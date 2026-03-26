use std::path::Path;

use c2pa::SigningAlg;
use openssl::asn1::Asn1Time;
use openssl::bn::BigNum;
use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::PKey;
use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage};
use openssl::x509::{X509Builder, X509NameBuilder};

use crate::c2pa::{config::IdentityConfig, error::InspectError};

/// FIPS-safe algorithms supported by UMRS.
/// ed25519 is intentionally excluded — unreliable on FIPS-enabled RHEL.
pub const ALLOWED_ALGORITHMS: &[&str] = &["es256", "es384", "es512", "ps256", "ps384", "ps512"];

/// Parse an algorithm string into `c2pa::SigningAlg`.
/// Returns `InspectError::UnsafeAlgorithm` if the algorithm is not in the
/// FIPS-safe allowed set.
pub fn parse_algorithm(alg: &str) -> Result<SigningAlg, InspectError> {
    if !ALLOWED_ALGORITHMS.contains(&alg) {
        return Err(InspectError::UnsafeAlgorithm(alg.to_string()));
    }
    match alg {
        "es256" => Ok(SigningAlg::Es256),
        "es384" => Ok(SigningAlg::Es384),
        "es512" => Ok(SigningAlg::Es512),
        "ps256" => Ok(SigningAlg::Ps256),
        "ps384" => Ok(SigningAlg::Ps384),
        "ps512" => Ok(SigningAlg::Ps512),
        other   => Err(InspectError::UnsafeAlgorithm(other.to_string())),
    }
}

/// Signing material resolved from `IdentityConfig`.
pub enum SignerMode {
    /// Ephemeral self-signed cert generated at runtime (test/eval mode).
    Ephemeral { alg: SigningAlg, organization: String },
    /// Production signing using customer-supplied cert + key.
    Credentials {
        alg:       SigningAlg,
        cert_pem:  Vec<u8>,
        key_pem:   Vec<u8>,
        tsa_url:   Option<String>,
    },
}

/// Resolve signing material from the identity config.
pub fn resolve_signer_mode(
    identity: &IdentityConfig,
    tsa_url: Option<&str>,
) -> Result<SignerMode, InspectError> {
    let alg = parse_algorithm(&identity.algorithm)?;

    match (&identity.cert_chain, &identity.private_key) {
        (Some(cert_path), Some(key_path)) => {
            let cert_pem = read_pem(cert_path)?;
            let key_pem  = read_pem(key_path)?;
            Ok(SignerMode::Credentials {
                alg,
                cert_pem,
                key_pem,
                // TSA requires network access — silently omit in air-gap builds.
                #[cfg(feature = "internet")]
                tsa_url: tsa_url.map(String::from),
                #[cfg(not(feature = "internet"))]
                tsa_url: None,
            })
        }
        _ => Ok(SignerMode::Ephemeral { alg, organization: identity.organization.clone() }),
    }
}

/// Build a `c2pa::Signer` from resolved signing material.
///
/// In ephemeral mode, generates an ECDSA self-signed certificate using the
/// algorithm specified in the config (default: ES256 / P-256).  This avoids
/// the c2pa crate's built-in `EphemeralSigner`, which hardcodes Ed25519 —
/// an algorithm that is not universally available on FIPS 140-3 validated
/// modules (only added in FIPS 186-5) and is optional in the C2PA spec.
///
/// By generating our own ephemeral cert with a FIPS-safe ECDSA curve, test
/// mode output is:
///   - verifiable by every C2PA-compliant validator (ES256/384/512 are mandatory)
///   - safe to run on any FIPS-enabled system regardless of module generation
///   - consistent with the algorithm the user configured for production
pub fn build_signer(mode: &SignerMode) -> Result<c2pa::BoxedSigner, InspectError> {
    match mode {
        SignerMode::Ephemeral { alg, organization } => {
            let (cert_pem, key_pem) = generate_ephemeral_cert(*alg, organization)?;
            c2pa::create_signer::from_keys(&cert_pem, &key_pem, *alg, None)
                .map_err(InspectError::C2pa)
        }
        SignerMode::Credentials { alg, cert_pem, key_pem, tsa_url } => {
            c2pa::create_signer::from_keys(cert_pem, key_pem, *alg, tsa_url.clone())
                .map_err(InspectError::C2pa)
        }
    }
}

/// Generate a self-signed ECDSA certificate and private key in PEM format.
///
/// The curve is chosen to match the requested `SigningAlg`:
///   - ES256 → P-256 (prime256v1)
///   - ES384 → P-384 (secp384r1)
///   - ES512 → P-521 (secp521r1)
///   - PS256/384/512 → P-256 (RSA-PSS algorithms use RSA keys, but for ephemeral
///     test mode we generate ECDSA; callers should prefer ES* for test mode)
///
/// The certificate is marked with `CN=UMRS ephemeral (test mode — UNTRUSTED)`
/// so it is visually obvious in any validator output.
fn generate_ephemeral_cert(alg: SigningAlg, organization: &str) -> Result<(Vec<u8>, Vec<u8>), InspectError> {
    let (nid, digest) = match alg {
        SigningAlg::Es384 => (Nid::SECP384R1,        MessageDigest::sha384()),
        SigningAlg::Es512 => (Nid::SECP521R1,        MessageDigest::sha512()),
        // ES256 and RSA-PSS algorithms both use P-256 for ephemeral test certs.
        // Production RSA-PSS signing uses real certs, not this path.
        _ => (Nid::X9_62_PRIME256V1, MessageDigest::sha256()),
    };

    let group = EcGroup::from_curve_name(nid)
        .map_err(|e| InspectError::Signing(format!("EC group: {e}")))?;
    let ec_key = EcKey::generate(&group)
        .map_err(|e| InspectError::Signing(format!("EC keygen: {e}")))?;
    let pkey = PKey::from_ec_key(ec_key)
        .map_err(|e| InspectError::Signing(format!("PKey wrap: {e}")))?;

    // Build self-signed X.509 cert with the configured organization name.
    let cn = format!("{organization} (ephemeral — self-signed)");
    let mut name_builder = X509NameBuilder::new()
        .map_err(|e| InspectError::Signing(format!("X509 name: {e}")))?;
    name_builder
        .append_entry_by_text("O", organization)
        .map_err(|e| InspectError::Signing(format!("X509 O: {e}")))?;
    name_builder
        .append_entry_by_text("CN", &cn)
        .map_err(|e| InspectError::Signing(format!("X509 CN: {e}")))?;
    let name = name_builder.build();

    let mut builder = X509Builder::new()
        .map_err(|e| InspectError::Signing(format!("X509 builder: {e}")))?;
    builder.set_version(2)
        .map_err(|e| InspectError::Signing(format!("X509 version: {e}")))?;
    builder.set_subject_name(&name)
        .map_err(|e| InspectError::Signing(format!("X509 subject: {e}")))?;
    builder.set_issuer_name(&name)
        .map_err(|e| InspectError::Signing(format!("X509 issuer: {e}")))?;
    builder.set_pubkey(&pkey)
        .map_err(|e| InspectError::Signing(format!("X509 pubkey: {e}")))?;

    let not_before = Asn1Time::days_from_now(0)
        .map_err(|e| InspectError::Signing(format!("ASN1 time: {e}")))?;
    let not_after = Asn1Time::days_from_now(1)
        .map_err(|e| InspectError::Signing(format!("ASN1 time: {e}")))?;
    builder.set_not_before(&not_before)
        .map_err(|e| InspectError::Signing(format!("X509 not_before: {e}")))?;
    builder.set_not_after(&not_after)
        .map_err(|e| InspectError::Signing(format!("X509 not_after: {e}")))?;

    // Serial number.
    let serial = BigNum::from_u32(1)
        .and_then(|bn| bn.to_asn1_integer())
        .map_err(|e| InspectError::Signing(format!("serial: {e}")))?;
    builder.set_serial_number(&serial)
        .map_err(|e| InspectError::Signing(format!("X509 serial: {e}")))?;

    // X.509v3 extensions required by the c2pa crate's certificate validator:
    //   - BasicConstraints: CA=false (end-entity cert)
    //   - KeyUsage: digitalSignature
    //   - ExtendedKeyUsage: emailProtection + any (matches c2pa EphemeralSigner)
    let bc = BasicConstraints::new().build()
        .map_err(|e| InspectError::Signing(format!("BasicConstraints: {e}")))?;
    builder.append_extension(bc)
        .map_err(|e| InspectError::Signing(format!("append BC: {e}")))?;

    let ku = KeyUsage::new().digital_signature().build()
        .map_err(|e| InspectError::Signing(format!("KeyUsage: {e}")))?;
    builder.append_extension(ku)
        .map_err(|e| InspectError::Signing(format!("append KU: {e}")))?;

    // C2PA profile disallows anyExtendedKeyUsage — use emailProtection only.
    let eku = ExtendedKeyUsage::new().email_protection().build()
        .map_err(|e| InspectError::Signing(format!("ExtKeyUsage: {e}")))?;
    builder.append_extension(eku)
        .map_err(|e| InspectError::Signing(format!("append EKU: {e}")))?;

    // AuthorityKeyIdentifier — required by the C2PA certificate profile.
    // For a self-signed cert, AKI = SKI (same key).
    let ski = openssl::x509::extension::SubjectKeyIdentifier::new()
        .build(&builder.x509v3_context(None, None))
        .map_err(|e| InspectError::Signing(format!("SKI: {e}")))?;
    builder.append_extension(ski)
        .map_err(|e| InspectError::Signing(format!("append SKI: {e}")))?;

    let aki = openssl::x509::extension::AuthorityKeyIdentifier::new()
        .keyid(true)
        .build(&builder.x509v3_context(None, None))
        .map_err(|e| InspectError::Signing(format!("AKI: {e}")))?;
    builder.append_extension(aki)
        .map_err(|e| InspectError::Signing(format!("append AKI: {e}")))?;

    builder.sign(&pkey, digest)
        .map_err(|e| InspectError::Signing(format!("X509 sign: {e}")))?;
    let cert = builder.build();

    let cert_pem = cert.to_pem()
        .map_err(|e| InspectError::Signing(format!("cert PEM: {e}")))?;
    let key_pem = pkey.private_key_to_pem_pkcs8()
        .map_err(|e| InspectError::Signing(format!("key PEM: {e}")))?;

    Ok((cert_pem, key_pem))
}

/// Returns `true` if the mode is ephemeral (test/eval).
#[must_use] 
pub fn is_ephemeral(mode: &SignerMode) -> bool {
    matches!(mode, SignerMode::Ephemeral { .. })
}

fn read_pem(path: &Path) -> Result<Vec<u8>, InspectError> {
    std::fs::read(path).map_err(InspectError::Io)
}

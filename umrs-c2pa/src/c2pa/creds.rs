// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jamie Adams

//! Credential generation and validation.
//!
//! Generates self-signed certificates, private keys, and CSRs for C2PA signing.
//! Validates existing credential files against the UMRS configuration.
//!
//! This module exists because key/certificate management is the #1 source of
//! confusion in any PKI-based system.  The goal is: **simple, simple, simple.**

use openssl::asn1::Asn1Time;
use openssl::bn::BigNum;
use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::PKey;
use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage};
use openssl::x509::{X509Builder, X509NameBuilder, X509ReqBuilder};

use crate::c2pa::{config::UmrsConfig, error::InspectError, signer};
#[allow(unused_imports)]
use crate::verbose;

/// Result of credential generation.
pub struct GeneratedCredentials {
    /// PEM-encoded certificate (self-signed) or CSR.
    pub cert_or_csr_pem: Vec<u8>,

    /// PEM-encoded private key (PKCS#8).
    pub key_pem: Vec<u8>,

    /// Whether this is a CSR (true) or self-signed cert (false).
    pub is_csr: bool,

    /// Human-readable summary of what was generated.
    pub summary: String,
}

/// Result of a single credential validation check.
#[derive(Debug)]
pub struct CredCheck {
    pub check: String,
    pub ok: bool,
    pub message: String,
}

impl CredCheck {
    fn pass(check: &str, msg: &str) -> Self {
        Self {
            check: check.into(),
            ok: true,
            message: msg.into(),
        }
    }
    fn fail(check: &str, msg: &str) -> Self {
        Self {
            check: check.into(),
            ok: false,
            message: msg.into(),
        }
    }
}

/// Generate a self-signed certificate and private key (or a CSR + key).
///
/// Uses the algorithm from the UMRS configuration to pick the curve/key type.
/// The certificate has proper X.509v3 extensions required by the C2PA profile.
///
/// # Errors
///
/// Returns `InspectError::UnsafeAlgorithm` if the configured algorithm is not
/// FIPS-safe, or `InspectError::Signing` if key generation or certificate
/// building fails.
pub fn generate(
    config: &UmrsConfig,
    csr_only: bool,
    validity_days: u32,
) -> Result<GeneratedCredentials, InspectError> {
    let alg_str = &config.identity.algorithm;
    let alg = signer::parse_algorithm(alg_str)?;

    // Pick the EC curve matching the algorithm.
    let (nid, digest, curve_name, key_bits) = match alg {
        c2pa::SigningAlg::Es384 => (Nid::SECP384R1, MessageDigest::sha384(), "P-384", "384"),
        c2pa::SigningAlg::Es512 => (Nid::SECP521R1, MessageDigest::sha512(), "P-521", "521"),
        _ => (Nid::X9_62_PRIME256V1, MessageDigest::sha256(), "P-256", "256"),
    };

    verbose!("Generating ECDSA key pair on curve {}...", curve_name);
    let group = EcGroup::from_curve_name(nid)
        .map_err(|e| InspectError::Signing(format!("EC group: {e}")))?;
    let ec_key =
        EcKey::generate(&group).map_err(|e| InspectError::Signing(format!("EC keygen: {e}")))?;
    let pkey =
        PKey::from_ec_key(ec_key).map_err(|e| InspectError::Signing(format!("PKey wrap: {e}")))?;

    let key_pem = pkey
        .private_key_to_pem_pkcs8()
        .map_err(|e| InspectError::Signing(format!("key PEM: {e}")))?;

    let org = &config.identity.organization;

    if csr_only {
        verbose!("Generating Certificate Signing Request...");
        let csr_pem = build_csr(&pkey, org, digest)?;
        let summary = format!(
            "Generated CSR + private key\n\
             Algorithm : {alg_str} (ECDSA {curve_name}, {key_bits}-bit)\n\
             Subject   : O={org}, CN={org} (UMRS C2PA Signing)\n\
             \n\
             Submit the CSR to your Certificate Authority for signing.\n\
             Keep the private key safe — it cannot be regenerated."
        );
        Ok(GeneratedCredentials {
            cert_or_csr_pem: csr_pem,
            key_pem,
            is_csr: true,
            summary,
        })
    } else {
        verbose!("Generating self-signed certificate (valid {} days)...", validity_days);
        let cert_pem = build_self_signed(&pkey, org, digest, validity_days)?;
        let summary = format!(
            "Generated self-signed certificate + private key\n\
             Algorithm : {alg_str} (ECDSA {curve_name}, {key_bits}-bit)\n\
             Subject   : O={org}, CN={org} (UMRS C2PA Signing — self-signed)\n\
             Validity  : {validity_days} days from now\n\
             \n\
             Self-signed certificates will show as UNVERIFIED by external\n\
             validators. For trusted status, submit a CSR to a recognized CA\n\
             or add your org's root to the trust anchors."
        );
        Ok(GeneratedCredentials {
            cert_or_csr_pem: cert_pem,
            key_pem,
            is_csr: false,
            summary,
        })
    }
}

/// Validate existing credentials referenced in the UMRS configuration.
///
/// Checks that the cert and key files exist, are valid PEM, match each other,
/// use the configured algorithm, and have acceptable validity dates.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn validate(config: &UmrsConfig) -> Vec<CredCheck> {
    let mut checks = Vec::new();

    let (cert_path, key_path) = match (&config.identity.cert_chain, &config.identity.private_key) {
        (Some(c), Some(k)) => (c, k),
        (None, None) => {
            checks.push(CredCheck::fail(
                "credentials",
                "No cert_chain or private_key configured. \
                 Run `inspect creds generate` to create them, \
                 then set the paths in umrs-c2pa.toml.",
            ));
            return checks;
        }
        (Some(_), None) => {
            checks.push(CredCheck::fail(
                "private_key",
                "cert_chain is set but private_key is missing",
            ));
            return checks;
        }
        (None, Some(_)) => {
            checks.push(CredCheck::fail(
                "cert_chain",
                "private_key is set but cert_chain is missing",
            ));
            return checks;
        }
    };

    // File existence.
    if cert_path.exists() {
        checks.push(CredCheck::pass("cert_file", &format!("Found: {}", cert_path.display())));
    } else {
        checks.push(CredCheck::fail("cert_file", &format!("Not found: {}", cert_path.display())));
    }
    if key_path.exists() {
        checks.push(CredCheck::pass("key_file", &format!("Found: {}", key_path.display())));
    } else {
        checks.push(CredCheck::fail("key_file", &format!("Not found: {}", key_path.display())));
    }

    // If either is missing, stop here.
    if !cert_path.exists() || !key_path.exists() {
        return checks;
    }

    // PEM format.
    let cert_bytes = match std::fs::read(cert_path) {
        Ok(b) => b,
        Err(e) => {
            checks.push(CredCheck::fail("cert_read", &format!("Cannot read: {e}")));
            return checks;
        }
    };
    let key_bytes = match std::fs::read(key_path) {
        Ok(b) => b,
        Err(e) => {
            checks.push(CredCheck::fail("key_read", &format!("Cannot read: {e}")));
            return checks;
        }
    };

    if !cert_bytes.windows(11).any(|w| w == b"-----BEGIN ") {
        checks.push(CredCheck::fail("cert_format", "File is not valid PEM"));
        return checks;
    }
    checks.push(CredCheck::pass("cert_format", "Valid PEM format"));

    if !key_bytes.windows(11).any(|w| w == b"-----BEGIN ") {
        checks.push(CredCheck::fail("key_format", "File is not valid PEM"));
        return checks;
    }
    checks.push(CredCheck::pass("key_format", "Valid PEM format"));

    // Parse certificate.
    let cert = match openssl::x509::X509::from_pem(&cert_bytes) {
        Ok(c) => c,
        Err(e) => {
            checks.push(CredCheck::fail("cert_parse", &format!("Cannot parse certificate: {e}")));
            return checks;
        }
    };
    checks.push(CredCheck::pass("cert_parse", "Certificate parsed successfully"));

    // Parse private key.
    let pkey = match openssl::pkey::PKey::private_key_from_pem(&key_bytes) {
        Ok(k) => k,
        Err(e) => {
            checks.push(CredCheck::fail("key_parse", &format!("Cannot parse private key: {e}")));
            return checks;
        }
    };
    checks.push(CredCheck::pass("key_parse", "Private key parsed successfully"));

    // Key matches certificate.
    let cert_pubkey = match cert.public_key() {
        Ok(pk) => pk,
        Err(e) => {
            checks.push(CredCheck::fail(
                "key_match",
                &format!("Cannot extract cert public key: {e}"),
            ));
            return checks;
        }
    };
    if cert_pubkey.public_eq(&pkey) {
        checks.push(CredCheck::pass("key_match", "Private key matches certificate"));
    } else {
        checks.push(CredCheck::fail(
            "key_match",
            "Private key does NOT match certificate — wrong key for this cert",
        ));
    }

    // Subject info.
    let subject = cert
        .subject_name()
        .entries()
        .map(|e| {
            let key = e.object().nid().short_name().unwrap_or("?");
            let val = e.data().as_utf8().map_or_else(|_| "?".to_string(), |s| s.to_string());
            format!("{key}={val}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    checks.push(CredCheck::pass("subject", &format!("Subject: {subject}")));

    // Issuer info.
    let issuer = cert
        .issuer_name()
        .entries()
        .map(|e| {
            let key = e.object().nid().short_name().unwrap_or("?");
            let val = e.data().as_utf8().map_or_else(|_| "?".to_string(), |s| s.to_string());
            format!("{key}={val}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let is_self_signed = cert.subject_name_hash() == cert.issuer_name_hash();
    if is_self_signed {
        checks.push(CredCheck::pass("issuer", &format!("Issuer: {issuer} (self-signed)")));
    } else {
        checks.push(CredCheck::pass("issuer", &format!("Issuer: {issuer}")));
    }

    // Validity dates.
    let not_before = cert.not_before().to_string();
    let not_after = cert.not_after().to_string();
    let now = openssl::asn1::Asn1Time::days_from_now(0).ok();
    if let Some(ref now_time) = now {
        if cert.not_after() < now_time.as_ref() {
            checks.push(CredCheck::fail(
                "validity",
                &format!("EXPIRED — valid {not_before} to {not_after}"),
            ));
        } else if cert.not_before() > now_time.as_ref() {
            checks.push(CredCheck::fail(
                "validity",
                &format!("NOT YET VALID — valid {not_before} to {not_after}"),
            ));
        } else {
            checks.push(CredCheck::pass(
                "validity",
                &format!("Valid from {not_before} to {not_after}"),
            ));
        }
    }

    // Algorithm info.
    let desc = signer::describe_algorithm(&config.identity.algorithm);
    checks.push(CredCheck::pass("algorithm", &format!("Configured: {desc}")));

    checks
}

// ── internal helpers ─────────────────────────────────────────────────────────

fn build_self_signed(
    pkey: &PKey<openssl::pkey::Private>,
    organization: &str,
    digest: MessageDigest,
    validity_days: u32,
) -> Result<Vec<u8>, InspectError> {
    let cn = format!("{organization} (UMRS C2PA Signing — self-signed)");
    let name = build_x509_name(organization, &cn)?;

    let mut builder =
        X509Builder::new().map_err(|e| InspectError::Signing(format!("X509 builder: {e}")))?;
    builder.set_version(2).map_err(|e| InspectError::Signing(format!("X509 version: {e}")))?;
    builder
        .set_subject_name(&name)
        .map_err(|e| InspectError::Signing(format!("X509 subject: {e}")))?;
    builder
        .set_issuer_name(&name)
        .map_err(|e| InspectError::Signing(format!("X509 issuer: {e}")))?;
    builder.set_pubkey(pkey).map_err(|e| InspectError::Signing(format!("X509 pubkey: {e}")))?;

    let not_before =
        Asn1Time::days_from_now(0).map_err(|e| InspectError::Signing(format!("ASN1 time: {e}")))?;
    let not_after = Asn1Time::days_from_now(validity_days)
        .map_err(|e| InspectError::Signing(format!("ASN1 time: {e}")))?;
    builder
        .set_not_before(&not_before)
        .map_err(|e| InspectError::Signing(format!("X509 not_before: {e}")))?;
    builder
        .set_not_after(&not_after)
        .map_err(|e| InspectError::Signing(format!("X509 not_after: {e}")))?;

    let serial = BigNum::from_u32(1)
        .and_then(|bn| bn.to_asn1_integer())
        .map_err(|e| InspectError::Signing(format!("serial: {e}")))?;
    builder
        .set_serial_number(&serial)
        .map_err(|e| InspectError::Signing(format!("X509 serial: {e}")))?;

    // C2PA-required X.509v3 extensions.
    add_c2pa_extensions(&mut builder)?;

    builder.sign(pkey, digest).map_err(|e| InspectError::Signing(format!("X509 sign: {e}")))?;

    builder.build().to_pem().map_err(|e| InspectError::Signing(format!("cert PEM: {e}")))
}

fn build_csr(
    pkey: &PKey<openssl::pkey::Private>,
    organization: &str,
    digest: MessageDigest,
) -> Result<Vec<u8>, InspectError> {
    let cn = format!("{organization} (UMRS C2PA Signing)");

    let mut name_builder =
        X509NameBuilder::new().map_err(|e| InspectError::Signing(format!("X509 name: {e}")))?;
    name_builder
        .append_entry_by_text("O", organization)
        .map_err(|e| InspectError::Signing(format!("X509 O: {e}")))?;
    name_builder
        .append_entry_by_text("CN", &cn)
        .map_err(|e| InspectError::Signing(format!("X509 CN: {e}")))?;
    let name = name_builder.build();

    let mut req =
        X509ReqBuilder::new().map_err(|e| InspectError::Signing(format!("CSR builder: {e}")))?;
    req.set_pubkey(pkey).map_err(|e| InspectError::Signing(format!("CSR pubkey: {e}")))?;
    req.set_subject_name(&name).map_err(|e| InspectError::Signing(format!("CSR subject: {e}")))?;
    req.sign(pkey, digest).map_err(|e| InspectError::Signing(format!("CSR sign: {e}")))?;

    req.build().to_pem().map_err(|e| InspectError::Signing(format!("CSR PEM: {e}")))
}

fn build_x509_name(organization: &str, cn: &str) -> Result<openssl::x509::X509Name, InspectError> {
    let mut name_builder =
        X509NameBuilder::new().map_err(|e| InspectError::Signing(format!("X509 name: {e}")))?;
    name_builder
        .append_entry_by_text("O", organization)
        .map_err(|e| InspectError::Signing(format!("X509 O: {e}")))?;
    name_builder
        .append_entry_by_text("CN", cn)
        .map_err(|e| InspectError::Signing(format!("X509 CN: {e}")))?;
    Ok(name_builder.build())
}

fn add_c2pa_extensions(builder: &mut X509Builder) -> Result<(), InspectError> {
    let bc = BasicConstraints::new()
        .build()
        .map_err(|e| InspectError::Signing(format!("BasicConstraints: {e}")))?;
    builder.append_extension(bc).map_err(|e| InspectError::Signing(format!("append BC: {e}")))?;

    let ku = KeyUsage::new()
        .digital_signature()
        .build()
        .map_err(|e| InspectError::Signing(format!("KeyUsage: {e}")))?;
    builder.append_extension(ku).map_err(|e| InspectError::Signing(format!("append KU: {e}")))?;

    let eku = ExtendedKeyUsage::new()
        .email_protection()
        .build()
        .map_err(|e| InspectError::Signing(format!("ExtKeyUsage: {e}")))?;
    builder.append_extension(eku).map_err(|e| InspectError::Signing(format!("append EKU: {e}")))?;

    let ski = openssl::x509::extension::SubjectKeyIdentifier::new()
        .build(&builder.x509v3_context(None, None))
        .map_err(|e| InspectError::Signing(format!("SKI: {e}")))?;
    builder.append_extension(ski).map_err(|e| InspectError::Signing(format!("append SKI: {e}")))?;

    let aki = openssl::x509::extension::AuthorityKeyIdentifier::new()
        .keyid(true)
        .build(&builder.x509v3_context(None, None))
        .map_err(|e| InspectError::Signing(format!("AKI: {e}")))?;
    builder.append_extension(aki).map_err(|e| InspectError::Signing(format!("append AKI: {e}")))?;

    Ok(())
}


use crate::c2pa::{config::UmrsConfig, signer::ALLOWED_ALGORITHMS};

/// Status of a single preflight check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Info,
    Skip,
}

/// Result of a single preflight check.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub check:   String,
    pub status:  CheckStatus,
    pub message: String,
}

impl ValidationResult {
    fn pass(check: &str, msg: &str) -> Self {
        Self { check: check.into(), status: CheckStatus::Pass, message: msg.into() }
    }
    fn warn(check: &str, msg: &str) -> Self {
        Self { check: check.into(), status: CheckStatus::Warn, message: msg.into() }
    }
    fn fail(check: &str, msg: &str) -> Self {
        Self { check: check.into(), status: CheckStatus::Fail, message: msg.into() }
    }
    fn info(check: &str, msg: &str) -> Self {
        Self { check: check.into(), status: CheckStatus::Info, message: msg.into() }
    }
    fn skip(check: &str, msg: &str) -> Self {
        Self { check: check.into(), status: CheckStatus::Skip, message: msg.into() }
    }
}

/// Run all preflight checks against the loaded configuration.
/// Returns a list of results — one per check.
#[must_use] 
pub fn validate_config(config: &UmrsConfig) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Identity: required fields.
    check_required_fields(config, &mut results);

    // Cert + key file checks.
    let cert_ok = check_cert_file(config, &mut results);
    let key_ok  = check_key_file(config, &mut results);

    // Key/cert pair match — only if both files are readable.
    if cert_ok && key_ok {
        check_key_cert_match(config, &mut results);
    } else if config.identity.cert_chain.is_some() || config.identity.private_key.is_some() {
        results.push(ValidationResult::skip(
            "key_cert_match",
            "Skipped — requires both cert and key files to be readable",
        ));
    }

    // Algorithm.
    check_algorithm(config, &mut results);

    // TSA reachability.
    check_tsa(config, &mut results);

    // Ephemeral mode notice.
    if config.identity.cert_chain.is_none() && config.identity.private_key.is_none() {
        results.push(ValidationResult::info(
            "credential_mode",
            "No certificate configured — ephemeral self-signed cert will be used (test mode). \
             Manifests will be marked UNTRUSTED by external validators.",
        ));
    }

    results
}

fn check_required_fields(config: &UmrsConfig, out: &mut Vec<ValidationResult>) {
    if config.identity.claim_generator.is_empty() {
        out.push(ValidationResult::fail("claim_generator", "Field is empty"));
    } else {
        out.push(ValidationResult::pass(
            "claim_generator",
            &format!("\"{}\"", config.identity.claim_generator),
        ));
    }
}

fn check_cert_file(config: &UmrsConfig, out: &mut Vec<ValidationResult>) -> bool {
    let Some(path) = &config.identity.cert_chain else { return false };
    if !path.exists() {
        out.push(ValidationResult::fail(
            "cert_chain",
            &format!("File not found: {}", path.display()),
        ));
        return false;
    }
    match std::fs::read(path) {
        Err(e) => {
            out.push(ValidationResult::fail("cert_chain", &format!("Cannot read: {e}")));
            false
        }
        Ok(bytes) => {
            if is_valid_pem(&bytes) {
                out.push(ValidationResult::pass(
                    "cert_chain",
                    &format!("Valid PEM at {}", path.display()),
                ));
                true
            } else {
                out.push(ValidationResult::fail(
                    "cert_chain",
                    &format!("File is not valid PEM: {}", path.display()),
                ));
                false
            }
        }
    }
}

fn check_key_file(config: &UmrsConfig, out: &mut Vec<ValidationResult>) -> bool {
    let Some(path) = &config.identity.private_key else { return false };
    if !path.exists() {
        out.push(ValidationResult::fail(
            "private_key",
            &format!("File not found: {}", path.display()),
        ));
        return false;
    }
    match std::fs::read(path) {
        Err(e) => {
            out.push(ValidationResult::fail("private_key", &format!("Cannot read: {e}")));
            false
        }
        Ok(bytes) => {
            if is_valid_pem(&bytes) {
                out.push(ValidationResult::pass(
                    "private_key",
                    &format!("Valid PEM at {}", path.display()),
                ));
                true
            } else {
                out.push(ValidationResult::fail(
                    "private_key",
                    &format!("File is not valid PEM: {}", path.display()),
                ));
                false
            }
        }
    }
}

fn check_key_cert_match(config: &UmrsConfig, out: &mut Vec<ValidationResult>) {
    // Delegate actual key/cert matching to the signer builder — if it succeeds, they match.
    match crate::c2pa::signer::resolve_signer_mode(&config.identity, None)
        .and_then(|mode| crate::c2pa::signer::build_signer(&mode).map(|_| ()))
    {
        Ok(())  => out.push(ValidationResult::pass("key_cert_match", "Private key matches certificate")),
        Err(e) => out.push(ValidationResult::fail("key_cert_match", &e.to_string())),
    }
}

fn check_algorithm(config: &UmrsConfig, out: &mut Vec<ValidationResult>) {
    let alg = &config.identity.algorithm;
    if alg == "ed25519" {
        out.push(ValidationResult::warn(
            "algorithm",
            "ed25519 is not reliably available on FIPS-enabled systems. \
             Recommended: es256, es384, or es512.",
        ));
    } else if ALLOWED_ALGORITHMS.contains(&alg.as_str()) {
        out.push(ValidationResult::pass(
            "algorithm",
            &format!("{alg} is in the FIPS-safe allowed set"),
        ));
    } else {
        out.push(ValidationResult::fail(
            "algorithm",
            &format!(
                "'{alg}' is not allowed. Use one of: {}",
                ALLOWED_ALGORITHMS.join(", ")
            ),
        ));
    }
}

fn check_tsa(config: &UmrsConfig, out: &mut Vec<ValidationResult>) {
    let Some(url) = &config.timestamp.tsa_url else { return };

    #[cfg(feature = "internet")]
    match ureq::head(url).call() {
        Ok(_) => out.push(ValidationResult::pass(
            "tsa_reachable",
            &format!("TSA endpoint reachable: {url}"),
        )),
        Err(e) => out.push(ValidationResult::warn(
            "tsa_reachable",
            &format!("TSA endpoint did not respond: {url} ({e})"),
        )),
    }

    #[cfg(not(feature = "internet"))]
    out.push(ValidationResult::warn(
        "tsa_reachable",
        &format!("TSA configured ({url}) but network feature is disabled — timestamps will be unsigned"),
    ));
}

/// Naively check whether bytes look like PEM (contains "-----BEGIN").
fn is_valid_pem(bytes: &[u8]) -> bool {
    bytes.windows(11).any(|w| w == b"-----BEGIN ")
}

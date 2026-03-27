// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jamie Adams

use crate::c2pa::{config::UmrsConfig, signer::ALLOWED_ALGORITHMS};
#[allow(unused_imports)]
use crate::verbose;

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
    pub check: String,
    pub status: CheckStatus,
    pub message: String,
}

impl ValidationResult {
    fn pass(check: &str, msg: &str) -> Self {
        Self {
            check: check.into(),
            status: CheckStatus::Pass,
            message: msg.into(),
        }
    }
    fn warn(check: &str, msg: &str) -> Self {
        Self {
            check: check.into(),
            status: CheckStatus::Warn,
            message: msg.into(),
        }
    }
    fn fail(check: &str, msg: &str) -> Self {
        Self {
            check: check.into(),
            status: CheckStatus::Fail,
            message: msg.into(),
        }
    }
    fn info(check: &str, msg: &str) -> Self {
        Self {
            check: check.into(),
            status: CheckStatus::Info,
            message: msg.into(),
        }
    }
    fn skip(check: &str, msg: &str) -> Self {
        Self {
            check: check.into(),
            status: CheckStatus::Skip,
            message: msg.into(),
        }
    }
}

/// Run all preflight checks against the loaded configuration.
/// Returns a list of results — one per check.
#[must_use]
pub fn validate_config(config: &UmrsConfig) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Identity: required fields.
    verbose!("Checking required identity fields...");
    check_required_fields(config, &mut results);

    // Cert + key file checks.
    verbose!("Checking certificate and key files...");
    let cert_ok = check_cert_file(config, &mut results);
    let key_ok = check_key_file(config, &mut results);

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
    verbose!("Checking signing algorithm...");
    check_algorithm(config, &mut results);

    // TSA reachability.
    verbose!("Checking TSA configuration...");
    check_tsa(config, &mut results);

    // Trust list checks.
    verbose!("Checking trust list configuration...");
    check_trust_config(config, &mut results);

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
    let Some(path) = &config.identity.cert_chain else {
        return false;
    };
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
    let Some(path) = &config.identity.private_key else {
        return false;
    };
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
        Ok(()) => {
            out.push(ValidationResult::pass("key_cert_match", "Private key matches certificate"));
        }
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
        let desc = crate::c2pa::signer::describe_algorithm(alg);
        out.push(ValidationResult::pass("algorithm", desc));
    } else {
        out.push(ValidationResult::fail(
            "algorithm",
            &format!("'{alg}' is not allowed. Use one of: {}", ALLOWED_ALGORITHMS.join(", ")),
        ));
    }
}

fn check_tsa(config: &UmrsConfig, out: &mut Vec<ValidationResult>) {
    let Some(url) = &config.timestamp.tsa_url else {
        return;
    };

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
        &format!(
            "TSA configured ({url}) but network feature is disabled — timestamps will be unsigned"
        ),
    ));
}

fn check_trust_config(config: &UmrsConfig, out: &mut Vec<ValidationResult>) {
    if !config.has_trust_config() {
        out.push(ValidationResult::info(
            "trust_config",
            "No trust lists configured — all manifests will show NO TRUST LIST. \
             See docs/trust-maintenance.md to set up trust anchors.",
        ));
        return;
    }

    // Check each trust file.
    if let Some(path) = &config.trust.trust_anchors {
        check_pem_file("trust_anchors", path, out);
    }
    if let Some(path) = &config.trust.user_anchors {
        check_pem_file("user_anchors", path, out);
    }
    if let Some(path) = &config.trust.allowed_list {
        if path.exists() {
            out.push(ValidationResult::pass(
                "allowed_list",
                &format!("File exists: {}", path.display()),
            ));
        } else {
            out.push(ValidationResult::fail(
                "allowed_list",
                &format!("File not found: {}", path.display()),
            ));
        }
    }
    if let Some(path) = &config.trust.trust_config {
        if path.exists() {
            out.push(ValidationResult::pass(
                "trust_eku_config",
                &format!("EKU config found: {}", path.display()),
            ));
        } else {
            out.push(ValidationResult::fail(
                "trust_eku_config",
                &format!("File not found: {}", path.display()),
            ));
        }
    }

    if let Some(url) = &config.trust.ocsp_responder {
        out.push(ValidationResult::info(
            "ocsp_responder",
            &format!("OCSP responder configured: {url} (not yet implemented — skeleton only)"),
        ));
    }
}

fn check_pem_file(name: &str, path: &std::path::Path, out: &mut Vec<ValidationResult>) {
    if !path.exists() {
        out.push(ValidationResult::fail(name, &format!("File not found: {}", path.display())));
        return;
    }
    match std::fs::read(path) {
        Err(e) => {
            out.push(ValidationResult::fail(name, &format!("Cannot read: {e}")));
        }
        Ok(bytes) => {
            if is_valid_pem(&bytes) {
                let cert_count =
                    bytes.windows(17).filter(|w| w == b"-----BEGIN CERT-").count().max(
                        // Fallback: count full BEGIN CERTIFICATE markers
                        String::from_utf8_lossy(&bytes)
                            .matches("-----BEGIN CERTIFICATE-----")
                            .count(),
                    );
                out.push(ValidationResult::pass(
                    name,
                    &format!("Valid PEM at {} ({} certificate(s))", path.display(), cert_count),
                ));
            } else {
                out.push(ValidationResult::fail(
                    name,
                    &format!("File is not valid PEM: {}", path.display()),
                ));
            }
        }
    }
}

/// Naively check whether bytes look like PEM (contains "-----BEGIN").
fn is_valid_pem(bytes: &[u8]) -> bool {
    bytes.windows(11).any(|w| w == b"-----BEGIN ")
}

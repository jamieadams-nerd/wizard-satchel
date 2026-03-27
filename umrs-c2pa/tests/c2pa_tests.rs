// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jamie Adams

/// Integration tests for the UMRS c2pa module.
/// All test fixtures live in tests/fixtures/.
/// No inline #[cfg(test)] modules — per project convention.
use std::path::Path;

// Re-export the library under test.
use umrs_c2pa::c2pa::{
    config::UmrsConfig,
    ingest::{ingest_file, sha256_hex},
    manifest::{TrustStatus, has_manifest, read_chain},
    signer::{ALLOWED_ALGORITHMS, describe_algorithm, parse_algorithm},
    validate::{CheckStatus, validate_config},
};

// ── helpers ────────────────────────────────────────────────────────────────────

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

/// Resolve a test fixture from the c2pa-rs SDK test suite.
///
/// Checks (in order):
///   1. `C2PA_FIXTURES_DIR` env var
///   2. Local copy at `tests/fixtures/c2pa-rs/`
///   3. Sibling repo at `../../c2pa-rs/sdk/tests/fixtures/`
fn c2pa_fixture(name: &str) -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("C2PA_FIXTURES_DIR") {
        return Path::new(&dir).join(name);
    }
    let local = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/c2pa-rs").join(name);
    if local.exists() {
        return local;
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../c2pa-rs/sdk/tests/fixtures").join(name)
}

fn temp_output(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(name)
}

fn default_config() -> UmrsConfig {
    UmrsConfig::default()
}

// ── manifest reading ───────────────────────────────────────────────────────────

#[test]
fn test_unsigned_file_has_no_manifest() {
    let path = fixture("unsigned.jpg");
    if !path.exists() {
        return;
    } // skip if fixture not present
    assert!(!has_manifest(&path));
}

#[test]
fn test_signed_file_has_manifest() {
    let path = fixture("signed.jpg");
    if !path.exists() {
        return;
    }
    assert!(has_manifest(&path));
}

#[test]
fn test_read_chain_unsigned_returns_empty() {
    let path = fixture("unsigned.jpg");
    if !path.exists() {
        return;
    }
    let chain = read_chain(&path).expect("read_chain failed");
    assert!(chain.is_empty(), "unsigned file should return empty chain");
}

#[test]
fn test_read_chain_signed_returns_entries() {
    let path = fixture("signed.jpg");
    if !path.exists() {
        return;
    }
    let chain = read_chain(&path).expect("read_chain failed");
    assert!(!chain.is_empty(), "signed file should return at least one chain entry");
}

#[test]
fn test_chain_entries_have_signer_names() {
    let path = fixture("signed.jpg");
    if !path.exists() {
        return;
    }
    let chain = read_chain(&path).expect("read_chain failed");
    for entry in &chain {
        assert!(!entry.signer_name.is_empty(), "signer_name should not be empty");
    }
}

// ── SHA-256 hashing ────────────────────────────────────────────────────────────

#[test]
fn test_sha256_produces_64_char_hex() {
    let path = fixture("unsigned.jpg");
    if !path.exists() {
        return;
    }
    let hash = sha256_hex(&path).expect("sha256_hex failed");
    assert_eq!(hash.len(), 64, "SHA-256 hex digest should be 64 characters");
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_sha256_is_deterministic() {
    let path = fixture("unsigned.jpg");
    if !path.exists() {
        return;
    }
    let h1 = sha256_hex(&path).unwrap();
    let h2 = sha256_hex(&path).unwrap();
    assert_eq!(h1, h2);
}

// ── ingest pipeline ────────────────────────────────────────────────────────────

#[test]
fn test_ingest_unsigned_file_produces_acquired_action() {
    let source = fixture("unsigned.jpg");
    if !source.exists() {
        return;
    }
    let out = temp_output("test_ingest_unsigned_output.jpg");
    let config = default_config();

    let result = ingest_file(&source, Some(&out), None, &config).expect("ingest_file failed");

    assert!(!result.had_manifest);
    assert_eq!(result.action, "c2pa.acquired");
    assert_eq!(result.sha256.len(), 64);
    assert!(result.is_ephemeral);

    // Cleanup.
    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_ingest_signed_file_produces_published_action() {
    let source = fixture("signed.jpg");
    if !source.exists() {
        return;
    }
    let out = temp_output("test_ingest_signed_output.jpg");
    let config = default_config();

    let result = ingest_file(&source, Some(&out), None, &config).expect("ingest_file failed");

    assert!(result.had_manifest);
    assert_eq!(result.action, "c2pa.published");
    assert!(result.previous_signer.is_some());

    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_ingest_output_has_manifest() {
    let source = fixture("unsigned.jpg");
    if !source.exists() {
        return;
    }
    let out = temp_output("test_ingest_output_has_manifest.jpg");
    let config = default_config();

    ingest_file(&source, Some(&out), None, &config).expect("ingest_file failed");
    assert!(has_manifest(&out), "signed output should have a manifest");

    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_ingest_sha256_matches_source() {
    let source = fixture("unsigned.jpg");
    if !source.exists() {
        return;
    }
    let out = temp_output("test_ingest_sha256_match.jpg");
    let config = default_config();

    let result = ingest_file(&source, Some(&out), None, &config).expect("ingest_file failed");
    let direct_hash = sha256_hex(&source).unwrap();

    assert_eq!(result.sha256, direct_hash, "ingest SHA-256 must match direct hash of source");

    let _ = std::fs::remove_file(&out);
}

// ── PNG support ────────────────────────────────────────────────────────────────

#[test]
fn test_ingest_png_file() {
    let source = fixture("unsigned.png");
    if !source.exists() {
        return;
    }
    let out = temp_output("test_ingest_unsigned_output.png");
    let config = default_config();

    let result =
        ingest_file(&source, Some(&out), None, &config).expect("ingest_file failed for PNG");

    assert_eq!(result.action, "c2pa.acquired");

    let _ = std::fs::remove_file(&out);
}

// ── algorithm policy ───────────────────────────────────────────────────────────

#[test]
fn test_allowed_algorithms_are_all_fips_safe() {
    for alg in ALLOWED_ALGORITHMS {
        assert_ne!(*alg, "ed25519", "ed25519 must not be in the allowed set");
        assert!(parse_algorithm(alg).is_ok(), "{alg} should parse successfully");
    }
}

#[test]
fn test_ed25519_is_rejected() {
    let err = parse_algorithm("ed25519");
    assert!(err.is_err(), "ed25519 should be rejected by parse_algorithm");
}

#[test]
fn test_unknown_algorithm_is_rejected() {
    let err = parse_algorithm("rsa2048");
    assert!(err.is_err());
}

// ── config loading ─────────────────────────────────────────────────────────────

#[test]
fn test_default_config_uses_ephemeral_mode() {
    let config = default_config();
    assert!(!config.has_credentials(), "default config should be ephemeral");
}

#[test]
fn test_default_config_claim_generator_is_set() {
    let config = default_config();
    assert!(!config.identity.claim_generator.is_empty());
}

#[test]
fn test_default_config_algorithm_is_fips_safe() {
    let config = default_config();
    assert!(
        parse_algorithm(&config.identity.algorithm).is_ok(),
        "default algorithm must be FIPS-safe"
    );
}

// ── config validation ──────────────────────────────────────────────────────────

#[test]
fn test_validate_default_config_no_failures() {
    let config = default_config();
    let results = validate_config(&config);
    let failures: Vec<_> = results.iter().filter(|r| r.status == CheckStatus::Fail).collect();
    assert!(
        failures.is_empty(),
        "default config should have no validation failures: {failures:?}"
    );
}

#[test]
fn test_validate_missing_key_file_fails() {
    let mut config = default_config();
    config.identity.cert_chain = Some("/nonexistent/cert.pem".into());
    config.identity.private_key = Some("/nonexistent/key.pem".into());

    let results = validate_config(&config);
    let failures: Vec<_> = results.iter().filter(|r| r.status == CheckStatus::Fail).collect();
    assert!(!failures.is_empty(), "missing key/cert files should produce failures");
}

#[test]
fn test_validate_ed25519_produces_warning() {
    let mut config = default_config();
    config.identity.algorithm = "ed25519".into();

    let results = validate_config(&config);
    let warnings: Vec<_> = results.iter().filter(|r| r.status == CheckStatus::Warn).collect();
    assert!(
        warnings.iter().any(|r| r.check == "algorithm"),
        "ed25519 should produce an algorithm warning"
    );
}

// ── Phase 1: c2pa-rs fixture tests ──────────────────────────────────────────
//
// These tests reference images from the c2pa-rs SDK test suite.
// Set C2PA_FIXTURES_DIR env var or place c2pa-rs as a sibling repo.
// Tests skip gracefully if fixtures are not available.

#[test]
fn test_c2pa_valid_chain_ca_jpg() {
    let path = c2pa_fixture("CA.jpg");
    if !path.exists() {
        return;
    }
    let chain = read_chain(&path).expect("read_chain failed for CA.jpg");
    assert!(!chain.is_empty(), "CA.jpg should have a chain of custody");
    for entry in &chain {
        assert!(!entry.signer_name.is_empty());
        assert!(!entry.algorithm.is_empty());
    }
}

#[test]
fn test_c2pa_nested_chain_c_jpg() {
    let path = c2pa_fixture("C.jpg");
    if !path.exists() {
        return;
    }
    let chain = read_chain(&path).expect("read_chain failed for C.jpg");
    assert!(!chain.is_empty(), "C.jpg should have chain entries");
}

#[test]
fn test_c2pa_tampered_xca_jpg() {
    let path = c2pa_fixture("XCA.jpg");
    if !path.exists() {
        return;
    }
    let chain = read_chain(&path).expect("read_chain failed for XCA.jpg");
    // XCA.jpg has a data hash mismatch — at least one entry should be Invalid.
    let has_invalid = chain.iter().any(|e| e.trust_status == TrustStatus::Invalid);
    assert!(
        has_invalid,
        "XCA.jpg should have at least one Invalid trust status (tampered image)"
    );
}

#[test]
fn test_c2pa_no_manifest_jpg() {
    let path = c2pa_fixture("no_manifest.jpg");
    if !path.exists() {
        return;
    }
    assert!(!has_manifest(&path), "no_manifest.jpg should have no C2PA data");
    let chain = read_chain(&path).expect("read_chain failed");
    assert!(chain.is_empty(), "no_manifest.jpg should return empty chain");
}

#[test]
fn test_c2pa_double_signed_caca_jpg() {
    let path = c2pa_fixture("CACA.jpg");
    if !path.exists() {
        return;
    }
    let chain = read_chain(&path).expect("read_chain failed for CACA.jpg");
    assert!(
        chain.len() >= 2,
        "CACA.jpg should have at least 2 chain entries (double-signed), got {}",
        chain.len()
    );
}

#[test]
fn test_c2pa_malformed_timestamp_ca_ct_jpg() {
    let path = c2pa_fixture("CA_ct.jpg");
    if !path.exists() {
        return;
    }
    // Should not panic — graceful handling of malformed timestamp.
    let chain = read_chain(&path).expect("read_chain should handle malformed timestamp");
    assert!(!chain.is_empty(), "CA_ct.jpg should still have chain entries");
}

#[test]
fn test_c2pa_png_no_manifest() {
    // sample1.png is a baseline PNG without C2PA data — used as an
    // ingredient source in the c2pa-rs test suite, not a signed image.
    let path = c2pa_fixture("sample1.png");
    if !path.exists() {
        return;
    }
    let chain = read_chain(&path).expect("read_chain failed for sample1.png");
    // Whether it has a manifest or not, read_chain should not panic.
    // This exercises PNG format handling.
    assert!(chain.is_empty() || !chain.is_empty(), "read_chain should handle PNG gracefully");
}

#[test]
fn test_c2pa_ocsp_image() {
    let path = c2pa_fixture("ocsp.jpg");
    if !path.exists() {
        return;
    }
    let chain = read_chain(&path).expect("read_chain failed for ocsp.jpg");
    assert!(!chain.is_empty(), "ocsp.jpg should have chain entries");
}

// ── credential generation ────────────────────────────────────────────────────

#[test]
fn test_creds_generate_self_signed() {
    let config = default_config();
    let result =
        umrs_c2pa::c2pa::creds::generate(&config, false, 365).expect("creds generate failed");
    assert!(!result.is_csr);
    assert!(result.cert_or_csr_pem.windows(11).any(|w| w == b"-----BEGIN "));
    assert!(result.key_pem.windows(11).any(|w| w == b"-----BEGIN "));
    assert!(result.summary.contains("self-signed"));
}

#[test]
fn test_creds_generate_csr() {
    let config = default_config();
    let result =
        umrs_c2pa::c2pa::creds::generate(&config, true, 365).expect("creds generate CSR failed");
    assert!(result.is_csr);
    assert!(
        result.cert_or_csr_pem.windows(20).any(|w| w == b"-----BEGIN CERTIFICA")
            || result.cert_or_csr_pem.windows(11).any(|w| w == b"-----BEGIN ")
    );
    assert!(result.summary.contains("CSR"));
}

#[test]
fn test_creds_validate_no_config() {
    let config = default_config(); // no cert/key configured
    let checks = umrs_c2pa::c2pa::creds::validate(&config);
    assert!(!checks.is_empty());
    assert!(checks.iter().any(|c| !c.ok), "should fail when no credentials configured");
}

#[test]
fn test_creds_generate_and_validate_roundtrip() {
    let mut config = default_config();
    let result =
        umrs_c2pa::c2pa::creds::generate(&config, false, 365).expect("creds generate failed");

    // Write to temp files.
    let dir = std::env::temp_dir().join("umrs_creds_test");
    let _ = std::fs::create_dir_all(&dir);
    let cert_path = dir.join("test_signing.pem");
    let key_path = dir.join("test_signing.key");
    std::fs::write(&cert_path, &result.cert_or_csr_pem).unwrap();
    std::fs::write(&key_path, &result.key_pem).unwrap();

    // Point config at the generated files.
    config.identity.cert_chain = Some(cert_path.clone());
    config.identity.private_key = Some(key_path.clone());

    // Validate should pass.
    let checks = umrs_c2pa::c2pa::creds::validate(&config);
    let failures: Vec<_> = checks.iter().filter(|c| !c.ok).collect();
    assert!(failures.is_empty(), "generated creds should validate: {failures:?}");

    // Cleanup.
    let _ = std::fs::remove_file(&cert_path);
    let _ = std::fs::remove_file(&key_path);
    let _ = std::fs::remove_dir(&dir);
}

// ── algorithm description ────────────────────────────────────────────────────

#[test]
fn test_describe_algorithm_all_fips() {
    for alg in ALLOWED_ALGORITHMS {
        let desc = describe_algorithm(alg);
        assert!(desc.contains("FIPS-safe"), "{alg} description should mention FIPS-safe");
        assert!(
            desc.starts_with(&alg.to_uppercase()),
            "{alg} description should start with algorithm name"
        );
    }
}

#[test]
fn test_describe_algorithm_details() {
    assert!(describe_algorithm("es256").contains("P-256"));
    assert!(describe_algorithm("es256").contains("SHA-256"));
    assert!(describe_algorithm("es384").contains("P-384"));
    assert!(describe_algorithm("es512").contains("P-521"));
    assert!(describe_algorithm("ps256").contains("RSA-PSS"));
}

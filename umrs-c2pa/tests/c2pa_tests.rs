/// Integration tests for the UMRS c2pa module.
/// All test fixtures live in tests/fixtures/.
/// No inline #[cfg(test)] modules — per project convention.

use std::path::Path;

// Re-export the library under test.
use umrs_c2pa::c2pa::{
    config::{IdentityConfig, UmrsConfig},
    ingest::{ingest_file, sha256_hex},
    manifest::{has_manifest, read_chain},
    signer::{parse_algorithm, ALLOWED_ALGORITHMS},
    validate::{validate_config, CheckStatus},
};

// ── helpers ────────────────────────────────────────────────────────────────────

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
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
    if !path.exists() { return; } // skip if fixture not present
    assert!(!has_manifest(&path));
}

#[test]
fn test_signed_file_has_manifest() {
    let path = fixture("signed.jpg");
    if !path.exists() { return; }
    assert!(has_manifest(&path));
}

#[test]
fn test_read_chain_unsigned_returns_empty() {
    let path = fixture("unsigned.jpg");
    if !path.exists() { return; }
    let chain = read_chain(&path).expect("read_chain failed");
    assert!(chain.is_empty(), "unsigned file should return empty chain");
}

#[test]
fn test_read_chain_signed_returns_entries() {
    let path = fixture("signed.jpg");
    if !path.exists() { return; }
    let chain = read_chain(&path).expect("read_chain failed");
    assert!(!chain.is_empty(), "signed file should return at least one chain entry");
}

#[test]
fn test_chain_entries_have_signer_names() {
    let path = fixture("signed.jpg");
    if !path.exists() { return; }
    let chain = read_chain(&path).expect("read_chain failed");
    for entry in &chain {
        assert!(!entry.signer_name.is_empty(), "signer_name should not be empty");
    }
}

// ── SHA-256 hashing ────────────────────────────────────────────────────────────

#[test]
fn test_sha256_produces_64_char_hex() {
    let path = fixture("unsigned.jpg");
    if !path.exists() { return; }
    let hash = sha256_hex(&path).expect("sha256_hex failed");
    assert_eq!(hash.len(), 64, "SHA-256 hex digest should be 64 characters");
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_sha256_is_deterministic() {
    let path = fixture("unsigned.jpg");
    if !path.exists() { return; }
    let h1 = sha256_hex(&path).unwrap();
    let h2 = sha256_hex(&path).unwrap();
    assert_eq!(h1, h2);
}

// ── ingest pipeline ────────────────────────────────────────────────────────────

#[test]
fn test_ingest_unsigned_file_produces_acquired_action() {
    let source = fixture("unsigned.jpg");
    if !source.exists() { return; }
    let out = temp_output("test_ingest_unsigned_output.jpg");
    let config = default_config();

    let result = ingest_file(&source, Some(&out), &config)
        .expect("ingest_file failed");

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
    if !source.exists() { return; }
    let out = temp_output("test_ingest_signed_output.jpg");
    let config = default_config();

    let result = ingest_file(&source, Some(&out), &config)
        .expect("ingest_file failed");

    assert!(result.had_manifest);
    assert_eq!(result.action, "c2pa.published");
    assert!(result.previous_signer.is_some());

    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_ingest_output_has_manifest() {
    let source = fixture("unsigned.jpg");
    if !source.exists() { return; }
    let out = temp_output("test_ingest_output_has_manifest.jpg");
    let config = default_config();

    ingest_file(&source, Some(&out), &config).expect("ingest_file failed");
    assert!(has_manifest(&out), "signed output should have a manifest");

    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_ingest_sha256_matches_source() {
    let source = fixture("unsigned.jpg");
    if !source.exists() { return; }
    let out = temp_output("test_ingest_sha256_match.jpg");
    let config = default_config();

    let result = ingest_file(&source, Some(&out), &config).expect("ingest_file failed");
    let direct_hash = sha256_hex(&source).unwrap();

    assert_eq!(result.sha256, direct_hash, "ingest SHA-256 must match direct hash of source");

    let _ = std::fs::remove_file(&out);
}

// ── PNG support ────────────────────────────────────────────────────────────────

#[test]
fn test_ingest_png_file() {
    let source = fixture("unsigned.png");
    if !source.exists() { return; }
    let out = temp_output("test_ingest_unsigned_output.png");
    let config = default_config();

    let result = ingest_file(&source, Some(&out), &config)
        .expect("ingest_file failed for PNG");

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
    config.identity.cert_chain  = Some("/nonexistent/cert.pem".into());
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

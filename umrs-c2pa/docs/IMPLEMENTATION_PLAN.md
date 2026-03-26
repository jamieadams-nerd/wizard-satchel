# UMRS C2PA — Implementation Plan

> Prototype reference implementation for C2PA manifest inspection, ingest signing,
> and chain-of-custody reporting. Designed to be adopted by downstream systems.
>
> Date: 2026-03-26
> Rust SDK: `c2pa` v0.78.6
> Target: FIPS-enabled RHEL (primary), Ubuntu non-FIPS (secondary)

---

## Table of Contents

1. [Background and Purpose](#1-background-and-purpose)
2. [What We Are Building](#2-what-we-are-building)
3. [Repository Layout](#3-repository-layout)
4. [Cargo.toml — Dependencies and Features](#4-cargotoml--dependencies-and-features)
5. [Module Design — src/c2pa/](#5-module-design--srcc2pa)
6. [Configuration File Design](#6-configuration-file-design)
7. [CLI Design](#7-cli-design)
8. [Ingest Pipeline Logic](#8-ingest-pipeline-logic)
9. [Chain of Custody Display](#9-chain-of-custody-display)
10. [Algorithm Policy — FIPS + C2PA Intersection](#10-algorithm-policy--fips--c2pa-intersection)
11. [OpenSSL Build Matrix](#11-openssl-build-matrix)
12. [Logging — systemd-journal-logger](#12-logging--systemd-journal-logger)
13. [Config Validation — inspect config validate](#13-config-validation--inspect-config-validate)
14. [Config Generation — inspect config generate](#14-config-generation--inspect-config-generate)
15. [Test Strategy](#15-test-strategy)
16. [Build and Run Instructions](#16-build-and-run-instructions)
17. [Future Phases](#17-future-phases)

---

## 1. Background and Purpose

### What Is C2PA?

C2PA (Coalition for Content Provenance and Authenticity) is an open technical
standard for attaching tamper-evident provenance metadata directly inside media
files. It answers: *where did this content come from, who touched it, and has it
been modified since?*

The standard is maintained by Adobe, Microsoft, Intel, the BBC, and others. It is
deployed in production cameras (Leica M11-P, Sony Alpha), generative AI tools
(Adobe Firefly), and social platforms.

C2PA does **not** prevent copying. It proves **origin and chain of custody**.

### What Is a Manifest?

A **C2PA Manifest** is a tamper-evident, cryptographically signed record embedded
directly inside the media file using **JUMBF** (JPEG Universal Metadata Box
Format). It is not a sidecar file — it travels with the asset.

```
Manifest
├── Assertions[]      ← typed metadata records (what happened, by whom)
├── Claim             ← signed summary referencing all assertions + asset hash
└── Claim Signature   ← X.509 cryptographic signature over the Claim
```

A **Manifest Store** contains multiple manifests — one per editing step — linked
into a verifiable chain of custody. The most recent is the **active manifest**.

### The Evidence Chain

When a tool processes an image, it embeds the previous manifest as an
**Ingredient** and adds a new signed manifest on top:

```
[Camera — Sony]        c2pa.created  — original capture, hash of raw pixels
      ↓ ingredient
[Adobe Lightroom]      c2pa.edited   — color grade, hash of edited pixels
      ↓ ingredient
[UMRS Ingest]          c2pa.published — accepted as-is, SHA-256 recorded at ingest
```

Each hop is independently verifiable. The SHA-256 hash at each step allows
downstream recipients to confirm the file has not changed since UMRS accepted it.

### Trust Levels

Each manifest entry in the chain is evaluated independently:

| Status | Meaning |
|--------|---------|
| **TRUSTED** | Cert chain leads to a root CA in the C2PA Trust List |
| **UNTRUSTED** | Signature valid but CA is not on the Trust List (self-signed, test cert) |
| **INVALID** | Signature failed, or asset hash does not match — tampering detected |
| **REVOKED** | Certificate was revoked by the issuing CA |

UMRS test deployments will show **UNTRUSTED** — correct and honest. Production
deployments with a cert from a recognized CA will show **TRUSTED**.

---

## 2. What We Are Building

### UMRS Ingest Signing

UMRS operates a trusted ingest dropbox. Files arrive and are processed as follows:

**Case 1 — File has no C2PA manifest:**
- File arrives from unknown origin
- UMRS computes SHA-256 of the file at ingest time
- UMRS signs with action `c2pa.acquired`
- Reason: *"Received at UMRS trusted ingest dropbox. Origin unknown. No modifications made."*
- Log entry records filename + SHA-256 + action

**Case 2 — File already has a C2PA manifest:**
- UMRS reads the existing manifest store
- Extracts the most recent signer (last entity in the chain) — name + timestamp
- UMRS adds its own manifest on top with action `c2pa.published`
- Reason: *"Received at UMRS trusted ingest dropbox with existing provenance. No modifications made."*
- Log entry records filename + SHA-256 + last previous signer + action

In both cases:
- The SHA-256 hash of the file bytes is recorded in the log and in the manifest
- The output file is a signed copy — the original is not modified
- The chain of custody is displayed to stdout

### What Downstream Recipients Can Verify

When a file leaves UMRS and reaches a downstream recipient, they can:
1. Verify UMRS's manifest signature → proves the file hasn't changed since UMRS signed it
2. Verify UMRS's `c2pa.hash.data` assertion → SHA-256 matches the file bytes
3. Walk the ingredient chain → see the full history before UMRS received it
4. Compare UMRS's hash to the previous signer's hash → proves UMRS made no changes

---

## 3. Repository Layout

```
umrs-c2pa/
├── Cargo.toml                  # workspace/crate manifest
├── Cargo.lock
├── umrs-c2pa.toml.example      # documented config template
├── C2PA_REPORT.md              # C2PA concepts reference
├── IMPLEMENTATION_PLAN.md      # this file
├── tests/
│   ├── fixtures/               # test images (JPEG, PNG, WebP, etc.)
│   └── c2pa_tests.rs           # integration tests
└── src/
    ├── main.rs                 # CLI only — argument parsing + output formatting
    └── c2pa/
        ├── mod.rs              # re-exports only
        ├── config.rs           # UmrsConfig struct, TOML loading, defaults
        ├── error.rs            # InspectError (thiserror)
        ├── ingest.rs           # ingest pipeline: sha256, acquire/publish logic
        ├── manifest.rs         # read_manifest(), chain-of-custody walker
        ├── signer.rs           # signing abstraction: ephemeral + callback
        ├── validate.rs         # config preflight validation logic
        └── report.rs           # chain-of-custody display + trust indicators
```

**Rule:** `main.rs` contains argument parsing and output formatting only.
All logic lives under `src/c2pa/`. Each file has a single clear responsibility.

---

## 4. Cargo.toml — Dependencies and Features

```toml
[package]
name    = "umrs-c2pa"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "inspect"
path = "src/main.rs"

[features]
default          = ["vendored-openssl"]
vendored-openssl = ["openssl/vendored"]   # hermetic build — no system OpenSSL required
system-openssl   = []                     # link system OpenSSL — use on FIPS RHEL

[dependencies]
# C2PA manifest read/write
c2pa = { version = "0.78.6", default-features = false, features = ["openssl", "file_io"] }

# OpenSSL — feature-gated (see build matrix)
openssl = { version = "0.10", optional = true }

# Error handling
thiserror = "1"       # typed errors in library code
anyhow    = "1"       # ergonomic error handling in binary

# CLI
clap = { version = "4", features = ["derive"] }   # subcommand CLI

# Config
toml       = "0.8"    # TOML config file parsing
serde      = { version = "1", features = ["derive"] }
serde_json = "1"      # JSON manifest output

# Logging
log                    = "0.4"   # logging facade
systemd-journal-logger = "2"     # journald backend (tag: umrs)

# Hashing
sha2 = "0.10"    # SHA-256 for file hash at ingest
hex  = "0.4"     # hex encoding of hash output
```

### Build Commands

```sh
# Default (vendored OpenSSL — works everywhere)
cargo build --release

# FIPS RHEL (system OpenSSL — uses RHEL's FIPS-validated module)
cargo build --release --no-default-features --features system-openssl
```

---

## 5. Module Design — src/c2pa/

### `mod.rs` — Re-exports only

```rust
pub mod config;
pub mod error;
pub mod ingest;
pub mod manifest;
pub mod report;
pub mod signer;
pub mod validate;

pub use config::UmrsConfig;
pub use error::InspectError;
pub use ingest::ingest_file;
pub use manifest::read_chain;
pub use report::print_chain;
```

### `error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum InspectError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("C2PA error: {0}")]
    C2pa(#[from] c2pa::Error),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Signing error: {0}")]
    Signing(String),
    #[error("Hash error: {0}")]
    Hash(String),
}
```

### `config.rs`

Loads and owns the `umrs-c2pa.toml` configuration. Provides `UmrsConfig::load(path)`
and `UmrsConfig::default()` (ephemeral signer, INFO logging, sensible policy defaults).

Key structs:
- `UmrsConfig` — top-level
- `IdentityConfig` — cert_chain, private_key, claim_generator, organization, algorithm
- `TimestampConfig` — optional TSA URL
- `PolicyConfig` — action labels + reason strings for unsigned/signed cases
- `LoggingConfig` — enabled flag + level

### `manifest.rs`

- `read_chain(path) -> Result<Vec<ChainEntry>, InspectError>`
  Reads the manifest store and returns the chain of custody as a `Vec<ChainEntry>`,
  ordered from oldest (deepest ingredient) to newest (active manifest).

- `ChainEntry` struct:
  - `signer_name: String` — from cert CN / claim_generator
  - `issuer: String` — CA that issued the signing cert
  - `signed_at: Option<DateTime<Utc>>`
  - `trust_status: TrustStatus` — Trusted / Untrusted / Invalid / Revoked
  - `alg: String` — signing algorithm used

### `signer.rs`

- `build_signer(config: &IdentityConfig) -> Result<Box<dyn c2pa::Signer>, InspectError>`
  - If cert_chain + private_key are set → `CallbackSigner` using those files
  - If not set → `EphemeralSigner` (test mode)
  - Validates that the algorithm is in the FIPS-safe allowed set

### `ingest.rs`

- `ingest_file(path, config) -> Result<IngestResult, InspectError>`
  - Computes SHA-256 of the file
  - Reads existing manifest chain (if any)
  - Chooses action: `c2pa.acquired` (no manifest) or `c2pa.published` (has manifest)
  - Builds and signs new manifest
  - Writes signed output file
  - Returns `IngestResult` with all fields needed for logging and display

### `validate.rs`

- `validate_config(config: &UmrsConfig) -> Vec<ValidationResult>`
  Runs all preflight checks, returns a list of pass/warn/fail items.
  Each item: `{ check: String, status: CheckStatus, message: String }`

### `report.rs`

- `print_chain(chain: &[ChainEntry], ingest: &IngestResult)`
  Renders the chain-of-custody table to stdout with trust indicators.
- `print_validation_report(results: &[ValidationResult])`
  Renders the config preflight report.

---

## 6. Configuration File Design

```toml
# umrs-c2pa.toml

[identity]
claim_generator = "UMRS Reference System/1.0"
organization    = "Acme Corp"
cert_chain      = "/etc/umrs/certs/signing.pem"
private_key     = "/etc/umrs/certs/signing.key"
# Allowed: es256 | es384 | es512 | ps256 | ps384 | ps512
# ed25519 excluded — unreliable on FIPS RHEL
# Strongest FIPS+C2PA intersection: es512
algorithm       = "es256"

[timestamp]
# Omit or comment out to disable TSA timestamps
tsa_url = "http://timestamp.digicert.com"

[policy]
unsigned_action = "c2pa.acquired"
unsigned_reason = "Received at UMRS trusted ingest dropbox. Origin unknown. No modifications made."

signed_action   = "c2pa.published"
signed_reason   = "Received at UMRS trusted ingest dropbox with existing provenance. No modifications made."

[logging]
enabled = true
level   = "info"    # off | error | warn | info | debug | trace
```

If no config file is found, UMRS runs in **ephemeral test mode** — self-signed cert
is generated at runtime, manifests are marked UNTRUSTED, logging defaults to INFO.

---

## 7. CLI Design

```
inspect c2pa <FILE>                    # read and display chain of custody
inspect c2pa --sign <FILE>             # ingest: sign file, display chain
inspect c2pa --json <FILE>             # emit full manifest store as JSON
inspect config validate [--config]     # preflight all config checks
inspect config generate [--output]     # write a starter config template
```

### Chain of Custody Output Example

```
Chain of Custody — photo.jpg
SHA-256: a3f1b2c4d5e6...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1  [TRUSTED]    Sony Corporation
                  Signed : 2025-09-14 08:12:33 UTC
                  Issuer : Sony Imaging Products CA
                  Alg    : es256

  2  [TRUSTED]    Adobe Inc. / Adobe Lightroom
                  Signed : 2025-10-02 14:45:01 UTC
                  Issuer : Adobe Content Credentials CA
                  Alg    : es256

  3  [UNTRUSTED]  UMRS Reference System/1.0
                  Signed : 2026-03-26 11:00:00 UTC
                  Issuer : Self-signed (test mode)
                  Alg    : es256
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Hash consistency: PASS — file unchanged across all signing events
```

### Config Validate Output Example

```
inspect config validate --config /etc/umrs/umrs-c2pa.toml

[OK]   TOML syntax valid
[OK]   Required fields present
[OK]   cert_chain file exists: /etc/umrs/certs/signing.pem
[OK]   private_key file exists: /etc/umrs/certs/signing.key
[OK]   Certificate chain parses (2 certs)
[OK]   Private key matches certificate
[WARN] Certificate expires in 14 days
[OK]   Algorithm es256 is FIPS-safe
[OK]   TSA endpoint reachable: http://timestamp.digicert.com
────────────────────────────────────────────────────────
All checks passed (1 warning). Configuration is ready.
```

---

## 8. Ingest Pipeline Logic

```
ingest_file(path, config)
  │
  ├─ 1. Compute SHA-256 of file bytes
  │
  ├─ 2. Attempt c2pa::Reader::from_file(path)
  │     ├─ Error / no manifest  →  case = Acquired
  │     └─ Manifest found       →  case = Published
  │                                 extract last signer name + timestamp
  │
  ├─ 3. Build signer (ephemeral or from config certs)
  │
  ├─ 4. Build c2pa::Builder
  │     ├─ Set claim_generator from config
  │     ├─ Add c2pa.actions assertion (acquired or published)
  │     ├─ Add reason string from config
  │     ├─ If existing manifest: add as Ingredient
  │     └─ c2pa.hash.data covers full file bytes
  │
  ├─ 5. builder.sign() → write signed output file
  │
  └─ 6. Log to journald (tag: umrs)
        ├─ Acquired: file, sha256, action=c2pa.acquired
        └─ Published: file, sha256, previous_signer, signed_at, action=c2pa.published
```

---

## 9. Chain of Custody Display

Walking the chain:

```
c2pa::Reader::from_file(path)
  → ManifestStore::active_manifest()
      → Manifest::ingredients()          ← parent manifests (recursive)
          → extract signature_info
              → signer name (cert CN or claim_generator)
              → issuer (cert issuer CN)
              → signed_at (timestamp)
              → trust_status (from validation_status())
              → algorithm
```

Ordered oldest-first for display. Each entry tagged with trust indicator.
Hash consistency check: compare `c2pa.hash.data` hashes across all entries —
if they all match, the file has not been modified at any step.

---

## 10. Algorithm Policy — FIPS + C2PA Intersection

### Venn Diagram

```
        C2PA Allowed          FIPS 140-3 Approved
       ┌──────────────────────────────────────┐
       │  ed25519              ╔══════════════╗│
       │  (C2PA only —         ║  es256       ║│
       │   avoid on FIPS RHEL) ║  es384       ║│
       │                       ║  es512  ←────╬╪── strongest
       │                       ║  ps256       ║│
       │                       ║  ps384       ║│
       │                       ║  ps512  ←────╬╪── strongest RSA
       │                       ╚══════════════╝│
       └──────────────────────────────────────┘
                         ↑
               Use only this intersection
```

### Allowed Algorithms in UMRS

| Algorithm | Type | Strength | FIPS Safe | Recommended |
|-----------|------|----------|-----------|-------------|
| `es256` | ECDSA P-256 | Good | ✅ | Default |
| `es384` | ECDSA P-384 | Strong | ✅ | ✅ |
| `es512` | ECDSA P-521 | Strongest ECDSA | ✅ | ✅ Best |
| `ps256` | RSA-PSS 2048 | Good | ✅ | Acceptable |
| `ps384` | RSA-PSS 3072 | Strong | ✅ | Acceptable |
| `ps512` | RSA-PSS 4096 | Strongest RSA | ✅ | Acceptable |
| `ed25519` | EdDSA | — | ❌ Avoid | ❌ Excluded |

`ed25519` is accepted by the C2PA spec but excluded from UMRS — unreliable on
FIPS-enabled RHEL OpenSSL providers. `inspect config validate` warns if it appears.

---

## 11. OpenSSL Build Matrix

| Target | Feature Flags | OpenSSL Source | FIPS Validated | Network |
|--------|--------------|----------------|----------------|---------|
| Dev / CI / Ubuntu | `vendored-openssl internet` (default) | Bundled OpenSSL 3.x | No | ✅ Yes |
| FIPS RHEL production | `system-openssl internet` | RHEL system library | ✅ Yes | ✅ Yes |
| Air-gap (any platform) | `vendored-openssl` (no `internet`) | Bundled OpenSSL 3.x | No | ❌ None |
| Air-gap FIPS RHEL | `system-openssl` (no `internet`) | RHEL system library | ✅ Yes | ❌ None |

```sh
# Default (vendored OpenSSL + internet/TSA enabled)
cargo build --release

# FIPS RHEL with internet
cargo build --release --no-default-features --features system-openssl,internet

# Air-gap (vendored OpenSSL, no internet, no TSA)
cargo build --release --no-default-features --features vendored-openssl

# Air-gap FIPS RHEL (system OpenSSL, no internet, no TSA)
cargo build --release --no-default-features --features system-openssl
```

**Important:** The vendored build uses FIPS-approved *algorithms* but is not the
NIST-validated *module*. For strict FIPS compliance (government, regulated
industries), use `system-openssl` on RHEL.

**Air-gap behaviour:** When built without the `internet` feature, the `tsa_url`
config field is accepted but ignored at signing time — no outbound connections
are made. `inspect config validate` will emit a warning if `tsa_url` is set,
reminding the operator that timestamps will be unsigned (local clock only).

---

## 12. Logging — systemd-journal-logger

Tag: `umrs` (lowercase)

```rust
// Initialization in main.rs
systemd_journal_logger::init_with_extra_fields(vec![
    ("SYSLOG_IDENTIFIER", "umrs"),
]).expect("Failed to initialize journald logger");
log::set_max_level(config.logging.level.into());
```

If `logging.enabled = false` in config → `log::set_max_level(LevelFilter::Off)`.
All `log::info!()` calls compile to no-ops at zero runtime cost.

### Log Entry Formats

**Case 1 — No prior manifest:**
```
umrs [INFO] ingest file="photo.jpg" sha256="a3f1b2..." manifest=none action=c2pa.acquired
```

**Case 2 — Prior manifest exists:**
```
umrs [INFO] ingest file="photo.jpg" sha256="a3f1b2..." previous_signer="Sony Corporation" signed_at="2025-09-14T08:12:33Z" action=c2pa.published
```

**Config validation:**
```
umrs [INFO] config_validate result=ok path="/etc/umrs/umrs-c2pa.toml"
umrs [WARN] config_validate check="cert_expiry" message="Certificate expires in 14 days"
umrs [ERROR] config_validate check="private_key" message="File not found: /etc/umrs/certs/signing.key"
```

---

## 13. Config Validation — inspect config validate

Runs every check that would otherwise fail silently at runtime.

| Check | Pass | Warn | Fail |
|-------|------|------|------|
| TOML parses without error | ✅ | | ❌ |
| Required fields present | ✅ | | ❌ |
| cert_chain file exists | ✅ | | ❌ |
| private_key file exists | ✅ | | ❌ |
| cert_chain is valid PEM | ✅ | | ❌ |
| private_key is valid PEM | ✅ | | ❌ |
| Private key matches cert | ✅ | | ❌ |
| Cert not yet valid (NotBefore) | ✅ | | ❌ |
| Cert not expired (NotAfter) | ✅ | ⚠️ <30 days | ❌ expired |
| Algorithm in FIPS-safe set | ✅ | ⚠️ ed25519 | |
| TSA URL reachable (if set) | ✅ | ⚠️ timeout | |
| No cert/key → ephemeral mode | | ℹ️ test mode | |

Exit code: `0` = all pass (warnings OK), `1` = one or more failures.

---

## 14. Config Generation — inspect config generate

Writes a fully-commented starter `umrs-c2pa.toml` to stdout or `--output` path.
Every field is present, every field has an inline comment explaining it.
Equivalent to `cargo init` for the config.

```sh
inspect config generate --output /etc/umrs/umrs-c2pa.toml
```

---

## 15. Test Strategy

All tests live in `tests/c2pa_tests.rs`. No inline `#[cfg(test)]` modules.

### Test Cases

| Test | Description |
|------|-------------|
| `test_read_unsigned_file` | Read a plain PNG with no manifest — expect `None` |
| `test_read_signed_file` | Read a fixture JPEG with manifest — expect `Ok(chain)` |
| `test_ingest_unsigned` | Ingest an unsigned file — expect `c2pa.acquired` manifest written |
| `test_ingest_signed` | Ingest a signed file — expect `c2pa.published` + ingredient present |
| `test_chain_hash_consistency` | Walk chain — all hashes match |
| `test_sha256_recorded` | SHA-256 in log entry matches `sha256sum` of file |
| `test_config_load_defaults` | No config file → ephemeral mode, no panic |
| `test_config_load_toml` | Load example TOML → all fields populate correctly |
| `test_validate_missing_key` | Validate config with missing key file → FAIL result |
| `test_algorithm_fips_set` | `ed25519` rejected by signer builder |
| `test_json_output` | `--json` flag emits valid JSON manifest store |

### Test Fixtures

Location: `tests/fixtures/`

Downloaded from `contentauth/c2pa-rs` official test suite:
- Signed JPEG with manifest
- Signed PNG with manifest
- Unsigned JPEG (no manifest)
- Unsigned PNG (no manifest)
- Multi-hop chain JPEG (2+ manifests in store)

---

## 16. Build and Run Instructions

```sh
# Clone and build (default — vendored OpenSSL)
git clone <repo>
cd umrs-c2pa
cargo build --release

# FIPS RHEL build
cargo build --release --no-default-features --features system-openssl

# Run tests
cargo test

# Read a file's chain of custody
./target/release/inspect c2pa tests/fixtures/signed.jpg

# Read as JSON
./target/release/inspect c2pa --json tests/fixtures/signed.jpg

# Ingest (sign) a file
./target/release/inspect c2pa --sign tests/fixtures/unsigned.png

# Validate config
./target/release/inspect config validate --config umrs-c2pa.toml

# Generate starter config
./target/release/inspect config generate --output umrs-c2pa.toml
```

---

## 17. Future Phases

These are **not** in scope for this prototype but are planned:

| Phase | Description |
|-------|-------------|
| SELinux labels | Read SELinux file labels at ingest — `inspect label <FILE>` |
| Apache module | `mod_umrs_c2pa` — inspect C2PA at HTTP request time, C FFI surface |
| TUI config editor | `inspect config edit` — interactive TOML editor with live validation |
| EXIF / XMP inspection | Surface IPTC/XMP metadata alongside C2PA chain |
| Audio/video probing | Symphonia or ffprobe integration for format metadata |
| Trust List integration | Check signer certs against live C2PA Trust List |
| OCSP / CRL checking | Real-time certificate revocation checking per chain entry |

---

## References

- [c2pa-rs Rust SDK](https://github.com/contentauth/c2pa-rs)
- [C2PA Technical Specification 2.2](https://spec.c2pa.org/specifications/specifications/2.2/specs/C2PA_Specification.html)
- [Manifest Examples — CAI Open Source](https://opensource.contentauthenticity.org/docs/manifest/manifest-examples/)
- [Understanding Manifests — CAI Open Source](https://opensource.contentauthenticity.org/docs/manifest/understanding-manifest/)
- [Identity Assertion — Creator Assertions Working Group](https://cawg.io/identity/1.2/)
- [c2pa-attacks Security Test Tool](https://github.com/contentauth/c2pa-attacks)
- [Content Credentials Online Verifier](https://contentcredentials.org/verify)
- [systemd-journal-logger crate](https://crates.io/crates/systemd-journal-logger)
- [OpenSSL crate (Rust)](https://crates.io/crates/openssl)

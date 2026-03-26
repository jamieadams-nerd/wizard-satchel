# IMPLEMENTATION.md — umrs-c2pa

This document describes the design, implementation, and operational behavior of
the `umrs-c2pa` module. It is written for engineers maintaining the code,
security auditors reviewing the cryptographic choices, and evaluators assessing
fitness for integration into a higher-assurance host system.

---

## 1. Overview

`umrs-c2pa` is a Rust library and CLI binary for C2PA (Coalition for Content
Provenance and Authenticity) manifest inspection and ingest signing. It is
part of the UMRS (Unified Mandatory Reference System) project.

The tool serves two purposes:

1. **Inspection** — read an existing C2PA manifest from a media file and display
   the chain of custody: who signed it, when, with what algorithm, and whether
   the certificate chain is trusted.

2. **Ingest signing** — when a file enters a UMRS-controlled boundary, sign it
   with a new C2PA manifest entry that records the ingest event. If the file
   already carries a manifest from a prior system (e.g., a camera or AI
   content platform), the prior chain is preserved as an ingredient and the
   UMRS entry is appended.

The binary is named `inspect`. All business logic lives in the `umrs_c2pa`
library crate. This separation is intentional: the library is designed to be
re-imported by higher-assurance host crates without modification.

---

## 2. Architecture

### Library + thin binary

`main.rs` contains argument parsing and output formatting only. It calls into
`umrs_c2pa::c2pa::*` for all logic. This constraint is enforced by convention,
not by the compiler, but the separation is clean in the current codebase.

### Module layout

```
src/
├── main.rs              CLI: argument parsing, dispatch, output
├── lib.rs               Re-exports pub mod c2pa
└── c2pa/
    ├── mod.rs           Public re-exports for the c2pa submodule
    ├── config.rs        UmrsConfig: TOML deserialization, defaults
    ├── error.rs         InspectError: unified typed error enum
    ├── ingest.rs        ingest_file(), sha256_hex(), IngestResult
    ├── manifest.rs      read_chain(), has_manifest(), manifest_json()
    ├── report.rs        print_chain(), print_validation_report()
    ├── signer.rs        ECDSA ephemeral cert generation, SignerMode
    └── validate.rs      validate_config(), preflight checks
```

### Module responsibilities

| Module | Responsibility |
|---|---|
| `config` | Load and parse `umrs-c2pa.toml`. Provide typed structs with serde defaults. |
| `error` | Single `InspectError` enum covering IO, C2PA SDK, config, signing, and algorithm errors. |
| `ingest` | Orchestrate the ingest pipeline: hash, detect manifest, select action, sign, write output, log. |
| `manifest` | Read manifest store JSON from a file. Walk the ingredient chain. Extract chain entries. |
| `signer` | Parse and validate algorithm selection. Generate ECDSA ephemeral certs. Build `c2pa::BoxedSigner`. |
| `validate` | Preflight checks run before any signing: required fields, cert/key files, key-cert match, algorithm, TSA reachability. |
| `report` | Format and print chain-of-custody reports and config validation results to stdout. |

### Data flow: read-only inspection

```
file
  -> manifest::read_chain()
     -> c2pa::Reader::from_file()
     -> parse manifest store JSON
     -> walk ingredient chain (oldest-first)
     -> Vec<ChainEntry>
  -> report::print_chain_readonly()
```

### Data flow: ingest signing

```
file + config
  -> ingest::sha256_hex()                    SHA-256 of source bytes
  -> manifest::has_manifest()                detect prior manifest
  -> select action: c2pa.acquired or c2pa.published
  -> signer::resolve_signer_mode()           ephemeral or credentials
  -> signer::build_signer()                  BoxedSigner
  -> c2pa::Builder                           construct manifest
     -> set claim_generator_info
     -> add_assertion("c2pa.actions", ...)
     -> add_ingredient (if prior manifest)
  -> builder.sign()                          write signed output file
  -> log::info! to journald
  -> IngestResult
  -> manifest::read_chain() on output file
  -> report::print_chain()
```

---

## 3. Running the Binary

The binary is named `inspect`. Use `cargo run --` to pass arguments during
development.

### Read-only chain display

```
cargo run -- c2pa <FILE>
```

Reads the C2PA manifest store from `<FILE>`, walks the ingredient chain
oldest-first, and prints the chain of custody: signer name, issuer, signing
timestamp, algorithm, and trust status. Also prints the SHA-256 of the file.

### JSON manifest dump

```
cargo run -- c2pa <FILE> --json
```

Emits the full manifest store as pretty-printed JSON. This is the raw output
from `c2pa::Reader::json()` — useful for debugging or piping to `jq`.

### Sign and ingest (default output path)

```
cargo run -- c2pa <FILE> --sign
```

Signs the file and writes the signed copy to `<FILE_STEM>_umrs_signed.<EXT>`
in the same directory. Displays the full chain of custody of the signed output,
including the newly appended UMRS entry.

Example: `image.jpg` produces `image_umrs_signed.jpg`.

### Sign with custom output path

```
cargo run -- c2pa <FILE> --sign --output <PATH>
```

As above, but writes the signed output to `<PATH>`.

### Validate configuration

```
cargo run -- config validate
```

Reads the config file (default: `umrs-c2pa.toml` in the current directory) and
runs preflight checks:

- `claim_generator` is non-empty
- `cert_chain` file exists and is valid PEM (if configured)
- `private_key` file exists and is valid PEM (if configured)
- Private key matches the certificate (if both are configured)
- Algorithm is in the allowed set
- TSA endpoint is reachable (if configured and `internet` feature is enabled)
- Notifies of ephemeral (test) mode if no credentials are configured

Exits 0 if all checks pass, 1 if any check fails.

### Generate starter config

```
cargo run -- config generate
```

Prints a commented config template to stdout.

```
cargo run -- config generate --output umrs-c2pa.toml
```

Writes the template to the specified file.

### Specifying a config file

All subcommands accept `--config <PATH>` (global flag):

```
cargo run -- --config /etc/umrs/umrs-c2pa.toml c2pa <FILE>
```

If the specified file does not exist, `UmrsConfig::default()` is used silently.

### Feature flags for release builds

The default build uses vendored OpenSSL and enables network access. For
production and FIPS deployments, select features explicitly:

```
# FIPS RHEL — link system OpenSSL, allow network
cargo run --release --no-default-features --features system-openssl,internet -- <args>

# Air-gap FIPS RHEL — link system OpenSSL, no network
cargo run --release --no-default-features --features system-openssl -- <args>
```

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | File not found, manifest error, signing failure, config validation failed |
| 2 | Usage error (clap argument parse failure) |

---

## 4. Feature Flags and Build Matrix

### Feature definitions

| Feature | Effect |
|---|---|
| `vendored-openssl` | Statically links a bundled OpenSSL. No system OpenSSL required. Not FIPS-validated. |
| `system-openssl` | Links the host system's OpenSSL. Required for FIPS-validated deployments on RHEL. |
| `internet` | Enables `ureq` for TSA timestamp requests and TSA reachability probing in `config validate`. |

`vendored-openssl` and `internet` are on by default. They are not mutually
exclusive with `system-openssl`, but using both OpenSSL flags together is
unusual and untested.

### Build matrix

| Scenario | Cargo flags | Notes |
|---|---|---|
| Development / non-FIPS | `--features default` (implicit) | Fast compile. Vendored OpenSSL. TSA support. |
| FIPS RHEL + internet | `--no-default-features --features system-openssl,internet` | Links RHEL OpenSSL 3.x FIPS provider. TSA timestamps available. |
| Air-gap vendored | `--no-default-features --features vendored-openssl` | No network code compiled in. Bundled OpenSSL. For isolated development environments. |
| Air-gap FIPS RHEL | `--no-default-features --features system-openssl` | Links RHEL FIPS provider. No network code. For production air-gap deployments. |

When `internet` is absent:

- `ureq` is not compiled in.
- TSA timestamps are silently omitted even if `tsa_url` is set in the config.
- `config validate` will emit a WARN for any configured TSA URL instead of
  probing it.
- The `tsa_url` field in `SignerMode::Credentials` is set to `None` at
  compile time (via `#[cfg(not(feature = "internet"))]`).

The unused variable warning for `tsa_url` in air-gap builds is expected
behavior from the cfg gate. It is not a bug.

---

## 5. Cryptography — Design Decisions

### Algorithm choice

The allowed algorithm set is defined as a compile-time constant in `signer.rs`:

```rust
pub const ALLOWED_ALGORITHMS: &[&str] = &["es256", "es384", "es512", "ps256", "ps384", "ps512"];
```

Any value outside this set causes `parse_algorithm()` to return
`InspectError::UnsafeAlgorithm` before any signing occurs.

The default algorithm is `es256` (ECDSA with P-256, SHA-256). This is the
correct choice for most deployments.

**ES256/ES384/ES512 (ECDSA)** are:
- Defined in FIPS 186-4 and FIPS 186-5
- Mandatory in the C2PA specification — every compliant validator must support
  them
- Available in every generation of FIPS 140-2 and FIPS 140-3 validated OpenSSL
  modules
- Supported by the widest range of deployed C2PA verification tools

**PS256/PS384/PS512 (RSA-PSS)** are supported for environments that require
RSA keys. They are FIPS-approved and C2PA-mandatory. RSA-PSS requires
externally provisioned credentials — the ephemeral cert path generates ECDSA
regardless.

### Why Ed25519 is excluded

Ed25519 is not in the allowed set. This is a deliberate policy decision, not
an oversight.

The reasoning:

1. **FIPS module coverage.** Ed25519 (EdDSA) was added to FIPS 186-5, published
   in 2023. FIPS 140-3 is the module validation standard. A FIPS 140-3
   validation does not automatically include Ed25519 — the specific tested
   module must implement it. Most deployed FIPS-validated OpenSSL builds were
   validated under FIPS 186-4, which does not include EdDSA at all. Attempting
   to use Ed25519 on such a system either silently falls back to a non-FIPS
   code path or fails at runtime. Neither outcome is acceptable.

2. **C2PA optionality.** Ed25519 is explicitly optional in the C2PA
   specification. A conformant C2PA validator is not required to support it.
   Using Ed25519 means signed manifests may be unverifiable by a subset of
   compliant validators. ECDSA algorithms are mandatory — if a validator cannot
   verify ES256, it is non-compliant.

3. **The c2pa crate's EphemeralSigner.** The `c2pa` crate provides a built-in
   `EphemeralSigner` that hardcodes Ed25519. Using it on a FIPS-enabled RHEL
   system causes failures. This is the original reason a custom ephemeral
   signer was written.

4. **RHEL 10 note.** OpenSSL 3.5.5 on RHEL 10 with the Red Hat FIPS provider
   active does include Ed25519 under FIPS 186-5. The exclusion is forward-
   looking: we cannot assume all customer RHEL deployments are on RHEL 10 or
   that their FIPS module was validated against 186-5. ECDSA works on every
   FIPS-enabled RHEL version. The conservative choice is ECDSA.

### Custom ephemeral signer

When no credentials are configured (`cert_chain` and `private_key` are both
absent from the config), `resolve_signer_mode()` returns
`SignerMode::Ephemeral`. The `build_signer()` function then calls
`generate_ephemeral_cert()` to produce a self-signed certificate at runtime.

`generate_ephemeral_cert()` in `signer.rs`:

- Selects the EC curve from the configured algorithm:
  - ES256 → P-256 (`prime256v1`)
  - ES384 → P-384 (`secp384r1`)
  - ES512 → P-521 (`secp521r1`)
  - PS256/PS384/PS512 → P-256 (RSA-PSS algorithms require real keys for
    production; P-256 is used for ephemeral test mode only)
- Generates a fresh EC key pair using OpenSSL
- Builds an X.509 v3 self-signed certificate with the extensions required by
  the c2pa crate's certificate profile validator:

  | Extension | Value | Reason |
  |---|---|---|
  | `BasicConstraints` | CA=false | End-entity cert |
  | `KeyUsage` | `digitalSignature` | Signing use only |
  | `ExtendedKeyUsage` | `emailProtection` | c2pa rejects `anyExtendedKeyUsage` |
  | `SubjectKeyIdentifier` | hash of public key | Required by c2pa profile |
  | `AuthorityKeyIdentifier` | keyid (= SKI for self-signed) | Required by c2pa profile |

- Sets `CN=UMRS ephemeral (test mode — UNTRUSTED)` so the cert is
  identifiable in any validator's output
- Sets validity: now to now+1 day
- Discards the private key after signing — the key exists only in memory during
  the signing call

The private key material is never written to disk in ephemeral mode.

The signed output is standard C2PA. Any C2PA-compliant validator can read and
cryptographically verify the manifest structure. The signature will verify as
valid because the public key is embedded in the manifest. The cert will be
marked UNTRUSTED because it is self-signed and not anchored to any trust list.
This is the correct behavior for test mode.

### Interoperability

ES256 produces the most portable output:

- Mandatory in C2PA 1.x and 2.x specifications
- Verified working against OpenAI/DALL-E images (which use ES256)
- Supported by every OpenSSL version in common use
- Smaller signatures than RS256 with equivalent security

ES384 and ES512 are valid choices for higher assurance requirements. All three
are C2PA-mandatory, so external validators must support them. The algorithm is
embedded in the manifest's `signature_info.alg` field and is readable by
`read_chain()`.

If a downstream validator cannot verify the output, the cause is not the
algorithm — it is a non-compliant validator.

### FIPS context on RHEL 10

When built with `--no-default-features --features system-openssl` on RHEL 10
with the Red Hat FIPS provider active:

- OpenSSL 3.5.5 is used
- FIPS mode is confirmed active (`/proc/sys/crypto/fips_enabled` = 1)
- The FIPS provider includes all ECDSA curves (P-256, P-384, P-521) and
  Ed25519 (FIPS 186-5)
- UMRS selects ECDSA regardless, to maintain portability across FIPS module
  generations

The build does not probe `/proc/sys/crypto/fips_enabled` at runtime. It relies
on OpenSSL's internal FIPS mode, which is activated by the system provider
configuration in `/etc/pki/tls/openssl.cnf`. If FIPS mode is required, it must
be verified at the OS level before running the binary.

---

## 6. Configuration

The config file is TOML, defaulting to `umrs-c2pa.toml` in the working
directory. Pass `--config <PATH>` to use a different location. If the file does
not exist, all defaults apply silently.

Generate a commented starter config:

```
cargo run -- config generate --output umrs-c2pa.toml
```

### Config structure

```toml
[identity]
claim_generator = "UMRS Reference System/1.0"  # embedded in every manifest
organization    = "Your Organization"           # display only, not in manifest
# cert_chain    = "/etc/umrs/certs/signing.pem" # PEM chain, leaf first
# private_key   = "/etc/umrs/certs/signing.key" # PEM private key
algorithm       = "es256"                       # see allowed set

[timestamp]
# tsa_url = "http://timestamp.digicert.com"     # omit for air-gap

[policy]
unsigned_action = "c2pa.acquired"
unsigned_reason = "Received at UMRS trusted ingest dropbox. Origin unknown. No modifications made."
signed_action   = "c2pa.published"
signed_reason   = "Received at UMRS trusted ingest dropbox with existing provenance. No modifications made."

[logging]
enabled = true
level   = "info"   # off | error | warn | info | debug | trace
```

### Runtime behavior by credential mode

| Config state | Signing mode | Manifest trust status |
|---|---|---|
| No `cert_chain`, no `private_key` | Ephemeral ECDSA self-signed cert | UNTRUSTED |
| Both `cert_chain` and `private_key` configured | Production credentials | Depends on cert chain |
| Only one of the two configured | Ephemeral (credentials ignored if incomplete) | UNTRUSTED |

`has_credentials()` returns `true` only if both `cert_chain` and `private_key`
are `Some`. Partial configuration falls through to ephemeral mode.

`config validate` reports the active mode as an INFO entry:

```
  [INFO]  credential_mode: No certificate configured — ephemeral self-signed cert
          will be used (test mode). Manifests will be marked UNTRUSTED by
          external validators.
```

---

## 7. Chain of Custody Model

### Files arriving without a C2PA manifest

UMRS records the ingest event with action `c2pa.acquired`. This action signals
that UMRS received the file but does not claim to be its creator.

The assertion embedded in the manifest:

```json
{
  "actions": [
    {
      "action": "c2pa.acquired",
      "reason": "Received at UMRS trusted ingest dropbox. Origin unknown. No modifications made.",
      "softwareAgent": "<claim_generator from config>"
    }
  ]
}
```

No ingredient is added. The resulting file has a single-entry chain.

Hash consistency reporting in the output:

```
Hash consistency : N/A  — no prior manifest (first signature)
```

### Files arriving with a C2PA manifest

UMRS records the ingest event with action `c2pa.published`. The existing
manifest is embedded as a C2PA ingredient, which preserves the prior chain.

```json
{
  "actions": [
    {
      "action": "c2pa.published",
      "reason": "Received at UMRS trusted ingest dropbox with existing provenance. No modifications made.",
      "softwareAgent": "<claim_generator from config>"
    }
  ]
}
```

`c2pa::Ingredient::from_file()` reads the existing manifest store from the
source file and embeds it in the new manifest. The c2pa SDK includes a hash of
the ingredient's manifest data in the new claim, creating a tamper-evident link.

`read_chain()` walks the ingredient chain recursively (cycle-guarded) and
returns entries oldest-first. The UMRS entry is always last.

Hash consistency reporting:

```
Hash consistency : PASS — file unchanged across all signing events
```

This PASS reflects that the c2pa SDK successfully verified the ingredient
hashes when constructing the new manifest. If the file had been modified
between signing events, `Ingredient::from_file()` would fail or the manifest
would contain a `validation_status` entry with a `mismatch` code.

### Trust status derivation

`derive_trust()` in `manifest.rs` inspects the `validation_status` array in
each manifest's JSON:

| Condition | TrustStatus |
|---|---|
| Any status code contains `"revoked"` | `Revoked` |
| Any code contains `"mismatch"` or `"failed"` | `Invalid` |
| Code `"signingCredential.trusted"` is present | `Trusted` |
| No validation_status array, or no matching codes | `Untrusted` |

The c2pa SDK sets `"signingCredential.trusted"` only when the cert chain
validates against a trust anchor in the C2PA Trust List. Self-signed ephemeral
certs will always resolve to `Untrusted`, not `Invalid`. This is the correct
and expected behavior.

---

## 8. Real-World Validation

The tool was validated against images produced by OpenAI/ChatGPT/DALL-E, which
embed C2PA manifests by default.

### What was confirmed

- OpenAI images contain C2PA manifests signed with ES256 (ECDSA P-256)
- `read_chain()` successfully parses the manifest store
- The chain entry shows: `claim_generator: "ChatGPT"`, SDK version `c2pa_rs
  0.67.1`
- UMRS successfully appended a `c2pa.published` entry to the chain
- The signed output was written and read back correctly
- Hash consistency check: PASS across all signing events

### Read-only inspection of an OpenAI/DALL-E image

This image was generated by ChatGPT and already carries two C2PA manifests
from OpenAI's pipeline:

```
$ cargo run --release --no-default-features --features system-openssl,internet \
    -- c2pa tests/sandbox/jamie_desk.png

Chain of Custody — tests/sandbox/jamie_desk.png
SHA-256: 3b6c04def733ee21d0fef1fa4e594e9a9b9c93132f5bd0a1a1473684a9f41cca
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1   [UNKNOWN]      OpenAI
                      Issuer : OpenAI
                      Alg    : Es256

  2   [UNKNOWN]      OpenAI
                      Issuer : OpenAI
                      Alg    : Es256

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

Both entries are signed with ES256. Trust status is UNKNOWN (not INVALID)
because trust list validation is not yet configured. The chain is structurally
valid.

### JSON manifest dump of the same image

```
$ cargo run --release --no-default-features --features system-openssl,internet \
    -- c2pa tests/sandbox/jamie_desk.png --json | head -30

{
    "active_manifest": "urn:c2pa:6422cc8d-5ce8-4ffe-b9ce-0849e1412c78",
    "manifests": {
        "urn:c2pa:6422cc8d-5ce8-4ffe-b9ce-0849e1412c78": {
            "claim_generator_info": [
                {
                    "name": "ChatGPT",
                    "org.contentauth.c2pa_rs": "0.67.1"
                }
            ],
            "title": "image.png",
            ...
```

This confirms OpenAI's pipeline uses the `c2pa` Rust SDK (version 0.67.1) and
identifies the claim generator as "ChatGPT".

### UMRS ingest signing of the OpenAI image

```
$ cargo run --release --no-default-features --features system-openssl,internet \
    -- c2pa tests/sandbox/jamie_desk.png --sign --output /tmp/jamie_desk_signed.png

Chain of Custody — tests/sandbox/jamie_desk.png
SHA-256: 3b6c04def733ee21d0fef1fa4e594e9a9b9c93132f5bd0a1a1473684a9f41cca
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1   [UNKNOWN]      OpenAI
                      Issuer : OpenAI
                      Alg    : Es256

  2   [UNKNOWN]      OpenAI
                      Issuer : OpenAI
                      Alg    : Es256

  3   [UNKNOWN]      Unknown
                      Issuer : Unknown
                      Alg    : Es256

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Hash consistency : PASS — file unchanged across all signing events
UMRS action      : c2pa.published
UMRS output      : /tmp/jamie_desk_signed.png
UMRS identity    : ephemeral self-signed cert (test mode — UNTRUSTED)
```

Entry 3 is the UMRS ingest record. The tool:

1. Detected the existing OpenAI chain (2 entries)
2. Chose `c2pa.published` because a prior manifest was present
3. Embedded the OpenAI chain as a C2PA ingredient
4. Signed with a fresh ES256 ephemeral certificate
5. Verified hash consistency across all three signing events: **PASS**

### Signing an unsigned file

```
$ cargo run --release --no-default-features --features system-openssl,internet \
    -- c2pa tests/sandbox/wallpaper.jpeg --sign

Chain of Custody — tests/sandbox/wallpaper.jpeg
SHA-256: de053eeb03f30afe55d5812df55da8aaf856173955fec8cf9d4f08f7408fdee2
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1   [UNKNOWN]      Unknown
                      Issuer : Unknown
                      Alg    : Es256

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Hash consistency : N/A  — no prior manifest (first signature)
UMRS action      : c2pa.acquired
UMRS output      : tests/sandbox/wallpaper_umrs_signed.jpeg
UMRS identity    : ephemeral self-signed cert (test mode — UNTRUSTED)
```

No prior manifest, so the action is `c2pa.acquired` and there is a single
chain entry.

---

## 9. Limitations and Known Constraints

**Ephemeral mode is test/eval only.** Manifests produced in ephemeral mode
carry a self-signed certificate with no trust anchor. Any C2PA validator will
mark the UMRS entry UNTRUSTED. This is correct behavior and the CN makes it
visually obvious. Do not use ephemeral mode for production ingest without
understanding that the chain will be marked UNTRUSTED downstream.

**No integration tests.** The `tests/` directory does not yet exist. There are
no automated tests against known-good C2PA files, no round-trip signing tests,
and no regression corpus. This is a prototype. Tests should be added before
production deployment.

**Test corpus is limited.** Validation was done against OpenAI-generated images
only. No camera-generated C2PA images (Sony, Leica, Nikon), Adobe-signed PDFs,
or other real-world signed assets have been tested. The manifest parsing code
may encounter edge cases in other manifest store layouts.

**Trust list integration is not implemented.** The `TrustStatus::Trusted`
variant exists in the type but can only be set by the c2pa SDK's validation
layer when it has access to a trust list. The current build does not configure
a trust list. All manifests resolve to UNTRUSTED or Unknown in practice.

**Logging is journald only.** The `systemd-journal-logger` backend is
unconditional. Running on a system without journald (e.g., macOS, non-systemd
Linux) will fail at startup. macOS support was explicitly removed. The binary
is Linux/systemd only.

**TSA unused-variable warning.** In air-gap builds (no `internet` feature), the
`tsa_url` field in `SignerMode::Credentials` is set to `None` unconditionally
by a `cfg` gate. The compiler emits an unused variable warning for the `tsa_url`
parameter in `resolve_signer_mode()`. This is expected and benign.

**MIME type detection is extension-based.** `mime_for_path()` uses the file
extension to determine the MIME type passed to `c2pa::Builder::sign()`. There
is no magic-byte or libmagic detection. A misnamed file will be signed with an
incorrect MIME type.

**No key rotation or revocation support.** There is no mechanism to rotate
credentials, check OCSP, or manage CRL. These are out of scope for the current
prototype.

---

## 10. Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `c2pa` | 0.78.6 | C2PA manifest read/write — reference Rust SDK. `openssl` and `file_io` features enabled; default features disabled to control OpenSSL linkage. |
| `openssl` | 0.10 | ECDSA key generation and X.509 certificate construction for the ephemeral signer. |
| `openssl-sys` | 0.9 | Low-level OpenSSL bindings. Used to forward the `vendored` feature flag. |
| `thiserror` | 1 | Typed error enum (`InspectError`) in library code. |
| `anyhow` | 1 | Ergonomic error handling in `main.rs` only. |
| `clap` | 4 | CLI argument parsing with `derive` feature. |
| `toml` | 0.8 | TOML config file parsing. |
| `serde` | 1 | Serialization/deserialization for config structs and JSON output. `derive` feature. |
| `serde_json` | 1 | JSON manifest output and assertion construction. |
| `log` | 0.4 | Logging facade. |
| `systemd-journal-logger` | 2 | journald logging backend. Linux/systemd only. |
| `sha2` | 0.10 | SHA-256 file hash at ingest. |
| `hex` | 0.4 | Hex encoding of SHA-256 digests. |
| `chrono` | 0.4 | Timestamp parsing from manifests. `serde` feature for deserialization. |
| `ureq` | 2 | HTTP client for TSA timestamp requests and reachability probing. Optional; gated behind `internet` feature. |

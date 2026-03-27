# c2pa-rs / c2patool — Source Code Research

**Date:** 2026-03-27
**Source:** `/media/psf/repos/c2pa-rs` (Adobe c2pa-rs workspace, version 0.78.6)
**Researcher:** Claude (for Jamie)

---

## 1. License and Attribution

**License:** MIT OR Apache-2.0 (dual-licensed, user's choice)
**Copyright:** © 2020 Adobe. All rights reserved.

**License files at repo root:**
- `LICENSE-MIT`
- `LICENSE-APACHE`

**Declared in `Cargo.toml`:** `license = "MIT OR Apache-2.0"`

### What This Means for Us

Under either license we may:

- **Use the SDK as a dependency** — no issue; we already depend on `c2pa = "0.78.6"`.
- **Copy test images and fixtures** — permitted, but we must:
  - Retain the copyright notice (MIT) or note that changes were made (Apache-2.0).
  - Include a copy of the license with any substantial portion we redistribute.
  - Add an attribution line in any file or NOTICE where we copy fixtures verbatim.
- **Study and reimplement patterns** — fully permitted under both licenses. Clean-room
  reimplementation requires no attribution; copying code does.

**Practical rule:** If we copy a file verbatim (test image, PEM bundle, config), drop a
one-line comment or `ATTRIBUTION.md` entry noting the source and license. If we write
our own code inspired by their patterns, no attribution is needed beyond our existing
`Cargo.toml` dependency.

**CLA note:** Adobe requires a Contributor License Agreement for contributions *to their
repo*. This does not affect our downstream use.

---

## 2. Repository Structure

The c2pa-rs workspace contains six crates:

| Crate | Purpose |
|---|---|
| `sdk/` | Core C2PA Rust library — manifest reading, signing, validation, trust |
| `cli/` | `c2patool` command-line application |
| `c2pa_c_ffi/` | C/FFI bindings for the SDK |
| `macros/` | Procedural macros |
| `export_schema/` | JSON schema export utilities |
| `make_test_images/` | Utility to regenerate test fixtures |

The SDK is the heart — 233KB `builder.rs`, 326KB `store.rs`, 169KB `claim.rs`. It is
a mature, production-grade implementation of C2PA 2.x with CAWG identity extensions.

**MSRV:** Rust 1.88.0+

---

## 3. Trust Lists — Deep Dive

This is the area most relevant to our `NoTrustList` gap. The c2pa-rs trust system is
layered and configurable.

### 3.1 Core Type: `CertificateTrustPolicy`

Located in `sdk/src/crypto/cose/certificate_trust_policy.rs`. Stores:

- `trust_anchor_ders: Vec<Vec<u8>>` — C2PA-official root CA certs (DER)
- `user_trust_anchor_ders: Vec<Vec<u8>>` — user-supplied root CAs (DER)
- `end_entity_cert_set: HashSet<String>` — SHA-256 hashes of allowed leaf certs
- `additional_ekus: HashSet<String>` — allowed Extended Key Usage OIDs

### 3.2 Trust Validation Flow

```
1. End-entity check: Is leaf cert hash in allowlist?
   → Yes: return TrustAnchorType::EndEntity (skip chain walk)

2. EKU filter: Does leaf cert have at least one allowed EKU OID?
   → No: CertificateTrustError::InvalidEku

3. Chain build: leaf → intermediates → root
   Walk upward, verify each signature

4. Anchor match: Does root match a system or user trust anchor?
   → System anchor: TrustAnchorType::System
   → User anchor:   TrustAnchorType::User
   → Neither:       CertificateTrustError::CertificateNotTrusted
```

### 3.3 Default EKU OIDs

Loaded from internal `valid_eku_oids.cfg`:

| OID | Description |
|---|---|
| `1.3.6.1.5.5.7.3.4` | Email Protection |
| `1.3.6.1.5.5.7.3.36` | Document Signing |
| `1.3.6.1.5.5.7.3.8` | Timestamping |
| `1.3.6.1.5.5.7.3.9` | OCSP Signing |
| `1.3.6.1.4.1.311.76.59.1.9` | MS C2PA Signing |
| `1.3.6.1.4.1.62558.2.1` | C2PA Signing Certificate |

### 3.4 Configuration Format (TOML)

```toml
[verify]
verify_trust = true              # enable trust validation
verify_timestamp_trust = true    # validate TSA cert chains too

[trust]
trust_anchors = """
-----BEGIN CERTIFICATE-----
(C2PA official root CAs as PEM)
-----END CERTIFICATE-----
"""

user_anchors = """
-----BEGIN CERTIFICATE-----
(your org's internal root CAs)
-----END CERTIFICATE-----
"""

trust_config = """
// C2PA signing certificate
1.3.6.1.4.1.62558.2.1
// Document signing
1.3.6.1.5.5.7.3.36
"""

allowed_list = """
-----BEGIN CERTIFICATE-----
(specific end-entity certs to always trust)
-----END CERTIFICATE-----
(or base64 SHA-256 hashes, one per line)
"""
```

### 3.5 CLI Exposure

`c2patool trust` subcommand with flags:

| Flag | Env Var | Purpose |
|---|---|---|
| `--trust_anchors <PATH_OR_URL>` | `C2PATOOL_TRUST_ANCHORS` | Root CA PEM bundle |
| `--user_anchors <PATH_OR_URL>` | (none) | User root CAs |
| `--allowed_list <PATH_OR_URL>` | `C2PATOOL_ALLOWED_LIST` | End-entity allowlist |
| `--trust_config <PATH_OR_URL>` | `C2PATOOL_TRUST_CONFIG` | EKU OID file |

Supports both local file paths and HTTP/HTTPS URLs for remote trust lists.

### 3.6 Passthrough Mode

`CertificateTrustPolicy::passthrough()` skips all validation, returns
`TrustAnchorType::NoCheck`. Used for testing/development only.

### 3.7 What We Need to Do

The c2pa SDK already performs chain validation internally — we just never feed it trust
config. The path forward is:

1. Add trust fields to `UmrsConfig` (PEM paths, EKU file path, allowed list path)
2. Build a c2pa `Settings` TOML string from those fields
3. Pass it to `Context::new()` or inject via the SDK's settings API before reading
4. Our `derive_trust()` in `manifest.rs` will then see real `signingCredential.trusted`
   codes instead of always landing on `NoTrustList`

This is **configuration plumbing**, not reimplementation. The hard crypto is in the SDK.

---

## 4. Validation System

### 4.1 Three-Tier Validation State

The SDK defines three validation outcomes:

| State | Meaning |
|---|---|
| **Valid** | Signature cryptographically correct, cert was valid at signing time |
| **Trusted** | Valid AND signer cert chains to a trust anchor |
| **Invalid** | Any structural or cryptographic failure |

### 4.2 Status Codes (Comprehensive)

**Success codes:**
- `claimSignature.validated` — signature crypto checks pass
- `claimSignature.insideValidity` — cert was valid at signing time
- `signingCredential.trusted` — cert chains to trust anchor
- `signingCredential.ocsp.notRevoked` — OCSP confirms good standing
- `assertion.dataHash.match` — file content hash matches manifest
- `assertion.hashedURI.match` — assertion hash matches declaration
- `ingredient.manifest.validated` — parent manifest hash verified
- `timeStamp.validated` — RFC 3161 timestamp is well-formed and valid
- `timeStamp.trusted` — TSA cert is in trust list

**Failure codes:**
- `claimSignature.mismatch` — signature does not verify
- `claimSignature.outsideValidity` — signed outside cert validity window
- `signingCredential.expired` — cert has expired
- `signingCredential.revoked` — OCSP indicates revoked
- `signingCredential.untrusted` — cert not in any trust list
- `assertion.dataHash.mismatch` — file has been tampered with
- `assertion.undeclared` — assertion present but not declared in claim
- `ingredient.manifest.mismatch` — parent manifest has been altered
- `timeStamp.mismatch` — timestamp digest doesn't match content
- `timeStamp.malformed` — invalid timestamp structure

**Informational codes:**
- `signingCredential.ocsp.skipped` — OCSP checking disabled
- `signingCredential.ocsp.inaccessible` — couldn't reach OCSP responder
- `ingredient.unknownProvenance` — parent has no C2PA manifest

### 4.3 Error Behavior Modes

```rust
pub enum ErrorBehavior {
    StopOnFirstError,       // Hard validation — fail fast
    ContinueWhenPossible,   // Soft validation — collect all errors (default)
}
```

Soft mode is the default, allowing comprehensive error reports even when multiple
validations fail.

### 4.4 Verification Settings

| Setting | Default | Purpose |
|---|---|---|
| `verify_after_reading` | true | Validate when reading an asset |
| `verify_after_sign` | false | Validate after signing (catches signer bugs) |
| `verify_trust` | true | Check cert against trust lists |
| `verify_timestamp_trust` | true | Verify TSA certificate chains |
| `ocsp_fetch` | false | Fetch live OCSP status |
| `remote_manifest_fetch` | true | Fetch remote manifests |
| `strict_v1_validation` | false | Apply strict C2PA v1 rules |

### 4.5 OCSP Revocation Checking

Located in `sdk/src/crypto/ocsp/mod.rs`. Checks:

1. OCSP staple embedded in the COSE signature
2. If `ocsp_fetch` enabled, live fetch from the OCSP responder URL in the cert
3. Falls back to `CertificateStatus` assertions if available

States: `Good` (with time window check), `Revoked` (with reason code), `Unknown`.

Non-fatal: invalid OCSP data is treated as absent per spec.

### 4.6 Timestamp Validation

Located in `sdk/src/crypto/time_stamp/verify.rs`. Validates:

1. TimeStampResponse and SignedData structure parsing
2. Message imprint verification (digest matches signed content)
3. TSA certificate signature verification (PKIX)
4. TSA certificate validity period check
5. TSA certificate EKU check (OID `1.3.6.1.5.5.7.3.8`)
6. Optional TSA cert chain trust validation

Timestamp failures are logged as informational, not hard failures.

---

## 5. Assertion Types

The SDK implements 20+ assertion types in `sdk/src/assertions/`. Beyond what we
currently parse (`c2pa.actions`, `umrs.security-label`):

### 5.1 Hard Bindings (Non-Redactable)

| Label | Type | Purpose |
|---|---|---|
| `c2pa.hash.data` | DataHash | Full-file hash with exclusion ranges |
| `c2pa.hash.bmff` | BmffHash | BMFF-specific hash with Merkle tree (video) |
| `c2pa.hash.boxes` | BoxHash | Box/chunk-based hash |
| `c2pa.hash.collection.data` | CollectionHash | Hash for multi-file collections |

### 5.2 Actions

| Label | Purpose |
|---|---|
| `c2pa.actions` / `c2pa.actions.v2` | Edit history (created, opened, edited, color_adjustments, converted, etc.) |

25+ action types defined, including `c2pa.created`, `c2pa.opened`, `c2pa.edited`,
`c2pa.color_adjustments`, `c2pa.converted`, and more.

### 5.3 Metadata and Identity

| Label | Purpose |
|---|---|
| `c2pa.metadata` | Hardware/software-generated metadata |
| `cawg.metadata` | Human-generated metadata (CAWG extension) |
| `stds.exif` | EXIF photo metadata |
| `stds.iptc.photo-metadata` | IPTC photo metadata |
| `schema.org.CreativeWork` | Schema.org creative work data |
| `schema.org.ClaimReview` | Fact-check / claim review data |

### 5.4 Thumbnails and Visual

| Label | Purpose |
|---|---|
| `c2pa.thumbnail.claim` | Thumbnail of the claimed asset |
| `c2pa.thumbnail.ingredient` | Thumbnail of an ingredient |
| `c2pa.icon` | Icon for the claim generator |
| `c2pa.depthmap` / `c2pa.depthmap.GDepth` | Depth map data |

### 5.5 Other

| Label | Purpose |
|---|---|
| `c2pa.soft-binding` | Soft bindings (perceptual hash, watermark) |
| `c2pa.cloud-data` | Cloud-hosted supplemental data |
| `c2pa.embedded-data` | Raw embedded binary data |
| `c2pa.time-stamp` | RFC 3161 timestamp tokens |
| `c2pa.certificate-status` | Certificate status (OCSP response) |
| `c2pa.asset-ref` | Cross-asset references |
| Custom assertions | Any reverse-domain-format label (e.g., `umrs.security-label`) |

### 5.6 Assertion Versioning

Labels support version suffix `.v{N}` and instance suffix `__{N}`:
- `c2pa.ingredient.v2` — version 2 of ingredient assertion
- `c2pa.ingredient__1` — second instance
- `c2pa.ingredient.v2__3` — version 2, fourth instance

---

## 6. Builder and Signer APIs

### 6.1 Builder

The `Builder` type (`sdk/src/builder.rs`) supports three intents:

| Intent | Use Case | Requirements |
|---|---|---|
| `Create(DigitalSourceType)` | New asset creation | Must specify source type |
| `Edit` | Editing existing asset | Must have parent ingredient |
| `Update` | Metadata-only update | Restricted actions, no data hash |

**DigitalSourceType** (subset):
- `DigitalCapture`, `ComputationalCapture`, `DigitalCreation`
- `TrainedAlgorithmicMedia`, `AlgorithmicMedia`
- `Composite`, `CompositeCapture`, `CompositeSynthetic`
- `ScreenCapture`, `VirtualRecording`

**Key methods:**
- `from_json()` / `with_definition()` — configure from manifest definition
- `set_intent()` — set the builder intent
- `add_assertion()` / `add_assertion_json()` — add typed or JSON assertions
- `add_ingredient_from_stream()` — add parent asset
- `sign()` / `sign_async()` / `sign_file()` — sign and embed manifest
- `to_archive()` / `from_archive()` — serialize/deserialize builder state

### 6.2 Signer Trait

```rust
pub trait Signer {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn alg(&self) -> SigningAlg;
    fn certs(&self) -> Result<Vec<Vec<u8>>>;  // DER cert chain
    fn reserve_size(&self) -> usize;
    fn time_authority_url(&self) -> Option<String>;
    fn ocsp_val(&self) -> Option<Vec<u8>>;
}
```

Also: `AsyncSigner` for async contexts, `RawSigner` for low-level signing,
`DynamicAssertion` for signer-injected assertions.

### 6.3 Context API

Thread-safe `Arc<Context>` wraps settings + HTTP resolvers + signer config. Allows
sharing configuration across Builder and Reader instances. Settings can come from
JSON or TOML strings.

---

## 7. CLI Features (c2patool)

### 7.1 Features We Don't Have Yet

| Feature | c2patool Flag | Description |
|---|---|---|
| **Tree view** | `--tree` | Visual manifest store hierarchy |
| **Certificate extraction** | `--certs` | Export the signer's certificate chain |
| **Ingredient extraction** | `--ingredient` | Create reusable parent ingredient definition |
| **Detailed report** | `--detailed` | Low-level C2PA manifest dump |
| **Info report** | `--info` | Lightweight manifest overview |
| **External manifest** | `--external-manifest` | Validate sidecar `.c2pa` files |
| **Custom signer** | `--signer-path` | External signer executable (HSM/KMS) |
| **Fragment signing** | `fragment` subcommand | DASH/fragmented MP4 support |
| **Remote manifest** | `--remote` | Fetch manifest from URL |
| **Trust management** | `trust` subcommand | Full trust list configuration |

### 7.2 Configuration File

c2patool supports a TOML settings file (default `$XDG_CONFIG_HOME/c2pa/c2pa.toml`)
with environment variable override `C2PATOOL_SETTINGS`. This is more sophisticated
than our single `umrs.toml`.

---

## 8. Test Fixtures — Inventory

### 8.1 Test Images (Key Ones)

**Valid C2PA manifests:**

| File | Size | Scenario |
|---|---|---|
| `CA.jpg` | 163K | Valid chain with agent ingredient (baseline "good") |
| `C.jpg` | 130K | Valid nested chain |
| `CACA.jpg` | 274K | Double-signed chain (multi-manifest traversal) |
| `CIE-sig-CA.jpg` | 347K | Complex embedding with multiple signers |
| `CACAE-uri-CA.jpg` | 380K | Chain with URI-based manifest references |
| `ocsp.jpg` | 279K | Valid manifest with OCSP certificate validation |
| `cloud.jpg` | 180K | Remote manifest URL reference |
| `C_with_CAWG_data.jpg` | 137K | CAWG identity validation data |

**Invalid / tampered:**

| File | Size | Scenario |
|---|---|---|
| `XCA.jpg` | 140K | **Intentional tamper** — data hash mismatch |
| `CA_ct.jpg` | 163K | Malformed timestamp |
| `no_manifest.jpg` | 96K | Clean baseline, no C2PA data |
| `no_alg.jpg` | 85K | Missing algorithm specification |

**Legacy formats:**

| File | Size | Scenario |
|---|---|---|
| `adobe-20220124-E-clm-CAICAI.jpg` | 641K | Adobe legacy double-agent chain |
| `legacy.mp4` | 812K | Legacy MP4 signing format |
| `legacy_ingredient_hash.jpg` | 371K | Legacy ingredient hash reference |
| `prerelease.jpg` | 367K | Pre-release version test image |

**Other formats:**

| File | Size | Scenario |
|---|---|---|
| `sample1.png` | 293K | PNG with C2PA |
| `video1.mp4` | 810K | MP4 with manifest |
| `video1_no_manifest.mp4` | 780K | MP4 without manifest |
| `sample1.webp` | 50K | WebP format |
| `sample1.avif` | 96K | AVIF format |
| Basic PDF variants | 9-74K | PDF with/without signatures |

### 8.2 Test Certificates

Located in `sdk/tests/fixtures/certs/`:

**Algorithm coverage (key pairs + root certs):**
- ES256 (P-256), ES384 (P-384), ES512 (P-521)
- PS256, PS384, PS512 (RSA-PSS)
- RS256 (RSA PKCS#1)
- ED25519

**Trust config fixtures** (`certs/trust/`):
- `store.cfg` — EKU OID configuration (6 OIDs)
- `test_cert_root_bundle.pem` — root CA bundle for tests
- `allowed_list.pem` — end-entity cert allowlist
- `test_settings.toml` — complete trust configuration example

**OCSP fixtures** (`crypto/ocsp/`):
- `response_good.der` — valid OCSP response
- `response_revoked.der` — revoked certificate response
- `response_unknown.der` — unknown status response

### 8.3 Known-Good JSON Outputs

Located in `sdk/tests/known_good/`:
- `CA.json` — expected Reader output for CA.jpg
- `C.json` — expected Reader output for C.jpg
- `XCA.json` — expected output for XCA.jpg (includes validation failures)
- `CA_test.json` — expected output after signing/editing CA.jpg

These enable **regression testing** — read a fixture, compare output to known-good JSON.

### 8.4 Test Settings Files

- `test_settings.json` (20K) — JSON format settings
- `test_settings.toml` (26K) — TOML format settings
- `test_settings_with_cawg_signing.toml` (30K) — CAWG identity settings

### 8.5 `make_test_images` Crate

The workspace includes a utility crate (`make_test_images/`) that can regenerate test
fixtures programmatically. This means the test images are **reproducible** — they're
built from known certificates and configurations, not opaque binaries.

---

## 9. Areas to Explore Further

### 9.1 Context/Settings API Integration

The SDK's `Context` type (`sdk/src/context.rs`, 50KB) is the recommended way to pass
configuration — including trust settings — into both `Reader` and `Builder`. We should
study how to construct a `Context` from our `UmrsConfig` and pass it through.

**Key question:** Can we build a `Context` with trust config and pass it to
`Reader::from_file()` to get real trust validation without any changes to our
validation code?

### 9.2 Embeddable API

The SDK has a low-level "embeddable API" for custom I/O workflows:
- `Builder::placeholder()` — pre-size manifest for embedding
- `Builder::update_hash_from_stream()` — compute hash after embedding
- `Builder::sign_embeddable()` — sign with explicit embedding control

This could be relevant for our future Apache module (`mod_media_inspect`) where we
need to read manifests from HTTP streams rather than files.

### 9.3 CAWG Identity Assertions

The SDK supports CAWG (Creator Assertions Working Group) identity assertions — a newer
extension to C2PA for binding real-world identities to claims. This includes:
- X.509 certificate-based identity
- W3C Verifiable Credentials
- CAWG-specific trust settings (`[cawg_trust]` in TOML)

Not immediately needed, but relevant for UMRS identity verification scenarios.

### 9.4 `make_test_images` Reproduction

We should investigate whether we can use or adapt the `make_test_images` crate to
generate our own test fixtures with known properties, rather than depending solely on
copied images.

### 9.5 Sidecar Manifest Support

The SDK supports `.c2pa` sidecar files — external manifest stores that reference an
asset without embedding in it. This is useful for formats that don't support JUMBF
embedding (e.g., some PDFs, raw formats). We don't support this yet.

---

## 10. Summary of Gaps (Our Code vs. c2pa-rs)

| Capability | c2pa-rs | umrs-c2pa | Gap |
|---|---|---|---|
| Trust list loading | Full (system + user + allowlist + EKU) | None (`NoTrustList`) | **Critical** |
| Trust validation | Active chain walk | Passive (SDK code decoding only) | **High** |
| OCSP revocation | Staple + live fetch | None | Medium |
| Timestamp validation | Full RFC 3161 | Embed only, no verify | Medium |
| Validation reporting | 30+ status codes, 3 tiers | Single `TrustStatus` enum | Medium |
| Assertion extraction | 20+ types | 2 types (actions, security-label) | Low |
| Format support | JPEG, PNG, MP4, WebP, AVIF, PDF, etc. | JPEG, PNG | Low |
| Test fixtures | 140+ files, all scenarios | 25 files, basic scenarios | **High** |
| Tree view | `--tree` flag | None | Nice-to-have |
| Certificate export | `--certs` flag | None | Nice-to-have |
| Sidecar manifests | Full support | None | Future |
| Custom signer (HSM/KMS) | `--signer-path` | None | Future |

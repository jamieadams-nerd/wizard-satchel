# umrs-c2pa Improvement Plan

**Date:** 2026-03-27
**Based on:** [c2patool Research](c2patool-research.md)
**Status:** Draft — for discussion

---

## Overview

After scanning the c2pa-rs source, tests, and documentation, this plan identifies
concrete improvements to umrs-c2pa organized into four phases. Each phase builds on
the previous one and delivers standalone value.

The guiding principle: **the c2pa SDK already implements the hard parts** (chain
validation, OCSP, timestamps, hash verification). Most of our gaps are configuration
plumbing — wiring our config into the SDK's `Settings`/`Context` API so it can do its
job.

---

## Phase 1: Test Fixture Expansion

**Goal:** Broaden test coverage using real C2PA test images from c2pa-rs.
**Effort:** Small (file copy + new tests)
**Dependencies:** None

### 1.1 Reference c2pa-rs Test Fixtures

**Per D2:** Reference test images directly from the c2pa-rs source tree rather than
copying them into our repo. Tests use a configurable path (env var `C2PA_FIXTURES_DIR`,
defaulting to `../c2pa-rs/sdk/tests/fixtures` or similar). Tests skip gracefully if
the directory is absent (same pattern we already use).

**Key images to reference** from `c2pa-rs/sdk/tests/fixtures/`:

| File | Why |
|---|---|
| `CA.jpg` | Baseline valid chain with agent ingredient |
| `C.jpg` | Valid nested chain |
| `XCA.jpg` | **Tampered image** — data hash mismatch (critical for validation tests) |
| `no_manifest.jpg` | Clean baseline, no C2PA |
| `CA_ct.jpg` | Malformed timestamp |
| `CACA.jpg` | Double-signed chain |
| `sample1.png` | PNG with C2PA manifest |
| `ocsp.jpg` | OCSP validation data embedded |

**Trust config fixtures** from `sdk/tests/fixtures/certs/trust/`:

| File | Why |
|---|---|
| `store.cfg` | EKU OID configuration example |
| `test_cert_root_bundle.pem` | Root CA bundle for trust tests |
| `allowed_list.pem` | End-entity allowlist example |

### 1.2 New Integration Tests

Write tests using the copied fixtures:

- **`test_read_valid_chain`** — read `CA.jpg`, verify chain has expected entries
- **`test_read_nested_chain`** — read `C.jpg`, verify nested ingredients
- **`test_read_tampered_image`** — read `XCA.jpg`, verify trust status is `Invalid`
- **`test_read_no_manifest`** — read `no_manifest.jpg`, verify graceful handling
- **`test_read_double_signed`** — read `CACA.jpg`, verify multi-manifest traversal
- **`test_read_malformed_timestamp`** — read `CA_ct.jpg`, check timestamp handling
- **`test_png_with_manifest`** — read `sample1.png`, verify PNG C2PA support

### 1.3 Known-Good Regression Tests

Adopt the c2pa-rs pattern of comparing Reader output to known-good JSON:

1. Read each fixture with our `read_chain()` function
2. Serialize the result to JSON
3. Compare against a checked-in `tests/known_good/<name>.json` file
4. Any change in output requires explicit update to the known-good file

This catches regressions when we change chain parsing or trust evaluation logic.

---

## Phase 2: Trust List Support

**Goal:** Enable real trust validation by wiring trust config into the c2pa SDK.
**Effort:** Medium (config changes + SDK integration)
**Dependencies:** Phase 1 (test fixtures for validation)

### 2.1 Extend UmrsConfig

**Per D1:** Ship a PEM file, operator updates manually. No fetch logic.

Add a new `[trust]` section to `UmrsConfig`:

```toml
[trust]
# All paths are fully configurable — no hardcoded default location.
# Operator chooses where to store trust material for their deployment.

# Path to PEM bundle of root CA certificates.
# Operator updates this file manually (works air-gapped and connected).
# See docs/trust-maintenance.md for update instructions.
trust_anchors = "/path/to/your/trust/c2pa-anchors.pem"

# Path to PEM bundle of additional user/org root CAs
user_anchors = "/path/to/your/trust/org-roots.pem"

# Path to end-entity certificate allowlist (PEM or base64 SHA-256 hashes)
allowed_list = "/path/to/your/trust/allowed-signers.pem"

# Path to EKU OID configuration file (one OID per line)
trust_config = "/path/to/your/trust/ekus.cfg"

# Enable trust validation (default: true when any trust file is configured)
verify_trust = true

# OCSP responder URL (skeleton — not fully implemented yet, per D4)
# Organizations can point this to their own OCSP server when ready.
# ocsp_responder = "http://ocsp.internal.example.com"
```

### 2.2 Build SDK Settings from UmrsConfig

Create a function that constructs a c2pa `Settings` TOML string from our trust config:

```rust
/// Build a c2pa settings TOML string from UmrsConfig trust settings.
pub fn build_c2pa_settings(config: &UmrsConfig) -> Result<String, InspectError> {
    // Read PEM files, construct TOML with [trust] and [verify] sections
    // Return string suitable for Context::new() or Settings::from_str()
}
```

### 2.3 Inject Settings into Reader

Modify `read_chain()` (and the ingest pipeline) to:

1. Load trust config from `UmrsConfig`
2. Build c2pa `Settings` TOML
3. Create a `Context` with those settings
4. Pass the `Context` to `Reader::from_file()` (or equivalent)

This should cause the SDK to perform real chain validation, and our existing
`derive_trust()` function will then see `signingCredential.trusted` (or `.untrusted`)
codes instead of never matching a trust list.

### 2.4 Config Validation Updates

Extend `validate_config()` to check:

- Trust anchor file exists and is valid PEM
- User anchor file exists and is valid PEM (if configured)
- Allowed list file exists (if configured)
- EKU config file is parseable (if configured)
- Warn if no trust config is set (all manifests will be `Untrusted`)

### 2.5 Trust List Tests

Using the copied trust fixtures:

- **`test_trust_with_anchors`** — configure trust anchors, read `CA.jpg`, verify
  trust status is `Trusted` (not `NoTrustList`)
- **`test_trust_without_anchors`** — no trust config, verify `Untrusted`
- **`test_trust_with_allowlist`** — configure end-entity allowlist, verify trusted
- **`test_invalid_trust_config`** — bad PEM file, verify config validation catches it

---

## Phase 3: Validation Enrichment

**Goal:** Surface richer validation information to the user.
**Effort:** Medium (new types + report formatting)
**Dependencies:** Phase 2 (trust validation generates real status codes)

### 3.1 Structured Validation Report

Replace the single `TrustStatus` with a richer structure:

```rust
/// Full validation result for a manifest.
pub struct ValidationReport {
    /// Overall state: Valid, Trusted, or Invalid
    pub state: ValidationState,

    /// Successful validation checks
    pub success: Vec<StatusEntry>,

    /// Informational notes (non-failures)
    pub informational: Vec<StatusEntry>,

    /// Validation failures
    pub failures: Vec<StatusEntry>,
}

pub struct StatusEntry {
    pub code: String,           // e.g., "claimSignature.validated"
    pub description: String,    // human-readable explanation
    pub assertion_url: Option<String>,  // which assertion, if applicable
}

pub enum ValidationState {
    Trusted,
    Valid,
    Invalid,
}
```

### 3.2 Enhanced Report Output

Update `print_chain()` to optionally display:

- All validation codes grouped by success/info/failure
- Specific failure reasons (not just "Invalid" but *why*)
- Timestamp validation details
- OCSP status if available

Add a `--validation-detail` flag (or `--verbose`) to the CLI.

### 3.3 JSON Validation Output

Extend `--json` output to include structured validation data:

```json
{
  "chain": [...],
  "validation": {
    "state": "valid",
    "success": ["claimSignature.validated", "assertion.dataHash.match"],
    "informational": ["signingCredential.ocsp.skipped"],
    "failures": ["signingCredential.untrusted"]
  }
}
```

### 3.4 Assertion Extraction

**Per D5:** Stay minimal — actions + security-label only. No additional assertion
types unless they become directly relevant to chain-of-custody tracking.

---

## Phase 4: CLI Enhancements and Polish

**Goal:** Feature parity with c2patool's most useful inspection features.
**Effort:** Small-to-medium per feature
**Dependencies:** Phase 3

### 4.1 Tree View

Add `--tree` flag to display manifest store hierarchy:

```
Manifest Store
├── urn:uuid:abc123 (active)
│   ├── Signer: Adobe Photoshop
│   ├── Algorithm: ES256
│   └── Ingredients:
│       └── urn:uuid:def456
│           ├── Signer: Canon EOS R5
│           └── Algorithm: ES256
```

### 4.2 Certificate Extraction

Add `inspect c2pa --certs <FILE>` to export the signer's certificate chain as PEM.
Useful for building allowlists and debugging trust issues.

### 4.3 Detailed / Info Modes

- `--detailed` — raw C2PA manifest dump (all assertions, all fields)
- `--info` — one-line summary (has manifest? signed? trusted? format?)

### 4.4 Verify-After-Sign

Add `verify_after_sign = true` option to `UmrsConfig`. After signing a file, re-read
it and validate the manifest to catch signing bugs. c2pa-rs supports this natively.

---

## Decisions (Resolved 2026-03-27)

### D1: Trust Anchor Source — Ship a file, manual updates only

**Decision:** Ship the C2PA official root CA list as a PEM file. The path is fully
configurable in `umrs.toml` — no hardcoded default location. Operators update the
file manually — no auto-fetch code. Works identically for air-gapped and connected
environments. See `docs/trust-maintenance.md` for update procedures.

**License note:** The c2pa-rs project (and its trust anchors) is MIT OR Apache-2.0
(Adobe copyright). We are free to distribute the official CA list. Test images and
copied fixtures require a one-line attribution note.

### D2: Which Test Images — Reference c2pa-rs directory, don't copy into repo

**Decision:** Reference the critical test images directly from the c2pa-rs source
directory in our test code (configurable path, e.g., `C2PA_FIXTURES_DIR` env var).
Images do not need to live in our repo. Focus on the ones that improve our validation
testing: `CA.jpg`, `XCA.jpg`, `no_manifest.jpg`, `CACA.jpg`, `CA_ct.jpg`,
`sample1.png`, `ocsp.jpg`. Add more formats as we add support.

### D3: Known-Good JSON — Field-level assertions, not snapshot comparison

**Decision:** Use field-level assertions (trust status, chain length, signer name,
specific validation codes) rather than full JSON snapshot comparison. More resilient
to output format changes. Revisit snapshots later if the format stabilizes.

### D4: OCSP — Skeleton only, defer full implementation

**Decision:** Keep OCSP as a configurable skeleton in the code (config field for
OCSP responder URL, plumbing in place) but do not implement live fetching yet. When
ready, support pointing to an organization's own OCSP server — not just public
responders. Decide later based on demand.

### D5: Assertion Extraction — Minimal, chain-of-custody focus

**Decision:** Stay minimal — actions + security-label. We are tracking chain of
custody, not doing forensics. If a specific assertion type becomes relevant to the
custody story, add it then.

---

## Phase Summary

| Phase | Scope | Effort | Value |
|---|---|---|---|
| **1: Test Fixtures** | Copy images, write tests, attribution | Small | High — immediate test coverage |
| **2: Trust Lists** | Config, SDK integration, validation | Medium | **Critical** — closes biggest gap |
| **3: Validation** | Rich reporting, assertion extraction | Medium | High — better user experience |
| **4: CLI Polish** | Tree view, certs, detail modes | Small each | Medium — feature parity |

Phases 1 and 2 are the priority. Phase 3 naturally follows once trust validation
produces real status codes. Phase 4 is incremental polish.

---

## Non-Goals (Explicitly Out of Scope)

- **Reimplementing chain validation** — the SDK does this; we configure it.
- **CAWG identity assertions** — interesting but not needed for CUI inspection.
- **Fragment/DASH support** — streaming media is not in our use case.
- **C FFI bindings** — we're Rust-native; FFI is for the Apache module phase.
- **WASM support** — not relevant for RHEL server deployments.

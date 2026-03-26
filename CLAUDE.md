# CLAUDE.md — Media Inspection Toolkit

## Project Identity

Fast-prototype repository for media file inspection tooling. Output is intended
for eventual integration into a higher-assurance host project. Code is written as
**library + thin binary** so every capability is importable as a crate later.

Primary near-term focus: C2PA (Coalition for Content Provenance and Authenticity)
manifest inspection and signing. Later phases will include Apache module integration
for reading SELinux file labels from media at ingest.

---

## Repository Layout

```
umrs-c2pa/
├── Cargo.toml
├── docs/                     # all documentation lives here
│   ├── IMPLEMENTATION.md     # design, crypto decisions, usage examples
│   └── ...
├── src/
│   ├── main.rs               # CLI only: arg parsing, dispatch, output
│   ├── lib.rs                # re-exports pub mod c2pa
│   └── c2pa/
│       ├── mod.rs            # public re-exports
│       ├── config.rs         # UmrsConfig TOML loading
│       ├── error.rs          # InspectError (thiserror)
│       ├── signer.rs         # ECDSA ephemeral cert gen, SignerMode
│       ├── ingest.rs         # ingest pipeline, SHA-256
│       ├── manifest.rs       # chain reading, JSON walking
│       ├── report.rs         # formatted output
│       └── validate.rs       # config preflight checks
├── tests/
│   ├── sandbox/              # writable copies of test images
│   └── fixtures/             # static test files
└── jamie_images/             # read-only source images (not committed)
```

**Rule:** `main.rs` contains argument parsing and output formatting only.
All logic must be in `umrs_c2pa::c2pa::*`. No business logic in `main.rs`.

**Rule:** Documentation (*.md) goes under `docs/` inside the crate.

---

## Methodology — Report, Plan, Build, Document

The rapid-fire workflow for this repo:

1. **Report** — research the problem space. Survey crates, APIs, constraints.
   Produce a short capability summary for human review.
2. **Plan** — discuss the approach with Jamie. Agree on what to build.
3. **Implement** — write the code after confirmation. Run clippy pedantic.
4. **Document** — write up what was built, why, and how it works. Real terminal
   output in the docs, not placeholders. Docs go under `docs/` in the crate.

This is a prototyping repo. Jamie and Claude build capabilities here; the team
integrates the results into the larger UMRS project. The write-up is how the
team understands what was done and why.

This applies especially to new crates (c2pa, xmp-toolkit, lopdf, symphonia,
ffprobe bindings, etc.). Do not assume a crate is absent; check `Cargo.toml`
and `Cargo.lock` first.

---

## Coding Standards

### General

- Prototype quality: readable, functional, not gold-plated.
- Library API should be clean enough to re-export without embarrassment.
- Prefer `thiserror` for error types in the library; `anyhow` is acceptable in
  the binary only.
- No `unwrap()` or `expect()` in library code. In binary code, `expect()` with a
  clear message is acceptable during prototyping.
- No `unsafe` without a comment explaining why and what invariant is upheld.

### Structure

- Every public function, struct, and enum in the library gets a doc comment.
- Internal helpers do not require doc comments but benefit from a one-liner.
- Feature flags in `Cargo.toml` gate optional heavy dependencies (e.g., `c2pa`,
  `ffmpeg`). Default features should keep the compile fast.

### Error Handling

```rust
// Library: typed errors via thiserror
#[derive(Debug, thiserror::Error)]
pub enum InspectError {
    #[error("C2PA manifest not found in file: {0}")]
    ManifestNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Binary: anyhow is fine
fn main() -> anyhow::Result<()> { ... }
```

### Tests

- **No inline tests.** All tests live under `tests/` as integration tests or in a
  `tests/` directory alongside the crate.
- Test file naming: `tests/<module_name>_tests.rs`
- Use `#[test]` and standard `assert_eq!` / `assert!`; no test framework required
  unless complexity demands it.

---

## C2PA Guidance

C2PA (Coalition for Content Provenance and Authenticity) is an open technical
standard for attaching tamper-evident provenance metadata to media files.

### Core Concepts

| Concept | Description |
|---|---|
| **Manifest** | Top-level container for all provenance data in a file |
| **Claim** | Asserts who created/modified the asset and when |
| **Assertion** | Individual metadata claim (e.g., `c2pa.actions`, `c2pa.hash.data`) |
| **Signature** | Cryptographic binding of the claim to a certificate (X.509) |
| **Ingredient** | A referenced parent asset embedded in a manifest |
| **JUMBF** | The binary embedding format used inside JPEG/MP4/etc. |
| **Trust List** | Set of known root CAs used to validate signatures |

### Rust Crate

The [`c2pa`](https://crates.io/crates/c2pa) crate is the reference Rust SDK.

```toml
[dependencies]
c2pa = { version = "0.78.6", default-features = false, features = ["openssl", "file_io"] }
```

Key types: `Reader`, `ManifestStore`, `Manifest`, `Ingredient`, `Builder`, `Signer`.

### Workflow Patterns

**Inspect (read):**
```
file → c2pa::Reader::from_file() → ManifestStore → iterate Manifests → print Claims/Assertions
```

**Sign (write):**
```
asset + certificate + private key → c2pa::Builder → embed manifest → signed output file
```

### Showcase Targets

- [x] Read a JPEG/PNG and dump its manifest store as JSON
- [x] Pretty-print chain of custody (signer, issuer, algorithm, trust)
- [ ] Validate signature chain against a trust anchor (trust list not yet integrated)
- [x] Sign a new asset with an ECDSA ephemeral certificate (ES256, FIPS-safe)
- [x] Detect missing or existing manifests (acquired vs. published policy)
- [x] Extend an existing chain (OpenAI/DALL-E images verified)

---

## Dependency Philosophy

- Add crates deliberately. Every new dependency gets a one-line comment in
  `Cargo.toml` explaining why it is present.
- Prefer pure-Rust crates. C bindings are acceptable when no Rust alternative
  exists (e.g., libmagic, ffmpeg), but document the native lib requirement in
  `README.md`.
- Version-pin workspace dependencies to avoid silent drift.

```toml
[dependencies]
c2pa       = { version = "0.78.6", default-features = false, features = ["openssl", "file_io"] }
openssl    = "0.10"                                      # ECDSA ephemeral cert generation
thiserror  = "1"                                         # typed errors in lib
anyhow     = "1"                                         # error ergonomics in bin
clap       = { version = "4", features = ["derive"] }    # CLI arg parsing
serde_json = "1"                                         # JSON output
```

---

## CLI Conventions

- Single binary: `inspect`
- Subcommand structure even if only one subcommand exists today (anticipate growth):

```
inspect c2pa <FILE>              # read and print manifest
inspect c2pa --sign <FILE>       # sign a file (prototype)
inspect label <FILE>             # (future) read SELinux label
```

- Output defaults to human-readable. `--json` flag emits machine-readable JSON.
- Exit codes: 0 = success, 1 = file/manifest error, 2 = usage error.

---

## Future Phases (do not implement yet)

- **Apache module** (`mod_media_inspect`): reads file SELinux labels at HTTP
  request time. Will be a separate crate with a C-compatible FFI surface.
- **EXIF / XMP inspection** via `kamadak-exif` or `xmp-toolkit-rs`.
- **Audio/video format probing** via `symphonia` or `ffprobe` subprocess.
- **Watermark detection** hooks (FFmpeg-based, phase TBD).

---

## Agent Rules

1. Always check `Cargo.toml` and existing source before creating new types or modules.
2. Summarize crate capabilities for human review before writing showcase code.
3. Tests go in `tests/` — never `#[cfg(test)]` inline modules.
4. Do not add `println!` debug output to library code; use the `log` crate (or
   `tracing`) with appropriate levels.
5. When uncertain about scope, ask — this is a prototype, pivoting is cheap.

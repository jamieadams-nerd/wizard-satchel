# C2PA — Coalition for Content Provenance and Authenticity
## Technical Report: Manifests, Signers, and the Authenticity Model

> Prepared for: wizard-satchel / umrs-c2pa prototype
> Date: 2026-03-26
> Rust SDK: `c2pa` v0.78.6

---

## Table of Contents

1. [What Is C2PA?](#1-what-is-c2pa)
2. [What Is a Manifest?](#2-what-is-a-manifest)
3. [Assertions — The Building Blocks](#3-assertions--the-building-blocks)
4. [The Evidence Chain](#4-the-evidence-chain)
5. [Signer Types and the Trust Model](#5-signer-types-and-the-trust-model)
6. [How Authenticity Is Proven at Verification Time](#6-how-authenticity-is-proven-at-verification-time)
7. [Supported File Formats](#7-supported-file-formats)
8. [Test Images — Where to Get Valid Samples](#8-test-images--where-to-get-valid-samples)
9. [Planned Module Structure](#9-planned-module-structure)
10. [References](#10-references)

---

## 1. What Is C2PA?

C2PA (Coalition for Content Provenance and Authenticity) is an open technical
standard for attaching tamper-evident provenance metadata to media files. It
answers the question: *where did this content come from, who touched it, and
what did they do to it?*

The standard is maintained jointly by Adobe, Microsoft, Intel, the BBC, and
others, and is increasingly deployed in cameras (Leica, Sony), generative AI
tools (Adobe Firefly, DALL-E), and social platforms.

C2PA does **not** prevent copying or distribution. It proves **origin and chain
of custody**. Think of it as a cryptographically-sealed shipping manifest that
travels inside the file itself.

---

## 2. What Is a Manifest?

A **C2PA Manifest** is a tamper-evident, cryptographically signed record
embedded directly inside a media file. It is stored using **JUMBF** (JPEG
Universal Metadata Box Format) — a binary container that works across JPEG,
PNG, MP4, WAV, PDF, and other formats.

> A manifest is not a sidecar file. It lives inside the asset. If the manifest
> is stripped, that omission is itself detectable.


### Manifest Anatomy

```
Manifest
├── Assertions[]        ← typed metadata records (what happened, by whom)
├── Claim               ← signed summary referencing all assertions + asset hash
└── Claim Signature     ← X.509 cryptographic signature over the Claim
```

A file can contain a **Manifest Store** — multiple manifests, one per editing
step, linked into a chain of custody. The most recent manifest is the **active
manifest**.

### Example Manifest (JSON representation)

```json
{
  "active_manifest": "urn:uuid:abc123",
  "manifests": {
    "urn:uuid:abc123": {
      "claim_generator": "Adobe Lightroom/6.0 c2pa-rs/0.78",
      "title": "photo.jpg",
      "format": "image/jpeg",
      "instance_id": "xmp:iid:a1b2c3d4",
      "assertions": [
        {
          "label": "c2pa.actions",
          "data": {
            "actions": [
              {
                "action": "c2pa.edited",
                "softwareAgent": "Adobe Lightroom",
                "digitalSourceType": "http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia"
              }
            ]
          }
        },
        {
          "label": "c2pa.hash.data",
          "data": {
            "exclusions": [{ "start": 0, "length": 1024 }],
            "alg": "sha256",
            "hash": "base64encodedHashHere=="
          }
        }
      ],
      "signature_info": {
        "issuer": "Adobe Inc.",
        "cert_serial_number": "03:a1:...",
        "time": "2025-11-14T10:23:00Z"
      },
      "ingredients": []
    }
  }
}
```

---

## 3. Assertions — The Building Blocks

An assertion is a typed, labeled data record inside a manifest. It represents a statement made by the signer about the asset — what was done, by whom, and under what conditions.

### Standard Assertion Types

| Label | Purpose |
|-------|---------|
| `c2pa.actions` | Editing history: `c2pa.created`, `c2pa.edited`, `c2pa.published`, `c2pa.transcoded`, etc. |
| `c2pa.actions.v2` | Updated actions schema (v2 spec) — preferred in new manifests |
| `c2pa.hash.data` | Cryptographic hash binding the manifest to specific byte ranges of the asset |
| `c2pa.hash.bmff` | Hash binding for BMFF-container formats (MP4, MOV, HEIF) |
| `c2pa.thumbnail.claim.jpeg` | Thumbnail of the asset as it appeared at signing time |
| `c2pa.thumbnail.ingredient.jpeg` | Thumbnail of an ingredient (parent asset) |
| `c2pa.ingredient` | A parent asset that was used to create or derive this one |
| `c2pa.training-mining` | Whether the asset permits AI training or data mining use |
| `c2pa.soft-binding` | Perceptual hash / watermark anchor for tracking |
| `stds.iptc.photo-metadata` | Standard IPTC photo metadata (creator, copyright, location, description) |
| `com.adobe.*` | Vendor-specific extensions (Adobe, camera makers, etc.) |

### The `c2pa.actions` Vocabulary

The action field records what the `claim_generator` did. Common values:

| Action | Meaning |
|--------|---------|
| `c2pa.created` | Asset was created fresh (no prior ingredient) |
| `c2pa.edited` | Asset was modified from an ingredient |
| `c2pa.cropped` | Spatial crop was applied |
| `c2pa.filtered` | Color/tone/style filter was applied |
| `c2pa.resampled` | Resolution was changed |
| `c2pa.transcoded` | Format conversion (e.g., RAW → JPEG) |
| `c2pa.published` | Asset was distributed or published |
| `c2pa.converted` | File format changed |
| `c2pa.placed` | Ingredient was composited into the scene |

---

## 4. The Evidence Chain

The `ingredients` field is the mechanism that turns a manifest into an **evidence chain**.

When a tool edits an image, it:
1. Reads the existing manifest from the original file
2. Embeds that original manifest as an `Ingredient` in the new manifest
3. Records what it changed via `c2pa.actions` assertions
4. Signs the new manifest with its own key

This creates a verifiable, append-only chain of custody:

```
[Camera firmware]
  signs: c2pa.created, c2pa.hash.data, thumbnail
  identity: camera manufacturer CA
        ↓ (embedded as ingredient)
[Adobe Lightroom]
  signs: c2pa.edited (crop, color), c2pa.hash.data
  identity: Adobe CA
        ↓ (embedded as ingredient)
[Adobe Firefly]
  signs: c2pa.edited (AI inpaint), c2pa.training-mining
  identity: Adobe CA
        ↓ (embedded as ingredient)
[News publisher CMS]
  signs: c2pa.published
  identity: publisher CA
```

Each link is independently verifiable. An inspector can walk the chain and determine:
- Which steps used AI
- Which tool made which edit
- The exact order of operations
- Whether any step in the chain has been tampered with

---

## 5. Signer Types and the Trust Model

### X.509 Trust Hierarchy

C2PA signing uses standard **X.509 certificates** — the same infrastructure as HTTPS/TLS. The signer's identity is their certificate, issued by a Certificate Authority (CA) on the C2PA Trust List.

```
C2PA Trust List (approved root CAs)
    └── Intermediate CA  (e.g., Adobe Intermediate CA)
            └── End-entity signing cert  (e.g., "Lightroom Content Credentials Key")
                    └── Signs the Claim in the manifest
```

Verification walks this chain. A manifest is:
- **Valid + Trusted** — signature checks out and root CA is in the Trust List
- **Valid + Untrusted** — signature is cryptographically sound but the root CA is unknown (typical for self-signed test certs)
- **Invalid** — signature is broken, or asset bytes were modified after signing
- **Unsigned** — no manifest present

### Signer Types in the Rust SDK (`c2pa` v0.78.6)

| Type | Use Case | When to Use |
|------|----------|-------------|
| `EphemeralSigner` | Generates a temporary self-signed cert at runtime | Development and testing — no real cert needed |
| `CallbackSigner` | You supply a signing callback function | Production — delegate to HSM, KMS, or cloud signing service |
| Custom `Signer` trait impl | Full manual control over crypto operations | Advanced integrations, custom key formats |
| `AsyncSigner` trait | Async version of the `Signer` trait | Async runtimes (Tokio, async-std) |

For the umrs-c2pa prototype, `EphemeralSigner` is the right starting point.

### Supported Signing Algorithms

| Algorithm | Type | Key Size |
|-----------|------|----------|
| `es256` | ECDSA | P-256 (default) |
| `es384` | ECDSA | P-384 |
| `es512` | ECDSA | P-521 |
| `ps256` | RSA-PSS | 2048+ bit |
| `ps384` | RSA-PSS | 2048+ bit |
| `ps512` | RSA-PSS | 2048+ bit |
| `ed25519` | EdDSA | 255-bit |

`es256` is the default in the SDK and widely supported by C2PA validators.

---

## 6. How Authenticity Is Proven at Verification Time

When `c2pa::Reader::from_file()` reads a file, it executes the following verification pipeline:

```
Step 1: Extract JUMBF box from the file container
Step 2: Deserialize the Manifest Store from JUMBF
Step 3: Recompute the expected hash of the asset bytes
         (excluding the manifest box itself, using the exclusion map in c2pa.hash.data)
Step 4: Compare recomputed hash to the stored hash assertion
         → FAIL = file was modified after signing
Step 5: Verify the Claim Signature using the certificate's public key
         → FAIL = signature is broken or cert doesn't match
Step 6: Validate the certificate chain up to a trusted root CA
         → FAIL = untrusted signer (may still be valid, just unknown)
Step 7: Check the signing timestamp against a Time Stamp Authority (TSA)
         → Proves *when* the manifest was signed
Step 8: Recurse into Ingredients and verify each parent manifest
         → Walks the full evidence chain
```

All results are exposed through `ManifestStore::validation_status()`, which returns a list of status codes. The key codes:

| Status Code | Meaning |
|-------------|---------|
| `claimSignature.validated` | Signature is cryptographically valid |
| `signingCredential.trusted` | Signer cert is in the Trust List |
| `timeStamp.trusted` | TSA timestamp is valid |
| `assertion.dataHash.match` | Asset bytes match the hash — file not modified |
| `assertion.dataHash.mismatch` | **File was tampered with after signing** |
| `claimSignature.failed` | Signature verification failed |
| `manifest.inaccessible` | Manifest present but unreadable |

---

## 7. Supported File Formats

The `c2pa` v0.78.6 crate supports all of the following out of the box (no extra features needed beyond `file_io`):

| Format | MIME Type | Container |
|--------|-----------|-----------|
| JPEG | `image/jpeg` | JFIF / EXIF |
| PNG | `image/png` | PNG chunks |
| WebP | `image/webp` | RIFF |
| TIFF | `image/tiff` | TIFF IFD |
| AVIF | `image/avif` | BMFF/ISOBMFF |
| HEIC / HEIF | `image/heic` | BMFF/ISOBMFF |
| MP4 / MOV | `video/mp4` | BMFF/ISOBMFF |
| WAV | `audio/wav` | RIFF |
| MP3 | `audio/mpeg` | ID3 |
| PDF | `application/pdf` | *(requires `pdf` feature)* |
| ZIP | `application/zip` | ZIP |

Format is auto-detected from the file contents — no need to specify it manually.

---

## 8. Test Images — Where to Get Valid Samples

Many older sample image repositories have gone stale. The following sources are confirmed active:

| Source | Formats | Notes |
|--------|---------|-------|
| [c2pa-rs `sdk/tests/fixtures/`](https://github.com/contentauth/c2pa-rs/tree/main/sdk/tests/fixtures) | JPEG, PNG, WebP, more | Official SDK test fixtures — clone the repo |
| [c2pa-attacks `sample/C.jpg`](https://github.com/contentauth/c2pa-attacks) | JPEG | Simple known-good signed image with manifest |
| [contentauthenticity.org/examples](https://contentauthenticity.org/examples) | JPEG, PNG, video | CAI curated showcase images, downloadable |
| [contentcredentials.org/verify](https://contentcredentials.org/verify) | Any supported | Drag-drop online verifier — inspect manifests in-browser |

**Recommended approach:** Clone the c2pa-rs repo and use its `sdk/tests/fixtures/` directory — it is purpose-built for testing and covers multiple formats and edge cases.

```sh
git clone https://github.com/contentauth/c2pa-rs
ls c2pa-rs/sdk/tests/fixtures/
```

---

## 9. Planned Module Structure

The `umrs-c2pa` crate will be structured as a drop-in module:

```
umrs-c2pa/
├── Cargo.toml
└── src/
    ├── main.rs          ← CLI only: clap argument parsing + output formatting
    └── c2pa/
        └── mod.rs       ← all library logic: read_manifest(), sign_asset(), etc.
```

A downstream crate can consume this by copying the `c2pa/` directory and adding `mod c2pa;`.

### `Cargo.toml` Key Dependencies

```toml
[dependencies]
# C2PA manifest read/write — vendored OpenSSL for hermetic builds
c2pa    = { version = "0.78.6", default-features = false, features = ["openssl", "file_io"] }
openssl = { version = "0.10", features = ["vendored"] }  # builds OpenSSL from source, no system lib required

# Error handling
thiserror  = "1"   # typed errors in library code
anyhow     = "1"   # ergonomic error handling in binary

# CLI and output
clap       = { version = "4", features = ["derive"] }   # subcommand CLI
serde_json = "1"                                         # JSON manifest output
```

### CLI Surface (planned)

```
inspect c2pa <FILE>           # read and print manifest store (human-readable)
inspect c2pa --json <FILE>    # emit manifest store as JSON
inspect c2pa --sign <FILE>    # sign a file with an ephemeral test cert
```

---

## 10. References

- [c2pa-rs Rust SDK — GitHub](https://github.com/contentauth/c2pa-rs)
- [C2PA Technical Specification 2.2](https://spec.c2pa.org/specifications/specifications/2.2/specs/C2PA_Specification.html)
- [Manifest Examples — CAI Open Source Docs](https://opensource.contentauthenticity.org/docs/manifest/manifest-examples/)
- [Understanding Manifests — CAI Open Source Docs](https://opensource.contentauthenticity.org/docs/manifest/understanding-manifest/)
- [Manifest Definition File (c2patool schema)](https://github.com/contentauth/c2patool/blob/main/docs/manifest.md)
- [Identity Assertion — Creator Assertions Working Group](https://cawg.io/identity/1.2/)
- [c2pa-attacks Security Test Tool](https://github.com/contentauth/c2pa-attacks)
- [c2pa-test-image-service](https://github.com/contentauth/c2pa-test-image-service)
- [Content Authenticity Initiative Examples](https://contentauthenticity.org/examples)
- [Content Credentials Online Verifier](https://contentcredentials.org/verify)

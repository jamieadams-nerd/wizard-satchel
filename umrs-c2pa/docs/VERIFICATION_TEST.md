# VERIFICATION_TEST.md — Tamper Detection and Security Label Verification

> Executed: 2026-03-26
> System: RHEL 10 (aarch64), OpenSSL 3.5.5, FIPS provider active
> Binary: `inspect` (umrs-c2pa 0.1.0)

This document records a live verification of the `umrs-c2pa` tamper detection
and security label (`umrs.security-label`) assertion. All output below is
real terminal output — nothing has been edited.

---

## Test Setup

Source image: `tests/sandbox/jamie_desk.png` — an OpenAI/ChatGPT-generated
image that already carries two C2PA manifests from OpenAI's signing pipeline.

---

## Test 1: Sign with a CUI Marking

Sign the OpenAI image with a `CUI//SP-CTI` security label and write the
output to a clean file.

```
$ cargo run -- c2pa tests/sandbox/jamie_desk.png --sign --marking "CUI//SP-CTI" --output /tmp/test_clean.png

Chain of Custody — tests/sandbox/jamie_desk.png
SHA-256: 3b6c04def733ee21d0fef1fa4e594e9a9b9c93132f5bd0a1a1473684a9f41cca
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1   *[NO TRUST LIST]  Truepic Lens CLI in Sora
                         Signed at : no timestamp provided
                         Issuer    : OpenAI
                         Alg       : Es256
                         Generator : ChatGPT

  2   *[NO TRUST LIST]  Truepic Lens CLI in Sora
                         Signed at : no timestamp provided
                         Issuer    : OpenAI
                         Alg       : Es256
                         Generator : ChatGPT

  3   *[NO TRUST LIST]  My Organization (ephemeral — self-signed)
                         Signed at : 2026-03-26T14:52:48Z UTC
                         Issuer    : My Organization
                         Alg       : Es256
                         Generator : UMRS Reference System/1.0 0.1.0
                         Marking   : CUI//SP-CTI

────────────────────────────────────────────────────────
  *[NO TRUST LIST] No trust list configured — trust could not be evaluated
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Hash consistency : PASS — file unchanged across all signing events
UMRS action      : c2pa.published
UMRS output      : /tmp/test_clean.png
UMRS identity    : ephemeral self-signed cert (test mode — UNTRUSTED)
```

**Result:** Three-entry chain. Entries 1-2 are OpenAI's original chain
(ChatGPT via Truepic's signing infrastructure). Entry 3 is the UMRS ingest
record with the `CUI//SP-CTI` security label. Hash consistency: PASS.

---

## Test 2: Read Back the Signed File

Read the signed output back to confirm the chain and marking persist.

```
$ cargo run -- c2pa /tmp/test_clean.png

Chain of Custody — /tmp/test_clean.png
SHA-256: be072ce78441bf1c7e56a0b01a2dc3c3c63a57e2eaeecaace1914352f5390fa9
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1   *[NO TRUST LIST]  Truepic Lens CLI in Sora
                         Signed at : no timestamp provided
                         Issuer    : OpenAI
                         Alg       : Es256
                         Generator : ChatGPT

  2   *[NO TRUST LIST]  Truepic Lens CLI in Sora
                         Signed at : no timestamp provided
                         Issuer    : OpenAI
                         Alg       : Es256
                         Generator : ChatGPT

  3   *[NO TRUST LIST]  My Organization (ephemeral — self-signed)
                         Signed at : 2026-03-26T14:52:48Z UTC
                         Issuer    : My Organization
                         Alg       : Es256
                         Generator : UMRS Reference System/1.0 0.1.0
                         Marking   : CUI//SP-CTI

────────────────────────────────────────────────────────
  *[NO TRUST LIST] No trust list configured — trust could not be evaluated
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Result:** Chain intact. Marking reads back as `CUI//SP-CTI`. The SHA-256
is different from the source (expected — the signed file includes the embedded
manifest data).

### JSON evidence chain

```
$ cargo run -- c2pa /tmp/test_clean.png --chain-json

[
  {
    "signer_name": "Truepic Lens CLI in Sora",
    "issuer": "OpenAI",
    "signed_at": null,
    "trust_status": "NO_TRUST_LIST",
    "algorithm": "Es256",
    "generator": "ChatGPT",
    "generator_version": null,
    "security_label": null
  },
  {
    "signer_name": "Truepic Lens CLI in Sora",
    "issuer": "OpenAI",
    "signed_at": null,
    "trust_status": "NO_TRUST_LIST",
    "algorithm": "Es256",
    "generator": "ChatGPT",
    "generator_version": null,
    "security_label": null
  },
  {
    "signer_name": "My Organization (ephemeral — self-signed)",
    "issuer": "My Organization",
    "signed_at": "2026-03-26T14:52:48Z",
    "trust_status": "NO_TRUST_LIST",
    "algorithm": "Es256",
    "generator": "UMRS Reference System/1.0",
    "generator_version": "0.1.0",
    "security_label": "CUI//SP-CTI"
  }
]
```

**Result:** The `security_label` field is `null` for the upstream OpenAI
entries (they didn't apply a marking) and `"CUI//SP-CTI"` for the UMRS entry.
This is exactly what another security tool would consume.

---

## Test 3: Tamper Detection — Modify One Byte

Copy the signed file and flip a single byte at offset 5000. This simulates
an attacker or a corrupted transfer modifying the image data after signing.

```
$ cp /tmp/test_clean.png /tmp/test_tampered.png
$ printf '\xff' | dd of=/tmp/test_tampered.png bs=1 seek=5000 count=1 conv=notrunc
```

Now read the tampered file:

```
$ cargo run -- c2pa /tmp/test_tampered.png

Chain of Custody — /tmp/test_tampered.png
SHA-256: 6d2f5833c7cd9a2e67ce5437e6c48a4e66fe0f1752cadfcf1ab091dc5d6f4543
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1   [INVALID]       Truepic Lens CLI in Sora
                       Signed at : no timestamp provided
                       Issuer    : OpenAI
                       Alg       : Es256
                       Generator : ChatGPT

  2   [INVALID]       Truepic Lens CLI in Sora
                       Signed at : no timestamp provided
                       Issuer    : OpenAI
                       Alg       : Es256
                       Generator : ChatGPT

  3   [INVALID]       My Organization (ephemeral — self-signed)
                       Signed at : 2026-03-26T14:52:48Z UTC
                       Issuer    : My Organization
                       Alg       : Es256
                       Generator : UMRS Reference System/1.0 0.1.0
                       Marking   : CUI//SP-CTI

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Result:** Every entry in the chain now shows `[INVALID]`. The c2pa SDK
detected `ingredient.manifest.mismatch` ("ingredient hash incorrect") in
the store-level validation status. A single flipped byte invalidated the
entire chain.

Note that the marking `CUI//SP-CTI` is still **readable** in the output —
the manifest structure survived the byte flip. But the `[INVALID]` status
tells the operator that the file has been tampered with and the manifest
cannot be trusted.

### JSON evidence chain (tampered)

```
$ cargo run -- c2pa /tmp/test_tampered.png --chain-json

[
  {
    "signer_name": "Truepic Lens CLI in Sora",
    "issuer": "OpenAI",
    "signed_at": null,
    "trust_status": "INVALID",
    "algorithm": "Es256",
    "generator": "ChatGPT",
    "generator_version": null,
    "security_label": null
  },
  {
    "signer_name": "Truepic Lens CLI in Sora",
    "issuer": "OpenAI",
    "signed_at": null,
    "trust_status": "INVALID",
    "algorithm": "Es256",
    "generator": "ChatGPT",
    "generator_version": null,
    "security_label": null
  },
  {
    "signer_name": "My Organization (ephemeral — self-signed)",
    "issuer": "My Organization",
    "signed_at": "2026-03-26T14:52:48Z",
    "trust_status": "INVALID",
    "algorithm": "Es256",
    "generator": "UMRS Reference System/1.0",
    "generator_version": "0.1.0",
    "security_label": "CUI//SP-CTI"
  }
]
```

**Result:** `trust_status: "INVALID"` on every entry. An automated tool
consuming this JSON would immediately flag this file.

---

## Test 4: SHA-256 Comparison

| File | SHA-256 | Status |
|---|---|---|
| Source (OpenAI original) | `3b6c04def733ee21d0fef1fa4e594e9a9b9c93132f5bd0a1a1473684a9f41cca` | No UMRS manifest |
| Signed (clean) | `be072ce78441bf1c7e56a0b01a2dc3c3c63a57e2eaeecaace1914352f5390fa9` | Chain valid |
| Tampered (1 byte flipped) | `6d2f5833c7cd9a2e67ce5437e6c48a4e66fe0f1752cadfcf1ab091dc5d6f4543` | `[INVALID]` — detected |

A single byte change produces a completely different SHA-256 and is detected
by the C2PA validation layer as `ingredient.manifest.mismatch`.

---

## How the Tamper Detection Works

When `inspect` reads a signed file, the c2pa SDK runs these validation checks
automatically:

```
assertion.hashedURI.match    — each assertion's URI hash matches the claim
assertion.dataHash.match     — the image data hash matches what was signed
ingredient.manifest.mismatch — an ingredient's manifest data was altered
signingCredential.untrusted  — the signing cert is not in the trust list
```

These are not application-layer checks. They are **cryptographic hash
verifications** built into the C2PA standard. The `umrs.security-label`
assertion participates in this same mechanism — its content is hashed and
bound to the claim signature. If anyone modifies the marking string after
signing, `assertion.hashedURI.match` fails and the manifest is invalidated.

This is the critical property: **the security label cannot be silently altered
or stripped without breaking the cryptographic chain.** This is fundamentally
different from metadata-based labeling (EXIF tags, filename conventions,
database records) where markings can be changed without detection.

---

## Test 5: Overwrite Safeguard

Attempting to re-sign a previously signed file is refused:

```
$ cargo run -- c2pa tests/sandbox/jamie_umrs_signed.png --sign

Error: Refusing to overwrite previously signed file: tests/sandbox/jamie_umrs_signed.png
```

**Result:** The safeguard prevents accidental double-signing.

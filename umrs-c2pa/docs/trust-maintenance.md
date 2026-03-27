# Trust List Maintenance Guide

**SPDX-License-Identifier: MIT**
**Copyright (c) 2025 Jamie Adams**

---

## Overview

UMRS C2PA validates media signatures against a set of trusted root certificates
(trust anchors). These certificates are stored as PEM files on disk. The operator
is responsible for obtaining, placing, and periodically updating these files.

There is no hardcoded default path. All trust file locations are set in `umrs.toml`
under the `[trust]` section.

---

## What Goes in the Trust Anchor File

The trust anchor file is a PEM bundle containing one or more X.509 root CA
certificates. When UMRS validates a C2PA manifest signature, it walks the
certificate chain from the signer's leaf cert up to a root and checks whether
that root appears in this file.

**If the root is present:** the manifest is marked `Trusted`.
**If the root is absent:** the manifest is marked `Untrusted` (signature may still
be cryptographically valid, but we don't recognize the issuer).

---

## Where to Get Trust Anchors

### C2PA Official Trust List

The C2PA consortium publishes an official list of recognized certificate authorities.
This is the primary trust list for validating content credentials from cameras,
software (Adobe, Microsoft, etc.), and other C2PA-compliant tools.

**Source:** The official trust list is maintained by the C2PA consortium and
distributed through the Content Credentials ecosystem. The c2pa-rs SDK
(`c2pa` crate) includes test root certificates in its source tree at:

```
c2pa-rs/sdk/tests/fixtures/certs/trust/test_cert_root_bundle.pem
```

For production use, obtain the current trust list from the C2PA consortium's
published trust infrastructure. The list is a standard PEM bundle.

**Note:** The test root bundle from c2pa-rs is for development and testing only.
Production deployments should use the officially published trust anchors.

### Organization Root CAs

If your organization issues its own signing certificates (e.g., for internal media
workflows), add your org's root CA to the `user_anchors` file. This is separate
from the C2PA official list so you can update them independently.

---

## Configuration

All paths are set in `umrs.toml`:

```toml
[trust]
# C2PA official root CAs
trust_anchors = "/opt/umrs/trust/c2pa-anchors.pem"

# Your organization's root CAs (optional)
user_anchors = "/opt/umrs/trust/org-roots.pem"

# End-entity certificate allowlist (optional)
# Directly trust specific signer certs without chain validation.
# Can contain PEM certificates or base64-encoded SHA-256 hashes.
allowed_list = "/opt/umrs/trust/allowed-signers.pem"

# EKU OID filter (optional)
# Only trust certificates with these Extended Key Usage OIDs.
# One OID per line, comments start with //
trust_config = "/opt/umrs/trust/ekus.cfg"

# Master switch (default: true when any trust file is configured)
verify_trust = true
```

Choose whatever directory layout suits your deployment. Common patterns:

| Environment | Example Path |
|---|---|
| Development / local | `./trust/c2pa-anchors.pem` |
| Server (FHS) | `/etc/umrs/trust/c2pa-anchors.pem` |
| Container | `/opt/umrs/trust/c2pa-anchors.pem` |
| Air-gapped RHEL | `/etc/pki/umrs/c2pa-anchors.pem` |

---

## Updating Trust Anchors

### Manual Update Procedure

1. **Obtain the updated PEM bundle** from the C2PA consortium or your CA
   administrator.

2. **Verify the file is valid PEM:**
   ```bash
   openssl x509 -in c2pa-anchors.pem -noout -text | head -20
   ```
   For a bundle with multiple certs, check the count:
   ```bash
   grep -c 'BEGIN CERTIFICATE' c2pa-anchors.pem
   ```

3. **Replace the file** at the configured path:
   ```bash
   cp c2pa-anchors.pem /opt/umrs/trust/c2pa-anchors.pem
   chmod 644 /opt/umrs/trust/c2pa-anchors.pem
   ```

4. **Verify UMRS reads the new list** by inspecting a known-signed file:
   ```bash
   inspect c2pa some-signed-image.jpg
   ```
   Look for `Trusted` in the chain-of-custody output where you previously saw
   `Untrusted` or `NoTrustList`.

### Update Frequency

- **C2PA official list:** Check quarterly, or when the consortium announces new
  CAs or revocations.
- **Organization roots:** Update when your PKI team rotates or reissues root
  certificates.
- **Allowed list:** Update when specific signer certificates are added or removed.

### Air-Gapped Environments

For systems without internet access:

1. Download the updated PEM on a connected workstation.
2. Transfer via approved media (USB, optical) per your security policy.
3. Place at the configured path.
4. Verify with `openssl x509` as above.

No UMRS code attempts to fetch trust material from the network.

---

## EKU Configuration File Format

The EKU (Extended Key Usage) config file filters which certificates are
acceptable based on their declared purpose. Format:

```
// C2PA signing certificate
1.3.6.1.4.1.62558.2.1

// Document signing
1.3.6.1.5.5.7.3.36

// Email protection (used by some early C2PA signers)
1.3.6.1.5.5.7.3.4

// Timestamping
1.3.6.1.5.5.7.3.8
```

Lines starting with `//` are comments. One OID per line in dotted-decimal notation.

If no EKU config is provided, the c2pa SDK uses its built-in default set (which
covers the standard C2PA OIDs listed above).

---

## End-Entity Allowlist Format

The allowlist lets you directly trust specific signer certificates without requiring
a chain to a root CA. This is useful for:

- Self-signed certificates (development, internal tools)
- Certificates from CAs not in the official trust list
- Emergency trust overrides

The file can contain:

1. **PEM certificates** — full certificate blocks
2. **Base64 SHA-256 hashes** — 44-character base64 strings (one per line), each
   being the SHA-256 hash of the certificate's DER encoding

Mixed format (some PEM blocks, some hashes) is supported.

---

## Troubleshooting

| Symptom | Likely Cause | Fix |
|---|---|---|
| All manifests show `NoTrustList` | No `[trust]` section in config | Add trust config to `umrs.toml` |
| All manifests show `Untrusted` | Trust file exists but signer's root CA isn't in it | Add the missing root CA to the PEM bundle |
| `Untrusted` for your org's certs only | Org root not in `user_anchors` | Add org root CA to `user_anchors` file |
| Config validation warns about trust | PEM file path doesn't exist or isn't readable | Check path and permissions (`644`) |
| `InvalidEku` errors | Signer cert's EKU not in allowed list | Add the OID to your EKU config file |

---

## Attribution

This trust system follows patterns established by the c2pa-rs project
(MIT OR Apache-2.0, Copyright 2020 Adobe). Test trust fixtures used during
development originate from that project. See `tests/fixtures/ATTRIBUTION.md`.

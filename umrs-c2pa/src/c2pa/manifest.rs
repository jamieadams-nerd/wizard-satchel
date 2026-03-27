// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jamie Adams

use std::path::Path;

use serde::Serialize;

use crate::c2pa::error::InspectError;
#[allow(unused_imports)]
use crate::verbose;

/// Trust evaluation for a single entry in the chain of custody.
///
/// | Status        | Display         | Meaning |
/// |---------------|-----------------|---------|
/// | `Trusted`     | `TRUSTED`       | Cert chain verified against a C2PA Trust List root CA |
/// | `Untrusted`   | `UNVERIFIED`    | Signature present but not validated against a trust list |
/// | `Invalid`     | `INVALID`       | Signature verification failed or asset hash mismatch |
/// | `Revoked`     | `REVOKED`       | Signing certificate was revoked by the issuing CA |
/// | `NoTrustList` | `NO TRUST LIST` | No trust list configured — cannot evaluate trust |
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TrustStatus {
    /// Cert chain leads to a root CA in the C2PA Trust List.
    #[serde(rename = "TRUSTED")]
    Trusted,
    /// Signature is present but the CA is not on the Trust List,
    /// or no trust list was configured. The signature has not been
    /// validated — it is not necessarily bad, just unverified.
    #[serde(rename = "UNVERIFIED")]
    Untrusted,
    /// Signature verification failed, or asset hash does not match.
    #[serde(rename = "INVALID")]
    Invalid,
    /// Certificate was revoked by the issuing CA.
    #[serde(rename = "REVOKED")]
    Revoked,
    /// No trust list is configured, so trust cannot be evaluated.
    /// Distinct from Untrusted: this means we did not even attempt validation.
    #[serde(rename = "NO_TRUST_LIST")]
    NoTrustList,
}

impl std::fmt::Display for TrustStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustStatus::Trusted => write!(f, "TRUSTED"),
            TrustStatus::Untrusted => write!(f, "UNVERIFIED"),
            TrustStatus::Invalid => write!(f, "INVALID"),
            TrustStatus::Revoked => write!(f, "REVOKED"),
            TrustStatus::NoTrustList => write!(f, "NO TRUST LIST"),
        }
    }
}

/// A single entry in the chain of custody, extracted from one manifest
/// in the manifest store.
#[derive(Debug, Clone, Serialize)]
pub struct ChainEntry {
    /// Signer identity — from the cert CN or `claim_generator` field.
    pub signer_name: String,

    /// CA that issued the signing certificate.
    pub issuer: String,

    /// Signing timestamp, if present.
    pub signed_at: Option<String>,

    /// Trust evaluation for this entry.
    pub trust_status: TrustStatus,

    /// Signing algorithm used (e.g. "es256").
    pub algorithm: String,

    /// Claim generator name (e.g. "`ChatGPT`", "UMRS Reference System").
    pub generator: String,

    /// Claim generator version, if available (e.g. "0.67.1").
    pub generator_version: Option<String>,

    /// Security label / marking from a `umrs.security-label` assertion, if present.
    pub security_label: Option<String>,
}

/// Read the chain of custody from a file's C2PA manifest store.
///
/// Returns entries ordered oldest-first (deepest ingredient → active manifest).
/// Returns an empty `Vec` if the file has no manifest.
///
/// # Errors
///
/// Returns `InspectError::C2pa` if the manifest store cannot be read, or
/// `InspectError::Config` if the manifest JSON is malformed.
pub fn read_chain(path: &Path) -> Result<Vec<ChainEntry>, InspectError> {
    verbose!("Opening C2PA manifest store...");
    let reader = match c2pa::Reader::from_file(path) {
        Ok(r) => r,
        Err(c2pa::Error::JumbfNotFound | c2pa::Error::ProvenanceMissing) => {
            verbose!("No C2PA manifest found in file");
            return Ok(Vec::new());
        }
        Err(e) => return Err(InspectError::C2pa(e)),
    };

    verbose!("Parsing manifest store JSON...");
    let store_json: serde_json::Value = serde_json::from_str(&reader.json())
        .map_err(|e| InspectError::Config(format!("manifest JSON parse error: {e}")))?;

    verbose!("Walking chain of custody...");
    let mut entries: Vec<ChainEntry> = Vec::new();
    collect_entries(&store_json, &mut entries);
    verbose!("Found {} chain entries", entries.len());
    Ok(entries)
}

/// Returns `true` if the file contains any C2PA manifest data.
#[must_use]
pub fn has_manifest(path: &Path) -> bool {
    c2pa::Reader::from_file(path).is_ok()
}

/// Returns the full manifest store as a pretty-printed JSON string.
///
/// This is the **raw c2pa SDK output** — the complete manifest store as the
/// crate emits it, including all assertions, ingredients, and signature info.
///
/// # Errors
///
/// Returns `InspectError::C2pa` if the manifest store cannot be read, or
/// `InspectError::Config` if JSON parsing or serialization fails.
pub fn manifest_json(path: &Path) -> Result<String, InspectError> {
    let reader = c2pa::Reader::from_file(path).map_err(InspectError::C2pa)?;
    let val: serde_json::Value = serde_json::from_str(&reader.json())
        .map_err(|e| InspectError::Config(format!("manifest JSON parse: {e}")))?;
    serde_json::to_string_pretty(&val)
        .map_err(|e| InspectError::Config(format!("JSON serialize: {e}")))
}

/// Returns the UMRS-parsed chain of custody as a JSON string.
///
/// Unlike `manifest_json()` which returns the raw c2pa SDK manifest store,
/// this function returns the **parsed evidence chain** — the same data
/// displayed in the human-readable report, serialized as JSON for
/// programmatic consumption by other tools.
///
/// The returned JSON is an array of objects ordered oldest-first:
///
/// ```json
/// [
///   {
///     "signer_name": "Truepic Lens CLI in Sora",
///     "issuer": "OpenAI",
///     "signed_at": null,
///     "trust_status": "NO_TRUST_LIST",
///     "algorithm": "Es256",
///     "generator": "ChatGPT",
///     "generator_version": null
///   }
/// ]
/// ```
///
/// Returns an empty array `[]` if the file has no C2PA manifest.
///
/// # Errors
///
/// Returns `InspectError::C2pa` if the manifest store cannot be read, or
/// `InspectError::Config` if JSON serialization fails.
pub fn chain_json(path: &Path) -> Result<String, InspectError> {
    let chain = read_chain(path)?;
    serde_json::to_string_pretty(&chain)
        .map_err(|e| InspectError::Config(format!("JSON serialize: {e}")))
}

/// Returns the most recent signer name and timestamp from the active manifest.
/// Used for the ingest log entry in the "has manifest" case.
///
/// # Errors
///
/// Returns `InspectError::C2pa` if the manifest store cannot be read, or
/// `InspectError::Config` if the manifest JSON is malformed.
pub fn last_signer(path: &Path) -> Result<Option<(String, Option<String>)>, InspectError> {
    let chain = read_chain(path)?;
    Ok(chain.last().map(|e| (e.signer_name.clone(), e.signed_at.clone())))
}

// Walk the manifest store JSON and collect chain entries.
// The store JSON has an `active_manifest` key and a `manifests` map.
// We walk from the active manifest back through ingredients.
fn collect_entries(store: &serde_json::Value, out: &mut Vec<ChainEntry>) {
    let Some(manifests) = store.get("manifests").and_then(|m| m.as_object()) else {
        return;
    };
    let Some(active_id) = store.get("active_manifest").and_then(|v| v.as_str()) else {
        return;
    };

    // Walk the chain recursively: ingredients first, then the active manifest.
    walk_manifest(active_id, manifests, out, &mut std::collections::HashSet::new());

    // Check store-level validation_status for tampering indicators that
    // affect the entire chain (e.g. ingredient.manifest.mismatch).
    let store_tampered =
        store.get("validation_status").and_then(|v| v.as_array()).is_some_and(|statuses| {
            statuses.iter().any(|s| {
                s.get("code")
                    .and_then(|v| v.as_str())
                    .is_some_and(|c| c.contains("mismatch") || c.contains("failed"))
            })
        });

    if store_tampered {
        for entry in out.iter_mut() {
            // Only override non-Invalid statuses — don't mask a worse status.
            if entry.trust_status != TrustStatus::Invalid {
                entry.trust_status = TrustStatus::Invalid;
            }
        }
    }
}

fn walk_manifest(
    id: &str,
    manifests: &serde_json::Map<String, serde_json::Value>,
    out: &mut Vec<ChainEntry>,
    visited: &mut std::collections::HashSet<String>,
) {
    if !visited.insert(id.to_string()) {
        return; // cycle guard
    }

    let Some(manifest) = manifests.get(id) else {
        return;
    };

    // Recurse into ingredients first (oldest-first ordering).
    if let Some(ingredients) = manifest.get("ingredients").and_then(|v| v.as_array()) {
        for ingredient in ingredients {
            if let Some(manifest_ref) = ingredient.get("active_manifest").and_then(|v| v.as_str()) {
                walk_manifest(manifest_ref, manifests, out, visited);
            }
        }
    }

    // Extract this manifest's entry.
    let entry = extract_entry(manifest);
    out.push(entry);
}

fn extract_entry(manifest: &serde_json::Value) -> ChainEntry {
    let sig_info = manifest.get("signature_info");

    // Signer identity: prefer common_name (cert CN, e.g. "Truepic Lens CLI
    // in Sora"), fall back to issuer, then claim_generator.
    let signer_name = sig_info
        .and_then(|s| s.get("common_name"))
        .and_then(|v| v.as_str())
        .or_else(|| sig_info.and_then(|s| s.get("issuer")).and_then(|v| v.as_str()))
        .or_else(|| manifest.get("claim_generator").and_then(|v| v.as_str()))
        .unwrap_or("Unknown")
        .to_string();

    // Issuer: the organization that issued the signing certificate.
    let issuer = sig_info
        .and_then(|s| s.get("issuer"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    // Timestamp: prefer TSA timestamp from signature_info, fall back to the
    // "when" field from the first action assertion (e.g. UMRS ingest time).
    let signed_at = sig_info
        .and_then(|s| s.get("time"))
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string)
        .or_else(|| extract_action_when(manifest));

    let algorithm = sig_info
        .and_then(|s| s.get("alg"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Extract claim_generator info — prefer structured claim_generator_info
    // array, fall back to parsing the claim_generator string.
    let (generator, generator_version) = extract_generator_info(manifest);

    // Derive trust status from validation_status codes if present.
    let trust_status = derive_trust(manifest);

    // Extract security label from umrs.security-label assertion.
    let security_label = extract_security_label(manifest);

    ChainEntry {
        signer_name,
        issuer,
        signed_at,
        trust_status,
        algorithm,
        generator,
        generator_version,
        security_label,
    }
}

/// Extract the `when` timestamp from the first action in `c2pa.actions` or
/// `c2pa.actions.v2`. Returns `None` if no action has a `when` field.
fn extract_action_when(manifest: &serde_json::Value) -> Option<String> {
    let assertions = manifest.get("assertions").and_then(|v| v.as_array())?;
    for assertion in assertions {
        let label = assertion.get("label").and_then(|v| v.as_str()).unwrap_or("");
        if label == "c2pa.actions" || label == "c2pa.actions.v2" {
            let actions =
                assertion.get("data").and_then(|d| d.get("actions")).and_then(|a| a.as_array())?;
            for action in actions {
                if let Some(when) = action.get("when").and_then(|v| v.as_str()) {
                    return Some(when.to_string());
                }
            }
        }
    }
    None
}

/// Extract a security label from a `umrs.security-label` assertion, if present.
fn extract_security_label(manifest: &serde_json::Value) -> Option<String> {
    let assertions = manifest.get("assertions").and_then(|v| v.as_array())?;
    for assertion in assertions {
        let label = assertion.get("label").and_then(|v| v.as_str()).unwrap_or("");
        if label == "umrs.security-label" {
            return assertion
                .get("data")
                .and_then(|d| d.get("marking"))
                .and_then(|v| v.as_str())
                .map(String::from);
        }
    }
    None
}

/// Extract generator name and version from the manifest.
///
/// Prefers `claim_generator_info` (structured array). Looks for:
///   1. `version` field (standard C2PA, e.g. UMRS sets this)
///   2. `org.contentauth.c2pa_rs` vendor extension (used by OpenAI/ChatGPT
///      to record the c2pa-rs SDK version — not the app version, but still
///      useful for forensics)
///
/// Falls back to parsing the `claim_generator` string, which often has the
/// form `"Name/Version"`.
fn extract_generator_info(manifest: &serde_json::Value) -> (String, Option<String>) {
    // Try claim_generator_info array first (C2PA 2.x style).
    if let Some(info_arr) = manifest.get("claim_generator_info").and_then(|v| v.as_array())
        && let Some(first) = info_arr.first()
    {
        let name = first.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");

        // Only use the explicit "version" field — vendor extensions like
        // "org.contentauth.c2pa_rs" are internal SDK version numbers,
        // not meaningful to end users.
        let version = first.get("version").and_then(|v| v.as_str()).map(String::from);

        return (name.to_string(), version);
    }

    // Fall back to claim_generator string — split on "/" for name/version.
    if let Some(cg) = manifest.get("claim_generator").and_then(|v| v.as_str()) {
        if let Some((name, version)) = cg.split_once('/') {
            return (name.trim().to_string(), Some(version.trim().to_string()));
        }
        return (cg.to_string(), None);
    }

    ("Unknown".to_string(), None)
}

fn derive_trust(manifest: &serde_json::Value) -> TrustStatus {
    let Some(statuses) = manifest.get("validation_status").and_then(|v| v.as_array()) else {
        // No validation_status array — common for ingredient manifests and
        // self-signed output.  No trust list was evaluated.
        return TrustStatus::NoTrustList;
    };

    if statuses.is_empty() {
        return TrustStatus::NoTrustList;
    }

    let codes: Vec<&str> =
        statuses.iter().filter_map(|s| s.get("code").and_then(|v| v.as_str())).collect();

    if codes.iter().any(|c| c.contains("revoked")) {
        return TrustStatus::Revoked;
    }
    if codes.iter().any(|c| c.contains("mismatch") || c.contains("failed")) {
        return TrustStatus::Invalid;
    }
    if codes.contains(&"signingCredential.trusted") {
        return TrustStatus::Trusted;
    }
    if codes.iter().any(|c| c.contains("untrusted")) {
        return TrustStatus::Untrusted;
    }
    TrustStatus::Untrusted
}

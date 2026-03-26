use std::path::Path;

use crate::c2pa::error::InspectError;

/// Trust evaluation for a single entry in the chain of custody.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustStatus {
    /// Cert chain leads to a root CA in the C2PA Trust List.
    Trusted,
    /// Signature is valid but the CA is not on the Trust List (e.g. self-signed).
    Untrusted,
    /// Signature verification failed, or asset hash does not match.
    Invalid,
    /// Certificate was revoked by the issuing CA.
    Revoked,
    /// Trust status could not be determined.
    Unknown,
}

impl std::fmt::Display for TrustStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustStatus::Trusted   => write!(f, "TRUSTED"),
            TrustStatus::Untrusted => write!(f, "UNTRUSTED"),
            TrustStatus::Invalid   => write!(f, "INVALID"),
            TrustStatus::Revoked   => write!(f, "REVOKED"),
            TrustStatus::Unknown   => write!(f, "UNKNOWN"),
        }
    }
}

/// A single entry in the chain of custody, extracted from one manifest
/// in the manifest store.
#[derive(Debug, Clone)]
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
}

/// Read the chain of custody from a file's C2PA manifest store.
///
/// Returns entries ordered oldest-first (deepest ingredient → active manifest).
/// Returns an empty `Vec` if the file has no manifest.
pub fn read_chain(path: &Path) -> Result<Vec<ChainEntry>, InspectError> {
    let reader = match c2pa::Reader::from_file(path) {
        Ok(r)  => r,
        Err(c2pa::Error::JumbfNotFound | c2pa::Error::ProvenanceMissing) => {
            return Ok(Vec::new());
        }
        Err(e) => return Err(InspectError::C2pa(e)),
    };

    let store_json: serde_json::Value = serde_json::from_str(&reader.json())
        .map_err(|e| InspectError::Config(format!("manifest JSON parse error: {e}")))?;

    let mut entries: Vec<ChainEntry> = Vec::new();
    collect_entries(&store_json, &mut entries);
    Ok(entries)
}

/// Returns `true` if the file contains any C2PA manifest data.
#[must_use] 
pub fn has_manifest(path: &Path) -> bool {
    c2pa::Reader::from_file(path).is_ok()
}

/// Returns the full manifest store as a pretty-printed JSON string.
pub fn manifest_json(path: &Path) -> Result<String, InspectError> {
    let reader = c2pa::Reader::from_file(path).map_err(InspectError::C2pa)?;
    let val: serde_json::Value = serde_json::from_str(&reader.json())
        .map_err(|e| InspectError::Config(format!("manifest JSON parse: {e}")))?;
    serde_json::to_string_pretty(&val)
        .map_err(|e| InspectError::Config(format!("JSON serialize: {e}")))
}

/// Returns the most recent signer name and timestamp from the active manifest.
/// Used for the ingest log entry in the "has manifest" case.
pub fn last_signer(path: &Path) -> Result<Option<(String, Option<String>)>, InspectError> {
    let chain = read_chain(path)?;
    Ok(chain.last().map(|e| (e.signer_name.clone(), e.signed_at.clone())))
}

// Walk the manifest store JSON and collect chain entries.
// The store JSON has an `active_manifest` key and a `manifests` map.
// We walk from the active manifest back through ingredients.
fn collect_entries(store: &serde_json::Value, out: &mut Vec<ChainEntry>) {
    let Some(manifests) = store.get("manifests").and_then(|m| m.as_object()) else { return };
    let Some(active_id) = store.get("active_manifest").and_then(|v| v.as_str()) else { return };

    // Walk the chain recursively: ingredients first, then the active manifest.
    walk_manifest(active_id, manifests, out, &mut std::collections::HashSet::new());
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

    let Some(manifest) = manifests.get(id) else { return };

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

    let signer_name = sig_info
        .and_then(|s| s.get("issuer"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            manifest.get("claim_generator").and_then(|v| v.as_str())
        })
        .unwrap_or("Unknown")
        .to_string();

    let issuer = sig_info
        .and_then(|s| s.get("issuer"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let signed_at = sig_info
        .and_then(|s| s.get("time"))
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string);

    let algorithm = sig_info
        .and_then(|s| s.get("alg"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Derive trust status from validation_status codes if present.
    let trust_status = derive_trust(manifest);

    ChainEntry { signer_name, issuer, signed_at, trust_status, algorithm }
}

fn derive_trust(manifest: &serde_json::Value) -> TrustStatus {
    let Some(statuses) = manifest.get("validation_status").and_then(|v| v.as_array()) else {
        return TrustStatus::Unknown;
    };

    let codes: Vec<&str> = statuses
        .iter()
        .filter_map(|s| s.get("code").and_then(|v| v.as_str()))
        .collect();

    if codes.iter().any(|c| c.contains("revoked")) {
        return TrustStatus::Revoked;
    }
    if codes.iter().any(|c| c.contains("mismatch") || c.contains("failed")) {
        return TrustStatus::Invalid;
    }
    if codes.contains(&"signingCredential.trusted") {
        return TrustStatus::Trusted;
    }
    TrustStatus::Untrusted
}

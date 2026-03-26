use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::c2pa::{
    config::UmrsConfig,
    error::InspectError,
    manifest,
    signer,
};

/// Result of ingesting a file into UMRS.
#[derive(Debug)]
pub struct IngestResult {
    /// Original file path.
    pub source_path: PathBuf,

    /// Output file path (signed copy).
    pub output_path: PathBuf,

    /// SHA-256 hex digest of the source file bytes, computed at ingest time.
    pub sha256: String,

    /// Whether the file had an existing C2PA manifest on arrival.
    pub had_manifest: bool,

    /// The C2PA action label applied by UMRS (acquired or published).
    pub action: String,

    /// Signer name of the previous last entry in the chain (if any).
    pub previous_signer: Option<String>,

    /// Signing timestamp of the previous last entry (if any).
    pub previous_signed_at: Option<String>,

    /// Whether UMRS signed with an ephemeral (test) cert.
    pub is_ephemeral: bool,
}

/// Ingest a file: compute its SHA-256, read any existing manifest, sign it,
/// and write the result to `output_path`.
///
/// If `output_path` is `None`, a default path is derived by appending
/// `_umrs_signed` before the extension.
pub fn ingest_file(
    source_path: &Path,
    output_path: Option<&Path>,
    config: &UmrsConfig,
) -> Result<IngestResult, InspectError> {
    // 1. Compute SHA-256 of source file bytes.
    let sha256 = sha256_hex(source_path)?;

    // 2. Check for existing manifest.
    let had_manifest = manifest::has_manifest(source_path);
    let (previous_signer, previous_signed_at) = if had_manifest {
        match manifest::last_signer(source_path)? {
            Some((name, ts)) => (Some(name), ts),
            None             => (None, None),
        }
    } else {
        (None, None)
    };

    // 3. Choose action label and reason.
    let (action, reason) = if had_manifest {
        (
            config.policy.signed_action.clone(),
            config.policy.signed_reason.clone(),
        )
    } else {
        (
            config.policy.unsigned_action.clone(),
            config.policy.unsigned_reason.clone(),
        )
    };

    // 4. Resolve signing material.
    let signer_mode = signer::resolve_signer_mode(
        &config.identity,
        config.timestamp.tsa_url.as_deref(),
    )?;
    let is_ephemeral = signer::is_ephemeral(&signer_mode);
    let signer = signer::build_signer(&signer_mode)?;

    // 5. Build the C2PA manifest.
    let format = mime_for_path(source_path);
    let mut builder = c2pa::Builder::new();
    {
        let mut cgi = c2pa::ClaimGeneratorInfo::default();
        cgi.name.clone_from(&config.identity.claim_generator);
        builder.set_claim_generator_info(cgi);
    }

    // Action assertion.
    let action_assertion = serde_json::json!({
        "actions": [
            {
                "action": action,
                "reason": reason,
                "softwareAgent": config.identity.claim_generator,
            }
        ]
    });
    builder
        .add_assertion("c2pa.actions", &action_assertion)
        .map_err(InspectError::C2pa)?;

    // If there's an existing manifest, embed it as an ingredient.
    if had_manifest {
        let ingredient = c2pa::Ingredient::from_file(source_path)
            .map_err(InspectError::C2pa)?;
        builder.add_ingredient(ingredient);
    }

    // 6. Sign and write output.
    let out_path = match output_path {
        Some(p) => p.to_path_buf(),
        None    => derive_output_path(source_path),
    };

    let source_bytes = std::fs::read(source_path).map_err(InspectError::Io)?;
    let mut out_file = std::fs::File::create(&out_path).map_err(InspectError::Io)?;

    builder
        .sign(signer.as_ref(), &format, &mut std::io::Cursor::new(source_bytes), &mut out_file)
        .map_err(InspectError::C2pa)?;

    // 7. Emit structured log entry.
    if had_manifest {
        log::info!(
            target: "umrs",
            "ingest file=\"{}\" sha256=\"{}\" previous_signer=\"{}\" signed_at=\"{}\" action={}",
            source_path.display(),
            sha256,
            previous_signer.as_deref().unwrap_or("unknown"),
            previous_signed_at.as_deref().unwrap_or("unknown"),
            action,
        );
    } else {
        log::info!(
            target: "umrs",
            "ingest file=\"{}\" sha256=\"{}\" manifest=none action={}",
            source_path.display(),
            sha256,
            action,
        );
    }

    Ok(IngestResult {
        source_path: source_path.to_path_buf(),
        output_path: out_path,
        sha256,
        had_manifest,
        action,
        previous_signer,
        previous_signed_at,
        is_ephemeral,
    })
}

/// Compute the SHA-256 hex digest of the file at `path`.
pub fn sha256_hex(path: &Path) -> Result<String, InspectError> {
    let bytes = std::fs::read(path).map_err(InspectError::Io)?;
    let digest = Sha256::digest(&bytes);
    Ok(hex::encode(digest))
}

/// Derive a default output path by inserting `_umrs_signed` before the extension.
fn derive_output_path(source: &Path) -> PathBuf {
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    let ext  = source.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let name = format!("{stem}_umrs_signed{ext}");
    source.with_file_name(name)
}

/// Best-effort MIME type from file extension.
fn mime_for_path(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()).map(str::to_lowercase).as_deref() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png")                => "image/png",
        Some("webp")               => "image/webp",
        Some("tiff" | "tif") => "image/tiff",
        Some("avif")               => "image/avif",
        Some("heic" | "heif")=> "image/heic",
        Some("mp4")                => "video/mp4",
        Some("mov")                => "video/quicktime",
        Some("wav")                => "audio/wav",
        Some("mp3")                => "audio/mpeg",
        Some("pdf")                => "application/pdf",
        _                          => "application/octet-stream",
    }
    .to_string()
}

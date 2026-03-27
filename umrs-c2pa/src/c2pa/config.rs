// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jamie Adams

use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::c2pa::error::InspectError;

/// Top-level UMRS configuration loaded from `umrs-c2pa.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UmrsConfig {
    #[serde(default)]
    pub identity: IdentityConfig,

    #[serde(default)]
    pub timestamp: TimestampConfig,

    #[serde(default)]
    pub policy: PolicyConfig,

    #[serde(default)]
    pub trust: TrustConfig,

    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Identity and signing credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    /// Human-readable name embedded in every manifest produced by this system.
    #[serde(default = "default_claim_generator")]
    pub claim_generator: String,

    /// Organization name for display in chain-of-custody reports.
    #[serde(default = "default_organization")]
    pub organization: String,

    /// Path to PEM-encoded certificate chain (leaf first, root last).
    /// If absent, ephemeral self-signed cert is generated at runtime (test mode).
    pub cert_chain: Option<PathBuf>,

    /// Path to PEM-encoded private key corresponding to the leaf certificate.
    /// If absent, ephemeral self-signed cert is generated at runtime (test mode).
    pub private_key: Option<PathBuf>,

    /// Signing algorithm. Must be in the FIPS-safe set.
    /// Allowed: es256 | es384 | es512 | ps256 | ps384 | ps512
    /// ed25519 is intentionally excluded — unreliable on FIPS RHEL.
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
}

/// Optional Time Stamp Authority configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimestampConfig {
    /// TSA URL for trusted signing timestamps (e.g. "<http://timestamp.digicert.com>").
    /// Omit to sign without a TSA timestamp.
    pub tsa_url: Option<String>,
}

/// Ingest policy — action labels and reason strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// C2PA action label for files arriving without an existing manifest.
    #[serde(default = "default_unsigned_action")]
    pub unsigned_action: String,

    /// Reason string embedded in the action assertion for unsigned files.
    #[serde(default = "default_unsigned_reason")]
    pub unsigned_reason: String,

    /// C2PA action label for files arriving with an existing manifest.
    #[serde(default = "default_signed_action")]
    pub signed_action: String,

    /// Reason string embedded in the action assertion for signed files.
    #[serde(default = "default_signed_reason")]
    pub signed_reason: String,
}

/// Trust list configuration for C2PA signature validation.
///
/// All paths are fully configurable — no hardcoded default location.
/// See `docs/trust-maintenance.md` for update procedures.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrustConfig {
    /// Path to PEM bundle of root CA certificates (C2PA official or org-specific).
    /// Operator updates this file manually.
    pub trust_anchors: Option<PathBuf>,

    /// Path to PEM bundle of additional user/org root CAs.
    pub user_anchors: Option<PathBuf>,

    /// Path to end-entity certificate allowlist (PEM or base64 SHA-256 hashes).
    pub allowed_list: Option<PathBuf>,

    /// Path to EKU OID configuration file (one OID per line, `//` comments).
    pub trust_config: Option<PathBuf>,

    /// Enable trust validation. Defaults to true when any trust file is configured.
    #[serde(default = "default_verify_trust")]
    pub verify_trust: bool,

    /// OCSP responder URL (skeleton — not fully implemented yet).
    /// Organizations can point this to their own OCSP server when ready.
    pub ocsp_responder: Option<String>,
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable or disable all logging output.
    #[serde(default = "default_logging_enabled")]
    pub enabled: bool,

    /// Minimum log level: off | error | warn | info | debug | trace
    #[serde(default = "default_log_level")]
    pub level: String,
}

impl UmrsConfig {
    /// Load configuration from a TOML file at `path`.
    ///
    /// # Errors
    ///
    /// Returns `InspectError::Io` if the file cannot be read, or
    /// `InspectError::Config` if the TOML is malformed.
    pub fn load(path: &Path) -> Result<Self, InspectError> {
        let text = std::fs::read_to_string(path).map_err(InspectError::Io)?;
        toml::from_str(&text).map_err(|e| InspectError::Config(e.to_string()))
    }

    /// Return the effective `LevelFilter` for the logging backend.
    #[must_use]
    pub fn log_level_filter(&self) -> LevelFilter {
        if !self.logging.enabled {
            return LevelFilter::Off;
        }
        match self.logging.level.to_lowercase().as_str() {
            "off" => LevelFilter::Off,
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        }
    }

    /// Returns `true` if the identity config has cert + key configured (production mode).
    #[must_use]
    pub fn has_credentials(&self) -> bool {
        self.identity.cert_chain.is_some() && self.identity.private_key.is_some()
    }

    /// Returns `true` if any trust list files are configured.
    #[must_use]
    pub fn has_trust_config(&self) -> bool {
        self.trust.trust_anchors.is_some()
            || self.trust.user_anchors.is_some()
            || self.trust.allowed_list.is_some()
    }
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            claim_generator: default_claim_generator(),
            organization: default_organization(),
            cert_chain: None,
            private_key: None,
            algorithm: default_algorithm(),
        }
    }
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            unsigned_action: default_unsigned_action(),
            unsigned_reason: default_unsigned_reason(),
            signed_action: default_signed_action(),
            signed_reason: default_signed_reason(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: default_logging_enabled(),
            level: default_log_level(),
        }
    }
}

// --- serde defaults ---

fn default_claim_generator() -> String {
    "UMRS Reference System/1.0".into()
}
fn default_organization() -> String {
    "UMRS".into()
}
fn default_algorithm() -> String {
    "es256".into()
}
fn default_unsigned_action() -> String {
    "c2pa.acquired".into()
}
fn default_signed_action() -> String {
    "c2pa.published".into()
}
fn default_logging_enabled() -> bool {
    true
}
fn default_log_level() -> String {
    "info".into()
}

fn default_verify_trust() -> bool {
    true
}

fn default_unsigned_reason() -> String {
    "Received at UMRS trusted ingest dropbox. Origin unknown. No modifications made.".into()
}

fn default_signed_reason() -> String {
    "Received at UMRS trusted ingest dropbox with existing provenance. No modifications made."
        .into()
}

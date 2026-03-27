// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jamie Adams

// verbose must be declared first so the `verbose!` macro is available
// to all sibling modules via `#[macro_export]`.
pub mod verbose;

pub mod config;
pub mod creds;
pub mod error;
pub mod ingest;
pub mod manifest;
pub mod report;
pub mod signer;
pub mod validate;

pub use config::UmrsConfig;
pub use error::InspectError;
pub use ingest::{ingest_file, sha256_hex};
pub use manifest::{chain_json, has_manifest, manifest_json, read_chain};
pub use report::{print_chain, print_chain_readonly, print_validation_report};
pub use signer::ALLOWED_ALGORITHMS;
pub use validate::validate_config;
pub use verbose::enable as enable_verbose;

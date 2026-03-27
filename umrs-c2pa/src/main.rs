// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jamie Adams

use umrs_c2pa::c2pa;
use umrs_c2pa::verbose;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use c2pa::{
    UmrsConfig,
    ingest::sha256_hex,
    manifest::read_chain,
    manifest_json,
    report::{print_chain, print_chain_readonly, print_validation_report},
    validate::validate_config,
};

/// UMRS media inspection and C2PA signing tool.
///
/// Inspect a file:   umrs-c2pa photo.jpg
/// Sign a file:      umrs-c2pa --sign photo.jpg
/// Manage creds:     umrs-c2pa creds generate --output ./certs
/// Manage config:    umrs-c2pa config validate
#[derive(Parser)]
#[command(name = "umrs-c2pa", version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)] // CLI flags — not a state machine
struct Cli {
    /// Path to UMRS configuration file.
    #[arg(long, global = true, default_value = "umrs-c2pa.toml")]
    config: PathBuf,

    /// Show step-by-step progress on stderr.
    #[arg(long, short, global = true)]
    verbose: bool,

    // ── default action: inspect/sign a file ──────────────────────────────
    /// Media file to inspect or sign.
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Sign (ingest) the file and record a UMRS chain-of-custody entry.
    #[arg(long)]
    sign: bool,

    /// Emit the full manifest store as JSON instead of the formatted report.
    #[arg(long)]
    json: bool,

    /// Emit the UMRS-parsed evidence chain as JSON.
    #[arg(long)]
    chain_json: bool,

    /// Security marking to embed in the manifest (e.g. "CUI" or "CUI//SP-CTI//NOFORN").
    /// Only applies when --sign is used.
    #[arg(long)]
    marking: Option<String>,

    /// Write the signed output to this path (default: <file>_`umrs_signed`.<ext>).
    #[arg(long)]
    output: Option<PathBuf>,

    // ── subcommands for non-file operations ──────────────────────────────
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Configuration management.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Signing credential management (certificates and keys).
    Creds {
        #[command(subcommand)]
        action: CredsAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Validate the configuration file and signing credentials.
    Validate,

    /// Generate a commented starter configuration file.
    Generate {
        /// Write the generated config to this path (default: stdout).
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum CredsAction {
    /// Generate a new signing certificate and private key.
    ///
    /// Creates a self-signed certificate by default.
    /// Use --csr to generate a Certificate Signing Request for your CA.
    ///
    /// After generating, set the paths in umrs-c2pa.toml:
    ///   [identity]
    ///   `cert_chain`  = "/path/to/signing.pem"
    ///   `private_key` = "/path/to/signing.key"
    Generate {
        /// Directory to write signing.pem and signing.key.
        #[arg(long, default_value = ".")]
        output: PathBuf,

        /// Generate a CSR instead of a self-signed certificate.
        #[arg(long)]
        csr: bool,

        /// Certificate validity in days (ignored with --csr). Default: 365.
        #[arg(long, default_value = "365")]
        days: u32,
    },

    /// Validate the configured signing credentials.
    ///
    /// Checks that cert and key files exist, are valid PEM, match each other,
    /// and reports certificate details (subject, issuer, validity, algorithm).
    Validate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Enable verbose console output if requested.
    if cli.verbose {
        c2pa::enable_verbose();
    }

    // Load config — fall back to defaults if the file doesn't exist.
    let config = if cli.config.exists() {
        verbose!("Loading config from {}", cli.config.display());
        UmrsConfig::load(&cli.config)
            .with_context(|| format!("Failed to load config: {}", cli.config.display()))?
    } else {
        verbose!("No config file at {} — using defaults", cli.config.display());
        UmrsConfig::default()
    };

    // Initialize logging — journald.
    systemd_journal_logger::JournalLog::new()
        .expect("Failed to connect to journald")
        .with_syslog_identifier("umrs".to_string())
        .install()
        .expect("Failed to initialize journald logger");
    log::set_max_level(config.log_level_filter());

    // Dispatch: subcommand takes priority, otherwise inspect a file.
    match cli.command {
        Some(Commands::Config {
            action,
        }) => match action {
            ConfigAction::Validate => {
                cmd_config_validate(&config);
            }
            ConfigAction::Generate {
                output,
            } => {
                cmd_config_generate(output.as_deref())?;
            }
        },
        Some(Commands::Creds {
            action,
        }) => match action {
            CredsAction::Generate {
                output,
                csr,
                days,
            } => {
                cmd_creds_generate(&config, &output, csr, days)?;
            }
            CredsAction::Validate => {
                cmd_creds_validate(&config);
            }
        },
        None => {
            // Default action: inspect or sign a file.
            let Some(file) = cli.file else {
                // No file and no subcommand — show help.
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                return Ok(());
            };
            cmd_c2pa(
                &file,
                cli.sign,
                cli.json,
                cli.chain_json,
                cli.marking.as_deref(),
                cli.output.as_deref(),
                &config,
            )?;
        }
    }

    Ok(())
}

// ── inspect / sign a file ────────────────────────────────────────────────────

fn cmd_c2pa(
    file: &std::path::Path,
    sign: bool,
    json: bool,
    chain_json: bool,
    marking: Option<&str>,
    output: Option<&std::path::Path>,
    config: &UmrsConfig,
) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
    }
    verbose!("Found file: {}", file.display());

    // JSON mode — emit raw manifest store and exit.
    if json {
        verbose!("Reading raw manifest store as JSON...");
        match manifest_json(file) {
            Ok(j) => println!("{j}"),
            Err(e) => eprintln!("No manifest or read error: {e}"),
        }
        return Ok(());
    }

    // Chain JSON mode — emit parsed evidence chain as JSON and exit.
    if chain_json {
        verbose!("Reading chain of custody as JSON...");
        match c2pa::chain_json(file) {
            Ok(j) => println!("{j}"),
            Err(e) => eprintln!("No manifest or read error: {e}"),
        }
        return Ok(());
    }

    verbose!("Computing SHA-256 digest...");
    let sha256 =
        sha256_hex(file).with_context(|| format!("Failed to hash file: {}", file.display()))?;
    verbose!("SHA-256: {}", sha256);

    if sign {
        verbose!("Signing mode — ingesting file into UMRS chain of custody...");
        if let Some(m) = marking {
            verbose!("Security marking: {}", m);
        }

        // Ingest mode: sign the file, display the resulting chain.
        let result = c2pa::ingest_file(file, output, marking, config)
            .with_context(|| format!("Ingest failed for: {}", file.display()))?;
        verbose!("Signed output written to: {}", result.output_path.display());

        verbose!("Reading chain of custody from signed output...");
        let chain = read_chain(&result.output_path)
            .with_context(|| "Failed to read chain from signed output")?;
        verbose!("Chain contains {} entries", chain.len());

        print_chain(&file.display().to_string(), &sha256, &chain, Some(&result));
    } else {
        verbose!("Read-only mode — inspecting existing chain of custody...");

        // Read-only mode: display the chain as-is.
        let chain = read_chain(file)
            .with_context(|| format!("Failed to read chain from: {}", file.display()))?;
        verbose!("Chain contains {} entries", chain.len());

        print_chain_readonly(&file.display().to_string(), &sha256, &chain);
    }

    Ok(())
}

// ── config validate ──────────────────────────────────────────────────────────

fn cmd_config_validate(config: &UmrsConfig) {
    verbose!("Running configuration preflight checks...");
    let results = validate_config(config);
    verbose!("{} checks completed", results.len());
    print_validation_report(&results);

    let failures = results.iter().filter(|r| r.status == c2pa::validate::CheckStatus::Fail).count();

    if failures > 0 {
        std::process::exit(1);
    }
}

// ── config generate ──────────────────────────────────────────────────────────

fn cmd_config_generate(output: Option<&std::path::Path>) -> Result<()> {
    let template = config_template();
    match output {
        Some(path) => {
            std::fs::write(path, template)
                .with_context(|| format!("Failed to write config to: {}", path.display()))?;
            println!("Config template written to: {}", path.display());
        }
        None => print!("{template}"),
    }
    Ok(())
}

fn config_template() -> &'static str {
    r#"# umrs-c2pa.toml — UMRS C2PA signing configuration
#
# Quick start:
#   1. umrs-c2pa creds generate --output ./certs    # create cert + key
#   2. Edit this file — set cert_chain and private_key paths
#   3. umrs-c2pa config validate                    # verify everything
#   4. umrs-c2pa --sign photo.jpg                   # sign your first file
#
# Run `umrs-c2pa config validate` to verify before use.
# Run `umrs-c2pa config generate --output <path>` to regenerate this template.

[identity]
# Human-readable name embedded in every manifest produced by this system.
claim_generator = "UMRS Reference System/1.0"

# Organization name for display in chain-of-custody reports.
organization = "Your Organization"

# Path to PEM-encoded signing certificate chain (leaf cert first, root last).
# Generate with: umrs-c2pa creds generate --output ./certs
# If omitted, an ephemeral self-signed cert is generated at runtime (test mode).
#cert_chain = "./certs/signing.pem"

# Path to PEM-encoded private key corresponding to the leaf certificate.
#private_key = "./certs/signing.key"

# Signing algorithm. Must be in the FIPS-safe set.
# Allowed : es256 | es384 | es512 | ps256 | ps384 | ps512
# Excluded: ed25519 — unreliable on FIPS-enabled RHEL
# Strongest FIPS+C2PA intersection: es512
algorithm = "es256"

[timestamp]
# Time Stamp Authority URL for trusted signing timestamps.
# Omit (or comment out) to sign without a TSA timestamp.
# tsa_url = "http://timestamp.digicert.com"

[policy]
# Action label and reason for files arriving WITHOUT an existing C2PA manifest.
# c2pa.acquired = "we received this; we are not the creator"
unsigned_action = "c2pa.acquired"
unsigned_reason = "Received at UMRS trusted ingest dropbox. Origin unknown. No modifications made."

# Action label and reason for files arriving WITH an existing C2PA manifest.
# c2pa.published = "we forwarded this as-is"
signed_action = "c2pa.published"
signed_reason = "Received at UMRS trusted ingest dropbox with existing provenance. No modifications made."

[trust]
# Trust list configuration for C2PA signature validation.
# All paths are configurable — no hardcoded default location.
# See docs/trust-maintenance.md for setup and update procedures.

# Path to PEM bundle of C2PA root CA certificates.
# Operator updates this file manually (works air-gapped and connected).
#trust_anchors = "/path/to/trust/c2pa-anchors.pem"

# Path to PEM bundle of your organization's root CAs (optional).
#user_anchors = "/path/to/trust/org-roots.pem"

# Path to end-entity certificate allowlist (optional).
# Directly trust specific signer certs without chain validation.
#allowed_list = "/path/to/trust/allowed-signers.pem"

# Path to EKU OID filter file (optional). One OID per line, // comments.
#trust_config = "/path/to/trust/ekus.cfg"

# Enable trust validation (default: true).
verify_trust = true

# OCSP responder URL (skeleton — not fully implemented yet).
# Organizations can point this to their own OCSP server when ready.
#ocsp_responder = "http://ocsp.internal.example.com"

[logging]
# Enable or disable all logging output.
enabled = true

# Minimum log level: off | error | warn | info | debug | trace
# Set to "off" in production if journald volume is a concern.
level = "info"
"#
}

// ── creds generate ───────────────────────────────────────────────────────────

fn cmd_creds_generate(
    config: &UmrsConfig,
    output_dir: &std::path::Path,
    csr: bool,
    days: u32,
) -> Result<()> {
    verbose!("Generating credentials in: {}", output_dir.display());

    // Create output directory if it doesn't exist.
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create directory: {}", output_dir.display()))?;
    }

    let result =
        c2pa::creds::generate(config, csr, days).with_context(|| "Credential generation failed")?;

    let cert_name = if result.is_csr {
        "signing.csr"
    } else {
        "signing.pem"
    };
    let cert_path = output_dir.join(cert_name);
    let key_path = output_dir.join("signing.key");

    // Safety: refuse to overwrite existing files.
    if cert_path.exists() {
        anyhow::bail!(
            "{} already exists at {}. Remove it first or choose a different --output directory.",
            cert_name,
            cert_path.display()
        );
    }
    if key_path.exists() {
        anyhow::bail!(
            "signing.key already exists at {}. Remove it first or choose a different --output directory.",
            key_path.display()
        );
    }

    std::fs::write(&cert_path, &result.cert_or_csr_pem)
        .with_context(|| format!("Failed to write {}", cert_path.display()))?;
    std::fs::write(&key_path, &result.key_pem)
        .with_context(|| format!("Failed to write {}", key_path.display()))?;

    // Restrict key file permissions (best-effort on non-Unix).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
    }

    println!("{}", result.summary);
    println!();
    println!("Files written:");
    println!("  {} : {}", cert_name, cert_path.display());
    println!("  signing.key : {}", key_path.display());
    println!();
    println!("Next step — add these to your umrs-c2pa.toml:");
    println!();
    if result.is_csr {
        println!("  [identity]");
        println!("  # After your CA signs the CSR, replace signing.csr with the signed cert:");
        println!("  cert_chain  = \"{}\"", output_dir.join("signing.pem").display());
        println!("  private_key = \"{}\"", key_path.display());
    } else {
        println!("  [identity]");
        println!("  cert_chain  = \"{}\"", cert_path.display());
        println!("  private_key = \"{}\"", key_path.display());
    }
    println!();
    println!("Then run: umrs-c2pa creds validate");

    Ok(())
}

// ── creds validate ───────────────────────────────────────────────────────────

fn cmd_creds_validate(config: &UmrsConfig) {
    verbose!("Validating configured signing credentials...");
    let checks = c2pa::creds::validate(config);

    let pass_mark = "\u{2714}"; // checkmark
    let fail_mark = "\u{2718}"; // x-mark

    println!();
    println!("Credential Validation");
    println!("{}", "\u{2501}".repeat(56));

    for check in &checks {
        let (mark, label) = if check.ok {
            (pass_mark, "PASS")
        } else {
            (fail_mark, "FAIL")
        };
        println!("  {mark} [{label}] {}: {}", check.check, check.message);
    }

    println!("{}", "\u{2501}".repeat(56));

    let failures = checks.iter().filter(|c| !c.ok).count();
    if failures > 0 {
        println!("{failures} check(s) failed.");
        println!();
        println!("To generate new credentials: umrs-c2pa creds generate --output /path/to/certs/");
        std::process::exit(1);
    } else {
        println!("All checks passed.");
    }
}

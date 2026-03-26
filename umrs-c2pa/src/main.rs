use umrs_c2pa::c2pa;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use c2pa::{
    ingest::sha256_hex,
    manifest::read_chain,
    manifest_json,
    report::{print_chain, print_chain_readonly, print_validation_report},
    validate::validate_config,
    UmrsConfig,
};

/// UMRS media inspection and C2PA signing tool.
#[derive(Parser)]
#[command(name = "inspect", version, about, long_about = None)]
struct Cli {
    /// Path to UMRS configuration file.
    #[arg(long, global = true, default_value = "umrs-c2pa.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// C2PA manifest inspection and ingest signing.
    C2pa {
        /// Media file to inspect or sign.
        file: PathBuf,

        /// Sign (ingest) the file and record a UMRS chain-of-custody entry.
        #[arg(long)]
        sign: bool,

        /// Emit the full manifest store as JSON instead of the formatted report.
        #[arg(long)]
        json: bool,

        /// Write the signed output to this path (default: <file>_`umrs_signed`.<ext>).
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Configuration management.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config — fall back to defaults if the file doesn't exist.
    let config = if cli.config.exists() {
        UmrsConfig::load(&cli.config)
            .with_context(|| format!("Failed to load config: {}", cli.config.display()))?
    } else {
        UmrsConfig::default()
    };

    // Initialize logging — journald.
    systemd_journal_logger::JournalLog::new()
        .expect("Failed to connect to journald")
        .with_syslog_identifier("umrs".to_string())
        .install()
        .expect("Failed to initialize journald logger");
    log::set_max_level(config.log_level_filter());

    match cli.command {
        Commands::C2pa { file, sign, json, output } => {
            cmd_c2pa(&file, sign, json, output.as_deref(), &config)?;
        }
        Commands::Config { action } => match action {
            ConfigAction::Validate => {
                cmd_config_validate(&config);
            }
            ConfigAction::Generate { output } => {
                cmd_config_generate(output.as_deref())?;
            }
        },
    }

    Ok(())
}

// ── c2pa subcommand ────────────────────────────────────────────────────────────

fn cmd_c2pa(
    file: &std::path::Path,
    sign: bool,
    json: bool,
    output: Option<&std::path::Path>,
    config: &UmrsConfig,
) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
    }

    // JSON mode — emit raw manifest store and exit.
    if json {
        match manifest_json(file) {
            Ok(j)  => println!("{j}"),
            Err(e) => eprintln!("No manifest or read error: {e}"),
        }
        return Ok(());
    }

    let sha256 = sha256_hex(file)
        .with_context(|| format!("Failed to hash file: {}", file.display()))?;

    if sign {
        // Ingest mode: sign the file, display the resulting chain.
        let result = c2pa::ingest_file(file, output, config)
            .with_context(|| format!("Ingest failed for: {}", file.display()))?;

        let chain = read_chain(&result.output_path)
            .with_context(|| "Failed to read chain from signed output")?;

        print_chain(
            &file.display().to_string(),
            &sha256,
            &chain,
            Some(&result),
        );
    } else {
        // Read-only mode: display the chain as-is.
        let chain = read_chain(file)
            .with_context(|| format!("Failed to read chain from: {}", file.display()))?;

        print_chain_readonly(
            &file.display().to_string(),
            &sha256,
            &chain,
        );
    }

    Ok(())
}

// ── config validate ────────────────────────────────────────────────────────────

fn cmd_config_validate(config: &UmrsConfig) {
    let results = validate_config(config);
    print_validation_report(&results);

    let failures = results
        .iter()
        .filter(|r| r.status == c2pa::validate::CheckStatus::Fail)
        .count();

    if failures > 0 {
        std::process::exit(1);
    }
}

// ── config generate ────────────────────────────────────────────────────────────

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
# Run `inspect config validate --config <this file>` to verify before use.
# Run `inspect config generate --output <path>` to regenerate this template.

[identity]
# Human-readable name embedded in every manifest produced by this system.
claim_generator = "UMRS Reference System/1.0"

# Organization name for display in chain-of-custody reports.
organization = "Your Organization"

# Path to PEM-encoded signing certificate chain (leaf cert first, root last).
# If omitted, an ephemeral self-signed cert is generated at runtime (test mode).
# Manifests signed in test mode will be marked UNTRUSTED by external validators.
#cert_chain = "/etc/umrs/certs/signing.pem"

# Path to PEM-encoded private key corresponding to the leaf certificate.
#private_key = "/etc/umrs/certs/signing.key"

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

[logging]
# Enable or disable all logging output.
enabled = true

# Minimum log level: off | error | warn | info | debug | trace
# Set to "off" in production if journald volume is a concern.
level = "info"
"#
}

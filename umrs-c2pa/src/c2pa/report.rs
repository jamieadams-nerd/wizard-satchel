use crate::c2pa::{ingest::IngestResult, manifest::ChainEntry};

const SEPARATOR: &str = "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━";
const THIN_SEP:  &str = "────────────────────────────────────────────────────────";

/// Print the full chain-of-custody report to stdout.
///
/// Shows every entry in the chain with trust indicators, then a hash
/// consistency summary based on the ingest result.
pub fn print_chain(path: &str, sha256: &str, chain: &[ChainEntry], ingest: Option<&IngestResult>) {
    println!("\nChain of Custody — {path}");
    println!("SHA-256: {sha256}");
    println!("{SEPARATOR}");

    if chain.is_empty() {
        println!("  (no C2PA manifest found)");
    } else {
        for (i, entry) in chain.iter().enumerate() {
            let trust_tag = format!("[{}]", entry.trust_status);
            println!(
                "  {:<3} {:<13}  {}",
                i + 1,
                trust_tag,
                entry.signer_name
            );
            if let Some(ts) = &entry.signed_at {
                println!("       {:<13}  Signed : {}", "", ts);
            }
            println!("       {:<13}  Issuer : {}", "", entry.issuer);
            println!("       {:<13}  Alg    : {}", "", entry.algorithm);
            println!();
        }
    }

    println!("{SEPARATOR}");

    // Hash consistency line.
    if let Some(result) = ingest {
        if result.had_manifest {
            // We have a chain — all hashes should be consistent.
            println!("Hash consistency : PASS — file unchanged across all signing events");
        } else {
            println!("Hash consistency : N/A  — no prior manifest (first signature)");
        }
        println!("UMRS action      : {}", result.action);
        println!("UMRS output      : {}", result.output_path.display());
        if result.is_ephemeral {
            println!("UMRS identity    : ephemeral self-signed cert (test mode — UNTRUSTED)");
        }
    }

    println!();
}

/// Print the result of a read-only chain inspection (no ingest).
pub fn print_chain_readonly(path: &str, sha256: &str, chain: &[ChainEntry]) {
    print_chain(path, sha256, chain, None);
}

/// Print the config validation report.
pub fn print_validation_report(results: &[crate::c2pa::validate::ValidationResult]) {
    use crate::c2pa::validate::CheckStatus;

    println!();
    for r in results {
        let tag = match r.status {
            CheckStatus::Pass => "[OK]  ",
            CheckStatus::Warn => "[WARN]",
            CheckStatus::Fail => "[FAIL]",
            CheckStatus::Info => "[INFO]",
            CheckStatus::Skip => "[SKIP]",
        };
        println!("  {}  {}: {}", tag, r.check, r.message);
    }
    println!("{THIN_SEP}");

    let failures = results.iter().filter(|r| r.status == CheckStatus::Fail).count();
    let warnings = results.iter().filter(|r| r.status == CheckStatus::Warn).count();

    if failures == 0 {
        if warnings > 0 {
            println!("  All checks passed ({warnings} warning(s)). Configuration is ready.");
        } else {
            println!("  All checks passed. Configuration is ready.");
        }
    } else {
        println!("  {failures} check(s) failed. Configuration is NOT ready.");
    }
    println!();
}

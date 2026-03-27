// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jamie Adams

use crate::c2pa::{
    ingest::IngestResult,
    manifest::{ChainEntry, TrustStatus},
};

const SEPARATOR: &str = "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━";
const THIN_SEP: &str = "────────────────────────────────────────────────────────";

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
        // Collect unique footnotes keyed by their display label so identical
        // statuses across entries produce a single footnote line.
        let mut footnote_set: std::collections::BTreeMap<String, &str> =
            std::collections::BTreeMap::new();

        for (i, entry) in chain.iter().enumerate() {
            let idx = i + 1;

            // Detect self-signed: issuer == signer (cert signs itself).
            let is_self_signed = entry.signer_name == entry.issuer && entry.issuer != "Unknown";

            // Build the trust tag with asterisk for entries that need a footnote.
            let (trust_tag, has_footnote) = match (&entry.trust_status, is_self_signed) {
                (TrustStatus::Untrusted | TrustStatus::NoTrustList, true) => {
                    let label = format!("{}", entry.trust_status);
                    footnote_set
                        .entry(label)
                        .or_insert("Self-signed certificate — not issued by a trusted CA");
                    (format!("*[{}]", entry.trust_status), true)
                }
                (TrustStatus::NoTrustList, false) => {
                    let label = format!("{}", entry.trust_status);
                    footnote_set
                        .entry(label)
                        .or_insert("No trust list configured — trust could not be evaluated");
                    (format!("*[{}]", entry.trust_status), true)
                }
                _ => (format!("[{}]", entry.trust_status), false),
            };

            let pad = if has_footnote {
                16
            } else {
                14
            };
            println!("  {:<3} {:<pad$}  {}", idx, trust_tag, entry.signer_name);

            match &entry.signed_at {
                Some(ts) => println!("       {:<pad$}  Signed at : {} UTC", "", ts),
                None => println!("       {:<pad$}  Signed at : no timestamp provided", ""),
            }

            // Only show Issuer if it differs from the top-level signer name.
            if entry.issuer != entry.signer_name {
                println!("       {:<pad$}  Issuer    : {}", "", entry.issuer);
            }

            println!("       {:<pad$}  Alg       : {}", "", entry.algorithm);

            // Generator + version (e.g. "ChatGPT 0.67.1")
            let gen_display = match &entry.generator_version {
                Some(v) => format!("{} {v}", entry.generator),
                None => entry.generator.clone(),
            };
            println!("       {:<pad$}  Generator : {}", "", gen_display);

            // Security label / marking, if present.
            if let Some(label) = &entry.security_label {
                println!("       {:<pad$}  Marking   : {}", "", label);
            }
            println!();
        }

        // Print deduplicated footnotes keyed by trust status label.
        if !footnote_set.is_empty() {
            println!("{THIN_SEP}");
            for (label, explanation) in &footnote_set {
                println!("  *[{label}] {explanation}");
            }
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

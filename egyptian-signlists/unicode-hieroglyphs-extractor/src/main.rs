use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::atomic::Ordering;

use anyhow::{Context, Result};
use serde::Serialize;

// Pull in shared infrastructure
use signlist_core::{VERBOSE};
use signlist_core::verbose;
use signlist_core::stderr_is_tty;

/* ============================================================
 * Data model
 * ============================================================
 */

#[derive(Debug, Serialize)]
struct HieroglyphUnicode {
    unicode_point: String,
    codepoint_hex: String,
    codepoint_dec: u32,
    char: String,
    unicode_name: String,
    unicode_id: String,
    family: String,
    is_format_control: bool,
}

fn is_format_control(name: &str, family: &str) -> bool {
    family == "H" && (
        name.contains("JOINER")
            || name.contains("INSERT")
            || name.contains("BEGIN")
            || name.contains("END")
            || name.contains("MIRROR")
            || name.contains("OVERLAY")
            || name.contains("ENCLOSURE")
            || name.contains("SEGMENT")
            || name.contains("BLANK")
    )
}

/* ============================================================
 * Main
 * ============================================================
 */

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let mut input: Option<String> = None;
    let mut output: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => {
                input = Some(
                    args.next().context("missing value for --input")?
                );
            }
            "--output" => {
                output = Some(
                    args.next().context("missing value for --output")?
                );
            }
            "--verbose" | "-v" => {
                VERBOSE.store(true, Ordering::Relaxed);
            }
            _ => {
                anyhow::bail!("unknown argument: {}", arg);
            }
        }
    }

    let input = input.context(
        "usage: unicode-hieroglyphs-extractor --input <file> --output <file> [--verbose]"
    )?;

    let output = output.context(
        "usage: unicode-hieroglyphs-extractor --input <file> --output <file> [--verbose]"
    )?;

    verbose!("reading input file: {}", input);
    verbose!("writing output file: {}", output);

    let file = File::open(&input)
        .with_context(|| format!("failed to open input file: {}", input))?;

    let reader = BufReader::new(file);
    let mut records: Vec<HieroglyphUnicode> = Vec::new();

    for (line_no, line) in reader.lines().enumerate() {
        let line = line
            .with_context(|| format!("failed reading line {}", line_no + 1))?;

        if !line.contains("EGYPTIAN HIEROGLYPH") {
            continue;
        }

        let mut parts = line.split_whitespace();
        let hex = parts.next().unwrap();
        let unicode_name = parts.collect::<Vec<_>>().join(" ");

        let unicode_id = unicode_name
            .split_whitespace()
            .last()
            .unwrap()
            .to_string();

        let family = unicode_id
            .chars()
            .next()
            .unwrap()
            .to_string();

        let codepoint_dec =
            u32::from_str_radix(hex, 16)
                .with_context(|| format!("invalid hex codepoint: {}", hex))?;

        let ch = char::from_u32(codepoint_dec)
            .unwrap_or('\u{FFFD}')
            .to_string();

        let is_format_control =
            is_format_control(&unicode_name, &family);

        verbose!(
            "line {}: {} {}",
            line_no + 1,
            unicode_id,
            unicode_name
        );

        records.push(HieroglyphUnicode {
            unicode_point: format!("U+{}", hex),
            codepoint_hex: hex.to_string(),
            codepoint_dec,
            char: ch,
            unicode_name,
            unicode_id,
            family,
            is_format_control,
        });
    }

    let json = serde_json::to_string_pretty(&records)?;
    std::fs::write(&output, json)
        .with_context(|| format!("failed to write output file: {}", output))?;

    println!(
        "Extracted {} Egyptian hieroglyph Unicode records.",
        records.len()
    );

    Ok(())
}


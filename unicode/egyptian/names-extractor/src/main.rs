use std::fs::File;
use std::io::{BufRead, BufReader};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct HieroglyphUnicode {
    /// "U+13000"
    unicode_point: String,

    /// "13000"
    codepoint_hex: String,

    /// 77824
    codepoint_dec: u32,

    /// "𓀀" (may not render everywhere)
    char: String,

    /// "EGYPTIAN HIEROGLYPH A001"
    unicode_name: String,

    /// "A001"
    unicode_id: String,

    /// "A", "D", "N", etc.
    family: String,

    /// True if this is a hieroglyphic *format/control* operator
    /// (joiners, inserts, begin/end markers, mirroring, etc.)
    is_format_control: bool,
}

fn is_format_control(name: &str, family: &str) -> bool {
    // Unicode puts hieroglyphic layout operators in family "H"
    // but not every H-sign is guaranteed to be a control forever,
    // so we also key off the Unicode name.
    family == "H"
        && (
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

fn main() -> std::io::Result<()> {
    let file = File::open("NamesList.txt")?;
    let reader = BufReader::new(file);

    let mut records: Vec<HieroglyphUnicode> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if !line.contains("EGYPTIAN HIEROGLYPH") {
            continue;
        }

        // Expected format:
        // <hex>\tEGYPTIAN HIEROGLYPH A001
        let mut parts = line.split_whitespace();
        let hex = parts.next().unwrap();
        let unicode_name = parts.collect::<Vec<_>>().join(" ");

        // Extract the trailing A001 / D021 / N005 / etc.
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

        let codepoint_dec = u32::from_str_radix(hex, 16).unwrap();

        let ch = char::from_u32(codepoint_dec)
            .unwrap_or('\u{FFFD}')
            .to_string();

        let is_format_control =
            is_format_control(&unicode_name, &family);

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

    let json = serde_json::to_string_pretty(&records).unwrap();
    std::fs::write("hieroglyphs_unicode.json", json)?;

    println!(
        "Extracted {} Egyptian hieroglyph Unicode records.",
        records.len()
    );

    Ok(())
}


use slint::{ModelRc, VecModel, SharedString};

mod unicode_blocks;
use unicode_blocks::*;

slint::include_modules!();

fn main() {
    let ui = UnicodeGrid::new().unwrap();

    //let block = &LATIN_EXTENDED_E;
    //let block = &BASIC_LATIN;
    //let block = &BOX_DRAWING;
    //let block = &RUNIC;
    let block = &BRAILLE_PATTERNS;

    // ---------------- glyphs ----------------
    let mut glyphs: Vec<SharedString> = Vec::new();
    let mut missing: Vec<bool> = Vec::new();

    for cp in block.start..=block.end {
        if let Some(ch) = char::from_u32(cp) {
            glyphs.push(ch.to_string().into());
            missing.push(false);
        } else {
            glyphs.push("�".into());
            missing.push(true);
        }
    }

    // ---------------- row labels ----------------
    let total = (block.end - block.start + 1) as usize;
    let rows = (total + 15) / 16;

    let mut row_labels: Vec<SharedString> = Vec::with_capacity(rows);

    for r in 0..rows {
        let base = block.start + (r as u32) * 16;
        row_labels.push(format!("0x{:04X}", base).into());
    }

    // ---------------- header ----------------
    ui.set_block_name(block.name.into());

    let range = format!(
        "0x{:04X} – 0x{:04X}",
        block.start,
        block.end
    );

    ui.set_block_range(range.into());
    ui.set_font_name("Menlo".into());


    // ---------------- models ----------------
    ui.set_glyphs(ModelRc::new(VecModel::from(glyphs)));
    ui.set_missing(ModelRc::new(VecModel::from(missing)));
    ui.set_row_labels(ModelRc::new(VecModel::from(row_labels)));

    ui.run().unwrap();
}


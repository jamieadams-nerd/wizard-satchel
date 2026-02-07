mod unicode_blocks;
use unicode_blocks::*;

use slint::{ModelRc, VecModel, SharedString};

slint::include_modules!();

fn main() {
    let ui = UnicodeGrid::new().unwrap();

    let block = &LATIN_1_SUPPLEMENT;   // ← change block here as needed

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

    ui.set_block_name(block.name.into());

    let range = format!(
        "0x{:04X} – 0x{:04X}",
        block.start,
        block.end
    );

    ui.set_block_range(range.into());
    //ui.set_font_name("Menlo".into());

    ui.set_glyphs(ModelRc::new(VecModel::from(glyphs)));
    ui.set_missing(ModelRc::new(VecModel::from(missing)));

    ui.run().unwrap();
}


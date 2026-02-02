use anyhow::{bail, Context, Result};
use std::fs;
use std::process::Command;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("setup") => setup(),
        _ => {
            eprintln!("Usage:");
            eprintln!("  cargo xtask setup");
            bail!("unknown xtask command");
        }
    }
}

fn setup() -> Result<()> {
    println!("== egyptian-signlists setup ==");

    ensure_dirs()?;
    build_binaries()?;
    run_extractors()?;
    run_correlator()?;
    copy_binaries()?;

    println!("== setup complete ==");
    Ok(())
}

fn ensure_dirs() -> Result<()> {
    for dir in ["generated", "reports", "dist"] {
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create directory `{}`", dir))?;
    }
    Ok(())
}

fn build_binaries() -> Result<()> {
    println!("building tools (release)");

    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "-p", "unicode-hieroglyphs-extractor",
            "-p", "gardiner-signlist-extractor",
            "-p", "signlist-correlator",
        ])
        .status()
        .context("failed to run cargo build")?;

    if !status.success() {
        bail!("cargo build failed");
    }

    Ok(())
}

fn run_extractors() -> Result<()> {
    println!("running unicode extractor");

    run_bin(
        "unicode-hieroglyphs-extractor",
        &[
            "--input", "data/unicode/NamesList.txt",
            "--output", "generated/hieroglyphs_unicode.json",
        ],
    )?;

    println!("running gardiner extractor");

    run_bin(
        "gardiner-signlist-extractor",
        &[
            "--input", "generated/jsesh.txt",
            "--output", "generated/jsesh_inventory.json",
        ],
    )?;

    Ok(())
}

fn run_bin(name: &str, args: &[&str]) -> Result<()> {
    let path = format!("target/release/{}", name);

    let status = Command::new(&path)
        .args(args)
        .status()
        .with_context(|| format!("failed to run {}", name))?;

    if !status.success() {
        bail!("{} failed", name);
    }

    Ok(())
}


fn run_correlator() -> Result<()> {
    println!("running correlator");

    run_bin(
        "signlist-correlator",
        &[
            "--unicode", "generated/hieroglyphs_unicode.json",
            "--jsesh", "generated/jsesh_inventory.json",
            "--output", "generated/signlist_merged.json",
            "--report", "reports/orphans.json",
        ],
    )
}


fn copy_binaries() -> Result<()> {
    println!("copying binaries to dist/");

    for bin in [
        "unicode-hieroglyphs-extractor",
        "gardiner-signlist-extractor",
        "signlist-correlator",
    ] {
        let src = format!("target/release/{}", bin);
        let dst = format!("dist/{}", bin);

        fs::copy(&src, &dst)
            .with_context(|| format!("failed to copy {} to dist/", bin))?;
    }

    Ok(())
}




// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Jamie Adams (a.k.a, Imodium Operator)

use serde::Serialize;

// Pull in shared infrastructure
use signlist_core::VERBOSE;
use signlist_core::verbose;
use signlist_core::stderr_is_tty;
use signlist_core::fail;

#[derive(Serialize)]
struct JseshSign {
    family: String,
    family_name: String,
    jsesh_code: String,
}

fn main() {
    use std::fs::read_to_string;
    use std::sync::atomic::Ordering;

    let mut args = std::env::args().skip(1);
    let mut input_path: Option<String> = None;
    let mut output_path: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--verbose" | "-v" => {
                VERBOSE.store(true, Ordering::Relaxed);
            }

            "--input" | "-i" => {
                let value = args
                    .next()
                    .unwrap_or_else(|| fail!(1, "missing value for {}", arg));

                if input_path.is_some() {
                    fail!(1, "duplicate {} argument", arg);
                }

                input_path = Some(value);
            }

            "--output" | "-o" => {
                let value = args
                    .next()
                    .unwrap_or_else(|| fail!(1, "missing value for {}", arg));

                if output_path.is_some() {
                    fail!(1, "duplicate {} argument", arg);
                }

                output_path = Some(value);
            }

            _ => {
                fail!(1, "unexpected argument: {}", arg);
            }
        }
    }

    let input_path = match input_path {
        Some(p) => p,
        None => fail!(
            0,
            "usage: gardiner-signlist-extractor [--verbose] --input <input-file> [--output <output-file>]"
        ),
    };

    let output_path = output_path.unwrap_or_else(|| "jsesh_inventory.json".to_string());

    verbose!("reading input file: {}", input_path);

    let text = match read_to_string(&input_path) {
        Ok(s) => s,
        Err(e) => {
            use std::io::ErrorKind::*;
            match e.kind() {
                NotFound => fail!(1, "input file not found: {}", input_path),
                PermissionDenied => fail!(1, "permission denied reading: {}", input_path),
                _ => fail!(1, "failed to read {}: {}", input_path, e),
            }
        }
    };

    let mut results = Vec::new();
    let mut current_family: Option<String> = None;
    let mut current_family_name: Option<String> = None;
    let mut awaiting_family_name = false;

    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(fam) = line.strip_suffix(" family") {
            verbose!(
                "line {}: \u{203B} Found family {}",
                line_no + 1,
                fam
            );
            current_family = Some(fam.to_string());
            current_family_name = None;
            awaiting_family_name = true;
            continue;
        }

        if awaiting_family_name {
            verbose!(
                "line {}:   family name = {}",
                line_no + 1,
                line
            );
            current_family_name = Some(line.to_string());
            awaiting_family_name = false;
            continue;
        }

        if let Some((_, code)) = line.split_once(' ') {
            if let (Some(f), Some(name)) = (&current_family, &current_family_name) {
                if code
                    .chars()
                    .next()
                    .map(|c| c.is_alphanumeric())
                    .unwrap_or(false)
                {
                    verbose!(
                        "line {}:   sign {} (family {})",
                        line_no + 1,
                        code,
                        f
                    );
                    results.push(JseshSign {
                        family: f.clone(),
                        family_name: name.clone(),
                        jsesh_code: code.to_string(),
                    });
                }
            }
        }
    }

    verbose!(" ");
    verbose!("\u{26C1} Creating {}...", output_path);

    if let Err(e) = std::fs::write(
        &output_path,
        serde_json::to_string_pretty(&results).unwrap(),
    ) {
        fail!(1, "failed to write output file {}: {}", output_path, e);
    }

    verbose!(
        "\u{26C3} Wrote {} records.\n",
        results.len()
    );
}


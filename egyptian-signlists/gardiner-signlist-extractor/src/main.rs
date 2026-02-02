// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Jamie Adams (a.k.a, Imodium Operator)

use serde::Serialize;
use std::io::IsTerminal;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

static STDERR_IS_TTY: OnceLock<bool> = OnceLock::new();

fn stderr_is_tty() -> bool {
    *STDERR_IS_TTY.get_or_init(|| std::io::stderr().is_terminal())
}

static VERBOSE: AtomicBool = AtomicBool::new(false);

macro_rules! verbose {
    ($fmt:expr $(, $arg:expr)*) => {
        if VERBOSE.load(std::sync::atomic::Ordering::Relaxed) {
            if stderr_is_tty() {
               eprintln!(
                   concat!("\x1b[36m\u{21E2}\x1b[0m ", $fmt)
                   $(, $arg)*
               );
            } else {
                eprintln!(concat!("\u{21E2} ", $fmt) $(, $arg)*);
            }
        }
    };
}

#[allow(unused)]
macro_rules! error {
    ($fmt:expr $(, $arg:tt)*) => {
        if stderr_is_tty() {
            eprintln!(
                concat!("\x1b[31m[ERROR]\x1b[0m ", $fmt)
                $(, $arg)*
            );
         } else {
            eprintln!(
                concat!("[ERROR] ", $fmt)
                $(, $arg)*
            );
         }
    };
}

macro_rules! fail {
    ($code:expr, $fmt:expr $(, $arg:expr)*) => {{
        eprintln!(
            concat!("\x1b[31m[ FAIL ]\x1b[0m ", $fmt)
            $(, $arg)*
        );
        std::process::exit($code);
    }};
}

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

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--verbose" | "-v" => {
                VERBOSE.store(true, Ordering::Relaxed);
            }
            _ => {
                if input_path.is_none() {
                    input_path = Some(arg);
                } else {
                    fail!(1, "unexpected argument: {}", arg);
                }
            }
        }
    }

    let input_path = match input_path {
        Some(p) => p,
        None => fail!(0, "usage: jsesh-extract [--verbose] <input-file>"),
    };

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
            verbose!("line {}: \u{203B} Found family {}", line_no + 1, fam);
            current_family = Some(fam.to_string());
            current_family_name = None;
            awaiting_family_name = true;
            continue;
        }

        if awaiting_family_name {
            verbose!("line {}:   family name = {}", line_no + 1, line);
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
                    verbose!("line {}:   sign {} (family {})", line_no + 1, code, f);
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
    verbose!("\u{26C1} Creating jsesh_inventory.json...");
    if let Err(e) = std::fs::write(
        "jsesh_inventory.json",
        serde_json::to_string_pretty(&results).unwrap(),
    ) {
        fail!(1, "failed to write output file: {}", e);
    }
    verbose!("\u{26C3} Wrote {} records.\n", results.len());
}

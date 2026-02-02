// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Jamie Adams (a.k.a, Imodium Operator)

use std::io::IsTerminal;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

#[allow(unused)]
static STDERR_IS_TTY: OnceLock<bool> = OnceLock::new();

#[allow(unused)]
pub fn stderr_is_tty() -> bool {
    *STDERR_IS_TTY.get_or_init(|| std::io::stderr().is_terminal())
}

#[allow(unused)]
pub static VERBOSE: AtomicBool = AtomicBool::new(false);

#[macro_export]
#[allow(unused)]
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

#[macro_export]
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

#[macro_export]
#[allow(unused)]
macro_rules! fail {
    ($code:expr, $fmt:expr $(, $arg:expr)*) => {{
        eprintln!(
            concat!("\x1b[31m[ FAIL ]\x1b[0m ", $fmt)
            $(, $arg)*
        );
        std::process::exit($code);
    }};
}

// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jamie Adams

//! Verbose console output for operator-facing progress messages.
//!
//! The `VERBOSE` flag is a global `AtomicBool` that library code checks before
//! printing progress messages to stderr.  The binary sets it from `--verbose`
//! at startup.  When off (the default), all `verbose!()` calls are silent.
//!
//! This is distinct from the `log` crate (which targets journald / audit):
//!   - `log::info!`  → audit log (structured, machine-readable)
//!   - `verbose!()`  → console progress (human-readable, ephemeral)

use std::sync::atomic::{AtomicBool, Ordering};

/// Global verbose flag — set once at startup, read everywhere.
pub static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Enable verbose output.  Called from `main()` when `--verbose` is passed.
pub fn enable() {
    VERBOSE.store(true, Ordering::Relaxed);
}

/// Check whether verbose output is enabled.
#[inline]
pub fn is_enabled() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

/// Print a verbose progress message to stderr.
///
/// Silent when `VERBOSE` is false (the default).  Messages go to stderr so
/// they don't interfere with `--json` or piped stdout output.
///
/// Must be `#[macro_export]` so it lands at crate root (`$crate::verbose!`).
/// Library modules and the binary both use this path.
#[macro_export]
macro_rules! verbose {
    ($fmt:expr $(, $arg:expr)*) => {
        if $crate::c2pa::verbose::is_enabled() {
            eprintln!(concat!("  ", $fmt) $(, $arg)*);
        }
    };
}

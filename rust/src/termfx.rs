// termfx.rs
// "Useless but cool-looking" terminal effects using ANSI escapes.
// No dependencies. Works on most modern terminals.
//
// Notes:
// - If stdout is not a TTY, you may want to disable styling.
// - For iTerm2, GNOME Terminal, and most Linux consoles, this is fine.
// - Windows cmd.exe historically needed special handling; on modern Windows Terminal it's OK.

use std::fmt::Write as _;
use std::io::{self, Write};
use std::time::{Duration, Instant};

pub mod ansi {
    pub const ESC: &str = "\x1b[";

    // Reset
    pub const RESET: &str = "\x1b[0m";

    // Attributes
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    // Basic colors (foreground)
    pub const FG_RED: &str = "\x1b[31m";
    pub const FG_GREEN: &str = "\x1b[32m";
    pub const FG_YELLOW: &str = "\x1b[33m";
    pub const FG_BLUE: &str = "\x1b[34m";
    pub const FG_MAGENTA: &str = "\x1b[35m";
    pub const FG_CYAN: &str = "\x1b[36m";
    pub const FG_GRAY: &str = "\x1b[90m";

    // Cursor control
    pub const CLEAR_LINE: &str = "\x1b[2K";
    pub const CURSOR_HIDE: &str = "\x1b[?25l";
    pub const CURSOR_SHOW: &str = "\x1b[?25h";
}

/// Guard: show cursor on drop (so spinners don't leave the cursor hidden).
pub struct CursorGuard;

impl CursorGuard {
    pub fn hide() -> Self {
        print!("{}", ansi::CURSOR_HIDE);
        let _ = io::stdout().flush();
        CursorGuard
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        print!("{}", ansi::CURSOR_SHOW);
        let _ = io::stdout().flush();
    }
}

/// True if output looks like a TTY. If false, you may choose to avoid ANSI.
/// This uses an environment heuristic; for a precise check you’d use libc/isatty.
pub fn likely_tty() -> bool {
    std::env::var("TERM").map(|t| t != "dumb").unwrap_or(false)
}

pub fn paint(s: &str, style: &str) -> String {
    format!("{style}{s}{}", ansi::RESET)
}

pub fn heading(title: &str) -> String {
    // A bold title with an underline
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{}{}{}{}",
        ansi::BOLD,
        title,
        ansi::RESET,
        ""
    );
    let _ = writeln!(out, "{}", "─".repeat(title.chars().count().max(3)));
    out
}

pub fn box_message(title: &str, body_lines: &[&str]) -> String {
    // Draw a unicode box; looks great on modern terminals.
    let mut lines: Vec<String> = Vec::new();
    lines.push(title.to_string());
    lines.extend(body_lines.iter().map(|s| s.to_string()));

    let inner_width = lines
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0)
        .max(10);

    let mut out = String::new();
    let _ = writeln!(out, "┌{}┐", "─".repeat(inner_width + 2));

    for (i, line) in lines.iter().enumerate() {
        let pad = inner_width.saturating_sub(line.chars().count());
        if i == 0 {
            // title line styled
            let styled = format!("{}{}{}",
                ansi::BOLD, line, ansi::RESET
            );
            // pad based on raw title length
            let _ = writeln!(out, "│ {}{}{} │", styled, " ".repeat(pad), "");
        } else {
            let _ = writeln!(out, "│ {}{} │", line, " ".repeat(pad));
        }
    }

    let _ = writeln!(out, "└{}┘", "─".repeat(inner_width + 2));
    out
}


//
// Status Messages
//
pub fn status_ok(msg: &str) -> String {
    format!(
        "[{}{}  OK  {}{}]  {}",
        ansi::FG_GREEN,
        ansi::BOLD,
        ansi::RESET,
        ansi::RESET,
        msg
    )
}

pub fn status_info(msg: &str) -> String {
    format!(
        "[{}{} INFO {}{}]  {}",
        ansi::FG_CYAN,
        ansi::BOLD,
        ansi::RESET,
        ansi::RESET,
        msg
    )
}

pub fn status_warn(msg: &str) -> String {
    format!(
        "[{}{} WARN {}{}]  {}",
        ansi::FG_YELLOW,
        ansi::BOLD,
        ansi::RESET,
        ansi::RESET,
        msg
    )
}

pub fn status_err(msg: &str) -> String {
    format!(
        "[{}{} ERR  {}{}]  {}",
        ansi::FG_RED,
        ansi::BOLD,
        ansi::RESET,
        ansi::RESET,
        msg
    )
}

/// A thin "rule" line across the terminal width (approx).
pub fn rule(width: usize) -> String {
    format!("{}{}", ansi::FG_GRAY, format!("{}{}", "─".repeat(width), ansi::RESET))
}


/// Print a 2-column key/value list with aligned keys.
pub fn kv_block(pairs: &[(&str, &str)]) -> String {
    let key_w = pairs.iter().map(|(k, _)| k.chars().count()).max().unwrap_or(0);
    let mut out = String::new();
    for (k, v) in pairs {
        let pad = key_w.saturating_sub(k.chars().count());
        let _ = writeln!(
            out,
            "{}{}{}{}{}: {}",
            ansi::FG_CYAN,
            k,
            ansi::RESET,
            " ".repeat(pad),
            "",
            v
        );
    }
    out
}


//
// Render a progress bar string (no printing). percent = 0.0..=100.0
// 
pub fn progress_bar(width: usize, percent: f64) -> String {
    let p = percent.clamp(0.0, 100.0);
    let filled = ((p / 100.0) * (width as f64)).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;

    // Use solid blocks for filled part
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
    format!(
        "{}[{}]{} {:>6.2}%",
        ansi::FG_BLUE,
        bar,
        ansi::RESET,
        p
    )
}

//
// A simple spinner that updates in-place.
// Call tick() in a loop; call finish() to print final line.
//
// Example:
//   let _cg = CursorGuard::hide();
//   let mut sp = Spinner::new("Hashing");
//   while work { sp.tick(); }
//   sp.finish("Hashing done");
//
pub struct Spinner {
    label: String,
    frames: &'static [&'static str],
    idx: usize,
    last: Instant,
    min_interval: Duration,
}

impl Spinner {
    pub fn new(label: &str) -> Self {
        Spinner {
            label: label.to_string(),
            // No emojis; clean ASCII-ish spinner.
            frames: &["|", "/", "-", "\\"],
            idx: 0,
            last: Instant::now(),
            min_interval: Duration::from_millis(80),
        }
    }

    pub fn tick(&mut self) {
        if self.last.elapsed() < self.min_interval {
            return;
        }
        self.last = Instant::now();
        self.idx = (self.idx + 1) % self.frames.len();

        print!(
            "\r{}{}{} {} {}{}",
            ansi::CLEAR_LINE,
            ansi::DIM,
            self.frames[self.idx],
            ansi::RESET,
            self.label,
            ansi::RESET
        );
        let _ = io::stdout().flush();
    }

    pub fn finish(&self, msg: &str) {
        print!("\r{}{}\n", ansi::CLEAR_LINE, msg);
        let _ = io::stdout().flush();
    }
}

//
// An attention-grabbing "stage" banner.
//
pub fn stage(label: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}{}==> {}{}", ansi::BOLD, ansi::FG_MAGENTA, label, ansi::RESET);
    out
}


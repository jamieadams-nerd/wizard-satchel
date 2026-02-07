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

//
// A simple spinner that updates in-place.
// Call tick() in a loop; call finish() to print final line.
//
// Example:
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
    pub fn set_frames(&mut self, new_frames: &'static [&'static str]) {
        self.frames = new_frames;
        self.idx = 0; // Reset index to avoid out-of-bounds on different sized arrays
    }

    pub fn use_block_theme(&mut self) {
        // These are static because they are hardcoded string literals
        self.set_frames(&["▖", "▘", "▝", "▗"]);
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
    let _ = writeln!(
        out,
        "{}{}==> {}{}",
        ansi::BOLD,
        ansi::FG_MAGENTA,
        label,
        ansi::RESET
    );
    out
}

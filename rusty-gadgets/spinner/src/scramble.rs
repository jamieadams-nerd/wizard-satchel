use rand::Rng;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

const CURSOR_HIDE: &str = "\x1b[?25l";
const CURSOR_SHOW: &str = "\x1b[?25h";
const CLEAR_LINE: &str = "\x1b[2K";

fn likely_tty() -> bool {
    std::env::var("TERM").map(|t| t != "dumb").unwrap_or(false)
}

/// Reveal `text` left-to-right with a "scrambling" active character.
/// - total_ms_per_char: total time spent on each character position
/// - fps: redraw rate (typ. 30-60)
/// - alphabet: characters used for scrambling
///
/// Works best for ASCII text. (Unicode graphemes are possible but more code.)
pub fn reveal_line(
    text: &str,
    total_ms_per_char: u64,
    fps: u64,
    alphabet: &str,
) -> io::Result<()> {
    // If not a tty, just print normally.
    if !likely_tty() {
        println!("{text}");
        return Ok(());
    }

    let mut stdout = io::stdout();

    // Hide cursor for the animation, ensure we show it at the end.
    write!(stdout, "{CURSOR_HIDE}")?;
    stdout.flush()?;

    let result = (|| -> io::Result<()> {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let frame_delay = if fps == 0 {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(1000 / fps)
        };

        let alpha: Vec<char> = alphabet.chars().collect();
        let mut rng = rand::thread_rng();

        // Pre-allocate a buffer we rewrite each frame
        let mut line = String::with_capacity(n + 32);

        for i in 0..n {
            let start = Instant::now();
            let per_char = Duration::from_millis(total_ms_per_char);

            while start.elapsed() < per_char {
                line.clear();

                // 1) already revealed prefix
                for j in 0..i {
                    line.push(chars[j]);
                }

                // 2) active scrambling character slot
                let active = alpha[rng.gen_range(0..alpha.len())];
                line.push(active);

                // 3) optional noisy tail (subtle, not too busy)
                //    If you prefer blanks, comment this out and push ' ' instead.
                for _ in (i + 1)..n {
                    let tail = alpha[rng.gen_range(0..alpha.len())];
                    line.push(tail);
                }

                // Draw
                write!(stdout, "\r{CLEAR_LINE}{line}")?;
                stdout.flush()?;

                thread::sleep(frame_delay);
            }

            // Lock in the real character at position i
            line.clear();
            for j in 0..=i {
                line.push(chars[j]);
            }
            // Tail as spaces (so the line doesn't keep "jumping")
            for _ in (i + 1)..n {
                line.push(' ');
            }

            write!(stdout, "\r{CLEAR_LINE}{line}")?;
            stdout.flush()?;
        }

        // Final clean print of full text, then newline
        write!(stdout, "\r{CLEAR_LINE}{text}\n")?;
        stdout.flush()?;

        Ok(())
    })();

    // Always show cursor again
    write!(stdout, "{CURSOR_SHOW}")?;
    stdout.flush()?;

    result
}

//fn main() -> io::Result<()> {
    // A good alphabet: looks “terminal-y” without being too chaotic.
    //let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{};:,.<>?";

    //scramble_reveal_line("SOLAR", 90, 60, alphabet)?;
    //scramble_reveal_line("UMRS INTAKE: RECEIPT ATTESTATION", 45, 60, alphabet)?;
    //Ok(())
//}

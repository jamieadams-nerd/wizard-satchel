use std::env;
use std::io::{self, Write};

fn gradient_bar(width: usize, msg: &str) {
    let msg_with_padding = format!("  {}  ", msg);
    
    // Determine if terminal supports TrueColor
    let is_truecolor = matches!(env::var("COLORTERM"), Ok(v) if v == "truecolor" || v == "24bit");

    // Cyberpunk Gold: 255, 190, 0 → 40, 30, 0
    // Deep Space Blue: 100, 150, 255 → 0, 10, 40
    // Alert Red: 255, 80, 80 → 50, 0, 0
    // Green: 150, 255, 150 -> 0, 40, 0
    //
    // Define Start (Bright Green) and End (Dark Forest)
    let (r_start, g_start, b_start) = (150, 255, 150);
    let (r_end, g_end, b_end) = (0, 40, 0);

    let start_fade_at = (width * 6) / 10;
    let fade_width = width - start_fade_at;

    for i in 0..width {
        // Get character at index or default to a space
        let c = msg_with_padding.chars().nth(i).unwrap_or(' ');

        let (r, g, b) = if i >= start_fade_at {
            let n = (i - start_fade_at) as i32;
            let fw = fade_width as i32;
            
            // Linear Interpolation
            let r_curr = r_start - ((r_start - r_end) * n / fw);
            let g_curr = g_start - ((g_start - g_end) * n / fw);
            let b_curr = b_start - ((b_start - b_end) * n / fw);
            (r_curr, g_curr, b_curr)
        } else {
            (r_start, g_start, b_start)
        };

        if is_truecolor {
            // \x1b[48;2;R;G;Bm = Background RGB | \x1b[38;2;0;0;0m = Black Text
            print!("\x1b[48;2;{};{};{}m\x1b[38;2;0;0;0m{}\x1b[0m", r, g, b, c);
        } else {
            // Simple ANSI Green fallback
            print!("\x1b[42;30m{}\x1b[0m", c);
        }
    }
    println!(); // New line
    io::stdout().flush().unwrap();
}

fn main() {
    gradient_bar(60, "RUST KERNELINITIALIZED");
    println!(" ");
    gradient_bar(60, "MEMORY SAFETY: VERIFIED");
    println!(" ");

    gradient_bar(80, "WARNING:");
    gradient_bar(80, "  RUST KERNEL INITIALIZED");
    gradient_bar(80, "  DONE.");

}


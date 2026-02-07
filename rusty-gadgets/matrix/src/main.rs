use crossterm::{
    cursor,
    event::{self, Event},
    execute,
    style::{self, Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{rngs::ThreadRng, Rng};

use std::{
    cmp::{max, min},
    env, fs,
    io::{self, Read, Stdout, Write, IsTerminal},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

impl Rect {
    fn contains(&self, px: u16, py: u16) -> bool {
        px >= self.x
            && px < self.x.saturating_add(self.w)
            && py >= self.y
            && py < self.y.saturating_add(self.h)
    }
}

#[derive(Debug)]
struct DropColumn {
    head_y: i32,
    speed: i32,
    length: i32,
}

impl DropColumn {
    fn new(rng: &mut ThreadRng, height: u16) -> Self {
        let h = height as i32;
        Self {
            head_y: rng.gen_range(-h..0),
            speed: rng.gen_range(1..=3),
            length: rng.gen_range(10..=max(12, h / 2)),
        }
    }

    fn step(&mut self, rng: &mut ThreadRng, height: u16) {
        self.head_y += self.speed;
        let h = height as i32;
        if self.head_y - self.length > h {
            *self = Self::new(rng, height);
        }
    }
}

#[derive(Debug)]
struct Args {
    seconds: u64,
    message: String,
}

// ****************************************************************************
//   Main Function
// ****************************************************************************
fn main() -> io::Result<()> {
    let args = parse_args()?;

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        cursor::Hide,
        Clear(ClearType::All)
    )?;

    let result = run(
        &mut stdout,
        &args.message,
        Duration::from_secs(args.seconds),
        //Duration::from_millis(50),
        Duration::from_millis(500),
    );

    // Always restore terminal
    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()?;

    result
}
// ****************************************************************************

fn parse_args() -> io::Result<Args> {
    let mut seconds: u64 = 8;
    let mut message_arg: Option<String> = None;
    let mut message_file: Option<String> = None;

    let mut it = env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--seconds" | "-s" => {
                if let Some(v) = it.next() {
                    seconds = v.parse::<u64>().unwrap_or(seconds);
                }
            }
            "--message" | "-m" => {
                if let Some(v) = it.next() {
                    message_arg = Some(v);
                }
            }
            "--message-file" | "-f" => {
                if let Some(v) = it.next() {
                    message_file = Some(v);
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                // Ignore unknown args (keeps it simple for quick demos)
            }
        }
    }

    // Priority:
    // 1) stdin (if piped)
    // 2) --message-file
    // 3) --message
    // 4) default
    let message = if stdin_is_piped() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        normalize_message(&buf)
    } else if let Some(path) = message_file {
        let content = fs::read_to_string(path)?;
        normalize_message(&content)
    } else if let Some(m) = message_arg {
        normalize_message(&m)
    } else {
        String::from("UMRS NOTICE\n\nOperation complete.\n\nPress any key to exit.")
    };

    Ok(Args { seconds, message })
}

fn print_help() {
    let txt = r#"matrix_box

USAGE:
  matrix_box [--seconds N] [--message "TEXT"] [--message-file PATH]

NOTES:
  - If stdin is piped, the message is read from stdin.
  - --message supports \n sequences for line breaks.
  - Press any key to exit early.

EXAMPLES:
  matrix_box --seconds 6 --message "UMRS NOTICE\n\nTransfer complete."
  echo -e "UMRS\n\nHello from stdin" | matrix_box --seconds 5
  matrix_box --message-file notice.txt --seconds 10
"#;
    print!("{txt}");
}

fn stdin_is_piped() -> bool {
    // If stdin is not a TTY, assume piped/redirected.
    !atty_stdin()
}

fn atty_stdin() -> bool {
    io::stdin().is_terminal()
}

fn normalize_message(s: &str) -> String {
    // Convert literal "\n" sequences into newlines, and trim only trailing whitespace.
    let replaced = s.replace("\\n", "\n");
    replaced.trim_end().to_string()
}

fn run(stdout: &mut Stdout, message: &str, run_for: Duration, tick: Duration) -> io::Result<()> {
    let start = Instant::now();
    let mut rng = rand::thread_rng();

    let (mut w, mut h) = terminal::size()?;
    w = max(w, 20);
    h = max(h, 10);

    //let mut box_rect = compute_center_box(w, h, message);
    let mut cols: Vec<DropColumn> = (0..w).map(|_| DropColumn::new(&mut rng, h)).collect();

    let mut box_rect; 
    while start.elapsed() < run_for {
        // Quit on any key
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(_) = event::read()? {
                break;
            }
        }

        // Handle resize
        let (nw, nh) = terminal::size()?;
        if nw != w || nh != h {
            w = max(nw, 20);
            h = max(nh, 10);
            cols = (0..w).map(|_| DropColumn::new(&mut rng, h)).collect();
        }

        // Recompute box each frame (so it stays centered and fits message if window changes)
        //box_rect = compute_center_box(w, h, message);

        box_rect = compute_center_box(w, 3, message);

        // Clear frame
        execute!(*stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;

        draw_rain(stdout, &mut rng, w, h, &mut cols, box_rect)?;
        draw_box_and_message(stdout, w, h, box_rect, message)?;

        stdout.flush()?;

        // Step state
        for c in cols.iter_mut() {
            c.step(&mut rng, h);
        }

        std::thread::sleep(tick);
    }

    Ok(())
}

fn compute_center_box(term_w: u16, term_h: u16, message: &str) -> Rect {
    let lines: Vec<&str> = message.lines().collect();
    let msg_w: u16 = lines
        .iter()
        .map(|s| s.chars().count() as u16)
        .max()
        .unwrap_or(0);
    let msg_h: u16 = max(lines.len() as u16, 1);

    // Box inner size constraints
    let inner_w = min(max(msg_w, 12), term_w.saturating_sub(6));
    let inner_h = min(max(msg_h, 5), term_h.saturating_sub(6));

    // Borders + padding
    let box_w = inner_w + 2 /*padding*/ + 2 /*borders*/;
    let box_h = inner_h + 2 /*padding*/ + 2 /*borders*/;

    let x = (term_w.saturating_sub(box_w)) / 2;
    let y = (term_h.saturating_sub(box_h)) / 2;

    Rect {
        x,
        y,
        w: box_w,
        h: box_h,
    }
}

fn draw_rain(
    stdout: &mut Stdout,
    rng: &mut ThreadRng,
    term_w: u16,
    term_h: u16,
    cols: &mut [DropColumn],
    clip_out: Rect,
) -> io::Result<()> {
    const GLYPHS: &[char] = &[
    //    '0', '1', '7', 'A', 'B', 'C', 'D', 'E', 'F', 'H', 'J', 'K', 'M', 'N', 'P', 'R', 'S', 'T',
        '\u{13000}', '\u{13001}', '\u{13140}', '\u{13143}', '\u{13080}',
        'V', 'X', 'Y', 'Z', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ',
        'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ',
    ];

    for x in 0..term_w {
        let c = &cols[x as usize];
        let head = c.head_y;
        let tail = head - c.length;

        for y in max(tail, 0)..=min(head, (term_h as i32) - 1) {
            let uy = y as u16;
            if clip_out.contains(x, uy) {
                continue;
            }

            let d = (head - y).max(0); // distance from head: 0=head

            // Multi-step "phosphor" fade:
            // 0: bright head
            // 1-2: green
            // 3-6: dark green
            // 7+: very dim
            let color = if d == 0 {
                Color::White
            } else if d <= 2 {
                Color::Green
            } else if d <= 6 {
                Color::DarkGreen
            } else {
                Color::DarkGrey
            };

            let ch = GLYPHS[rng.gen_range(0..GLYPHS.len())];

            execute!(
                *stdout,
                cursor::MoveTo(x, uy),
                SetForegroundColor(color),
                Print(ch)
            )?;
        }
    }

    Ok(())
}

fn draw_box_and_message(
    stdout: &mut Stdout,
    _term_w: u16,
    term_h: u16,
    r: Rect,
    message: &str,
) -> io::Result<()> {
    let x0 = r.x;
    let y0 = r.y;
    let x1 = r.x.saturating_add(r.w).saturating_sub(1);
    let y1 = r.y.saturating_add(r.h).saturating_sub(1);

    // Unicode box drawing
    let tl = '┌';
    let tr = '┐';
    let bl = '└';
    let br = '┘';
    let hz = '─';
    let vt = '│';

    // Border
    execute!(*stdout, SetForegroundColor(Color::White))?;

    execute!(*stdout, cursor::MoveTo(x0, y0), Print(tl))?;
    for x in (x0 + 1)..x1 {
        execute!(*stdout, cursor::MoveTo(x, y0), Print(hz))?;
    }
    execute!(*stdout, cursor::MoveTo(x1, y0), Print(tr))?;

    for y in (y0 + 1)..y1 {
        execute!(*stdout, cursor::MoveTo(x0, y), Print(vt))?;
        execute!(*stdout, cursor::MoveTo(x1, y), Print(vt))?;
    }

    execute!(*stdout, cursor::MoveTo(x0, y1), Print(bl))?;
    for x in (x0 + 1)..x1 {
        execute!(*stdout, cursor::MoveTo(x, y1), Print(hz))?;
    }
    execute!(*stdout, cursor::MoveTo(x1, y1), Print(br))?;

    // Interior
    let inner_x = x0 + 2;
    let inner_y = y0 + 2;
    let inner_w = r.w.saturating_sub(4);
    let inner_h = r.h.saturating_sub(4);

    let lines: Vec<&str> = message.lines().collect();
    let msg_h = max(lines.len() as u16, 1);
    let start_y = inner_y + (inner_h.saturating_sub(msg_h)) / 2;

    execute!(*stdout, SetForegroundColor(Color::White))?;

    for (i, line) in lines.iter().enumerate() {
        let y = start_y.saturating_add(i as u16);
        if y >= term_h {
            break;
        }


        let clipped: String = line.chars().take(inner_w as usize).collect();
        let line_len = clipped.chars().count() as u16;
        let x = inner_x + (inner_w.saturating_sub(line_len)) / 2;

        execute!(*stdout, cursor::MoveTo(x, y), Print(clipped))?;
    }

    Ok(())
}

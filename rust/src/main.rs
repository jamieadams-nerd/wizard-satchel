mod termfx;
mod scramble;

use std::{thread, time::Duration};

fn main() {

    // Headings + boxes
    print!("{}", termfx::heading("UMRS Intake Prototype"));
    print!("{}", termfx::stage("Computing hash"));
    print!(
        "{}",
        termfx::box_message(
            "Receipt Plan",
            &[
                "1) Store evidence as <sha256>",
                "2) Write <sha256>.json receipt",
                "3) Sign receipt as <sha256>.json.sig",
            ]
        )
    );

    // Status lines
    println!();
    println!("{}", termfx::status_ok("Vault directory is writable"));
    println!("{}", termfx::status_info("Vault directory is writable"));
    println!("{}", termfx::status_warn("Mime type is heuristic, record method/version"));
    println!("{}", termfx::status_err("Signature verification failed (demo message)"));
    println!();

    // Key/value block
    print!(
        "{}",
        termfx::kv_block(&[
            ("hash_alg", "sha256"),
            ("evidence_hash", "9a5c..."),
            ("bytes", "1048576"),
            ("detected_mime", "application/pdf"),
        ])
    );

    // Progress bar demo
    println!();
    for i in 0..=100 {
        let bar = termfx::progress_bar(80, i as f64);
        print!("\r{bar}");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        thread::sleep(Duration::from_millis(15));
    }
    println!();

    // Spinner demo
    let _cg = termfx::CursorGuard::hide();
    let mut sp = termfx::Spinner::new("Signing receipt");
    for _ in 0..80 {
        sp.tick();
        thread::sleep(Duration::from_millis(15));
    }
    sp.finish("Signing receipt: done");


    println!();

    // A good alphabet: looks “terminal-y” without being too chaotic.
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{};:,.<>?";
    
    scramble::reveal_line("SOLAR", 90, 60, alphabet);
    scramble::reveal_line("UMRS INTAKE: RECEIPT ATTESTATION", 45, 60, alphabet);

}





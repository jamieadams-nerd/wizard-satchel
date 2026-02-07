mod termfx;
mod scramble;

use std::{thread, time::Duration};

fn main() {

    println!();

    // Spinner demo
    let mut sp = termfx::Spinner::new("Loading");
    //sp.use_block_theme(); // Changes the frames inside the instance
    for _ in 0..80 {
        sp.tick();
        thread::sleep(Duration::from_millis(15));
    }
    // Do work here
    sp.finish("Signing receipt: done");


    println!("Can we start new messages...");
}





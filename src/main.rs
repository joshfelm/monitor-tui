use monitor_tui::tui::run_tui;

use std::env;

fn main() {
    let mut debug = false;
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        let argument = &args[1];
        if argument == "-d" {
            debug = true;
        }
    }

    if let Err(err) = run_tui(debug) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}


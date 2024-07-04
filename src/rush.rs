use std::process;
use std::env;
use carrot_libs::args;
use carrot_libs::input;
use serde_derive::{Deserialize, Serialize};
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::thread;

mod helpful;

#[derive(Serialize, Deserialize)]
struct RushConfig {
    prompt: String,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for RushConfig {
    fn default() -> Self { Self { prompt: "> ".into() } }
}

fn main() {
    let opts = args::opts();
    let (swcs, vals) = args::swcs();

    // Refuse to run when switches have been passed
    if ! swcs.is_empty() {
        eprintln!("This program does not support any switches and values!");
        process::exit(1);
    };
    if ! vals.is_empty() {
        eprintln!("This program does not support any switches and values!");
        process::exit(1);
    };

    // Create a new thread waiting for SIGINT
    // to prevent quiting with CTRL-C
    let mut signals = Signals::new([SIGINT]).unwrap();
    thread::spawn(move || {
        for sig in signals.forever() {
            if sig == 2 {
                println!("Got interrupt signal!");
                return;
            } else {
                println!("{sig}");

            }
        }
    });

    /*
    This shell will work in two modes:
    File mode - read lines from a file provided by the user via arguments
    Input mode - read lines from stdin
     */
    let mode = if !opts.is_empty() {
        "file"
    }
    else {
        "input"
    };

    if mode == "file" {
        todo!("File mode is not ready yet!");
    }
    else if mode == "input" {
        init_input_mode();
    }
}

fn init_input_mode() {
    // Always set $? (return code of previous command) to zero on start-up
    env::set_var("?", "0");
    loop {
        // Get all space separated words from user
        let cfg:RushConfig = confy::load("rush", "rush").unwrap();
        let console_input = match input::get(cfg.prompt, false) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Can't get user input: {e}");
                process::exit(1);
            }
        };
        // Separate commands from super commands
        let commands = helpful::split_commands(console_input, helpful::SPLIT_COMMANDS.to_vec(), true);
        // Execute those commands
        match commands {
            Ok(a) => helpful::detect_commands(&a),
            Err(e) => eprintln!("{e}!"),
        }
    };
}

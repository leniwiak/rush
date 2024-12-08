use std::io::Write;
use std::process;
use carrot_libs::args;
use carrot_libs::input;
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::thread;
use std::fs::OpenOptions;
use std::env;

mod helpful;
use helpful::RushConfig;

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
    loop {
        let cfg:RushConfig = match confy::load("rush", "rush") {
            Err(e) => { eprintln!("Failed to read config file: {}!", e); process::exit(1)},
            Ok(e) => e,
        };

        // Get all space separated words from user input
        let console_input = match input::get(cfg.prompt, false) {
            Ok(e) => {
                append_to_history(&e);
                // Return the list of words
                e
            },
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

fn append_to_history(e:&[String]) {
    if let Some(home_dir) = env::var_os("HOME") {
        if let Ok(motherfucking_home_dir) = home_dir.into_string() {
            let history_file_path = format!("{motherfucking_home_dir}/.rush_history.txt");
            // Open file (or create if does not exist) in append mode
            let mut file = OpenOptions::new().append(true).create(true).open(history_file_path);
            // If opening succeeds, write to the file
            if let Ok(ret) = &mut file {
                let mut text = String::new();
                e.iter().for_each(|x| {text.push_str(x); text.push(' ')} );
                // We don't care whether it succeeds or not
                let Ok(_) = writeln!(ret, "{text}") else {return};
                
            }
        };
    };
}
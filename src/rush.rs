use std::process;
use std::env;
use std::collections::HashMap;
use carrot_libs::args;
use carrot_libs::input;
use serde_derive::{Deserialize, Serialize};
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::thread;

mod exit;
mod getenv;
mod gt;
mod helpful;
mod setenv;
use {exit::*, getenv::*, gt::*, helpful::*, setenv::*};

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
        let commands = split_commands(console_input, SPLIT_COMMANDS.to_vec(), true);
        // Execute those commands
        detect_commands(commands);
    };
}

/* 
This function was created to run super commands or any other commands.
If non built-in command is inside some logical statements, don't do anything unless forced by logical statement
*/
pub fn detect_commands(commands:Vec<Vec<String>>) {
    let mut index = 0;
    let mut stop = false;
    
    /*
    This variable contains all required information about exit codes and statuses reported by ALL invoked commands
    this will be used by logic operators to find out if we can continue some operations or not
    Commands separation is done by split_commands()
    */
    let mut returns: HashMap<usize, CommandStatus> = HashMap::new();

    while index < commands.len() {
        // Check whether the first argument is a keyword or not
        if !stop {
            match commands[index][0].as_str() {
                "gt" => gt(&commands[index], index, &mut returns),
                // "help" | "?" => help(),
                "exit" | "quit" | "bye" => exit(&commands[index], index, &mut returns),
                "end" | "next" => (),
                "getenv" | "get" => getenv(&commands[index], index, &mut returns),
                "setenv" | "set" => setenv(&commands[index], index, &mut returns),
                "then" => then(&mut index, &mut returns, &commands, &mut stop),
                "exec" => helpful::exec(&commands[index], index, &mut returns),
                "panic" => panic!("Manually invoked panic message"),
                
                _ => helpful::exec(&commands[index], index, &mut returns)
            };
            if index < commands.len() {
                index+=1;
            };
        }
        else {
            return;
        };
    }
}

fn then(index_of_then:&mut usize, returns: &mut HashMap<usize, CommandStatus>, commands: &[Vec<String>], stop:&mut bool) {
    if *index_of_then == 0 {
        eprintln!("SYNTAX ERROR! Operator \"THEN\" doesn't work when there is nothing before it!");
        report_failure(*index_of_then, returns);
        *stop=true;
        return;
    }
    if *index_of_then == commands.len()-1 {
        eprintln!("SYNTAX ERROR! Operator \"THEN\" doesn't work when there is nothing after it!");
        report_failure(*index_of_then, returns);
        *stop=true;
        return;
    }
    // Compare exit status of previous and following commands
    let prev_index = *index_of_then-1;
    if returns.contains_key(&prev_index) {
        returns.get(&prev_index).unwrap().success
    }
    else {
        eprintln!("OPERATOR \"THEN\" FAILED! Unable to read exit code of the previous command!");
        *stop=true;
        false
    };

    // Go to the 'end' keyword
    let aaaaaaaaaa = match commands.iter().position(|x| x[0] == "end") {
        None => {
            eprintln!("OPERATOR \"THEN\" FAILED! Unable to find \"END\" keyword!");
            *stop=true;
            return;
        },
        Some(a) => { a },
    };

    *index_of_then=aaaaaaaaaa;
}

use carrot_libs::args;
use dialoguer;
use global::remove_quotationmarks;
use global::escape_slashes;
use std::fs;
use std::env;
use std::process;
use std::thread;
mod config;
mod directories;
mod exec;
mod global;
mod r#if;
mod variables;

use config::RushConfig;
use global::{
    allow_interrupts, index, interrupt_now, print_err, set_allow_interrupts, set_index,
    set_interrupt_now,
};

fn main() {
    // Get options and switches
    let opts = args::opts();
    let (swcs, vals) = args::swcs();

    // Refuse to run when switches have been passed
    if !swcs.is_empty() {
        eprintln!("This program does not support any switches and values!");
        process::exit(1);
    };
    if !vals.is_empty() {
        eprintln!("This program does not support any switches and values!");
        process::exit(1);
    };

    // Prevent quiting with CTRL-C
    let _ = ctrlc::set_handler(move || {
        eprintln!("^C");
        if allow_interrupts() {
            eprintln!("Interrupting...");
            set_interrupt_now(true);
        }
    });

    // Collect words in whole script
    // from interactive console or a file

    // If we have no options, run in interactive mode
    // Start text entry thing and wait for the user to type the command
    if opts.is_empty() {
        loop {
            set_index(0);
            set_allow_interrupts(true);
            set_interrupt_now(false);
            let cfg: RushConfig = match confy::load("rush", "rush") {
                Err(e) => {
                    eprintln!("Failed to read config file: {}!", e);
                    process::exit(1)
                }
                Ok(e) => e,
            };

            let cmd: Result<String, dialoguer::Error> = dialoguer::Input::new().interact_text();

            match cmd {
                Ok(e) => {
                    do_rest_of_magic_or_nothing(
                        e.split_whitespace().map(|x| x.to_string()).collect(),
                    );
                }
                Err(e) => {
                    eprintln!("Can't get user input: {e}");
                    process::exit(1);
                }
            };
        }
    }
    // If there are some options, read the file to the 'script' vector
    else {
        for o in opts {
            set_index(0);
            set_allow_interrupts(true);
            set_interrupt_now(false);
            match fs::read_to_string(o) {
                Ok(e) => do_rest_of_magic_or_nothing(
                    e.split_whitespace().map(|x| x.to_string()).collect(),
                ),
                Err(e) => {
                    eprintln!("Unable to read from script file: {:?}", e.kind());
                    process::exit(1);
                }
            }
        }
    };
}

fn do_rest_of_magic_or_nothing(script: Vec<String>) {
    // Do nothing if script is empty
    if script.is_empty() {
        print!("");
        return;
    }
    group_quotationmarks(script);
}

/*
This function joins separate arguments enclosed in single or double quotationmarks.
For example:
- "this would be tolerated as one argument"
- 'this is a big argument, too'
- th"is is also just o"ne
- "this is incorrect'
- this won't be accepted either
- that\'s good
- this sentence represents multiple arguments. no quotationmarks!
*/
fn group_quotationmarks(script: Vec<String>) {
    // Collect non-quoted and joined quoted commands from the script here
    let mut buf = Vec::new();
    let mut words_in_qmarks = String::new();

    let mut single_qmarks = false;
    let mut double_qmarks = false;

    for w in script {
        // Iterate through all the letters to find quotationmarks
        for (c_idx, c) in w.chars().enumerate() {
            // Toggle single/double_qmark variable if we find a quotationmark that is not
            // enclosed in other quotationmark AND is not preceded by a slash.
            if c == '\''
                && (c_idx == 0 || w.chars().nth(c_idx - 1).unwrap() != '\\')
                && !double_qmarks
            {
                single_qmarks = !single_qmarks;
            }

            if c == '"'
                && (c_idx == 0 || w.chars().nth(c_idx - 1).unwrap() != '\\')
                && !single_qmarks
            {
                double_qmarks = !double_qmarks;
            }
        }
        if single_qmarks || double_qmarks {
            if !words_in_qmarks.is_empty() {
                words_in_qmarks.push(' ');
            };
            words_in_qmarks.push_str(&w);
        } else if !words_in_qmarks.is_empty() {
            words_in_qmarks.push(' ');
            words_in_qmarks.push_str(&w);
            buf.push(words_in_qmarks.clone());
            words_in_qmarks.clear();
        } else {
            buf.push(w.clone());
        }
    }

    syntax_test(buf);
}

enum Builtins {
    Lock,
    If,
}

fn syntax_test(script: Vec<String>) {
    let mut line_number = 1;
    let mut used_builtins_history = Vec::new();

    // Build list of errors to show
    let mut errors = Vec::new();

    // Iterate through every word in script and catch some common errors
    for w in &script {
        // Errors messages that tell the user where problematic code is, are much more readable c;
        if w.ends_with('\n') {
            line_number += 1
        };

        // Catch usage of logical statements
        if w.to_lowercase() == "lock" {
            used_builtins_history.push(Builtins::Lock);
        };
        if w.to_lowercase() == "if" {
            used_builtins_history.push(Builtins::If);
        };

        // Any logical statements have to be ended with associated ending keywords like ENDLOCK or ENDIF
        // If you find it somewhere, remove the last logical statement from history
        if w.to_lowercase().trim() == "endlock" || w.to_lowercase().trim() == "endlock;" {
            match used_builtins_history.last() {
                Some(Builtins::Lock) => {used_builtins_history.pop();},
                _ => errors.push(format!("{line_number}: Usage of \"ENDLOCK\" outside of the \"LOCK\" statement is incorrect")),
            }
        }
        if w.to_lowercase().trim() == "endif" || w.to_lowercase().trim() == "endif;" {
            match used_builtins_history.last() {
                Some(Builtins::If) => {used_builtins_history.pop();},
                _ => errors.push(format!("{line_number}: Usage of \"ENDIF\" outside of the \"IF\" statement is incorrect")),
            }
        }

        // Catch use of free or continue
        if (w == "free" || w == "continue")
            && !used_builtins_history
                .iter()
                .any(|x| matches!(x, Builtins::Lock))
        {
            errors.push(format!("{line_number}: Usage of \"FREE\" or \"CONTINUE\" is not permited outside of the \"LOCK\" statement"));
        }

        // Disallow running empty commands like this: say hello; ; say bye
        // if w == ";" {
        //     errors.push(format!("{line_number}: Trying to run empty command!"))
        // }
    }

    // Summarize looking for unclosed logical statements
    if !used_builtins_history.is_empty() {
        for element in used_builtins_history {
            match element {
                Builtins::Lock => errors.push(("Unclosed \"LOCK\" statement").to_string()),
                Builtins::If => errors.push(("Unclosed \"IF\" statement").to_string()),
            }
        }
    }

    // Show errors
    if !errors.is_empty() {
        eprintln!("There are errors in your script that need to be fixed or they can cause serious issues!");
        for e in errors {
            eprintln!("\t{e}");
        }
        return;
    }

    make_script_thread(script);
}

fn make_script_thread(script: Vec<String>) {
    // Allow responses for SIGINT
    set_allow_interrupts(true);

    let parser = thread::spawn(move || {
        run_script(script);
    });
    if let Err(e) = parser.join() {
        eprintln!("Child process returned an error: {:?}", e);
    }
    set_allow_interrupts(false);
}

#[derive(Debug)]
enum ShellMode {
    // Run all commands inside IF/ELSEIF/ELSE block
    CmpSuccess,
    // Skip tasks inside current IF/ELSEIF/ELSE block and try running another one
    CmpFailure,
    // When shell mode is set to CmpSuccess, skip checking any other IF/ELSEIF/ELSE and included subcommands
    // IfDone status will be reset by ENDIF keywords
    IfDone,
    // After you reach "endlock", go back to LOCK defined in position_of_lock.
    // Allow usage of BREAK and CONTNUE
    Lock,
    // Skip executing commands until you reach ENDLOCK. Go back to LOCK.
    LockContinue,
    // Skip executing commands until you reach ENDLOCK. But do not go back to LOCK.
    LockFree,
}

struct ShellModes {
    list: Vec<ShellModes>
}

fn run_script(script: Vec<String>) {
    // When looking for words for a command that we are starting to work with,
    // where should we start searching from?
    let mut start_search_from = 0;
    
    // Create a buffer that will hold words from the script **that we are currently working on**.
    let buf = script[start_search_from..].iter().position(
        |x| !x.ends_with("\\;") && (x.ends_with(';') || x.ends_with('\n'))
    );
    start_search_from += buf.len();
    
}

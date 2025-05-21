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

#[derive(Debug)]
enum ShellModes {
    // Run all commands inside IF/ELSEIF/ELSE block
    CmpSuccess,
    // Skip tasks inside current IF/ELSEIF/ELSE block and try running another one
    CmpFailure,
    // When shell mode is set to CmpSuccess, skip checking any other IF/ELSEIF/ELSE and included subcommands
    // IfDone status will be reset by ENDIF keywords
    IfDone,
    // After you reach "endlock", go back to LOCK defined in position_of_lock.
    // Allow usage of BREAK and CONTNUE
    Lock(usize),
    // Skip executing commands until you reach ENDLOCK. Go back to LOCK.
    LockContinue(usize),
    // Skip executing commands until you reach ENDLOCK. But do not go back to LOCK.
    LockFree,
}

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

fn run_script(script: Vec<String>) {
    // Iterate through words in the script and slowly collect them to a buffer
    let mut buf: Vec<String> = Vec::new();
    // Count line numbers in the script for better error messages
    let mut line_number = 1;
    // How many nested blocks do we have?
    let mut nesting_level: usize = 0;

    // This variable defines status of the execution mode
    // We'll skip some elseif's and run loops over and over again based on the value of this variable.
    // More about this at the beginning of this source file.
    let mut shell_mode: Vec<ShellModes> = Vec::new();

    // Until we reach the end of the script
    while index() < script.len() {
        if interrupt_now() {
            return;
        }
        // Shorthand for currently iterated word of a script
        let w = script[index()].clone();
        // Is this the last word? The script is ending!
        let the_last_word_in_script = index() == script.len() - 1;
        // Add currently iterated word to a temporary buffer
        if !w.ends_with("\\;") && w.ends_with(';') {
            buf.push(w.strip_suffix(';').unwrap().to_string());
        } else {
            buf.push(w.clone());
        };

        let contains_if_done = shell_mode.iter().any(|s| matches!(s, ShellModes::IfDone));
        let contains_cmp_failure = shell_mode.iter().any(|s| matches!(s, ShellModes::CmpFailure));
        let contains_lock_free = shell_mode.iter().any(|s| matches!(s, ShellModes::LockFree));
        let contains_lock_continue = shell_mode
            .iter()
            .any(|s| matches!(s, ShellModes::LockContinue(_)));

        let block_execution =
            contains_if_done ||
            contains_cmp_failure ||
            contains_lock_free ||
            contains_lock_continue;


        // If we reach the end of a script OR some command separator like '\n' or ';' is found...
        if the_last_word_in_script || (!w.ends_with("\\;") && w.ends_with(';')) || w.ends_with('\n')
        // || (buf.len() < 2 && w == "lock")
        // || (buf.len() < 2 && w == "if")
        //
        // ... try running the command ...
        {
            dbg!(&buf);

            // First argument in the buffer (buf[0]) is a program name
            // Check whether it's something built into the shell or not.
            let program_name = buf[0].clone();
            if !block_execution {
                match remove_quotationmarks(&program_name).as_str() {
                    /*
                    marbulec was here
                    dito

                    When we reach the LOCK keyword, we have to remember it's position and set working shell_mode to 'Lock'
                    When lock mode is enabled, Rush will execute all commands inside of an lock block until it reaches END keyword.
                    Then, Rush will jump back to the position of LOCK and execute all of the commands again and again and again...
                    It's an endless loop.

                    But we can break out of this and set us free with the FREE keyword.
                    When we approach it, we'll set the shell_mode to LockFree and then, skip executing any command until END is found.

                    Additionally, you can rerun the loop from the beggining, skipping any following command.
                    This can be done using CONTINUE.
                    CONTINUE will change shell_mode to LockContinue, stop the execution of any command until END and then, run LOCK again.
                    */
                    "lock" => {
                        // Increment nesting count
                        nesting_level += 1;

                        // Set shell_mode to Lock and save it's position
                        let position_of_program_name_in_script = index();
                        shell_mode.push(ShellModes::Lock(position_of_program_name_in_script));
                    }
                    /*
                    set val 0
                    lock
                        set val 5
                        if $val = 5;
                            free
                            say "Don't run it"
                        endif
                        say "This can't be seen"
                    endlock

                    In the example below, FREE or CONTINUE keyword may be used when we're not in the LOCK directly.
                    Therefore, we need to look back for the LAST possible shell_mode with Lock value and change it to LockFree/LockContinue

                    To prevent execution of any command after FREE/CONTINUE (doesn't matter if it's still an IF stmt. or smth. outside of it in LOCK itself),
                    shell must remember that there is at least one LockFree/LockContinue value
                    */
                    "free" => {
                        let mut position_of_found_lock_mode_in_shellmodes = 0;
                        // Check if we are in a LOCK mode somewhere in shell_mode list (look for 'Lock' mode from the end of a list)
                        let inlock = shell_mode.iter().rev().enumerate().any(|s| {
                            let a = s.1;
                            position_of_found_lock_mode_in_shellmodes =
                                (&shell_mode.len() - 1) - s.0;
                            matches!(a, ShellModes::Lock { .. })
                        });
                        // If we are in a LOCK, change latest Lock to a LockFree.
                        if inlock {
                            shell_mode[position_of_found_lock_mode_in_shellmodes] =
                                ShellModes::LockFree;
                        }
                        // If we are NOT in a LOCK in any way
                        else {
                            let e = ("Usage of \"FREE\" is not permited outside of the \"LOCK\" statement.").to_string();
                            print_err(e, program_name.clone(), line_number)
                        }
                    }
                    "continue" => {
                        let mut position_of_found_lock_mode_in_shellmodes = 0;
                        // Check if we are in a LOCK mode somewhere in shell_mode list (look for 'Lock' mode from the end of a list)
                        let inlock = shell_mode.iter().rev().enumerate().any(|s| {
                            let a = s.1;
                            position_of_found_lock_mode_in_shellmodes =
                                (&shell_mode.len() - 1) - s.0;
                            matches!(a, ShellModes::Lock { .. })
                        });

                        // If we are in a LOCK, change latest Lock(position in script) shell_mode to a LockContinue(position in script).
                        if inlock {
                            match shell_mode[position_of_found_lock_mode_in_shellmodes] {
                                ShellModes::Lock(number) => {
                                    shell_mode[position_of_found_lock_mode_in_shellmodes] =
                                        ShellModes::LockContinue(number);
                                }
                                _ => {
                                    unreachable!("Program's logic contradics itself! Please, report this error!");
                                }
                            }
                        }
                        // If we are NOT in a LOCK in any way
                        else {
                            let e = ("Usage of \"CONTINUE\" is not permited outside of the \"LOCK\" statement.").to_string();
                            print_err(e, program_name.clone(), line_number)
                        }
                    }

                    "if" => {
                        // Increment nesting count
                        nesting_level += 1;

                        // Set shell_mode to Lock and save it's position
                        let position_of_program_name_in_script = index();
                        let out = r#if::logic(buf.clone());
                        match out {
                            Ok(true) => shell_mode.push(ShellModes::CmpSuccess),
                            Ok(false) => shell_mode.push(ShellModes::CmpFailure),
                            Err(e) => print_err(e, program_name.clone(), line_number),
                        };
                    }
                    "elseif" | "eif" => {
                        dbg!(&shell_mode[nesting_level]);
                        match shell_mode[nesting_level] {
                            // Is it running after unsuccessfull IF/ELSEIF?
                            ShellModes::CmpFailure => {
                                // Set shell_mode to Lock and save it's position
                                let position_of_program_name_in_script = index();
                                let out = r#if::logic(buf.clone());
                                match out {
                                    Ok(true) => shell_mode.push(ShellModes::CmpSuccess),
                                    Ok(false) => shell_mode.push(ShellModes::CmpFailure),
                                    Err(e) => print_err(e, program_name.clone(), line_number),
                                };
                            }
                            ShellModes::CmpSuccess => {
                                shell_mode[nesting_level] = ShellModes::IfDone
                            }
                            ShellModes::IfDone => (),
                            _ => {
                                let e =
                                    "Usage of \"ELSEIF\" outside of an IF statement is incorrect"
                                        .to_string();
                                print_err(e, program_name.clone(), line_number);
                            }
                        };
                    }
                    "else" => {
                        todo!();
                    }

                    "set" => {
                        if let Err(e) = variables::setenv(&buf) {
                            print_err(e, program_name.clone(), line_number);
                        };
                    }
                    "rem" => {
                        if let Err(e) = variables::remenv(&buf) {
                            print_err(e, program_name.clone(), line_number);
                        };
                    }
                    "get" => match variables::getenv(&buf) {
                        Ok(res) => println!("{res}"),
                        Err(e) => print_err(e, program_name.clone(), line_number),
                    },
                    "++" => {
                        if let Err(e) = variables::chenv(&buf, true) {
                            print_err(e, program_name.clone(), line_number);
                        };
                    }
                    "--" => {
                        if let Err(e) = variables::chenv(&buf, false) {
                            print_err(e, program_name.clone(), line_number);
                        };
                    }

                    "gt" => {
                        if let Err(e) = directories::gt(&buf) {
                            print_err(e, program_name.clone(), line_number);
                        };
                    }

                    "panic" => {
                        panic!("User invoked panic");
                    }
                    "exit" => {
                        process::exit(0);
                    }

                    // Comments will never be run
                    "#" | "endif" | "endlock" => {},
                    // Not built in?
                    _ => {
                            if let Err(e) = exec::exec(&buf) {
                                print_err(e, program_name.clone(), line_number);
                            }
                            // The fuck was this code about
                            //
                            // let mut resolved_buf = resolver(buf.iter().collect(), true, true);
                            // match resolved_buf {
                            //     Err(e) => print_err(e, program_name, line_number),
                            //     Ok(gg) => exec::exec(&gg.split_whitespace()),
                            // }
                    }
                };
            }
            // ... and finally, after command is done, clear the buffer
            buf.clear();

            match program_name.as_str() {
                "endlock" => {
                    let mut position_of_found_lock_mode_in_shellmodes = 0;
                    // Check if we are in a LOCK mode somewhere in shell_mode list (look for 'Lock' mode from the end of a list)
                    let inlock = shell_mode.iter().rev().enumerate().any(|s| {
                        let a = s.1;
                        position_of_found_lock_mode_in_shellmodes = (&shell_mode.len() - 1) - s.0;
                        matches!(a, ShellModes::Lock { .. })
                    });
                    if inlock {
                        match shell_mode[nesting_level-1] {
                            // If current shell_mode is set to Lock or LockContinue,
                            // go back to the position of LOCK so we'll execute it again as intended.
                            ShellModes::Lock(a) | ShellModes::LockContinue(a) => {
                                set_index(position_of_found_lock_mode_in_shellmodes);
                                shell_mode[nesting_level] = ShellModes::Lock(a);
                            }
                            // Set free from LOCK loop
                            ShellModes::LockFree => {
                                shell_mode.pop();
                                nesting_level -= 1;
                            }
                            _ => {
                                let e = "Usage of \"ENDLOCK\" in wrong nesting".to_string();
                                print_err(e, program_name, line_number);
                            }
                        }
                    }
                },
                "endif" => {
                    let mut position_of_found_if_mode_in_shellmodes = 0;
                    // Check if we are in a LOCK mode somewhere in shell_mode list (look for 'Lock' mode from the end of a list)
                    let inif = shell_mode.iter().rev().enumerate().any(|s| {
                        let a = s.1;
                        position_of_found_if_mode_in_shellmodes = (&shell_mode.len() - 1) - s.0;
                        matches!(a, ShellModes::CmpFailure)
                        || matches!(a, ShellModes::CmpSuccess)
                        || matches!(a, ShellModes::IfDone)
                    });
                    if inif {
                        match shell_mode[nesting_level-1] {
                            ShellModes::CmpSuccess | ShellModes::CmpFailure | ShellModes::IfDone => {
                                shell_mode.pop();
                                nesting_level -= 1;
                            }
                            _ => {
                                let e = "Usage of \"ENDIF\" in wrong nesting".to_string();
                                print_err(e, program_name, line_number);
                            }
                        }
                    }
                },
                _ => (),
            };
        }

        // Bump line number if we find a new line character
        if w.ends_with('\n') && !w.ends_with("\\n") {
            line_number += 1;
        }

        // Bump index number
        set_index(index() + 1);
    }
    // When you stop working with the buffer, reset indexer
    set_index(0);
}

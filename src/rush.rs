use std::process;
use carrot_libs::args;
use carrot_libs::input;
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::thread;
use std::fs;
mod helpful;
mod directories;
mod exec;
mod variables;
mod r#if;
mod global;
use global::{index, set_index, print_err};
use helpful::RushConfig;

enum ShellModes {
    /*
    Set this mode when comparison inside of an IF block succeeds.
    This will force the shell to run tasks inside of this particular IF/ELSE/ELSEIF block
    and skip others under it.
    */
    CmpSuccess,
    /*
    Set this mode when comparison inside of an IF block fails.
    This will force the shell to SKIP tasks inside of this particular IF/ELSE/ELSEIF block
    and try to run the others.
    */
    CmpFailure,
    /*
    After you reach END keyword, go back to LOCK defined in position_of_lock.
    Allow usage of BREAK and CONTNUE
     */
    Lock {position_of_lock:usize},
    /*
    Skip executing commands until you reach END. Go back to LOCK.
    */
    LockContinue {position_of_lock:usize},
    /*
    Skip executing commands until you reach END. But do not go back to LOCK.
    */
    LockFree,
}

fn main() {
    // Get options and switches
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

    // Prevent quiting with CTRL-C
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
    
    // Collect words in whole script
    // from interactive console or a file 

    // If we have no options, run in interactive mode
    // Start text entry thing and wait for the user to type the command
    if opts.is_empty() {
        loop {
            let cfg:RushConfig = match confy::load("rush", "rush") {
                Err(e) => { eprintln!("Failed to read config file: {}!", e); process::exit(1)},
                Ok(e) => e,
            };
            match input::get(cfg.prompt, false) {
                Ok(e) => {
                    // Return the list of words
                    parse_script(Some(e));
                },
                Err(e) => {
                    eprintln!("Can't get user input: {e}");
                    process::exit(1);
                }
            };
        };
    }
    // If there are some options, read the file to the 'script' vector
    else {
        for o in opts {
            match fs::read_to_string(o) {
                Ok(e) => parse_script(Some(e.split_whitespace().map(str::to_string).collect())),
                Err(e) => {
                    eprintln!("Unable to read from script file: {:?}", e.kind());
                    process::exit(1);
                }
            }
        }
    };
}

fn parse_script(script: Option<Vec<String>>) {   
    if let Some(script) = script {
        // Do nothing if script is empty
        if script.is_empty() {
            print!("");
            return;
        }

        // Iterate through words in the script and slowly collect them to a buffer
        let mut buf: Vec<String> = Vec::new();
        // Count line numbers in the script for better error messages
        let mut line_number = 1;
        // How many nested blocks do we have?
        let mut nesting_level = 0;

        // This variable defines status of the execution mode
        // We'll skip some elseif's and run loops over and over again based on the value of this variable.
        // More about this at the beginning of this source file.
        let mut shell_mode:Vec<ShellModes> = Vec::new();
        /* IMPORTANT!
        Since IF, LOOP, WHILE and other blocks can be nested, you always have to check for shell_mode in a good position.
        
        For example:
        
        
        if some_command do <<< Set shell_mode to [CmpSuccess] because this example assumes, that 'some_command' returned a success.
            say running another if... <<< Run this and other tasks below in this IF block 
                                          because of the status saved in shell_modes[0]
            if another_command do <<< Bump number in 'nesting_level' to 1 and push another ShellMode into shell_mode variable,
                                      so it might look something like this: [CmpSuccess, CmpSuccess]
                say OK <<< This task will be executed when 'another_command' returns success.
                            Check contents of shell_mode[1] to find out whether to run it or not.
            end <<< Restore 'nesting_level' to 0 and revert shell_mode to [CmpSuccess]
        elseif something_else do <<< This will be skipped because operation in the previous IF block reported success.
                                     We know this because of the shell_mode[0] 
            say NOT OK
        end <<< Revert shell_mode to [CmpReset]
        
         */

        // Until we reach the end of the script
        while index() < script.len() {
            // Shorthand for currently iterated word of a script
            let w = script[index()].clone();
            // Is this the last word? The script is ending! 
            let the_last_word_in_string = index() == script.len()-1;
            // Add currently iterated word to a temporary buffer
            buf.push(w.clone());

            // If we reach the end of a script OR some command separator like '\n' or, 'do' 'next'... 
            if the_last_word_in_string || w == "next" || w == "do" || w.ends_with('\n') {
                // ... try running the command ...
                // First argument in the buffer (buf[0]) is a program name
                // Check whether it's something built into the shell or not.
                let program_name = buf[0].clone();

                // In some scenarios, we can't run any command until we reach END
                // When this kind of lock is needed, we set this variable to true.
                let mut _ultimate_end_lock = false;

                // And these scenarios are simple:
                // Any shell_mode[] is set to LockContinue/LockFree
                // The last shell_mode is set to CmpFailure
                for a in &shell_mode {
                    _ultimate_end_lock =  { matches!(a, ShellModes::LockContinue { .. }) || matches!(shell_mode[nesting_level], ShellModes::CmpFailure) };
                }

                match (program_name.as_str(), false) {
                    /*
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
                    ("lock", _ultimate_end_lock) => {
                        // Increment nesting count
                        nesting_level+=1;
                        // Set shell_mode to Lock
                        shell_mode.push(ShellModes::Lock{position_of_lock:index()});
                    },
                    /*
                    set val 0
                    lock
                        set val 5
                        if $val = 5 do
                            free
                            say "Don't run it"
                        end
                        say "This can't be seen"
                    end

                    In the example below, FREE or CONTINUE keyword may be used when we're not in the LOCK directly.
                    Therefore, we need to look back for the LAST possible shell_mode with Lock value and change it to LockFree/LockContinue
                    
                    To prevent execution of any command after FREE/CONTINUE (doesn't matter if it's still an IF stmt. or smth. outside of it in LOCK itself),
                    shell must remember that there is at least one LockFree/LockContinue value
                     */
                    ("free", _ultimate_end_lock) => {
                        // Check if we are in a lock somewhere in shell_mode list
                        let mut inlock = false;
                        let mut last_lock_position = 0;
                        // Get the last possible lock position
                        for (i,a) in shell_mode.iter().rev().enumerate() {
                            if matches!(a, ShellModes::Lock { .. }) {
                                let j = shell_mode.len()-1-i;
                                inlock = true;
                                last_lock_position = j;
                                break;
                            }
                        }
                        // If we are in a LOCK, change latest Lock to a LockFree.
                        if inlock {
                            shell_mode[last_lock_position] = ShellModes::LockFree;
                        }
                        // If we are NOT in a LOCK in any way
                        else {
                            let e = ("Usage of \"FREE\" is not permited outside of the \"LOCK\" statement.").to_string();
                            print_err(e, program_name, line_number)
                        }
                    }
                    ("continue", _ultimate_end_lock) => {
                        // Check if we are in a lock somewhere in shell_mode list
                        let mut inlock = false;
                        let mut last_lock_position = 0;
                        // Get the last possible lock position
                        for (i,a) in shell_mode.iter().rev().enumerate() {
                            if matches!(a, ShellModes::Lock { .. }) {
                                let j = shell_mode.len()-1-i;
                                inlock = true;
                                last_lock_position = j;
                                break;
                            }
                        }
                        // If we are in a LOCK, change latest Lock to a LockContinue.
                        if inlock {
                            shell_mode[last_lock_position] = ShellModes::LockContinue{position_of_lock:last_lock_position}
                        }
                        // If we are NOT in a LOCK in any way
                        else {
                            let e = ("Usage of \"CONTINUE\" is not permited outside of the \"LOCK\" statement.").to_string();
                            print_err(e, program_name, line_number)
                        }
                    }


                    ("if", _ultimate_end_lock) => {
                        // Increment nesting count
                        nesting_level+=1;

                        match r#if::logic(buf.clone()) {
                            Err(e) => print_err(e, program_name, line_number),
                            Ok(res) => {
                                if res {shell_mode.push(ShellModes::CmpSuccess)}
                                else {shell_mode.push(ShellModes::CmpFailure)}
                            }
                        };
                    },
                    ("elseif", _ultimate_end_lock) => todo!("elseif"),
                    ("else", _ultimate_end_lock) => todo!("else"),


                    ("set", _ultimate_end_lock) => {
                        if let Err(e) = variables::setenv(&buf) {
                            print_err(e, program_name, line_number);
                        };
                    },
                    ("rem", _ultimate_end_lock) => {
                        if let Err(e) = variables::remenv(&buf) {
                            print_err(e, program_name, line_number);
                        };
                    },
                    ("get", _ultimate_end_lock) => {
                        match variables::getenv(&buf) {
                            Ok(res) => println!("{res}"),
                            Err(e) => print_err(e, program_name, line_number),
                        }
                    },
                    ("inc", _ultimate_end_lock) => {
                        if let Err(e) = variables::chenv(&buf, true) {
                            print_err(e, program_name, line_number);
                        };
                    },
                    ("dec", _ultimate_end_lock) => {
                        if let Err(e) = variables::chenv(&buf, false) {
                            print_err(e, program_name, line_number);
                        };
                    },


                    ("gt", _ultimate_end_lock) => {
                        if let Err(e) = directories::gt(&buf) {
                            print_err(e, program_name, line_number);
                        };
                    },

                    // END is never locked
                    ("end", false) => {
                        match shell_mode[nesting_level] {
                            // If current shell_mode is set to Lock, go back to the index number of lock
                            ShellModes::Lock { position_of_lock } | ShellModes::LockContinue{position_of_lock} => {
                                set_index(position_of_lock);
                            }
                            ShellModes::LockFree => {
                                shell_mode.remove(nesting_level);
                            }
                            ShellModes::CmpSuccess | ShellModes::CmpFailure => {
                                shell_mode.remove(nesting_level);
                            }
                        }
                        nesting_level-=1;
                    },
                    // Comments are always locked, meaning they will never run
                    ("#", false) => {},
                    // Not built in?
                    _ => {
                        if let Err(e) = exec::exec(&buf) {
                            print_err(e, program_name, line_number);
                        }
                    },
                }
                // ... and finally, after command is done, clear the buffer
                buf.clear();
            }

            // Bump line number if we find a new line character
            if w.ends_with('\n') {
                line_number+=1;
            }

            // Bump index number
            set_index(index()+1);
        }
        // When you stop working with the buffer, reset indexer
        set_index(0);
    } else {
        // Do nothing if script is empty
        print!("");
    }
}
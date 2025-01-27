use carrot_libs::args;
use carrot_libs::input;
use std::fs;
use std::process;
use std::thread;
mod directories;
mod exec;
mod global;
mod helpful;
mod r#if;
mod variables;

use global::{
    allow_interrupts, index, interrupt_now, print_err, set_allow_interrupts, set_index,
    set_interrupt_now,
};
use helpful::RushConfig;

#[derive(Debug)]
enum ShellModes {
    /*
    When you encouter an IF block, allow usage for ELSE and ELSEIF.
    Wait, till ;, ELSE or ELSEIF is found and when it happens - make comparison, check if it succeeds and when it does,
    change state of IF to CmpSuccess or CmpFailure.
    */
    If(usize),
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
    After you reach ";", go back to LOCK defined in position_of_lock.
    Allow usage of BREAK and CONTNUE
     */
    Lock(usize),
    /*
    Skip executing commands until you reach END. Go back to LOCK.
    */
    LockContinue(usize),
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
            match input::get(cfg.prompt, false) {
                Ok(e) => {
                    do_rest_of_magic_or_nothing(Some(e));
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
                Ok(e) => do_rest_of_magic_or_nothing(Some(
                    e.split_whitespace().map(str::to_string).collect(),
                )),
                Err(e) => {
                    eprintln!("Unable to read from script file: {:?}", e.kind());
                    process::exit(1);
                }
            }
        }
    };
}

fn do_rest_of_magic_or_nothing(script: Option<Vec<String>>) {
    if let Some(script) = script {
        // Do nothing if script is empty
        if script.is_empty() {
            print!("");
            return;
        }

        group_quotationmarks(script);
    } else {
        print!("");
    }
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

    remove_quotationmarks(buf);
}

// Remove all unescaped quotationmarks
fn remove_quotationmarks(script: Vec<String>) {
    let mut out = Vec::new();
    let mut str = String::new();
    for w in script {
        for c in w.chars() {
            // If current character is a qmark AND it's preceded by a slash, remove the slash from string
            if let Some(last) = str.chars().last() {
                //dbg!(&w, c, c=='\'' || c=='"',last!='\\');
                //println!();
                if (c == '\'' || c == '"') && last == '\\' {
                    str.pop();
                }
            }
            str.push(c);
        }
        out.push(str.clone());
        str.clear();
    }

    syntax_test(out);
}

enum Builtins {
    Lock(usize),
    If(usize),
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
        if w == "lock" {
            used_builtins_history.push(Builtins::Lock(line_number))
        };
        if w == "if" {
            used_builtins_history.push(Builtins::If(line_number))
        };

        // Any logical statement have to be ended with ';'
        // If you find it somewhere, remove the last logical statement from history
        if !w.ends_with("\\;") && w.ends_with(';') {
            used_builtins_history.pop();
        }

        // Catch use of free or continue
        if (w == "free" || w == "continue")
            && !used_builtins_history
                .iter()
                .any(|x| matches!(x, Builtins::Lock(_)))
        {
            errors.push(format!("{line_number}: Usage of \"FREE\" or \"CONTINUE\" is not permited outside of the \"LOCK\" statement"));
        }

        // Disallow running empty commands like this: say hello, , say bye
        if w == "," || w == ";" {
            errors.push(format!("{line_number}: Trying to run empty command!"))
        }
    }

    // Summarize looking for unclosed logical statements
    if !used_builtins_history.is_empty() {
        for element in used_builtins_history {
            match element {
                Builtins::Lock(a) => errors.push(format!("{a}: Unclosed \"LOCK\" statement")),
                Builtins::If(a) => errors.push(format!("{a}: Unclosed \"IF\" statement")),
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
        if interrupt_now() {
            return;
        }
        // Shorthand for currently iterated word of a script
        let w = script[index()].clone();
        // Is this the last word? The script is ending!
        let the_last_word_in_script = index() == script.len() - 1;
        // Add currently iterated word to a temporary buffer
        if !w.ends_with("\\,") && w.ends_with(',') {
            buf.push(w.strip_suffix(',').unwrap().to_string());
        } else if !w.ends_with("\\;") && w.ends_with(';') {
            buf.push(w.strip_suffix(';').unwrap().to_string());
        } else {
            buf.push(w.clone());
        };

        /*
        Comma (,) or line feed (\n) symbolizes next command
        Semicolor (;) symbolizes the end of a logical statement block
            */

        // If we reach the end of a script OR some command separator like '\n' or, ';'...
        if the_last_word_in_script
            || (!w.ends_with("\\,") && w.ends_with(','))
            || (!w.ends_with("\\;") && w.ends_with(';'))
            || w.ends_with('\n')
            || (buf.len() < 2 && w == "lock")
            || (buf.len() < 2 && w == "if")
        // ... try running the command ...
        {
            // First argument in the buffer (buf[0]) is a program name
            // Check whether it's something built into the shell or not.
            let program_name = buf[0].clone();

            /*
            In some scenarios, we can't run any command until we reach ";"
            When this kind of lock is needed, we set this variable to true.

            And these scenarios are simple:
            Any shell_mode[] is set to LockContinue, LockFree or If
            The LAST shell_mode is set to CmpFailure
            */
            let contains_lock_continue = shell_mode
                .iter()
                .any(|s| matches!(s, ShellModes::LockContinue(_)));
            let contains_lock_free = shell_mode.iter().any(|s| matches!(s, ShellModes::LockFree));
            let contains_if = shell_mode.iter().any(|s| matches!(s, ShellModes::If(_)));
            let ends_with_cmp_failure = !shell_mode.is_empty()
                && matches!(shell_mode[&shell_mode.len() - 1], ShellModes::CmpFailure);

            let ultimate_end_lock = contains_if
                || contains_lock_continue
                || contains_lock_free
                || ends_with_cmp_failure;

            //dbg!(&program_name, ultimate_end_lock, contains_lock_continue, contains_lock_free, ends_with_cmp_failure, &shell_mode);

            if !ultimate_end_lock {
                match program_name.as_str() {
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
                        if $val = 5 do
                            free
                            say "Don't run it"
                        ;
                        say "This can't be seen"
                    ;

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
                            print_err(e, program_name, line_number)
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
                            print_err(e, program_name, line_number)
                        }
                    }

                    "if" => {
                        // Increment nesting count
                        nesting_level += 1;

                        // Set shell_mode to Lock and save it's position
                        let position_of_program_name_in_script = index();
                        shell_mode.push(ShellModes::If(position_of_program_name_in_script));
                    }
                    "set" => {
                        if let Err(e) = variables::setenv(&buf) {
                            print_err(e, program_name, line_number);
                        };
                    }
                    "rem" => {
                        if let Err(e) = variables::remenv(&buf) {
                            print_err(e, program_name, line_number);
                        };
                    }
                    "get" => match variables::getenv(&buf) {
                        Ok(res) => println!("{res}"),
                        Err(e) => print_err(e, program_name, line_number),
                    },
                    "++" => {
                        if let Err(e) = variables::chenv(&buf, true) {
                            print_err(e, program_name, line_number);
                        };
                    }
                    "--" => {
                        if let Err(e) = variables::chenv(&buf, false) {
                            print_err(e, program_name, line_number);
                        };
                    }

                    "gt" => {
                        if let Err(e) = directories::gt(&buf) {
                            print_err(e, program_name, line_number);
                        };
                    }

                    "panic" => {
                        panic!("User invoked panic");
                    }

                    // Comments will never be run
                    "#" => {}
                    // Not built in?
                    _ => {
                        if let Err(e) = exec::exec(&buf) {
                            print_err(e, program_name, line_number);
                        }
                    }
                };
            };

            // Check previous command
            if !w.ends_with("\\;") && w.ends_with(';') {
                let nest = nesting_level.saturating_sub(1);
                //std::thread::sleep(std::time::Duration::from_millis(1000));
                if !shell_mode.is_empty() {
                    match shell_mode[nest] {
                        // If current shell_mode is set to Lock or LockContinue,
                        // go back to the index number of lock keyword position in script.
                        ShellModes::Lock(a) | ShellModes::LockContinue(a) => {
                            set_index(a);
                            shell_mode[nest] = ShellModes::Lock(a);
                        }
                        ShellModes::LockFree | ShellModes::CmpSuccess | ShellModes::CmpFailure => {
                            shell_mode.remove(nest);
                            nesting_level = nest;
                        }
                        ShellModes::If(_) => {
                            match r#if::logic(buf.clone()) {
                                Err(e) => {
                                    print_err(e, global::PROGRAM_NAME.to_string(), line_number)
                                }
                                Ok(res) => {
                                    if res {
                                        shell_mode.push(ShellModes::CmpSuccess)
                                    } else {
                                        shell_mode.push(ShellModes::CmpFailure)
                                    }
                                }
                            };
                        }
                    }
                }
            };

            // ... and finally, after command is done, clear the buffer
            buf.clear();
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

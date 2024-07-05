#![allow(dead_code)]
use std::collections::HashMap;
use std::env;
use std::process;
use std::process::Stdio;
use std::io;
use std::io::Write;

// Commands that separate inline commands.
pub const SPLIT_COMMANDS:[&str;2] = ["then", "next"];
// Commands wont be separated by shell by SPLIT_COMMANDS from point where logic operator is found, until "END" is reached.
pub const LOGIC_OPERATORS:[&str;3] = ["if", "loop", "while"];

//
const IF_SPLIT_COMMANDS:[&str;8] = ["if", "elseif", "else", "and", "or", "not", "do", "end"];
const IF_JUMP_SPOTS:[&str;3] = ["elseif", "else", "end"];

/* 
This function was created to run super commands or any other commands.
If non built-in command is inside some logical statements, don't do anything unless forced by logical statement
*/
pub fn detect_commands(commands:&[Vec<String>]) {
    let mut index = 0;
    let mut stop = false;
    
    /*
    This variable contains all required information about exit codes and statuses reported by ALL invoked commands
    this will be used by logic operators to find out if we can continue some operations or not
    Commands separation is done by split_commands()
    */
    let mut returns: HashMap<usize, bool> = HashMap::new();

    while index < commands.len() {
        // Check whether the first argument is a keyword or not
        if !stop {
            match commands[index][0].as_str() {
                "gt" => gt(&commands[index], index, &mut returns),
                // "help" | "?" => help(),
                "exit" | "quit" | "bye" => exit(&commands[index], index, &mut returns),
                "break" => r#break(&commands[index], index, &mut returns),
                "end" | "next" => (),
                "getenv" | "get" => getenv(&commands[index], index, &mut returns),
                "setenv" | "set" => setenv(&commands[index], index, &mut returns),
                "then" => then(&mut index, &mut returns, commands, &mut stop),
                "exec" => exec(&commands[index], index, &mut returns),
                "panic" => panic!("Manually invoked panic message"),
                
                _ => exec(&commands[index], index, &mut returns)
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

// Change working directory
pub fn gt(args:&[String], index:usize, returns:&mut HashMap<usize, bool>) {
    // Check if there is just ONE argument
    // We can't go to more than one directory at the same time
    if args.len() == 1 {
        eprintln!("Give me a directory path to go!");
        // As usual, run this function to report a failure.
        // "index" variable contains position of a command
        // "returns" contains information about all return codes that were reported by commands
        // Both variables are required because "returns" will be modified by "report_failure" according to the contents of "index"
        report_failure(index, returns);
    }
    else if args.len() > 2 {
        eprintln!("Cannot go to multiple directories simultaneously!");
        report_failure(index, returns)
    }
    else {
        match env::set_current_dir(&args[1]) { 
            Err(e) => {
                eprintln!("{}: Cannot go into this directory because of an error: {}", args[1], e.kind());
                report_failure(index, returns);
            },
            Ok(_) => {
                report_success(index, returns);
            }
        };
    };
}

// Just go away with specified exit code
pub fn exit(args:&[String], index:usize, returns:&mut HashMap<usize, bool>) {
    if args.len() == 1 {
        report_failure(index, returns);
        process::exit(0)
    }
    else if args.len() > 2 {
        report_failure(index, returns);
        eprintln!("Cannot exit with multiple exit codes!");
    }
    else {
        match args[1].parse::<i32>() {
            Err(e) => {
                eprintln!("Cannot exit with this code because of an error: {:?}", e.kind());
                report_failure(index, returns);
            },
            Ok(code) => { report_success(index, returns); process::exit(code); },
        }
    };
}

// Exit on special ocasions
pub fn r#break(args:&[String], mut break_keyword_position:usize, returns:&mut HashMap<usize, bool>) {
    // Break if no arguments were passed to BREAK
    if args.len() == 1 {
        process::exit(0);
    }
    // Break if comparison statement returns success
    else {
        // Split all arguments by IF specific IF_SPLIT_COMMANDS
        let all_commands = match split_commands(args[1..].to_owned(), IF_SPLIT_COMMANDS.to_vec(), false) {
            Err(e) => { eprintln!("IF OPERATOR FAILED! {e}!"); process::exit(1) },
            Ok(e) => e,
        };
        // dbg!(&all_commands);
        // Save position of break_keyword_position
        let bruh = break_keyword_position;
        make_comparison(&mut break_keyword_position, &all_commands, returns, false, Some(bruh+all_commands.len()))
    }
}

fn then(index_of_then:&mut usize, returns: &mut HashMap<usize, bool>, commands: &[Vec<String>], stop:&mut bool) {
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
        returns.get(&prev_index).unwrap()
    }
    else {
        eprintln!("OPERATOR \"THEN\" FAILED! Unable to read exit code of the previous command!");
        *stop=true;
        &false
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


// These functions will be used to report success or failure when built-in or super commands are running
// This is usefull because typically we don't want the shell to abnormally quit when syntax of "if" statement is incorrect
// Instead, we just want to say "Hey! There is a bug!"
// BUT when rush would work as a subshell just to execute a script, we won't even need it anymore
pub fn report_success(index:usize, returns:&mut HashMap<usize, bool>) {
    let command_status = true;
    returns.insert(index, command_status);
}
pub fn report_failure(index:usize, returns:&mut HashMap<usize, bool>) {
    let command_status = false;
    returns.insert(index, command_status);
}

pub fn split_commands(mut words:Vec<String>, spliting_keywords:Vec<&str>, split_when_inside_logic_op:bool) -> Result<Vec<Vec<String>>, String> {
    // This list contains all commands passed by the user 
    let mut commands: Vec<Vec<String>> = Vec::new();
    // List of words in one command
    let mut command = Vec::new();
    /*
    This will be used to separate built-in commands from anything else
    Expected output: ('af' 'file'), ('then'), ('ad' 'dir')
    */ 

    // First of all, when there's a word that starts with "$"
    // replace it with variable contents
    let mut i = 0;
    while i < words.len() {
        // Save currently tested word to "w"
        let w = &words[i];
        // Remove unnecessary prefix ($varname -> varname)
        let sp = w.strip_prefix('$');
        // If contents of "w" are not empty, replace word with variable contents
        if let Some(w) = sp {
            // Get variable contents
            let variable_contents=env::var_os(w).unwrap_or_default();
            if variable_contents.is_empty() {
                return Err(format!("Variable \"{}\" is empty or undefinned", sp.unwrap()));
            }
            // Remove current command
            words.remove(i);
            // Append contents of a variable
            words.insert(i, variable_contents.into_string().unwrap_or_default());
        }
        i += 1;
    };
   
    // Split commands in place of any new-line character
    let mut index = 0;
    while index < words.len() {
        if words[index].contains('\n') && !words[index].ends_with('\n') {
            println!("Got a newline!");
            println!("Word: {} at index: {}", words[index], index);
            let dont_die = words[index].clone();
            let word_splitted_by_newlines = dont_die.rsplit_terminator('\n');
            // dbg!(&word_splitted_by_newlines);
            // Remove old word from "words"
            words.remove(index);
            // Add new collection of words in place of older one
            for w in word_splitted_by_newlines {
                words.insert(index, w.to_string());
            }
            // Additionally, add "newline" keyword so the loop below will split commands where newline character was inserted
            words.insert(index, String::from("newline"));
            dbg!(&words);
        }
        // If word ends with new line character, add "newline" AFTER the current word
        else if words[index].contains('\n') && words[index].ends_with('\n') {
            words.insert(index+1, String::from("newline"));
        }
        index += 1;
    }

    let mut index = 0;
    /*
    The shell was designed to NOT split commands if we're inside of some logic. (In IF statement, for example)
    if cmp 11 = 11 do
        say "hello"
        say "it is equal to eleven" next say "so lucky."
    end

    In the code above, function split_commands() wouldn't split commands by newline character nor by the "next" keyword.
    The IF statement should do this manually.
    */
    let mut logic_operators_depth = 0;
    let logic_operators = LOGIC_OPERATORS.to_vec();
    while index < words.len() {
        // Words starting with a quote have to be tolerated as one, large command
        if words[index].starts_with('\'') || words[index].starts_with('"') {
            // Build one large argument from words in quotes
            let mut joined = String::new();
            loop {
                // Remove quotes from word, if any
                let stripped_word = strip_quotes(&words[index]);
                // println!("Got stripped word: {}", stripped_word);
                // Add word to 'joined' with additional space at the end
                joined.push_str(&format!("{} ", stripped_word));
                // println!("Joined contents: {joined}");
                // If we find the end of quotation
                if words[index].ends_with('\'') || words[index].ends_with('"') {
                    // Add final word to 'joined'
                    // println!("Index {index}: Word {} is ending a quote", words[index]);
                    joined = joined.strip_suffix(' ').unwrap().to_string(); // TIP: Space at the end of the word is no longer needed ;)
                    // println!("Joined contents: {joined}");
                    // Remove final word from 'words'
                    if !words.is_empty() {
                        words.remove(index);
                    }
                    // Add all collected words in quotes, stored in 'joined' to 'words' 
                    words.insert(index, joined);
                    // println!("Current index is: {}", index);
                    // println!("Words lenght is: {}", words.len());
                    // dbg!(index+1==words.len());
                    // No more words? 
                    if index+1 == words.len() {
                        // Add collected words to 'commands'.
                        commands.push(words[..index+1].to_vec());
                    }
                    break;
                }
                // Remove current word from 'words' list. We no longer need it since it is added to 'joined'.
                words.remove(index);
            }
            index+=1;
        }
        // Split commands in place of any built-in command
        else {
            if logic_operators.contains(&words[index].as_str()) && split_when_inside_logic_op {
                logic_operators_depth += 1;
            } else if words[index] == "end" && logic_operators_depth != 0 && split_when_inside_logic_op {
                logic_operators_depth -= 1;
            }

            let inside_logic_operator = logic_operators_depth != 0;

            // If built-in or newline keyword appears AND if we're not in logical operation
            if (spliting_keywords.contains(&words[index].as_str()) || spliting_keywords.contains(&"newline")) && !inside_logic_operator {
                // println!("Index {index}: Word {} looks like a keyword to split.", words[index]);
                // println!("Is keyword {} in a logic? {}", words[index], inside_logic_operator);

                // Separate CURRENT keyword from PREVIOUSLY collected words
                // Expected output: ('af' 'file'), ('then' 'ad' 'dir')
                let (before_keyword, right) = words.split_at(index);
                // Convert everything to a vector
                let (before_keyword, right) = (before_keyword.to_vec(), right.to_vec());

                // Separate keyword from NEXT words, that are not collected yet
                // Expected output: ('af' 'file'), ('then'), ('ad' 'dir')
                let (keyword, after_keyword) = {
                    let (keyword, after_keyword) = right.split_at(1);
                    (keyword.to_vec(), after_keyword.to_vec())
                };

                // Send previous words to "commands"
                // Example: ('af' 'file')
                if !before_keyword.is_empty() {
                    // Do not append anything if there is nothing before keyword!
                    commands.push(before_keyword.to_vec());
                }
                // Send keyword to "commands" exclusively
                // Example: ('then')
                commands.push(keyword.to_vec());
                // We no longer need to deal with ('af' 'file') and ('then') so remove them from words
                words = after_keyword.to_vec();
                // Start over with new words
                // Example: ('ad' 'dir')
                index = 0;
            }
            // If there are no built-in commands nor "newline" keyword or if we're in logical operator
            else if (!spliting_keywords.contains(&words[index].as_str()) && !spliting_keywords.contains(&"newline")) || inside_logic_operator {
                // println!("Index {index}: Word {} looks like a normal word.", words[index]);
                // Just add the words to the 'command' variable
                command.push(words[index].clone());
                index+=1;
                // No more words? 
                if index == words.len() {
                    // Add collected words to 'commands'.
                    commands.push(words[..index].to_vec());
                }
            };
        };
    }
    Ok(commands)
}

pub fn strip_quotes(input:&str) -> String {
    let mut output = input.to_string();
    if output.starts_with('\'') {
        output = output.strip_prefix('\'').unwrap().to_string();
    }
    if output.starts_with('"') {
        output = output.strip_prefix('"').unwrap().to_string();
    }

    if output.ends_with('\'') {
        output = output.strip_suffix('\'').unwrap().to_string();
    }
    if output.ends_with('"'){
        output = output.strip_suffix('"').unwrap().to_string();
    }
    output
}

// This will be used to execute commands!
pub fn exec(args:&[String], index:usize, returns:&mut HashMap<usize, bool>) {
    // Do nothing if nothing was requested
    // This might occur when the user presses ENTER without even typing anything
    if args.is_empty() || args[0].is_empty() {
        print!("");
    }

    // Run a command passed in "args[0]" with arguments in "args[1]" (and so on) and get it's status
    // using process::Command::new().args().status();
    match process::Command::new(&args[0]).args(&args[1..]).status() { 
        Err(e) => {
            eprintln!("{}: Command execution failed: {:?}", args[0], e.kind());
            report_failure(index, returns)
        },
        Ok(process) => {
            // If the command is possible to run, save it's status to "returns" variable
            let command_status = process.success();
            returns.insert(index, command_status);
        },
    }
    // Flush stdout
    io::stdout().flush().unwrap();
}

// This will be used to execute commands and get it's contents!
pub fn getoutput_exec(args:&[String]) -> process::Output {
    // Do nothing if nothing was requested
    // This might occur when the user presses ENTER without even typing anything
    if args.is_empty() || args[0].is_empty() {
        print!("");
    }
    
    // Run a command passed in "args[0]" with arguments in "args[1]" (and so on) and get it's status
    // using process::Command::new().args().status();
    match process::Command::new(&args[0]).args(&args[1..]).output() { 
        Err(e) => {
            eprintln!("{}: Command execution failed: {:?}", args[0], e.kind());
            process::exit(1);
        },
        Ok(process) => {
            // If the command is possible to run, save it's status to "returns" variable
            process
        },
    }
}

pub fn silent_exec(args:&[String], index:usize, returns:&mut HashMap<usize, bool>) {
    // Run a command passed in "args[0]" with arguments in "args[1]" (and so on) and collect it's status to "returns"
    match process::Command::new(&args[0]).args(&args[1..]).stdout(Stdio::null()).status() { 
        Err(e) => {
            eprintln!("{}: Command execution failed because of an error: {}", args[0], e.kind());
            report_failure(index, returns)
        },
        Ok(process) => {
            // If the command is possible to run, save it's status to "returns" variable
            let command_status = process.success();
            returns.insert(index, command_status);
        },
    }
}

use std::env::var_os;
use std::ffi::OsString;
pub fn getenv(args:&[String], index:usize, returns:&mut HashMap<usize, bool>) {
    // Check if there is just ONE argument
    // We can't check more than one variable at the same time
    if args.len() == 1 {
        eprintln!("Give me a variable name to check!");
        // As usual, run this function to report a failure.
        // "index" variable contains position of a command
        // "returns" contains information about all return codes that were reported by commands
        // Both variables are required because "returns" will be modified by "report_failure" according to the contents of "index"
        report_failure(index, returns);
    }
    else if args.len() > 2 {
        eprintln!("Cannot check multiple variables simultaneously!");
        report_failure(index, returns)
    }
    else {
        let variable = match var_os(&args[1]) {
            Some(ret) => ret,
            None => { 
                eprintln!("GETENV FAILED! Variable \"{}\" is not set!", args[1]);
                report_failure(index, returns);
                OsString::new()
            }
        };
        if let Ok(a) = variable.into_string() {
             if !a.is_empty() {
                println!("{}", a);
             }
        }
        report_success(index, returns);
    }
}

use std::env::set_var;
pub fn setenv(args:&[String], index:usize, returns:&mut HashMap<usize, bool>) {
    // Check if there is just ONE argument
    // We can't set more than one variable at the same time
    if args.len() == 1 {
        eprintln!("Give me a variable name to set!");
        // As usual, run this function to report a failure.
        // "index" variable contains position of a command
        // "returns" contains information about all return codes that were reported by commands
        // Both variables are required because "returns" will be modified by "report_failure" according to the contents of "index"
        report_failure(index, returns);
    }
    else if args.len() > 2 {
        eprintln!("Cannot set multiple variables simultaneously!");
        report_failure(index, returns)
    }
    else {
        match args[1].split_once('=') {
            Some((key, value)) => {
                if key.is_empty() || value.is_empty() {
                    eprintln!("OPEATOR \"SETENV\" FAILED! Incorretly requested variable!");
                    report_failure(index, returns);
                }
                else {
                    set_var(key, value);
                    report_success(index, returns);
                }
            }
            _ => {
                eprintln!("OPEATOR \"SETENV\" FAILED! Incorretly requested variable!");
                report_failure(index, returns);
            }
        };

    }
}

/*
Place where we invoke super commands
This function takes some usefull arguments:
idx - Location of currently running super operator.
args - All options passed to this program in unchanged form.
all_commands - List of commands splitted by IF-specific IF_SPLIT_COMMANDS constant. Usefull for comparison statement but not in the task.
returns - List of all return statuses from commands
run_as_else - Indicate that we're running as "else" command
jump_spot_position - Index of nearest jump spot position can be found automatically (if none is sent) or set to a fixed value
*/
pub fn make_comparison(idx:&mut usize, all_commands:&[Vec<String>], returns:&mut HashMap<usize, bool>, run_as_else:bool, jump_spot_position:Option<usize>) {
    // This is where current super operator (IF/ELSEIF/ELSE) is located in options
    // TIP: IF is not defined in options but let's assume that it's index number is zero if we're starting IF logic.
    let super_operator_index = *idx;

    // Find out where closest jump spot is located
    let jump_spot_position:usize = if let Some(a) = jump_spot_position {
        a
    } else {
        all_commands.iter().position(|x| IF_JUMP_SPOTS.contains(&x[0].as_str())).unwrap()
    };

    let (shall_we_move_on, task_commands) = if !run_as_else {
        // Position of commands between IF/ELSEIF and DO
        // Comparison operator and "DO" is not present in case of running as "ELSE"
        let comparison_statement_starting_position = super_operator_index+1;

        // Find out where "DO" is located
        let do_keyword_position = all_commands[super_operator_index..].iter().position(|x| x[0] == "do").unwrap() + super_operator_index;

        // Protect from writing "if do", "elseif do" and "else do". "DO" have to be preceeded with something different than just a
        // super operator "if"
        if do_keyword_position == super_operator_index+1 {
            eprintln!("SYNTAX ERROR! Comparison statement is empty!");
            process::exit(1);
        }

        // This is a list containing everything between current IF/ELSEIF/ELSE and DO
        // dbg!(comparison_statement_starting_position, do_keyword_position);
        let comparison_statement_commands = &all_commands[comparison_statement_starting_position..do_keyword_position].to_vec();
        // This is a list containing commands between DO and closest jump spot
        // NOTE: When separating task commands, do not use IF-specific SPLIT_COMMANDS. Use those defined in helpful instead.
        let task_commands = match split_commands(all_commands[do_keyword_position+1].to_owned(), SPLIT_COMMANDS.to_vec(), false) {
            Err(e) => {eprintln!("IF OPERATOR FAILED! {e}!"); process::exit(1)},
            Ok(e) => e,
        };
        // dbg!(&all_commands[do_keyword_position+1]);

        // Run all commands inside comparison statement
        let mut index = 0;
        while index < comparison_statement_commands.len() {
            // Execute all commands and collect their statuses to "returns"
            if !SPLIT_COMMANDS.contains(&comparison_statement_commands[index][0].as_str()) {
                silent_exec(&comparison_statement_commands[index], index+comparison_statement_starting_position, returns);
            }
            index += 1;
        }

        // When exit codes of all commands inside comparison statement are known - try executing AND, OR operators
        let mut index = 0;
        while index < comparison_statement_commands.len() && comparison_statement_commands[index][0] != "do" {
            match comparison_statement_commands[index][0].as_str() {
                "and" => and(index, returns),
                "or" => or(index, returns),
                "not" => not(index, returns),
                "else" | "elseif" | "end" | "if" => {
                    eprintln!("SYNTAX ERROR! Operator \"{}\" was found in a comparison statement!", comparison_statement_commands[index][0]);
                    process::exit(1);
                },
                _ => (),
            }
            index+=1;
        }

        // Final check - Is every command in comparison statement successfull?
        (
            check_statuses(returns, comparison_statement_starting_position, do_keyword_position),
            task_commands
        )
    }
    // We simply don't care about any comparison statement or "DO".
    // "Else" exists to run ANY command as a fallback to the situation, when nothing else works
    else {
        (
            true,
            match split_commands(all_commands[super_operator_index+1].to_vec(), SPLIT_COMMANDS.to_vec(), false) {
                Err(e) => {eprintln!("IF OPERATOR FAILED! {e}!"); process::exit(1)},
                Ok(e) => e,
            }
        )
    };

    // Check if every command after "IF" returned success.
    if shall_we_move_on {
        do_the_task(task_commands);
        // We don't want to run any other ELSEIF or ELSE after current task is done.
        process::exit(0);
    } else {
        //dbg!(&all_commands[jump_spot_position]);
        // Exit unsuccessfully if next jump spot is an "END"
        if all_commands[jump_spot_position][0] == "end" {
            process::exit(1);
        }
        // Else, change "idx" to the position of another jump spot ant try executing next elseifs or finishing else
        else {
            *idx += jump_spot_position;
        }
    }
}

// BEWARE! Function check_statuses() has to check them only for commands inside the comparison operator that is currently running!
// This is why this function scans through "returns" only for values fitting in range from "comparison_statement_starting_position"
// to "do_keyword_position".
pub fn check_statuses(returns:&HashMap<usize, bool>, start:usize, end:usize) -> bool {
    let mut ok = true;
    //dbg!(returns, start, end);
    let mut index = start;
    while index < end {
        // dbg!(returns.get(&index).unwrap());
        // If there is at least one unsuccessfull command - quit
        if !returns.get(&index).unwrap() {
            ok = false;
            break;
        }
        index += 1;
    };
    ok
}

pub fn do_the_task(commands: Vec<Vec<String>>) {
    detect_commands(&commands);
}

// This checks exit code of commands executed before and after.
// Then, it returns true ONLY IF BOTH return codes are positive
pub fn and(index_of_and:usize, returns: &mut HashMap<usize, bool>) {
    if index_of_and == 0 {
        eprintln!("SYNTAX ERROR! Operator \"AND\" doesn't work when there is nothing before it!");
        report_failure(index_of_and, returns);
        process::exit(1);
    }
    if !returns.contains_key(&(index_of_and+1)) {
        eprintln!("SYNTAX ERROR! Operator \"AND\" doesn't work when there is nothing after it!");
        report_failure(index_of_and, returns);
        process::exit(1);
    }
    // Compare exit status of previous and following commands
    let prev_index = index_of_and-1;
    let next_index = index_of_and+1;
    let prev_status = if returns.contains_key(&prev_index) {
        returns.get(&prev_index).unwrap()
    }
    else {
        eprintln!("OPERATOR \"AND\" FAILED! Unable to read exit code of the previous command!");
        process::exit(1);
    };
    let next_status = if returns.contains_key(&next_index) {
        returns.get(&next_index).unwrap()
    }
    else {
        eprintln!("OPERATOR \"AND\" FAILED! Unable to read exit code of the next command!");
        process::exit(1);
    };

    if *prev_status && *next_status {
        report_success(index_of_and, returns);
    } else {
        report_failure(index_of_and, returns);
    }
}

// This checks return code before and after it and returns true IF ANY return codes are positive
pub fn or(index_of_or:usize, returns: &mut HashMap<usize, bool>) {
    if index_of_or == 0 {
        eprintln!("SYNTAX ERROR! Operator \"OR\" doesn't work when there is nothing before it!");
        process::exit(1);
    }
    if !returns.contains_key(&(index_of_or+1)) {
        eprintln!("SYNTAX ERROR! Operator \"OR\" doesn't work when there is nothing after it!");
        process::exit(1);
    }
    // Compare previous and following commands
    let prev = index_of_or-1;
    let next = index_of_or+1;
    if *returns.get(&prev).unwrap() || *returns.get(&next).unwrap() {
        report_success(index_of_or, returns);
        // Overwrite the status of both exit codes to fool the if (or any other) logic that every command is ok
        report_success(prev, returns);
        report_success(next, returns);
    } else {
        report_failure(index_of_or, returns);
    }
}

// This changes the return code after it
pub fn not(index_of_not:usize, returns: &mut HashMap<usize, bool>) {
    if !returns.contains_key(&(index_of_not+1)) {
        eprintln!("SYNTAX ERROR! Operator \"NOT\" doesn't work when there is nothing after it!");
        process::exit(1);
    }
    // Return code of "NOT" doesn't matter
    report_success(index_of_not, returns);

    // Get exit code of the next command
    let next = index_of_not+1;
    if *returns.get(&next).unwrap() {
        // Overwrite the status of the next exit code
        report_failure(next, returns);
    } else {
        report_success(next, returns);
    }
}
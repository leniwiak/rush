#![allow(dead_code)]
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::process;
use std::process::Stdio;
use num_bigint::BigInt;

#[derive(Serialize, Deserialize)]
pub struct RushConfig {
    pub prompt: String,
    pub aliases: HashMap<String, String>,
}
// `Default` settings for `MyConfig`
impl ::std::default::Default for RushConfig {
    fn default() -> Self { 
        Self { 
            prompt: "> ".into(),
            aliases: HashMap::new(),
        } 
    }
}

// Commands that separate inline commands.
pub const SPLIT_COMMANDS:[&str;2] = ["then", "next"];
// Commands from point where logic operator is found, until "END"
// wont be separated automatically by shell by words defined in SPLIT_COMMANDS.
// Also, shell won't expand variable references ($varname syntax) between these words and next 'do'
pub const LOGIC_OPERATORS:[&str;3] = ["if", "loop", "for"];

//
const IF_SPLIT_COMMANDS:[&str;8] = ["if", "elseif", "else", "and", "or", "not", "do", "end"];
const IF_JUMP_SPOTS:[&str;3] = ["elseif", "else", "end"];
const FOR_SPLIT_COMMANDS:[&str;8] = ["for", "in", "persign", "perword", "perline", "step", "do", "end"];

/* 
This function was created to run commands with additional super commands detection.
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
        // Shortcut list containing all arguents of currently iterated command
        let mut this_command = commands[index].clone();

        // println!("Got command: {:?}", this_command);

        // When there's a word that starts with some particular character,
        // replace it with variable contents
        let mut i = 0;
        while i < this_command.len() {
            // Dolar sign is used to replace variable name with it's contents
            // Save currently tested word to "word"
            let word = &this_command[i];

            if word.starts_with('$') && (!word.ends_with("++") && !word.ends_with("--")) {
                // Remove unnecessary prefix ($varname -> varname)
                let stripped_prefix = word.strip_prefix('$');
                // If contents of "w" are not empty, replace word with variable contents
                if let Some(w) = stripped_prefix {
                    // Get variable contents
                    let variable_contents=env::var_os(w).unwrap_or_default();
                    if variable_contents.is_empty() {
                        eprintln!("ILLEGAL OPERATION! Mentioned variable is empty or undefinned: {}!", stripped_prefix.unwrap());
                        return;
                    }
                    // Remove current word from a command
                    this_command.remove(i);
                    // Append contents of a variable to a command
                    this_command.insert(i, variable_contents.into_string().unwrap_or_default());
                }
            }
            else if word.starts_with('~') {
                // Tilde will be replaced with user's full home directory path
                // Get variable contents
                let homedir=env::var_os("HOME").unwrap_or_default().into_string().unwrap();
                if homedir.is_empty() {
                    eprintln!("ILLEGAL OPERATION! User's home directory path is undefinned!");
                    return;
                }
                // Remove tilde from currently iterated word
                let mut modified_word = word.strip_prefix('~').unwrap().to_string();
                // Add homedir path to the start of the word
                modified_word.insert_str(0, &homedir);

                // Remove current word from a command
                this_command.remove(i);
                // Append contents of a variable to a command
                this_command.insert(i, modified_word.to_owned());
            }
            else if word.starts_with('$') && (word.ends_with("++") || word.ends_with("--")) {
                let addition = word.ends_with("++");
                // ++varname will be replaced with contents of a variable with one added to it
                // NOTE: This will only work if a variable contains a number
                
                // Remove "$" (if any) and "++" or "--" from currently iterated word
                let mut modified_word = word.strip_prefix('$').unwrap().to_string();
                modified_word = if addition {
                    modified_word.strip_suffix("++").unwrap().to_string()
                }
                else {
                    modified_word.strip_suffix("--").unwrap().to_string()
                };
                // Get variable contents
                let variable_contents=env::var_os(&modified_word).unwrap_or_default().into_string().unwrap();
                if variable_contents.is_empty() {
                    eprintln!("ILLEGAL OPERATION! Calculation failed because variable is empty or undefinned: {}!", modified_word);
                    return;
                }
                let mut parsed_variable = match variable_contents.parse::<BigInt>() {
                    Ok(res) => res,
                    Err(_) => {
                        eprintln!("ILLEGAL OPERATION! Calculation failed because variable is not a number: {}!", modified_word);
                        return;
                    },
                };
                // Add/substract one to/from a parsed variable
                if addition {
                    parsed_variable+=1;
                } else {
                    parsed_variable-=1;
                }
                // Save new variable contents to environment
                //set_var(modified_word, parsed_variable.to_string());
                // Remove current word from a command
                this_command.remove(i);
                // Append contents of a variable to a command
                this_command.insert(i, parsed_variable.to_string());
            }
            i += 1;
        };

        // Remove quotes from words
        // If strip_quotes find and (hopefuly) removes quotes, the for loop will
        // remove enquoted word and replace it with a new one
        for (i, w) in this_command.clone().iter().enumerate() {
            if let Some(dequoted) = strip_quotes(w) {
                this_command.remove(i);
                this_command.insert(i, dequoted);
            };
        }

        // Check whether the first argument is a keyword or not
        if !stop {
            match this_command[0].as_str() {
                // "help" | "?" => help(),
                "exit" | "quit" | "bye" => exit(&this_command, index, &mut returns),
                "break" => r#break(&this_command, index, &mut returns),
                "end" | "next" => (),
                "alias" => alias(&this_command, index, &mut returns),
                //"getenv" | "get" => getenv(&this_command, index, &mut returns),
                //"setenv" | "set" => setenv(&this_command, index, &mut returns),
                //"unsetenv" | "unset" => unsetenv(&this_command, index, &mut returns),
                "then" => then(&mut index, &mut returns, commands, &mut stop),
                //"exec" => exec(&this_command, index, &mut returns),
                "panic" => panic!("Manually invoked panic message"),
                
                _ => todo!()
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
pub fn r#break(args:&[String], break_keyword_position:usize, returns:&mut HashMap<usize, bool>) {
    // Break if no arguments were passed to BREAK
    if args.len() == 1 {
        process::exit(0);
    }
    // Break if comparison statement returns success
    else {
        //let comparison_statement_commands = &args[1..];
        // Split all arguments by normal splitting keywords
        let comparison_statement_commands = match split_commands(args[1..].to_vec(), SPLIT_COMMANDS.to_vec(), false) {
            Err(e) => { eprintln!("BREAK OPERATOR FAILED! {e}!"); process::exit(1) },
            Ok(e) => e,
        };

        // Run all commands inside comparison statement
        let mut index = 0;
        while index < comparison_statement_commands.len() {
            // Execute all commands and collect their statuses to "returns"
            if !SPLIT_COMMANDS.contains(&comparison_statement_commands[index][0].as_str()) {
                silent_exec(&comparison_statement_commands[index], index+break_keyword_position, returns);
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
        // dbg!(&returns, break_keyword_position, comparison_statement_commands.len()+break_keyword_position);
        if check_statuses(returns, break_keyword_position, comparison_statement_commands.len()+break_keyword_position) {
            process::exit(0);
        }
    }
}

pub fn alias(args:&[String], index:usize, returns:&mut HashMap<usize, bool>) {
    // Check if there is just ONE argument
    // We can't set more than one variable at the same time
    if args.len() == 1 {
        eprintln!("Give me at least one alias name to check!");
        // As usual, run this function to report a failure.
        // "index" variable contains position of a command
        // "returns" contains information about all return codes that were reported by commands
        // Both variables are required because "returns" will be modified by "report_failure" according to the contents of "index"
        report_failure(index, returns);
    }
    else {
        let cfg:RushConfig = match confy::load("rush", "rush") {
            Err(e) => { eprintln!("Failed to read config file: {}!", e); process::exit(1)},
            Ok(e) => e,
        };
        let aliases = cfg.aliases;

        for arg in args.iter().skip(1) {
            match aliases.get_key_value(arg) {
                None => eprintln!("{}: Alias not set!", arg),
                Some(e) => eprintln!("{}", e.1),
            };
        }
        report_success(index, returns);
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
   
    // Split commands in place of any new-line character
    let mut index = 0;
    while index < words.len() {
        if words[index].contains('\n') && !words[index].ends_with('\n') {
            // println!("Got a newline!");
            // println!("Word: {} at index: {}", words[index], index);
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
            // dbg!(&words);
        }
        // If word ends with new line character, add "newline" AFTER the current word
        else if words[index].contains('\n') && words[index].ends_with('\n') {
            words.insert(index+1, String::from("newline"));
        }
        index += 1;
    }

    // (Almost) always replace aliased words with their value
    // TIP: Don't do anything if previous word is "alias",
    // Writing the name of "alias" command and using aliased word typically means that user wants to check
    // if alias is set. This is an exception to the rule above
    let cfg:RushConfig = confy::load("rush", "rush").unwrap();
    // TIP: Using .unwrap() above is okay. We couldn't even load the shell if loading config file would be impossible

    if !cfg.aliases.is_empty() {
        let mut index = 0;
        while index < words.len() {
            if cfg.aliases.contains_key(&words[index]) && !(index != 0 && words[index-1] == "alias") {
                if let Some(ret) = cfg.aliases.get_key_value(&words[index]) {
                    // Remove aliased word from the list of words
                    words.remove(index);
                    // Separate words by space in alias's value
                    let words_in_alias_value = ret.1.split_whitespace();
                    for w in words_in_alias_value.rev() {
                        words.insert(index, w.to_owned());
                    }
                }
            }
            index += 1;
        }
    }

    /*
    The shell was designed NOT to split commands if we're inside of some logic. (In IF statement, for example)
    The entire if statement block must be accessible for if command to work

    In the code below, function split_commands() wouldn't split commands by newline character nor by the "next" keyword.
    The IF statement should do this manually.

    if cmp 11 = 11 then
        say "hello"
        say "it is equal to eleven" next say "so lucky."
    end
    */

    // Connect words in quotes
    let mut index = 0;
    let mut logic_operators_depth = 0;
    let logic_operators = LOGIC_OPERATORS.to_vec();
    while index < words.len() {
        // Words starting with a quote have to be tolerated as one, large command
        if words[index].starts_with('\'') || words[index].starts_with('"') {
            // Build one large argument from words in quotes
            let mut joined = String::new();
            loop {
                // println!("Got stripped word: {}", stripped_word);
                // Add word to 'joined' with additional space at the end
                joined.push_str(&format!("{} ", words[index]));
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

pub fn strip_quotes(input:&str) -> Option<String> {
    let mut did_it_change = false;
    let mut output = input.to_string();
    if output.starts_with('\'') {
        did_it_change = true;
        output = output.strip_prefix('\'').unwrap().to_string();
    }
    if output.starts_with('"') {
        did_it_change = true;
        output = output.strip_prefix('"').unwrap().to_string();
    }
    if output.ends_with('\'') {
        did_it_change = true;
        output = output.strip_suffix('\'').unwrap().to_string();
    }
    if output.ends_with('"'){
        did_it_change = true;
        output = output.strip_suffix('"').unwrap().to_string();
    }
    if did_it_change {
        Some(output)
    } else {
        None
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


/*
Place where we invoke super commands
This function takes some usefull arguments:
idx - Location of currently running super operator.
args - All options passed to this program in unchanged form.
all_commands - List of commands splitted by IF-specific IF_SPLIT_COMMANDS constant. Usefull for comparison statement but not in the task.
returns - List of all return statuses from commands
run_as_else - Indicate that we're running as "else" command
*/
pub fn make_comparison(idx:&mut usize, all_commands:&[Vec<String>], returns:&mut HashMap<usize, bool>, run_as_else:bool) {
    // This is where current super operator (IF/ELSEIF/ELSE) is located in options
    // TIP: IF is not defined in options but let's assume that it's index number is zero if we're starting IF logic.
    let super_operator_index = *idx;

    // Find out where closest jump spot is located
    // or use fixed value from "end_comparison" when running from the break().
    let jump_spot_position = all_commands.iter().position(|x| IF_JUMP_SPOTS.contains(&x[0].as_str())).unwrap();

    let (shall_we_move_on, task_commands) = if !run_as_else {
        // Position of commands between IF/ELSEIF and DO
        // TIP: Comparison operator and "DO" is not present in case of running as "ELSE"
        let comparison_statement_starting_position = super_operator_index+1;

        // Find out where "DO" is located
        // or use fixed value from "end_comparison" when running from the break().
        let do_keyword_position = all_commands[super_operator_index..].iter().position(|x| x[0] == "do").unwrap();

        // Protect from writing "if do", "elseif do" and "else do".
        // "DO" has to be preceeded with something different than just a super operator (if/else or elseif)
        // DO NOT run this test when end_comparison is defined.

        // If this is the case, we are running from
        // break(), so "do" keyword is not present!
        if do_keyword_position == super_operator_index+1 {
            // eprintln!("{super_operator_index} {do_keyword_position}");
            eprintln!("SYNTAX ERROR! Comparison statement is empty!");
            process::exit(1);
        }

        // This is a list containing everything between current IF/ELSEIF/ELSE and DO
        // dbg!(&all_commands, comparison_statement_starting_position, do_keyword_position);
        let comparison_statement_commands = &all_commands[comparison_statement_starting_position..do_keyword_position].to_vec();

        // This is a list containing commands between DO and closest jump spot
        // NOTE: When separating task commands, do not use IF-specific SPLIT_COMMANDS. Use those defined in helpful instead.
        let task_commands = match split_commands(all_commands[do_keyword_position+1].to_owned(), SPLIT_COMMANDS.to_vec(), false) {
            Err(e) => {eprintln!("IF OPERATOR FAILED! {e}!"); process::exit(1)},
            Ok(e) => e,
        };
        // dbg!(&all_commands[do_keyword_position+1]);

        // Run all NORMAL commands inside comparison statement
        //dbg!(comparison_statement_commands);
        let mut index = 0;
        while index < comparison_statement_commands.len() && comparison_statement_commands[index][0] != "do" {
            // Execute all NORMAL commands and collect their statuses to "returns"
            // println!("COMMAND {:?}", &comparison_statement_commands[index]);
            if !IF_SPLIT_COMMANDS.contains(&comparison_statement_commands[index][0].as_str()) {
                // println!("IS BEING RUN. IT HAS INDEX NUMER {}", index+comparison_statement_starting_position);
                silent_exec(&comparison_statement_commands[index], index+comparison_statement_starting_position, returns);
            }
            index += 1;
        }


        // When exit codes of all NORMAL commands inside comparison statement are known
        // Run AND, OR, NOT operators
        // dbg!(comparison_statement_commands);
        let mut index = 0;
        while index < comparison_statement_commands.len() && comparison_statement_commands[index][0] != "do" {
            // println!("COMMAND {:?}", &comparison_statement_commands[index]);
            match comparison_statement_commands[index][0].as_str() {
                "and" => and(index+comparison_statement_starting_position, returns),
                "or" => or(index+comparison_statement_starting_position, returns),
                "not" => not(index+comparison_statement_starting_position, returns),
                "else" | "elseif" | "end" | "if" => {
                    eprintln!("SYNTAX ERROR! Operator \"{}\" was found in a comparison statement!", comparison_statement_commands[index+comparison_statement_starting_position][0]);
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
        // If there is at least one unsuccessfull command - quit
        // println!("Looking for result with index: {index}");
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

// This checks exit code of commands executed before and after a keyword.
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

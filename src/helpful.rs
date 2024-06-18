#![allow(dead_code)]

use std::collections::HashMap;
use std::env;
use std::process;
use std::process::Stdio;
use std::io;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::thread;

#[derive(Debug)]
 pub struct CommandStatus {
    pub code: Option<i32>,
    pub success: bool,
    pub signal: Option<i32>,
    pub core_dumped: bool 
}

// Commands that separate inline commands
pub const SPLIT_COMMANDS:[&str;3] = ["then", "next", "end"];

// These functions will be used to report success or failure when built-in or super commands are running
// This is usefull because typically we don't want the shell to abnormally quit when syntax of "if" statement is incorrect
// Instead, we just want to say "Hey! There is a bug!"
// BUT when rush would work as a subshell just to execute a script, we won't even need it anymore
pub fn report_success(index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    let command_status = CommandStatus {code: Some(0),success: true,signal: None,core_dumped: false};
    returns.insert(index, command_status);
}
pub fn report_failure(index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    let command_status = CommandStatus {code: Some(1),success: false,signal: None,core_dumped: false};
    returns.insert(index, command_status);
}

pub fn split_commands(mut words:Vec<String>, spliting_keywords:Vec<&str>) -> Vec<Vec<String>> {
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
        if words[index].contains('\n') {
            let dont_die = words[index].clone();
            let word_splitted_by_newlines = dont_die.split_terminator('\n');
            // Remove old word from "words"
            words.remove(index);
            // Add new collection of words in place of older one
            for w in word_splitted_by_newlines {
                words.insert(index, w.to_string());
            }
        }
        index += 1;
    }

    // Split commands in place of any built-in command
    let mut index = 0;
    while index < words.len() {
        // Word starts with a quote
        if words[index].starts_with('\'') || words[index].starts_with('"') {
            // Build one large argument from words in quotes
            let mut joined = String::new();
            loop {
                // Remove quotes from word, if any
                let stripped_word = strip_quotes(&words[index]);
                println!("Got stripped word: {}", stripped_word);
                // Add word to 'joined' with additional space at the end
                joined.push_str(&format!("{} ", stripped_word));
                println!("Joined contents: {joined}");
                // If we find the end of quotation
                if words[index].ends_with('\'') || words[index].ends_with('"') {
                    // Add final word to 'joined'
                    println!("Index {index}: Word {} is ending a quote", words[index]);
                    joined = joined.strip_suffix(' ').unwrap().to_string(); // TIP: Space at the end of the word is no longer needed ;)
                    println!("Joined contents: {joined}");
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
        } else {
            // If built-in keyword appears
            if spliting_keywords.contains(&words[index].as_str()) {
                //println!("Index {index}: Word {} looks like a keyword to split.", words[index]);

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
            // If there are no built-in commands
            else if !spliting_keywords.contains(&words[index].as_str()) {
                //println!("Index {index}: Word {} looks like a normal word.", words[index]);
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
    commands
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
pub fn exec(args:&[String], index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    // Do nothing if nothing was requested
    // This might occur when the user presses ENTER without even typing anything
    if args.is_empty() || args[0].is_empty() {
        print!("");
    }

    // Create a new thread waitinh for SIGINT
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

    // Run a command passed in "args[0]" with arguments in "args[1]" (and so on) and get it's status
    // using process::Command::new().args().status();
    match process::Command::new(&args[0]).args(&args[1..]).status() { 
        Err(e) => {
            eprintln!("{}: Command execution failed: {:?}", args[0], e.kind());
            report_failure(index, returns)
        },
        Ok(process) => {
            // If the command is possible to run, save it's status to "returns" variable
            let command_status = CommandStatus {
                code: process.code(),
                success: process.success(),
                signal: process.signal(),
                core_dumped: process.core_dumped()
            };
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

    // Create a new thread waitinh for SIGINT
    let mut signals = Signals::new([SIGINT]).unwrap();
    thread::spawn(move || {
        for sig in signals.forever() {
            if sig == 2 {
                return;
            }
        }
    });
    
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

pub fn silent_exec(args:&[String], index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    // Create a new thread waitinh for SIGINT
    let mut signals = Signals::new([SIGINT]).unwrap();
    thread::spawn(move || {
        for sig in signals.forever() {
            if sig == 2 {
                return;
            }
        }
    });

    // Run a command passed in "args[0]" with arguments in "args[1]" (and so on) and collect it's status to "returns"
    match process::Command::new(&args[0]).args(&args[1..]).stdout(Stdio::null()).status() { 
        Err(e) => {
            eprintln!("{}: Command execution failed because of an error: {}", args[0], e.kind());
            report_failure(index, returns)
        },
        Ok(process) => {
            // If the command is possible to run, save it's status to "returns" variable
            let command_status = CommandStatus {
                code: process.code(),
                success: process.success(),
                signal: process.signal(),
                core_dumped: process.core_dumped()
            };
            returns.insert(index, command_status);
        },
    }
}
#![allow(dead_code)]

use std::io;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::process;
use std::env;
use std::collections::HashMap;
use carrot_libs::args;
use carrot_libs::input;

mod helpful;
mod gt;
mod exit;
use {helpful::*, gt::*, exit::*};

pub const SPLIT_COMMANDS:[&str;4] = ["then", "next", "end", "else"];
pub const NESTABLE_OPERATORS:[&str;1] = ["test"];
pub const CMP_OPERATORS:[&str;2] = ["test", "else"];
pub const END_LOGIC:[&str;2] = ["end", "else"];


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
        let console_input = input::get(String::from("> "));
        // Separate commands from super commands
        let commands = split_commands(console_input, SPLIT_COMMANDS.to_vec());
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
                "help" | "?" => help(),
                "exit" | "quit" | "bye" => exit(&commands[index], index, &mut returns),
                "next" | "end" => stop=false,
                "else" => command_else(&mut index, &mut returns, &commands, &mut stop),
                "then" => then(&mut index, &mut returns, &commands, &mut stop),
                _ => runcommand(&commands[index], index, &mut returns)
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

fn help() {
    todo!("Help!");
}

// This will be used to execute commands!
fn runcommand(args:&[String], index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    // Do nothing if nothing was requested
    // This might occur when the user presses ENTER without even typing anything
    if args.is_empty() || args[0].is_empty() {
        print!("");
    }
    // Run a command passed in "args[0]" with arguments in "args[1]" (and so on) and get it's status
    // using process::Command::new().args().status();
    match process::Command::new(&args[0]).args(&args[1..]).status() { 
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
    // Flush stdout
    io::stdout().flush().unwrap();
}


pub fn jump_to_end(index:&mut usize, is_already_inside_cmp_operator:u8, status:bool, stop:&mut bool, returns: &mut HashMap<usize, CommandStatus>, commands: &[Vec<String>]) {
    /*
    Find "END" keyword and save it's position
    TIP: There can be multiple END keywords after "THEN". Comparison operations can be nested like in the example below:

    test ad /test then
        say "Operation succeeded!"
        test ( math 1+1 = 2 ) then
            say "It is equal"
        end
    end
     */
    let mut level = is_already_inside_cmp_operator;
    let mut index_of_end = 0;
    for (i,c) in commands[*index+1..].iter().enumerate() {
        if NESTABLE_OPERATORS.contains(&c[0].as_str()) {
            level+=1;
        }
        if END_LOGIC.contains(&c[0].as_str()) || &c[0] == "else" {
            level-=1;
        }
        if level == 0 {
            index_of_end=*index+i;
            break;
        }
        if *index+i == commands.len()-1 && !NESTABLE_OPERATORS.contains(&c[0].as_str()) && level != 0 {
            report_failure(*index, returns);
            // Tell detect_commands() that it can't execute commands anymore
            *stop=true;
            return;
        }
    }

    if status {
        report_success(*index, returns);
    }
    else {
        *index=index_of_end;
    }
    
}

// This function goes back in commands history to find closest comparison operator like "TEST"
// If that found operator reported "success", find "END" and jump straight to that.
// Don't do anything between "ELSE" and "END" or another "ELSE".
// otherwise, (so if previous operator failed) try launching all commands until next "END" or "ELSE"
fn command_else(index_of_else:&mut usize, returns: &mut HashMap<usize, CommandStatus>, commands: &[Vec<String>], stop:&mut bool) {
    if *index_of_else == 0 {
        eprintln!("SYNTAX ERROR! Operator \"ELSE\" doesn't work when there is nothing before it!");
        report_failure(*index_of_else, returns);
        *stop=true;
    }
    if *index_of_else == commands.len()-1 {
        eprintln!("SYNTAX ERROR! Operator \"ELSE\" doesn't work when there is nothing after it!");
        report_failure(*index_of_else, returns);
        *stop=true;
    }

    // Look for the nearest possible previous comparison operator
    let mut index_of_nearest_cmp_operator = *index_of_else-1;
    loop {
        if CMP_OPERATORS.contains(&commands[index_of_nearest_cmp_operator][0].as_str()) {
            break;
        }
        if index_of_nearest_cmp_operator == 0 && !CMP_OPERATORS.contains(&commands[index_of_nearest_cmp_operator][0].as_str()) {
            eprintln!("SYNTAX ERROR! Operator \"ELSE\" is NOT preceded by any comparison operator!");
            report_failure(*index_of_else, returns);
            *stop=true;
            break;
        }
        index_of_nearest_cmp_operator -= 1;
    }

    // Check if previous cmp operator succeeded
    let status_of_cmp_operator = if returns.contains_key(&index_of_nearest_cmp_operator) {
        returns.get(&index_of_nearest_cmp_operator).unwrap().success
    }
    else {
        eprintln!("OPERATOR \"ELSE\" FAILED! Unable to read exit code of the previous comparison operator!");
        *stop=true;
        false
    };
    // Do the test - jump to "END" or "ELSE" if needed
    jump_to_end(index_of_else, 0, !status_of_cmp_operator, stop, returns, commands);


}

fn then(index_of_then:&mut usize, returns: &mut HashMap<usize, CommandStatus>, commands: &[Vec<String>], stop:&mut bool) {
    if *index_of_then == 0 {
        eprintln!("SYNTAX ERROR! Operator \"THEN\" doesn't work when there is nothing before it!");
        report_failure(*index_of_then, returns);
        *stop=true;
    }
    if *index_of_then == commands.len()-1 {
        eprintln!("SYNTAX ERROR! Operator \"THEN\" doesn't work when there is nothing after it!");
        report_failure(*index_of_then, returns);
        *stop=true;
    }
    // Compare exit status of previous and following commands
    let prev_index = *index_of_then-1;
    let prev_status = if returns.contains_key(&prev_index) {
        returns.get(&prev_index).unwrap().success
    }
    else {
        eprintln!("OPERATOR \"THEN\" FAILED! Unable to read exit code of the previous command!");
        *stop=true;
        false
    };
    
    jump_to_end(index_of_then, 1, prev_status, stop, returns, commands);
}
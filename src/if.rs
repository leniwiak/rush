#![allow(unused_imports)]
/*
Rust import system is so stupid that I can't import anything MY OWN modules in this particular project tree.
Trying to use mod or use in any way does not help.

The only solution that I found as of now is to create a directory named "IF" and then creating symbolic links to end.rs and helpful.rs.
Then, I have to add "#![allow(clippy::duplicate_mod)]" to "rush.rs" to tell the compiler, that I want to ignore the fact, 
that some modules are imported multiple times (like... WHAT???)

It's just broken or I am sick and I can't read the docs properly to find the correct solution.
TODO
*/

use std::collections::HashMap;
use std::process;
use carrot_libs::args;

mod helpful;
mod end;
use crate::helpful::*;
// Why this import is considered to be useless?
use crate::end::*;

const SPLIT_COMMANDS:[&str;3] = ["and", "or", "not"];

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

    if opts.is_empty() {
        eprintln!("The \"TEST\" statement requires at least one argument!");
        process::exit(1);
    }

    // Split all arguments by super commands
    let commands = split_commands(opts.clone(), SPLIT_COMMANDS.to_vec());
    // Collect exit statuses here
    let mut returns: HashMap<usize, CommandStatus> = HashMap::new();

    // Run all standard commands
    let mut index = 0;
    while index < commands.len() {
        // Execute all standard commands and collect their statuses
        if !SPLIT_COMMANDS.contains(&commands[index][0].as_str()) {
            silent_exec(&commands[index], index, &mut returns);
        }
        index+=1;
    }

    // When exit codes of standard commands are known - try executing AND, OR operators
    let mut index = 0;
    while index < commands.len() {
        // Execute all standard commands and collect their statuses
        match commands[index][0].as_str() {
            "and" => and(index, &mut returns),
            "or" => or(index, &mut returns),
            "not" => not(index, &mut returns),
            _ => (),
        }
        index+=1;
    }

    // Check if every command between "IF" and "DO" returned success
    let mut ok = true;
    for (i,_r) in returns.iter().enumerate() {
        // If there is at least one unsuccessfull command - quit
        if !returns.get(&i).unwrap().success {
            ok = false;
            break;
        }
    };
    if ok {
        process::exit(0);        
    } else {
        process::exit(1);
    }
}

// This function goes back in commands history to find closest comparison operator like "IF"
// If that previously found operator reported "success", find "END" and jump straight to that.
// Don't do anything inside "ELSE" and "END" or another "ELSE".
// otherwise, (so if previous comparision operator failed) try launching all commands until another "END" or "ELSE"
pub fn command_else(index_of_else:&mut usize, returns: &mut HashMap<usize, CommandStatus>, commands: &[Vec<String>], stop:&mut bool) {
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

    // Look back for the nearest possible comparison operator
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
    end::jump_to_end(index_of_else, 0, !status_of_cmp_operator, stop, returns, commands);


}

// This checks exit code of commands executed before and after.
// Then, it returns true ONLY IF BOTH return codes are positive
pub fn and(index_of_and:usize, returns: &mut HashMap<usize, CommandStatus>) {
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
        returns.get(&prev_index).unwrap().success
    }
    else {
        eprintln!("OPERATOR \"AND\" FAILED! Unable to read exit code of the previous command!");
        process::exit(1);
    };
    let next_status = if returns.contains_key(&next_index) {
        returns.get(&next_index).unwrap().success
    }
    else {
        eprintln!("OPERATOR \"AND\" FAILED! Unable to read exit code of the next command!");
        process::exit(1);
    };

    if prev_status && next_status {
        report_success(index_of_and, returns);
    } else {
        report_failure(index_of_and, returns);
    }
}

// This checks return code before and after it and returns true IF ANY return codes are positive
pub fn or(index_of_or:usize, returns: &mut HashMap<usize, CommandStatus>) {
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
    if returns.get(&prev).unwrap().success || returns.get(&next).unwrap().success {
        report_success(index_of_or, returns);
        // Overwrite the status of both exit codes to fool the if (or any other) logic that every command is ok
        report_success(prev, returns);
        report_success(next, returns);
    } else {
        report_failure(index_of_or, returns);
    }
}

// This changes the return code after it
pub fn not(index_of_not:usize, returns: &mut HashMap<usize, CommandStatus>) {
    if !returns.contains_key(&(index_of_not+1)) {
        eprintln!("SYNTAX ERROR! Operator \"NOT\" doesn't work when there is nothing after it!");
        process::exit(1);
    }
    // Return code of "NOT" doesn't matter
    report_success(index_of_not, returns);

    // Get exit code of the next command
    let next = index_of_not+1;
    if returns.get(&next).unwrap().success {
        // Overwrite the status of the next exit code
        report_failure(next, returns);
    } else {
        report_success(next, returns);
    }
}
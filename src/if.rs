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

// This command uses it's own set of splitting words
const SPLIT_COMMANDS:[&str;7] = ["and", "or", "not", "do", "else", "elseif", "end"];
const JUMP_SPOTS:[&str;3] = ["elseif", "else", "end"];

/*
While working with this code, it might be usefull for you to get used to some of my own terminology.
I don't know if names of parts of IF statements are standarized somewhere or something like that but nevermind...

just know that many of the functions, variables, comments and lot's of other stuff in the code below uses some magical
words because...

The whole code below is an example of an IF statement. Everything starting with an "IF" and ending with an "END" is an IF statement.

-- This is an "IF" keyword. It starts this particular type of logic.
-- "IF", "ELSEIF" and "ELSE" are called "super operators".
|
|   -- Everything between "IF" and "DO" is a "comparison statement"
|   |
|   |        -- This is an "OR" keyword. Both "OR" and "AND" (not used in this example) keywords are called "operators".
|   |        |
|   |        |             -- This is a "DO" keyword. It's also called a "summarizer" because it sums up all the return codes
|   |        |             -- from commands in a "comparison statement". If everythig went fine (returned a success) - execute a "task"
|   |        |             -- if not (there is at least one command that returned a failure) - skip a "task" (I'll explain this in a moment)
|   |        |             -- and jump to "jump spot" (Will explain this too).
|   |        |             |
if cmp 1 = 4 or cmp 1 = 1 do
    say "Equal to one!"   -- This is a "task"
    say "or to four!"     -- This also is a "task". Like everything between "DO" and "ELSEIF" or "ELSE"

  -- As you can see, the logic is simple. If everything inside comparison statement succeeds, we do the "task",
  -- if not, we skip the "task" and jump to the next "ELSEIF", "ELSE" or "END".
  -- This is why those three keywords are often called a "jump spot".
  |
elseif cmp 1 = 2 or cmp 1 = 3 do
    say "Equal to two or three!"  -- Another "task"
else                              -- Another "jump spot"
    say "I don't get it"          -- Task once again
end                               -- And a final jump spot.
*/

fn main() {
    let opts = args::opts();
    let (swcs, vals) = args::swcs();

    // Refuse to run when switches have been passed
    if ! swcs.is_empty() || ! vals.is_empty() {
        eprintln!("This program does not support any switches and values!");
        process::exit(1);
    };

    if opts.is_empty() {
        eprintln!("The \"IF\" statement requires at least one argument!");
        process::exit(1);
    }

    // IF is not defined in options but let's assume that it's index number is zero.
    let super_operator_index = 0;
    let comparison_statement_starting_position = super_operator_index+1;
    
    // If there is any "IF" in options, that means, that user probably requested IF multiple times
    // you can't do that while working with IF's
    if opts.contains(&"if".to_string()) {
        eprintln!("SYNTAX ERROR! Repeated \"IF\" operator inside of an IF statement!");
        process::exit(1);
    }
    // If these keywords are not present - The syntax is surelly incorrect
    if !opts.contains(&"do".to_string()) {
        eprintln!("SYNTAX ERROR! Missing \"DO\" operator inside of an IF statement!");
        process::exit(1);
    }
    if !opts.contains(&"end".to_string()) {
        eprintln!("SYNTAX ERROR! Missing \"END\" operator inside of an IF statement!");
        process::exit(1);
    }
    
    // Protect from writing "do end". The task cannot be empty. "END" have to be preceeded with something different than "DO"
    // TODO

    // Split all arguments by splitting keywords
    let all_commands = helpful::split_commands(opts.clone(), SPLIT_COMMANDS.to_vec());
    // Collect exit statuses here
    let mut returns: HashMap<usize, helpful::CommandStatus> = HashMap::new();

    // Find out where "DO" and closest jump spot is located
    let do_keyword_position = all_commands.iter().position(|x| x[0] == "do").unwrap();
    let jump_spot_position = all_commands.iter().position(|x| JUMP_SPOTS.contains(&x[0].as_str())).unwrap();
    // This is a list containing everything between current IF/ELSEIF/ELSE and DO
    let comparison_statement_commands = &all_commands[comparison_statement_starting_position..do_keyword_position].to_vec();
    // This is a list containing everything between DO and closest jump spot
    // When separating task commands, do not use IF-specific SPLIT_COMMANDS. Use those defined in helpful instead.
    let task_commands = helpful::split_commands(opts[do_keyword_position+1..jump_spot_position].to_vec(), helpful::SPLIT_COMMANDS.to_vec());

    // Protect from writing "if do", "elseif do" and "else do". "DO" have to be preceeded with something different than just a
    // super operator "if"
    prevent_empty_comparison_statement(0, comparison_statement_commands);

    // Run all commands inside comparison statement
    let mut index = 0;
    while index < comparison_statement_commands.len() {
        // Execute all commands and collect their statuses to "returns"
        if !SPLIT_COMMANDS.contains(&comparison_statement_commands[index][0].as_str()) {
            helpful::silent_exec(&comparison_statement_commands[index], index, &mut returns);
        }
        index+=1;
    }

    // When exit codes of all commands inside comparison statement are known - try executing AND, OR operators
    let mut index = 0;
    while comparison_statement_commands[index][0] != "do" {
        match comparison_statement_commands[index][0].as_str() {
            "and" => and(index, &mut returns),
            "or" => or(index, &mut returns),
            "not" => not(index, &mut returns),
            "else" | "elseif" | "end" | "if" => {
                eprintln!("SYNTAX ERROR! Operator \"{}\" was found in comparison statement!", comparison_statement_commands[index][0]);
                process::exit(1);
            },
            _ => (),
        }
        index+=1;
    }

    // Check if every command after "IF" returned success.
    if check_statuses(&returns, comparison_statement_starting_position, do_keyword_position) {
        do_the_task(task_commands);
        // We don't want to run any other ELSEIF or ELSE after current task is done.
        process::exit(0);
    } else {
        process::exit(1);
    }
}

// BEWARE! Function check_statuses() has to check them only for commands inside the comparison operator that is currently running!
// This is why this function scans through "returns" only for values fitting in range from "comparison_statement_starting_position"
// to "do_keyword_position".
fn check_statuses(returns:&HashMap<usize, helpful::CommandStatus>, start:usize, end:usize) -> bool {
    let mut ok = true;
    for (i,_r) in returns.iter().enumerate() {
        if i >= start && i <= end {
            // If there is at least one unsuccessfull command - quit
            if !returns.get(&i).unwrap().success {
                ok = false;
                break;
            }
        }
    };
    ok
}

// Make sure, that there is something interesting after if/elseif/else
fn prevent_empty_comparison_statement(super_operator_index:usize, commands:&[Vec<String>]) {
    if commands[super_operator_index+1][0] == "do" {
        eprintln!("SYNTAX ERROR! Comparison statement is empty!");
        process::exit(1);
    }
}

fn do_the_task(commands: Vec<Vec<String>>) {
    for c in commands {
        helpful::exec(&c, 0, &mut HashMap::new());
    }
}

// This checks exit code of commands executed before and after.
// Then, it returns true ONLY IF BOTH return codes are positive
pub fn and(index_of_and:usize, returns: &mut HashMap<usize, helpful::CommandStatus>) {
    if index_of_and == 0 {
        eprintln!("SYNTAX ERROR! Operator \"AND\" doesn't work when there is nothing before it!");
        helpful::report_failure(index_of_and, returns);
        process::exit(1);
    }
    if !returns.contains_key(&(index_of_and+1)) {
        eprintln!("SYNTAX ERROR! Operator \"AND\" doesn't work when there is nothing after it!");
        helpful::report_failure(index_of_and, returns);
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
        helpful::report_success(index_of_and, returns);
    } else {
        helpful::report_failure(index_of_and, returns);
    }
}

// This checks return code before and after it and returns true IF ANY return codes are positive
pub fn or(index_of_or:usize, returns: &mut HashMap<usize, helpful::CommandStatus>) {
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
        helpful::report_success(index_of_or, returns);
        // Overwrite the status of both exit codes to fool the if (or any other) logic that every command is ok
        helpful::report_success(prev, returns);
        helpful::report_success(next, returns);
    } else {
        helpful::report_failure(index_of_or, returns);
    }
}

// This changes the return code after it
pub fn not(index_of_not:usize, returns: &mut HashMap<usize, helpful::CommandStatus>) {
    if !returns.contains_key(&(index_of_not+1)) {
        eprintln!("SYNTAX ERROR! Operator \"NOT\" doesn't work when there is nothing after it!");
        process::exit(1);
    }
    // Return code of "NOT" doesn't matter
    helpful::report_success(index_of_not, returns);

    // Get exit code of the next command
    let next = index_of_not+1;
    if returns.get(&next).unwrap().success {
        // Overwrite the status of the next exit code
        helpful::report_failure(next, returns);
    } else {
        helpful::report_success(next, returns);
    }
}
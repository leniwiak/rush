use std::process;
use carrot_libs::args;
mod helpful;

/*
Example loop statement:

set INDEX=1
loop
    say "HELLO"
    say "GOOD MORNING"
    say "GOODBYE!"
    break cmp $INDEX = 1
    say "THIS SHOULD NEVER BE EXECUTED!"
end

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
        eprintln!("The \"LOOP\" statement requires at least one argument!");
        process::exit(1);
    }
    
    // If there is any "LOOP" in options, that means, that user probably requested LOOP multiple times
    // you can't do that while working with LOOPs
    if opts.contains(&"loop".to_string()) {
        eprintln!("SYNTAX ERROR! Repeated \"LOOP\" operator inside of a LOOP statement!");
        process::exit(1);
    }
    // If these keywords are not present - The syntax is surelly incorrect
    if ! opts.contains(&"end".to_string()) {
        eprintln!("SYNTAX ERROR! Missing \"END\" operator inside of a LOOP statement!");
        process::exit(1);
    }

    // Protect from writing "loop end".
    if opts[0] == "end" {
        eprintln!("SYNTAX ERROR! There is nothing between keywords \"LOOP\" and \"END\"!");
        process::exit(1);
    }
    
    magic(&opts);
    
}

/*
MAGIC is a place where we invoke super commands

This function takes some usefull arguments:
all_commands - List of commands splitted by LOOP-specific SPLIT_COMMANDS constant. Usefull for comparison statement but not in the task.
*/
fn magic(all_commands:&[String]) {
    loop {
        let task_commands = match helpful::split_commands(all_commands[..all_commands.len()-1].to_owned(), helpful::SPLIT_COMMANDS.to_vec(), false) {
            Err(e) => {eprintln!("LOOP OPERATOR FAILED! {e}!"); process::exit(1)},
            Ok(e) => e,
        };
        //dbg!(&task_commands);
        helpful::detect_commands(&task_commands);
    }
}
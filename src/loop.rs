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

/*
Example loop statement:

loop
    say "HELLO"
    say "GOOD MORNING"
    say "GOODBYE!"
    break
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
        let task_commands = helpful::split_commands(all_commands.to_owned(), helpful::SPLIT_COMMANDS.to_vec(), false);
        //dbg!(&task_commands);
        let mut idx = 0;
        while idx < task_commands.len() {
            //println!("Running: {}", task_commands[idx][0]);
            if task_commands[idx][0] == "break" {
                process::exit(0);
            }
            if task_commands[idx][0] == "end" || helpful::SPLIT_COMMANDS.contains(&task_commands[idx][0].as_str()) {
                idx+=1;
                continue;
            }
            helpful::exec(&task_commands[idx], 0, &mut HashMap::new());
            idx+=1;
        }
    }
    
}
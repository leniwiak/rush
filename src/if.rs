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
use helpful::*;

// This command uses it's own set of splitting words
const SPLIT_COMMANDS:[&str;8] = ["if", "elseif", "else", "and", "or", "not", "do", "end"];

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
else                              -- Another "jump spot". "ELSE" does have a "comparison statement" nor "DO" keyword
    say "I don't get it"          -- Task once again
end                               -- And a final jump spot.
*/

fn main() {
    let args = args::args();
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
    
    // If there is any "IF" in options, that means, that user probably requested IF multiple times
    // you can't do that while working with IFs
    if opts.contains(&"if".to_string()) {
        eprintln!("SYNTAX ERROR! Repeated \"IF\" operator inside of an IF statement!");
        process::exit(1);
    }
    // If these keywords are not present - The syntax is surelly incorrect
    if ! opts.contains(&"do".to_string()) {
        eprintln!("SYNTAX ERROR! Missing \"DO\" operator inside of an IF statement!");
        process::exit(1);
    }
    if ! opts.contains(&"end".to_string()) {
        //dbg!(&opts);
        eprintln!("SYNTAX ERROR! Missing \"END\" operator inside of an IF statement!");
        process::exit(1);
    }
    
    // Protect from writing "if do". The comparison statement cannot be empty.
    if opts[0] == "do" {
        eprintln!("SYNTAX ERROR! There is nothing between keywords \"IF\" and \"DO\"!");
        process::exit(1);
    }

    // Split all arguments by splitting keywords
    let all_commands = match helpful::split_commands(args.clone(), SPLIT_COMMANDS.to_vec(), false) {
        Err(e) => { eprintln!("IF OPERATOR FAILED! {e}!"); process::exit(1) },
        Ok(e) => e,
    };
    // Collect exit statuses here
    let mut returns: HashMap<usize, bool> = HashMap::new();
    
    let mut idx = 0;
    // Protect from writing "do end". The task cannot be empty.
    let do_position = all_commands.iter().position(|x| x[0] == "do" ).unwrap();
    if all_commands[do_position+1][0] == "end" {
        eprintln!("SYNTAX ERROR! There is nothing between keywords \"DO\" and \"END\"!");
        process::exit(1);
    }

    let end_position = all_commands.iter().position(|x| x[0] == "end" ).unwrap();
    // dbg!(&all_commands);
    while idx != end_position {
        // println!("LET'S GO: {idx}!!!");
        match all_commands[idx][0].as_str() {
            "if" | "elseif" => make_comparison(&mut idx, &all_commands, &mut returns, false),
            "else" => make_comparison(&mut idx, &all_commands, &mut returns, true),
            a => {
                panic!("Internal error! Logic jumped to unknown super operator: {a}!");
            }
        }
    }
}

use std::collections::HashMap;
use std::process;
use carrot_libs::args;
mod helpful;
use helpful::*;

// This command uses it's own set of splitting words
const SPLIT_COMMANDS:[&str;8] = ["for", "in", "persign", "perword", "perline", "step", "do", "end"];

/*
Same situation as in the IF.RS code. Get used to some of my own definitions and have fun.

-- This is an "FOR" keyword. It starts this particular type of logic.
|
|   -- Everything between "FOR" and "DO" is a "iteration statement"
|   |
|   | -- This is an "IN" keyword. "IN", ""PERSIGN", "PERWORD", "PERLINE" (not used here) are called "separation methods".
|   | |
|   | |        -- This is a "STEP" keyword. It tells the loop to skip some values in "$LIST".
|   | |        -- Negative numbers will cause FOR to iterate from the end.
|   | |        |
|   | |        |      -- This is a "DO" keyword. It will be used to end "iteration statement" and start the definition of a new task.
|   | |        |      -- Every sign, word or line in a variable (depending on which separation method is used) will be used to set contents of
|   | |        |      -- "a". "a" can be used in a loop to do some actions per every sign/word/line in requested variable or command's output.
|   | |        |      |
for a in $LIST step 2 do
    say "a"                -- This is a "task"
    say "Nice, buddy!"     -- This also is a "task". Like everything between "DO" and "END"
end                        -- Jump spot.
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
        eprintln!("The \"FOR\" statement requires some arguments!");
        process::exit(1);
    }
    
    // If there is any "FOR" in options, that means, that user probably requested "FOR" multiple times
    // you can't do that while working with FORs
    if opts.contains(&"for".to_string()) {
        eprintln!("SYNTAX ERROR! Repeated \"FOR\" operator inside of an FOR statement!");
        process::exit(1);
    }
    // If these keywords are not present - The syntax is surelly incorrect
    if !opts.contains(&"do".to_string()) {
        eprintln!("SYNTAX ERROR! Missing \"DO\" operator inside of an FOR statement!");
        process::exit(1);
    }
    if !opts.contains(&"end".to_string()) {
        //dbg!(&opts);
        eprintln!("SYNTAX ERROR! Missing \"END\" operator inside of an FOR statement!");
        process::exit(1);
    }
    if !opts.contains(&"in".to_string()) 
    || !opts.contains(&"persign".to_string())
    || !opts.contains(&"perword".to_string())
    || !opts.contains(&"perline".to_string())
    {
        //dbg!(&opts);
        eprintln!("SYNTAX ERROR! Missing separation method inside of an FOR statement!");
        process::exit(1);
    }
    
    // TODO: Protect from writing "for in/per...", "for do", "step" without a value, etc.
    if opts[0] == "do" {
        eprintln!("SYNTAX ERROR! There is nothing between keywords \"FOR\" and \"DO\"!");
        process::exit(1);
    }

    // Split all arguments by splitting keywords
    let all_commands = match helpful::split_commands(args.clone(), SPLIT_COMMANDS.to_vec(), false) {
        Err(e) => { eprintln!("FOR OPERATOR FAILED! {e}!"); process::exit(1) },
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

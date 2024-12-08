use std::process;
use carrot_libs::args;
mod helpful;

// This command uses it's own set of splitting words
const SPLIT_COMMANDS:[&str;6] = ["for", "persign", "perword", "perline", "do", "end"];

/*
Same situation as in the IF.RS code. Get used to some of my own definitions and have fun.

-- This is an "FOR" keyword. It starts this particular type of logic.
|
|    -- Any optional switches and their values to change the way how "for" statement works
|    |
|    |    -- This is a current iteration referer
|    |    |
|    |    | -- This is a separator method. It defines how things should be separated in it.Possible separator methods are: "PERSIGN", "PERWORD", "PERLINE".
|    |    | |              -- This is a "DO" keyword. It will be used to end "iteration statement" and start the definition of a new task.
|    |    | |              -- Every sign, word or line in a variable (depending on which separation method is used) will be used to set contents of
|    |    | |              -- "a". "a" can be used in a loop to do some actions per every sign/word/line in requested variable or command's output.
|    |    | |              |
for -s=2 a perword $LIST do
    say "$a"               -- This is a "task"
    say "Nice, buddy!"     -- This also is a "task". Like everything between "DO" and "END"
end                        -- Jump spot.
*/

fn main() {
    let args = args::args();
    let (swcs, vals) = args::swcs();

    let mut position_of_separator = 0;
    let mut position_of_do = 0;
    let mut position_of_end = 0;

    // Refuse to run when switches have been passed
    if ! swcs.is_empty() || ! vals.is_empty() {
        eprintln!("This program does not support any switches and values!");
        process::exit(1);
    };
    
    // If these keywords are not present - The syntax is surelly incorrect
    if !args.contains(&"do".to_string()) {
        eprintln!("SYNTAX ERROR! Missing \"DO\" operator inside a FOR statement!");
        process::exit(1);
    }
    if !args.contains(&"end".to_string()) {
        eprintln!("SYNTAX ERROR! Missing \"END\" operator inside a FOR statement!");
        process::exit(1);
    }
    if !args.contains(&"persign".to_string())
    || !args.contains(&"perword".to_string())
    || !args.contains(&"perline".to_string())
    {
        eprintln!("SYNTAX ERROR! Missing separation method inside a \"FOR\" statement!");
        process::exit(1);
    }

    // Find out where „for” statement's keywords are located in arguments list
    for (idx, word) in args.clone().into_iter().enumerate() {
        match word.as_str() {    
            "persign" | "perword" | "perline" | "in" => position_of_separator = idx,
            "do" => position_of_do = idx,
            "end" => position_of_end = idx,
            _ => ()
        };
    }

    // „For” will support switches soon.
    // They need to be defined first. Even before an iteration referer
    let mut position_of_last_switch = 0;
    for (idx, word) in args.clone().into_iter().enumerate() {
        if word.starts_with('-') {
            position_of_last_switch = idx;
        }
    }

    // Protect from writing switches anywhere after iteration referer
    // Example: "for a perline -s=2 $LIST do ..."
    // The only way is to write it like thid: "for -s=2 a perline $LIST do ..."
    if position_of_last_switch > position_of_separator-1 {
        eprintln!("SYNTAX ERROR! Switches for the \"FOR\" block have to be defined first!");
        process::exit(1);
    }
    
    // Protect from running for loop without iteration reference name.
    // Example: "for perline do..."
    if position_of_separator != position_of_last_switch+2 {
        eprintln!("SYNTAX ERROR! Separation method must be defined after current iteration reference!");
        process::exit(1);
    };

    // Protect from running for loop with „do” in the wrong place.
    // Example: "for a do perword $LIST..."
    // In other words, make sure that "do" is after separation method
    if position_of_do < position_of_separator {
        eprintln!("SYNTAX ERROR! \"DO\" is misplaced in a \"FOR\" block!");
        process::exit(1);
    }
    // Prevent from running for loop without tasks in it
    // Example: "for a perword $LIST do end"
    if position_of_end == position_of_do+1 {
        eprintln!("SYNTAX ERROR! \"FOR\" block is missing tasks to run!");
        process::exit(1);
    }

    // Make sure that "end" keyword is the last
    if position_of_end < position_of_do+1 {
        eprintln!("SYNTAX ERROR! \"END\" keyword must be the last in \"FOR\" block!");
        process::exit(1);
    }

    // Possible list that the end-user wants us to loop through
    // This may be a reference to command's OUT/ERR output
    // Example: "for a perline OUT:p file.txt do ..."
    let between_sep_and_do = &args[position_of_separator+1..position_of_do];

    if between_sep_and_do.is_empty() {
        unreachable!("Program's logic contradicts itself! Please, report a bug!");
    }

    // Tasks inside of a for block
    let tasks = match helpful::split_commands(args[position_of_do+1..position_of_end].to_owned(), helpful::SPLIT_COMMANDS.to_vec(), false) {
        Err(e) => { eprintln!("FOR OPERATOR FAILED! {e}!"); process::exit(1) },
        Ok(e) => e,
    };

    
    
}

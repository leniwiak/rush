use std::process;
use carrot_libs::args;
// This is how to import other rush files when the current source file is defined as binary in Cargo.toml
mod helpful;
use crate::helpful::*;

#[allow(clippy::needless_late_init)]
/*
Possible comprarison operations

FOR NUMERIC VARIABLES:
    >           Larger than
    <           Smaller than
    => / >=     Larger or equal
    =< / <=     Smaller or equal
    = / ==      Equal
    != / =!     Different
FOR TEXT VALUES:
    = / ==      Equal
    ~           Contained
    != / =!     Different

When you want to use the output of a command while comparing - use [command]
When you want to use the output of a variable while comparing - use $variable
*/
fn main() {
    let opts = args::opts();
    let (swcs, vars) = args::swcs();

    if !swcs.is_empty() || !vars.is_empty() {
        eprintln!("Operator \"cmp\" doesn't support any switches nor it's values!");
        process::exit(1);
    }

    if opts.len() < 3 {
        eprintln!("Operator \"cmp\" doesn't work when there's nothing to compare!");
        process::exit(1);
    }

    // Contents of things to compare on the left and right side
    let mut left = String::new();
    let mut action = String::new();
    let mut right = String::new();
    // List of known operators
    let operators = [">", "<", "=>", ">=", "=<", "<=", "=", "==", "!=", "=!", "~"];

    let mut i = 0;
    while i < opts.len() {
        let curop = &opts[i].as_str();
        if operators.contains(curop) && i == 0 {
            eprintln!("SYNTAX ERROR! Missing left value before operator!");
            process::exit(1);
        }
        if operators.contains(curop) && i > 0 {
            for opt in opts[..i].iter() {
                left.push_str(&format!("{opt} "));
            }
            action.clone_from(&opts[i]);
            for opt in opts[i+1..].iter() {
                right.push_str(&format!("{opt} "));
            }
        }   
        i+=1;
    }
    if right.is_empty() {
        eprintln!("SYNTAX ERROR! Missing right value after operator!");
        process::exit(1);
    }

    // Replace contents of left/right operator with the output of a command
    // if requested text starts with a colon or "ERR:" or "CODE:"
    left = replace_contents(left.strip_suffix(' ').unwrap().to_string());
    right = replace_contents(right.strip_suffix(' ').unwrap().to_string());

    // Are we comparing numbers?
    let left_is_numeric = left.parse::<usize>().is_ok();
    let right_is_numeric = right.parse::<usize>().is_ok();
    // dbg!(left_is_numeric, right_is_numeric);

    // The variable "comparison_status" will be used by process::exit to return 0 or 1
    let comparison_status:bool;
    // Compare data
    match (left_is_numeric, right_is_numeric) {
        (true, true) => {
            let left_number = left.parse::<usize>().unwrap();
            let right_number = right.parse::<usize>().unwrap();
            comparison_status = match action.as_str() {
                ">" => left_number > right_number,
                "<" => left_number < right_number,
                "=>" | ">=" => left_number >= right_number,
                "=<" | "<=" => left_number <= right_number,
                "=" | "==" => left_number == right_number,
                "!=" | "=!" => left_number != right_number,
                _ => {
                    eprintln!("SYNTAX ERROR! Unknown comparison operator!");
                    process::exit(1);
                }
            };
        }
        (false, false) => {
            comparison_status = match action.as_str() {
                "=" | "==" => left.trim() == right.trim(),
                "~" => left.trim().contains(right.trim()),
                "!=" | "=!" => left.trim() != right.trim(),
                _ => {
                    eprintln!("SYNTAX ERROR! Unknown comparison operator!");
                    process::exit(1);
                }
            };
        }
        _ => {
            eprintln!("OPERATOR \"CMP\" FAILED! Values differ in type!");
            process::exit(1);
        }
    };

    // Exit from the program either successfully or not
    if comparison_status {
        println!("OK");
        process::exit(0);
    } else {
        process::exit(1);
    }
}

fn replace_contents(text:String) -> String {
    if text.starts_with(':') || text.starts_with("ERR:") || text.starts_with("CODE:") {
        let command = if text.starts_with(':') {
            text.strip_prefix(':')
        } else if text.starts_with("ERR:") {
            text.strip_prefix("ERR:")
        } else if text.starts_with("CODE:") {
            text.strip_prefix("CODE:")
        } else {
            unreachable!("Program's logic contradicts itself. Please, report a bug!");
        };
        
        // Make a list of words because "cmd_content" does not allow strings
        let mut ewygerfeue = Vec::new();
        println!("{}", command.unwrap());
        for w in command.unwrap().split_whitespace() {
            ewygerfeue.push(w.to_string());
            println!("{:?}", ewygerfeue);
        }
        
        let output = getoutput_exec(&ewygerfeue);
        
        if text.starts_with(':') {
            String::from_utf8(output.stdout).unwrap()
        } else if text.starts_with("ERR:") {
            String::from_utf8(output.stderr).unwrap()
        } else if text.starts_with("CODE:") {
            output.status.code().unwrap().to_string()
        } else {
            unreachable!("Program's logic contradicts itself. Please, report a bug!");
        }
    } else {
        text
    }
        
}
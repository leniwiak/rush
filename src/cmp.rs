use std::process;
use carrot_libs::args;
use num_bigint::BigInt;
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

When you want to use command's STDOUT of while comparing - use :command
When you want to use command's STDERR of while comparing - use ERR:command
When you want to use command's exit code while comparing - use CODE:command
When you want to use the output of a variable while comparing - use $variable
*/
fn main() {
    let args = args::args();

    if args.len() < 4 {
        eprintln!("Operator \"cmp\" doesn't work when there's nothing to compare!");
        process::exit(1);
    }

    // Contents of things to compare on the left and right side
    let mut left = String::new();
    let mut action = String::new();
    let mut right = String::new();
    // List of known operators
    let operators = [">", "<", "=>", ">=", "=<", "<=", "=", "==", "!=", "=!", "~"];

    let mut i = 1;
    while i < args.len() {
        let curop = &args[i].as_str();
        if operators.contains(curop) && i == 0 {
            eprintln!("SYNTAX ERROR! Missing left value before operator!");
            process::exit(1);
        }
        if operators.contains(curop) && i > 0 {
            for arg in args[1..i].iter() {
                left.push_str(&format!("{} ", arg));
            }
            action.clone_from(&args[i]);
            for arg in args[i+1..].iter() {
                right.push_str(&format!("{} ", arg));
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
    left = replace_contents(left.to_string());
    right = replace_contents(right.to_string());

    // Are we comparing numbers?
    let left_is_numeric = left.trim().parse::<BigInt>().is_ok();
    let right_is_numeric = right.trim().parse::<BigInt>().is_ok();
    //dbg!(&left, left_is_numeric, &right, right_is_numeric);

    // The variable "comparison_status" will be used by process::exit to return 0 or 1
    let comparison_status:bool;
    // Compare data
    match (left_is_numeric, right_is_numeric) {
        (true, true) => {
            let left_number = left.trim().parse::<BigInt>().unwrap();
            let right_number = right.trim().parse::<BigInt>().unwrap();
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
            //dbg!(left_is_numeric, right_is_numeric);
            process::exit(1);
        }
    };

    // Exit from the program either successfully or not
    if comparison_status {
        println!("TRUE");
        process::exit(0);
    } else {
        println!("FALSE");
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
        // println!("{}", command.unwrap());
        for w in command.unwrap().split_whitespace() {
            ewygerfeue.push(w.to_string());
            // println!("{:?}", ewygerfeue);
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

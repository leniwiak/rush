use std::process;
use carrot_libs::args;
#[allow(clippy::needless_late_init)]
/*
Possible comprarison operations

FOR NUMERIC VARIABLES:
    >           Larger than
    <           Smaller than
    => / >=     Larger or equal
    =< / <=     Smaller or equal
    = / ==      Equal
    ! / != / =! Different
FOR TEXT VALUES:
    = / ==      Equal
    ~ / ~= / =~ Contained
    ! / != / =! Different

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

    if opts.len() > 3 {
        eprintln!("Operator \"cmp\" doesn't support more than three options!");
        process::exit(1);
    }

    let left = opts[0].clone();
    let action = opts[1].clone();
    let right = opts[2].clone();
    // Are we comparing numbers?
    let left_is_numeric = left.chars().all(char::is_alphanumeric);
    let right_is_numeric = right.chars().all(char::is_alphanumeric);

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
                "!" | "!=" | "=!" => left_number != right_number,
                _ => {
                    eprintln!("SYNTAX ERROR! Unknown comparison operator!");
                    process::exit(1);
                }
            };
        }
        (false, false) => {
            comparison_status = match action.as_str() {
                "=" | "==" => left.trim() == right.trim(),
                "~" | "~=" | "=~" => left.trim().contains(right.trim()),
                "!" | "!=" | "=!" => left.trim() != right.trim(),
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
        process::exit(0);
    } else {
        process::exit(1);
    }
}
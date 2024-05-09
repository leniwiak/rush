use std::collections::HashMap;
// This is how to import other files from rush when the current source file is imported in "rush.rs"
use crate::helpful::*;
use crate::end::*;

// This function goes back in commands history to find closest comparison operator like "TEST"
// If that found operator reported "success", find "END" and jump straight to that.
// Don't do anything between "ELSE" and "END" or another "ELSE".
// otherwise, (so if previous operator failed) try launching all commands until next "END" or "ELSE"
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

    // Look for the nearest possible previous comparison operator
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
    jump_to_end(index_of_else, 0, !status_of_cmp_operator, stop, returns, commands);


}
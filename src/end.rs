use std::collections::HashMap;
use crate::helpful::*;

pub fn jump_to_end(index:&mut usize, is_already_inside_cmp_operator:u8, status:bool, stop:&mut bool, returns: &mut HashMap<usize, CommandStatus>, commands: &[Vec<String>]) {
    /*
    Find "END" keyword and save it's position
    TIP: There can be multiple END keywords after "THEN". Comparison operations can be nested like in the example below:

    if ad /test then
        say "Operation succeeded!"
        if ( math 1+1 == 2 ) then
            say "It is equal"
        end
    end
     */
    let mut level = is_already_inside_cmp_operator;
    let mut found_end_operators = 0;
    let mut index_of_end = 0;
    for (i,c) in commands[*index+1..].iter().enumerate() {
        // If you find logic operator, bump up the 'level' variable
        // This means that there is probably another 'if' (or anything like that) which should
        // have it's own 'end'.
        if NESTABLE_OPERATORS.contains(&c[0].as_str()) {
            level+=1;
        }
        // Lower the 'level' when 'end' is found
        if END_LOGIC.contains(&c[0].as_str()) || &c[0] == "else" {
            level-=1;
            found_end_operators+=1;
        }
        // If the last END operator was found, save it's position and quit
        if level == 0 {
            index_of_end=*index+i;
            break;
        }
        // If we are at the last possible commands and it is not an END keyword, print an error
        if found_end_operators == 0 && i == commands.len() {
            eprintln!("SYNTAX ERROR! The \"END\" operator cannot be found!");
            report_failure(*index, returns);
            // Tell detect_commands() that it can't execute commands anymore
            *stop=true;
            return;
        }
    }

    if status {
        report_success(*index, returns);
    }
    else {
        *index=index_of_end;
    }
    
}

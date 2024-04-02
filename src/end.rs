use std::collections::HashMap;
use crate::helpful::*;

pub fn jump_to_end(index:&mut usize, is_already_inside_cmp_operator:u8, status:bool, stop:&mut bool, returns: &mut HashMap<usize, CommandStatus>, commands: &[Vec<String>]) {
    /*
    Find "END" keyword and save it's position
    TIP: There can be multiple END keywords after "THEN". Comparison operations can be nested like in the example below:

    test ad /test then
        say "Operation succeeded!"
        test ( math 1+1 = 2 ) then
            say "It is equal"
        end
    end
     */
    let mut level = is_already_inside_cmp_operator;
    let mut index_of_end = 0;
    for (i,c) in commands[*index+1..].iter().enumerate() {
        if NESTABLE_OPERATORS.contains(&c[0].as_str()) {
            level+=1;
        }
        if END_LOGIC.contains(&c[0].as_str()) || &c[0] == "else" {
            level-=1;
        }
        if level == 0 {
            index_of_end=*index+i;
            break;
        }
        if *index+i == commands.len()-1 && !NESTABLE_OPERATORS.contains(&c[0].as_str()) && level != 0 {
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
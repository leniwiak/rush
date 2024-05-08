#![allow(dead_code)]

use std::collections::HashMap;
use std::env;

#[derive(Debug)]
 pub struct CommandStatus {
    pub code: Option<i32>,
    pub success: bool,
    pub signal: Option<i32>,
    pub core_dumped: bool 
}

pub const SPLIT_COMMANDS:[&str;4] = ["then", "next", "end", "else"];
pub const NESTABLE_OPERATORS:[&str;1] = ["test"];
pub const CMP_OPERATORS:[&str;2] = ["test", "else"];
pub const END_LOGIC:[&str;2] = ["end", "else"];

// These functions will be used to report success or failure when built-in or super commands are running
// This is usefull because typically we don't want the shell to abnormally quit when syntax of "test" statement is incorrect
// Instead, we just want to say "Hey! There is a bug!"
// BUT when rush would work as a subshell just to execute a script, we won't even need it anymore
pub fn report_success(index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    let command_status = CommandStatus {code: Some(0),success: true,signal: None,core_dumped: false};
    returns.insert(index, command_status);
}
pub fn report_failure(index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    let command_status = CommandStatus {code: Some(1),success: false,signal: None,core_dumped: false};
    returns.insert(index, command_status);
}


pub fn split_commands(mut words:Vec<String>, spliting_keywords:Vec<&str>) -> Vec<Vec<String>> {
    // This list contains all commands passed by the user 
    let mut commands: Vec<Vec<String>> = Vec::new();
    // List of words in one command
    let mut command = Vec::new();
    /*
    This will be used to separate built-in commands from anything else
    Expected output: ('af' 'file'), ('then'), ('ad' 'dir')
    */ 

    // First of all, when there's a word that starts with "$"
    // replace it with variable contents
    let mut i = 0;
    while i < words.len() {
        // Save currently tested word to "w"
        let w = &words[i];
        // Remove unnecessary prefix ($varname -> varname)
        let sp = w.strip_prefix('$');
        // If contents of "w" are not empty, replace word with variable contents
        if let Some(w) = sp {
            // Get variable contents
            let variable_contents=env::var_os(w).unwrap_or_default();
            // Remove current command
            words.remove(i);
            // Append contents of a variable
            words.insert(i, variable_contents.into_string().unwrap_or_default());
        }
        i += 1;
    };
   
    // Find words enclosed in quotes
    let mut index = 0;
    // This is where ranges of words will be stored for those that are inside quotes
    // EDIT: Look for comments inside nearest while loop
    let mut quote_positions = Vec::new();
    let mut quote_starting_points = Vec::new();
    let mut quote_ending_points = Vec::new();
    let mut start_quote = None;
    let mut end_quote = None;

    while index < words.len() {
        // Save position of starting/ending quotes
        if words[index].starts_with('\'') || words[index].starts_with('"') { start_quote=Some(index); }
        if words[index].ends_with('\'') || words[index].ends_with('"') { end_quote=Some(index+1); }

        // If both, starting and ending positions are defined, add them to quote_positions
        if start_quote.is_some() && end_quote.is_some() {
            // I've lost my patience
            // Instead of list containing ranges of enquoted words, just make a list with
            // EVERY possible number containing identifiers of enquoted words
            // Old structure: [[1..3], [4..9], [35,45]]
            // New structure: [1,2,3,4,5,6,7,8,9,35,36,37,38,39,40, ..., you get the point]
            // Another funny thing: My program needs to know where another quote is starting/ending
            // the easiest way to do it now is to create another list with all quotation starting points
            // Example: Starting points [1, 4, 35], ending points [3, 9, 45]
            quote_starting_points.push(start_quote.unwrap());
            quote_ending_points.push(end_quote.unwrap());
            for i in start_quote.unwrap()..end_quote.unwrap() {
                quote_positions.push(i);
            }
            start_quote = None;
            end_quote = None;
        }
        index+=1;
        // Searching for enquoted words is finished. If there is any quote left, throw syntax
        // error.
        if index == words.len() && (start_quote.is_some() && end_quote.is_none()) {
            eprintln!("SYNTAX ERROR! There are unclosed quotes in your input!");
        }
    }

    let mut quoted = false;
    let mut natychmiast_wypierdalaj = false;
    // Split commands in place of any built-in command
    let mut index = 0;
    while index < words.len() {
        if natychmiast_wypierdalaj {
            break;
        }
        // If currently tested word is a part of enquoted sentence,
        // add the entire quote contents to 'commands' without any spacing between words
        // then jump to words after current quote.
        if quote_positions.contains(&index) {
            println!("Tutaj {index}");
            quoted=true;
            if quote_starting_points.contains(&index) {
                let mut inner_index = index;
                // Get all the enquoted words until end of nearest quote is found
                let mut joined = String::new();
                loop {
                    joined.push_str(&words[inner_index]);
                    inner_index+=1;
                    println!("Hallo {inner_index}");
                    if !quote_ending_points.contains(&inner_index) {
                        index=inner_index+1;
                        break;
                    }
                }
                // Add entire, enquoted text to 'commands' list
                commands.push(Vec::from([joined]));
            }
            else {
                quoted=false;
            }
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
            // KURWA!
        }
        // If built-in keyword appears
        else if spliting_keywords.contains(&words[index].as_str()) && !quoted {
            // Separate CURRENT keyword from PREVIOUSLY collected words
            // Expected output: ('af' 'file'), ('then' 'ad' 'dir')
            let (before_keyword, right) = words.split_at(index);
            // Convert everything to a vector
            let (before_keyword, right) = (before_keyword.to_vec(), right.to_vec());

            // Separate keyword from NEXT words, that are not collected yet
            // Expected output: ('af' 'file'), ('then'), ('ad' 'dir')
            let (keyword, after_keyword) = {
                let (keyword, after_keyword) = right.split_at(1);
                (keyword.to_vec(), after_keyword.to_vec())
            };

            // Send previous words to "commands"
            // Example: ('af' 'file')
            if !before_keyword.is_empty() {
                // Do not append anything if there is nothing before keyword!
                commands.push(before_keyword.to_vec());
            }
            // Send keyword to "commands" exclusively
            // Example: ('then')
            commands.push(keyword.to_vec());
            // We no longer need to deal with ('af' 'file') and ('then') so remove them from words
            words = after_keyword.to_vec();
            // Start over with new words
            // Example: ('ad' 'dir')
            index = 0;
        }
        // If there are no built-in commands
        else if !spliting_keywords.contains(&words[index].as_str()) && !quoted {
            // Just add the words to the 'command' variable
            command.push(words[index].clone());
            index += 1;
            // No more words? Add them to 'commands'.
            if index == words.len() {
                commands.push(words.clone());
            }
        };
    };

    println!("Tu powinno nakurwiać");
    dbg!(&commands);

    commands
}

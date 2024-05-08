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
    /*
    This will be used to separate SUPER COMMANDS from anything else
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

    let mut command = Vec::new();
   
    // Find words enclosed in quotes
    let mut index = 0;
    // This is where index numbers ranges will be stored for words inside quotes
    // Example text: some text "is quoted" for sure and we "all know it", don't we?
    // Example list contents: (2,3) and (8,10)
    let mut quote_positions = Vec::new();
    let mut start_quote = None;
    let mut end_quote = None;

    while index < words.len() {
        // Save position of starting/ending quotes
        if words[index].starts_with('\'') || words[index].starts_with('"') { start_quote=Some(index); }
        if words[index].ends_with('\'') || words[index].ends_with('"') { end_quote=Some(index+1); }

        // If both, starting and ending positions are defined, add them to quote_positions
        if start_quote.is_some() && end_quote.is_some() {
            quote_positions.push( start_quote.unwrap()..end_quote.unwrap() );
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

    // Split commands in place of any built-in command
    let mut index = 0;
    while index < words.len() {
        // If currently tested word is enquoted, add the entire quote contents to 'commands'
        // and jump to word after current quote.
        let mut quoted = false;
        let mut currently_quoted_in = 0..0;
        for range in &quote_positions {
            if range.contains(&index) {
                quoted=true;
                currently_quoted_in=range.clone()
            };
        }; 

        // If built-in keyword appears
        if spliting_keywords.contains(&words[index].as_str()) && !quoted {
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
            // Just add the words to the commands' bank
            command.push(words[index].clone());
            index += 1;
            if index == words.len() {
                commands.push(words.clone());
            };
        }
        // If we are in single/double quote mode
        else {
            // Get words in quotes in range
            let joined = words[currently_quoted_in.clone()].join(" ");
            // Add entire, enquoted text to 'commands'
            commands.push(Vec::from([joined.clone()]));
            // Set index to the number of the word after quote
            index+=currently_quoted_in.end;
        }
    };

    println!("Tu powinno nakurwiać");
    dbg!(&commands);

    commands
}

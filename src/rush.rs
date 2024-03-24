use std::io;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::process;
use std::env;
use std::collections::HashMap;
use carrot_libs::args;
use carrot_libs::input;

/*
There are three types of commands in RUSH
- Standard commands: When you try to run something like 'git' or 'htop', it will be executed by system with all it's arguments.
- Built-in commands: They also get their arguments as usual BUT they will be executed by shell
- SUPER COMMANDS: They are used to operate output, exit code, or anything else from previous or next commands
 */ 

/*
This struct will be used as a template for "return" variable.
"return" helps this shell to find commands that reported success on exit or not
You'll find more about it later
 */
#[derive(Debug)]
 pub struct CommandStatus {
    pub code: Option<i32>,
    pub success: bool,
    pub signal: Option<i32>,
    pub core_dumped: bool 
}

fn main() {
    let opts = args::opts();
    let (swcs, vals) = args::swcs();

    // Refuse to run when switches have been passed
    if ! swcs.is_empty() {
        eprintln!("This program does not support any switches and values!");
        process::exit(1);
    };
    if ! vals.is_empty() {
        eprintln!("This program does not support any switches and values!");
        process::exit(1);
    };

    /*
    This shell will work in two modes:
    File mode - read lines from a file provided by the user via arguments
    Input mode - read lines from stdin
     */
    let mode = if !opts.is_empty() {
        "file"
    }
    else {
        "input"
    };

    if mode == "file" {
        todo!("File mode is not ready yet!");
    }
    else if mode == "input" {
        init_input_mode();
    }
}

fn init_input_mode() {
    // Always set $? (return code of previous command) to zero on start-up
    env::set_var("?", "0");
    loop {
        // Get all space separated words from user
        let console_input = input::get(String::from("> "));
        // Separate commands from super commands
        let commands = split_commands(console_input, Vec::from(["then", "next", "end"]));
        // Execute those commands
        detect_commands(commands);
    };
}

/* 
This function was created to run super commands or any other commands.
If non built-in command is inside some logical statements, don't do anything unless forced by logical statement
*/
pub fn detect_commands(commands:Vec<Vec<String>>) {
    let mut index = 0;
    let mut stop = false;
    
    /*
    This variable contains all required information about exit codes and statuses reported by ALL invoked commands
    this will be used by logic operators to find out if we can continue some operations or not
    Commands separation is done by split_commands()
    */
    let mut returns: HashMap<usize, CommandStatus> = HashMap::new();

    while index < commands.len() {
        // Check whether the first argument is a keyword or not
        if !stop {
            match commands[index][0].as_str() {
                "gt" => gt(&commands[index], index, &mut returns),
                "help" | "?" => help(),
                "exit" | "quit" | "bye" => exit(&commands[index], index, &mut returns),
                "next" | "end" => print!(""),
                "then" => then(&mut index, &mut returns, &commands, &mut stop),
                _ => runcommand(&commands[index], index, &mut returns)
            };
            if index < commands.len() {
                index+=1;
            };
        }
        else {
            return;
        };
    }
}
    
pub fn split_commands(mut words:Vec<String>, spliting_keywords:Vec<&str>) -> Vec<Vec<String>> {
    // This list contains all commands passed by the user 
    let mut commands: Vec<Vec<String>> = Vec::new();
    /*
    This will be used to separate SUPER COMMANDS from anything else
    Expected output: ('af' 'file'), ('then'), ('ad' 'dir')
    */ 
    let mut command = Vec::new();
    let mut index = 0;
    while index < words.len() {
        // If built-in keyword appears
        if spliting_keywords.contains(&words[index].as_str()) {
            // Separate keyword from PREVIOUSLY collected words
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
        // If there is not built-in command 
        else {
            command.push(words[index].clone());
            index += 1;
            if index == words.len() {
                commands.push(words.clone());
            };
        };
    };
    commands
}


// Change working directory
fn gt(args:&[String], index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    // Check if there is just ONE argument
    // We can't go to more than one directory at the same time
    if args.len() == 1 {
        eprintln!("Give me a directory path to go!");
        // As usual, run this function to report a failure.
        // "index" variable contains position of a command
        // "returns" contains information about all return codes that were reported by commands
        // Both variables are required because "returns" will be modified by "report_failure" according to the contents of "index"
        report_failure(index, returns);
    }
    else if args.len() > 2 {
        eprintln!("Cannot go to multiple directories simultaneously!");
        report_failure(index, returns)
    }
    else {
        match env::set_current_dir(&args[1]) { 
            Err(e) => {
                eprintln!("{}: Cannot go into this directory because of an error: {}", args[1], e.kind());
                report_failure(index, returns);
            },
            Ok(_) => {
                report_success(index, returns);
            }
        };
    };
}

fn help() {
    todo!("Help!");
}

// Just go away
fn exit(args:&[String], index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    if args.len() == 1 {
        report_failure(index, returns);
        process::exit(0)
    }
    else if args.len() > 2 {
        report_failure(index, returns);
        eprintln!("Cannot exit with multiple exit codes!");
    }
    else {
        match args[1].parse::<i32>() {
            Err(e) => {
                eprintln!("Cannot exit with this code because of an error: {:?}", e.kind());
                report_failure(index, returns);
            },
            Ok(code) => { report_success(index, returns); process::exit(code); },
        }
    };
}

// This will be used to execute commands!
fn runcommand(args:&[String], index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    // Do nothing if nothing was requested
    // This might occur when the user presses ENTER without even typing anything
    if args.is_empty() || args[0].is_empty() {
        print!("");
    }
    // Run a command passed in "args[0]" with arguments in "args[1]" (and so on) and get it's status
    // using process::Command::new().args().status();
    match process::Command::new(&args[0]).args(&args[1..]).status() { 
        Err(e) => {
            eprintln!("{}: Command execution failed because of an error: {}", args[0], e.kind());
            report_failure(index, returns)
        },
        Ok(process) => {
            // If the command is possible to run, save it's status to "returns" variable
            let command_status = CommandStatus {
                code: process.code(),
                success: process.success(),
                signal: process.signal(),
                core_dumped: process.core_dumped()
            };
            returns.insert(index, command_status);
        },
    }
    // Flush stdout
    io::stdout().flush().unwrap();
}

// These functions will be used to report success or failure when built-in or super commands are running
// This is usefull because typically we don't want the shell to abnormally quit when syntax of if statement is incorrect
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

fn then(index_of_then:&mut usize, returns: &mut HashMap<usize, CommandStatus>, commands: &[Vec<String>], stop:&mut bool) {
    if *index_of_then == 0 {
        eprintln!("SYNTAX ERROR! Operator \"THEN\" doesn't work when there is nothing before it!");
        report_failure(*index_of_then, returns);
        process::exit(1);
    }
    if *index_of_then == commands.len()-1 {
        eprintln!("SYNTAX ERROR! Operator \"THEN\" doesn't work when there is nothing after it!");
        report_failure(*index_of_then, returns);
    }
    // Compare exit status of previous and following commands
    let prev_index = *index_of_then-1;
    let prev_status = if returns.contains_key(&prev_index) {
        returns.get(&prev_index).unwrap().success
    }
    else {
        eprintln!("OPERATOR \"THEN\" FAILED! Unable to read exit code of the previous command!");
        process::exit(1);
    };

    // Find "END" keyword and save it's position
    let mut level = 1;
    let mut index_of_end = 0;
    for (i,c) in commands[*index_of_then+1..].iter().enumerate() {
        if c[0]=="if" {
            level+=1;
        }
        if c[0]=="end" {
            level-=1;
        }
        if level == 0 {
            index_of_end=*index_of_then+i;
        }
        if *index_of_then+i == commands.len()-1 && c[0] != "end" && level != 0 {
            eprintln!("SYNTAX ERROR! Operator \"THEN\" isn't properly closed with \"END\" operator!");
            report_failure(*index_of_then, returns);
            // Tell detect_commands() that it can't execute commands anymore
            *stop=true;
            return;
        }
    }

    // If previous command succeeded, don't do anything special
    if prev_status {
        report_success(*index_of_then, returns);
    }
    // If it didn't, jump to the index of the farest possible "END" keyword
    else {
        *index_of_then=index_of_end;
    }
}
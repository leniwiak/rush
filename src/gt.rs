use std::collections::HashMap;
// Very fucking strange behavior in Rust - you have to write "use crate::blah blah" if you want to "mod" something
// BUT the file is already used as a module somewhere else
use crate::helpful::*;
use std::env;

// Change working directory
pub fn gt(args:&[String], index:usize, returns:&mut HashMap<usize, CommandStatus>) {
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
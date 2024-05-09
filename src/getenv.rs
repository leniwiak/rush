use std::env::var_os;
use std::ffi::OsString;
use std::collections::HashMap;
// This is how to import other files from rush when the current source file is imported in "rush.rs"
use crate::helpful::*;

pub fn getenv(args:&[String], index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    // Check if there is just ONE argument
    // We can't check more than one variable at the same time
    if args.len() == 1 {
        eprintln!("Give me a variable name to check!");
        // As usual, run this function to report a failure.
        // "index" variable contains position of a command
        // "returns" contains information about all return codes that were reported by commands
        // Both variables are required because "returns" will be modified by "report_failure" according to the contents of "index"
        report_failure(index, returns);
    }
    else if args.len() > 2 {
        eprintln!("Cannot check multiple variables simultaneously!");
        report_failure(index, returns)
    }
    else {
        let variable = match var_os(&args[1]) {
            Some(ret) => ret,
            None => { 
                eprintln!("GETENV FAILED! Variable \"{}\"is not set", args[1]);
                report_failure(index, returns);
                OsString::new()
            }
        };
        println!("{}", variable.into_string().expect("Bruh"));
        report_success(index, returns);
    }
}
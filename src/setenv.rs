use std::env::set_var;
use std::collections::HashMap;
use crate::helpful::*;

pub fn setenv(args:&[String], index:usize, returns:&mut HashMap<usize, CommandStatus>) {
    // Check if there is just ONE argument
    // We can't set more than one variable at the same time
    if args.len() == 1 {
        eprintln!("Give me a variable name to set!");
        // As usual, run this function to report a failure.
        // "index" variable contains position of a command
        // "returns" contains information about all return codes that were reported by commands
        // Both variables are required because "returns" will be modified by "report_failure" according to the contents of "index"
        report_failure(index, returns);
    }
    else if args.len() > 2 {
        eprintln!("Cannot set multiple variables simultaneously!");
        report_failure(index, returns)
    }
    else {
        match args[1].split_once('=') {
            Some((key, value)) => {
                if key.is_empty() || value.is_empty() {
                    eprintln!("OPEATOR \"SETENV\" FAILED! Incorretly requested variable!");
                    report_failure(index, returns);
                }
                else {
                    set_var(key, value);
                    report_success(index, returns);
                }
            }
            _ => {
                eprintln!("OPEATOR \"SETENV\" FAILED! Incorretly requested variable!");
                report_failure(index, returns);
            }
        };

    }
}
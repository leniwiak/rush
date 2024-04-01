use crate::helpful::*;
use std::collections::HashMap;
use std::process;

// Just go away
pub fn exit(args:&[String], index:usize, returns:&mut HashMap<usize, CommandStatus>) {
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
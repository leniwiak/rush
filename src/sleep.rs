use std::process;
use std::thread;
use std::time;
// This is how to import other files from rush when the current source file is NOT imported in "rush.rs"
use carrot_libs::args;

fn main() {
    let opts = args::opts();
    let (swcs, vals) = args::swcs();

    // Refuse to run when switches have been passed
    if !swcs.is_empty() {
        eprintln!("This program does not support any switches and values!");
        process::exit(1);
    };
    if !vals.is_empty() {
        eprintln!("This program does not support any switches and values!");
        process::exit(1);
    };

    if opts.is_empty() {
        eprintln!("The \"SLEEP\" statement requires at least one argument!");
        process::exit(1);
    }
    if opts.len() > 1 {
        eprintln!("This commands does not accept more than one option!");
        // As usual, run this function to report a failure.
        // "index" variable contains position of a command
        // "returns" contains information about all return codes that were reported by commands
        // Both variables are required because "returns" will be modified by "report_failure" according to the contents of "index"
        std::process::exit(1);
    } else {
        // Parse argument to make sure it is a number
        match opts[0].parse::<u64>() {
            Ok(ret) => {
                let d = time::Duration::from_millis(ret);
                thread::sleep(d);
                std::process::exit(0);
            }
            _ => {
                eprintln!("Couldn't parse the time value!");
                std::process::exit(1);
            }
        }
    };
}

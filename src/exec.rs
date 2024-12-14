use std::{io, process};
use io::stdout;
use io::Write;

pub fn exec(args:&[String]) -> Result<process::Output, String> {
    // Run a command passed in "args[0]" with arguments in "args[1..]" and get it's output with
    // using process::Command::new().args().stdout(stdout()).output();
    match process::Command::new(&args[0]).args(&args[1..]).stdout(stdout()).output() { 
        Err(e) => Err(format!("{}: Command execution failed: {:?}", args[0], e.kind())),
        Ok(process) => {io::stdout().flush().unwrap(); Ok(process)},
    }
    // Flush stdout
    
}
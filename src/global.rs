use std::sync::atomic::{AtomicUsize, Ordering};

// This indicates which command we are iterating by
static INDEX:AtomicUsize = AtomicUsize::new(0);

pub fn index() -> usize {
    INDEX.load(Ordering::SeqCst)
}

pub fn set_index(val:usize) {
    INDEX.store(val, Ordering::SeqCst);
}

// This function prints out an error that just occured and tells the user on which line it happened
pub fn print_err(e:String, program_name:String, line_number: usize) {
    eprintln!("Program \"{program_name}\" returned an error at line {line_number}:\n{e}");
    //process::exit(1);
}
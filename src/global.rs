use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub static PROGRAM_NAME: &str = "Rush";

// This indicates which command we are iterating by
static INDEX: AtomicUsize = AtomicUsize::new(0);

// Do we have to stop?
pub static ALLOW_INTERRUPTS: AtomicBool = AtomicBool::new(false);
pub static INTERRUPT_NOW: AtomicBool = AtomicBool::new(false);

pub fn index() -> usize {
    INDEX.load(Ordering::SeqCst)
}

pub fn set_index(val: usize) {
    INDEX.store(val, Ordering::SeqCst);
}

pub fn set_interrupt_now(val: bool) {
    INTERRUPT_NOW.store(val, Ordering::SeqCst);
}

pub fn interrupt_now() -> bool {
    INTERRUPT_NOW.load(Ordering::SeqCst)
}

pub fn set_allow_interrupts(val: bool) {
    ALLOW_INTERRUPTS.store(val, Ordering::SeqCst);
}

pub fn allow_interrupts() -> bool {
    ALLOW_INTERRUPTS.load(Ordering::SeqCst)
}

// This function prints out an error that just occured and tells the user on which line it happened
pub fn print_err(e: String, program_name: String, line_number: usize) {
    eprintln!("Program \"{program_name}\" returned an error at line {line_number}:\n{e}");
    //process::exit(1);
}

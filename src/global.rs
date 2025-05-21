use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::env;
use std::process;

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
pub fn print_err<S:AsRef<str>>(e: S, program_name: S, line_number: usize) {
    eprintln!("Program \"{}\" returned an error at line {line_number}:\n{}", program_name.as_ref(), e.as_ref());
    set_interrupt_now(true);
}

// Remove all unescaped quotationmarks even inside words
pub fn remove_quotationmarks<S:AsRef<str>>(input: S) -> String {
    let mut str = String::new();
    for c in input.as_ref().chars() {
        // If current character is a qmark AND it's preceded by a slash, remove the slash from string
        str.push(c);
        let last = str.chars().last().unwrap_or(' ');
        if (c == '\'' || c == '"') && last != '\\' {
            str.pop();
        }
    }
    str
}

enum ResolvingMode {
    SingleQuote,
    DoubleQuote,
    Variable,
    None
}

// This function removes unescaped slashes
pub fn escape_slashes<S:AsRef<str>>(input: S, remove_quotation_marks:bool, resolve_variables:bool) -> Result<String, String> {
    let mut previous_c= ' ';
    let mut variable_name = String::new();
    let mut output = String::new();
    let mut mode = ResolvingMode::None;
    for c in input.as_ref().chars() {
        if c == '\'' && previous_c != '\\' {
            match mode {
                ResolvingMode::None => mode = ResolvingMode::SingleQuote,
                ResolvingMode::SingleQuote => mode = ResolvingMode::None,
                _ => (),
            }
        }
        else if c == '"' && previous_c != '\\' {
            match mode {
                ResolvingMode::None => mode = ResolvingMode::DoubleQuote,
                ResolvingMode::DoubleQuote => mode = ResolvingMode::None,
                _ => (),
            }
        }
        else if c == '$' && previous_c != '\\' && resolve_variables {
            if let ResolvingMode::None = mode { mode = ResolvingMode::Variable }
        }
        else if c == ' ' {
            if let ResolvingMode::Variable = mode { mode = ResolvingMode::None }
        }
        else if c == '\\' && previous_c != '\\' {
            ();
        }
        else {
            match mode {
                ResolvingMode::Variable => variable_name.push(c),
                _ => {
                    if !variable_name.is_empty() {
                        let env = env::var(&variable_name);
                        match env {
                            Err(e) => return Err(format!("{variable_name}: Reference to a variable caused an error: {:?}", e)),
                            Ok(variable_contents) => output.push_str(&variable_contents),
                        }
                        variable_name.clear();
                    }
                    output.push(c);
                },
            }
        }

        if (c == '\'' || c == '"')  && !remove_quotation_marks {
            output.push(c);
        }

        previous_c = c;
    }
    Ok(output)
}
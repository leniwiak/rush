use carrot_libs::args;
use std::env;
use std::process;

fn main() {
    let opts = args::opts();
    let swcs = args::swcs();
    if !opts.is_empty() || !swcs.0.is_empty() || !swcs.1.is_empty() {
        eprintln!("This program does not support any options nor switches!");
        process::exit(1);
    }
    println!("{}", env::current_dir().unwrap().display())
}

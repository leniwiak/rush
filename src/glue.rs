use carrot_libs::args;
use std::io::{self, IsTerminal, Read};
use std::process;

fn main() {
    let mut args = args::args();

    if args.len() == 1 && io::stdin().is_terminal() {
        eprintln!("This program requires more arguments to work!");
        process::exit(1);
    }

    let mut i = 1;
    let mut large_text = String::new();
    // Process piped stuff
    if !io::stdin().is_terminal() {
        // Save contents of STDIN to a string
        let mut contents_of_stdin = String::new();
        io::stdin().lock().read_to_string(&mut contents_of_stdin).expect("Failed to get stdin contents!");
        // Split STDIN content by space chars
        let list_of_words_from_stdin: Vec<&str> = contents_of_stdin.rsplit(' ').collect();
        // Add every word from STDIN to args
        for c in list_of_words_from_stdin {
            args.insert(1, c.trim().to_owned());
        }
    }

    while i < args.len(){
        large_text.push_str(&args[i]);
        i+=1;
    }
    println!("{}", large_text);
}

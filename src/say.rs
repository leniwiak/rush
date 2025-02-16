use carrot_libs::args;

fn main() {
    let args = args::args();

    for (idx, a) in args.iter().skip(1).enumerate() {
        print!("{}", a);
        if idx < args.len() {
            print!(" ");
        }
    }
    println!();
}

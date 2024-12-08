use carrot_libs::args;

fn main() {
    let args = args::args();

    for a in args.iter().skip(1) {
        print!("{}", a);
    }
    println!();
}

mod args;
use args::Args;

#[paw::main]
fn main(args: Args) {
    println!("Hello, world! {}", args.file.display());
}

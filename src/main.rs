mod arg_parser;

use arg_parser::Args;
use clap::Parser;

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}

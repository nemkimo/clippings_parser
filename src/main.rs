mod arg_parser;
mod file_parser;

use crate::arg_parser::Args;
use crate::file_parser::parse_file;
use clap::Parser;

fn main() {
    let args = Args::parse();
    let result = parse_file(args.clippings).unwrap();
    println!("{:?}", result);
}

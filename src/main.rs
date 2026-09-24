use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use crate::lackey_parser::LackeyParser;

mod lackey_parser;
mod types;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let input_file = File::open(&args[1])?;
    let reader = BufReader::new(input_file);

    let mut parser = LackeyParser::new(reader.lines().collect::<Result<_, _>>().unwrap());

    while let Some(access) = parser.next_access() {
        //println!("{}", format!("{:x}", access.address));
    }

    Ok(())
}

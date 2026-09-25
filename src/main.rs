use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use clap::Parser;
use crate::lackey_parser::LackeyParser;
use crate::basic_parser::BasicParser;
use crate::cache::Cache;

mod lackey_parser;
mod basic_parser;
mod pseudorandom_policy;
mod types;
mod cache;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: String,

    #[arg(short, long)]
    cache: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let input_file = File::open(args.file)?;
    let reader = BufReader::new(input_file);

    //let mut parser = LackeyParser::new(reader.lines().collect::<Result<_, _>>().unwrap());
    let mut parser = BasicParser::new(reader.lines().collect::<Result<_, _>>().unwrap());

    let mut cache_config = args.cache.split(",");

    let Some(line_size_str) = cache_config.next() else {
        panic!("Failed to parse line_size");
    };

    let line_size = u32::from_str_radix(line_size_str, 10).expect("failed to parser line_size");

    let Some(nb_lines_str) = cache_config.next() else {
        panic!("Failed to parse nb_lines");
    };

    let nb_lines = u32::from_str_radix(nb_lines_str, 10).expect("failed to parser nb_lines");

    let Some(associativity_str) = cache_config.next() else {
        panic!("Failed to parse associativity");
    };

    let associativity = u32::from_str_radix(associativity_str, 10).expect("failed to parser associativity");

    let cache = Cache::new(line_size, nb_lines, associativity);

    while let Some(access) = parser.next_access() {
        //println!("{}", format!("{:x}", access.address));
    }

    Ok(())
}

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use clap::Parser;
use crate::parser::AccessParser;
use crate::lackey_parser::LackeyParser;
use crate::basic_parser::BasicParser;
use crate::cache::Cache;
use crate::types::AccessOperation;

mod lackey_parser;
mod basic_parser;
mod pseudorandom_policy;
mod parser;
mod types;
mod cache;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: String,

    #[arg(short, long)]
    cache: String,
}

fn make_parser(path: &str, lines: Vec<String>) -> Result<Box<dyn AccessParser>, String> {
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("dat") => Ok(Box::new(BasicParser::new(lines))),
        Some("lackey") => Ok(Box::new(LackeyParser::new(lines))),
        other => Err(format!("unsupported file extension {:?}", other))
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let input_file = File::open(&args.file)?;
    let reader = BufReader::new(input_file);
    let lines = reader.lines().collect::<Result<_, _>>().unwrap();

    let mut parser = make_parser(&args.file, lines).expect("cannot select parser");

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

    let mut cache = Cache::new(line_size, nb_lines, associativity);

    while let Some(access) = parser.next_access() {
        if access.operation == AccessOperation::WRITE {
            cache.write(access);
        } else if access.operation == AccessOperation::READ {
            cache.read(access);
        }
    }

    println!("Hits {}", cache.hits);
    println!("Misses {}", cache.misses);
    println!("Evictions {}", cache.evictions);

    Ok(())
}

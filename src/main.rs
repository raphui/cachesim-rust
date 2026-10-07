use std::env;
use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use clap::Parser;
use fdt::Fdt;
use fdt::node::FdtNode;
use crate::parser::AccessParser;
use crate::lackey_parser::LackeyParser;
use crate::basic_parser::BasicParser;
use crate::cache::Cache;
use crate::types::Access;
use crate::types::AccessOperation;
use crate::types::AccessType;
use crate::types::CacheType;

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

fn parse_cache(cache: FdtNode) -> Cache {
    let level = cache.property("cache-level").and_then(|p| p.as_usize()).expect("cache-level") as u32;
    let line_size = cache.property("line-size").and_then(|p| p.as_usize()).expect("line-size") as u32;
    let nb_lines = cache.property("nb-lines").and_then(|p| p.as_usize()).expect("nb-lines") as u32;
    let associativity = cache.property("associativity").and_then(|p| p.as_usize()).expect("associativity") as u32;
    //let policy = cache.property("policy").and_then(|p| p.as_str()).expect("policy");
    let type_ = if cache.name.contains("_i") {
        CacheType::INSTRUCTION
    } else if cache.name.contains("_d") {
        CacheType::DATA
    } else {
        CacheType::UNIFIED
    };

    return Cache::new(String::from(cache.name), type_, level, line_size, nb_lines, associativity)
}

fn add_cache(node: FdtNode, fdt: &Fdt, caches: &mut Vec<Cache>) -> usize {
    if let Some(idx) =  caches.iter().position(|c| c.name == node.name) {
        return idx;
    }

    let mut cache = parse_cache(node);

    if let Some(next_level_cache) = node.property("next-level-cache")
            .and_then(|p| p.as_usize())
            .and_then(|p| fdt.find_phandle(p as u32)) {

        let idx = add_cache(next_level_cache, fdt, caches);
        cache.next_level = Some(idx);
    }

    caches.push(cache);

    return caches.len() - 1;
}

fn run(caches: &mut Vec<Cache>, idx: usize, access: Access) {
    let mut current = Some(idx);

    while let Some(i) = current {
            if caches[i].perform(access) {
                break;
            }

            current = caches[i].next_level;
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let content = fs::read(&args.cache)?;
    let fdt = Fdt::new(&content).unwrap();
    let root_node = fdt.find_node("/").unwrap();

    let mut caches: Vec<Cache> = Vec::new();

    for child in root_node.children() {
        if child.property("cache-level").is_some() {
            add_cache(child, &fdt, &mut caches);
        }
    }

    let input_file = File::open(&args.file)?;
    let reader = BufReader::new(input_file);
    let lines = reader.lines().collect::<Result<_, _>>().unwrap();

    let mut parser = make_parser(&args.file, lines).expect("cannot select parser");

    let l1_i = caches.iter().position(|c| c.level == 1 && matches!(c.type_, CacheType::INSTRUCTION | CacheType::UNIFIED)).expect("no L1 instruction cache");
    let l1_d = caches.iter().position(|c| c.level == 1 && matches!(c.type_, CacheType::DATA | CacheType::UNIFIED)).expect("no L1 data cache");

    while let Some(access) = parser.next_access() {
        let cache_idx = match access.type_ {
            AccessType::INSTRUCTION => l1_i,
            AccessType::DATA => l1_d,
            _ => continue,
        };

        run(&mut caches, cache_idx, access);
    }

    //println!("Hits {}", cache.hits);
    //println!("Misses {}", cache.misses);
    //println!("Evictions {}", cache.evictions);

    Ok(())
}

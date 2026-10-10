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
use crate::types::InclusionPolicy;
use crate::types::Status;

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

fn parse_cache(node: FdtNode) -> Cache {
    let level = node.property("cache-level").and_then(|p| p.as_usize()).expect("cache-level") as u32;
    let line_size = node.property("line-size").and_then(|p| p.as_usize()).expect("line-size") as u32;
    let nb_lines = node.property("nb-lines").and_then(|p| p.as_usize()).expect("nb-lines") as u32;
    let associativity = node.property("associativity").and_then(|p| p.as_usize()).expect("associativity") as u32;
    //let policy = node.property("policy").and_then(|p| p.as_str()).expect("policy");
    let type_ = if node.name.contains("_i") {
        CacheType::INSTRUCTION
    } else if node.name.contains("_d") {
        CacheType::DATA
    } else {
        CacheType::UNIFIED
    };

    let mut cache = Cache::new(String::from(node.name), type_, level, line_size, nb_lines, associativity);

    if level > 1 {
        let inclusion_policy = if node.property("inclusion-policy")
            .and_then(|p| p.as_str()).expect("inclusion-policy").contains("inclusive") {
            InclusionPolicy::INCLUSIVE
        } else {
            InclusionPolicy::EXCLUSIVE
        };

        cache.inclusion_policy = Some(inclusion_policy);
    }

    return cache;
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

fn handle_inclusion_policy(caches: &mut Vec<Cache>, l1: usize, l2: usize, access: Access) {
    if let Some(inclusion_policy) =  caches[l2].inclusion_policy {
        match inclusion_policy {
            InclusionPolicy::INCLUSIVE => {
                match caches[l2].perform(access) {
                    Status::HIT => {
                    },
                    Status::MISS {evicted: None} => {
                        caches[l1].place(&access);
                    },
                    Status::MISS {evicted: Some(victim)} => {
                        caches[l1].place(&access);
                        caches[l1].invalidate(&victim);
                    }
                }
            },
            InclusionPolicy::EXCLUSIVE => {
                match caches[l2].perform(access) {
                    Status::HIT => {
                        caches[l2].invalidate(&access);

                        if let Some(evicted) = caches[l1].place(&access) {
                            caches[l2].place(&evicted);
                        }
                    },
                    Status::MISS {evicted: None} => {
                        caches[l1].place(&access);
                    },
                    Status::MISS {evicted: Some(victim)} => {
                        caches[l1].place(&access);

                        caches[l2].place(&victim);
                       
                    }
                }
            },
        };
    }
}

fn run(caches: &mut Vec<Cache>, l1: usize, access: Access) {
    if let Status::MISS {evicted: _} = caches[l1].perform(access) {
        if let Some(l2) = caches[l1].next_level {
            if access.operation == AccessOperation::READ {
                handle_inclusion_policy(caches, l1, l2, access);
            }
        }
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

    for cache in caches {
        println!("---- L{} STATS ----", cache.level);
        println!("Hits {}", cache.hits);
        println!("Misses {}", cache.misses);
        println!("Evictions {}", cache.evictions);
    }


    Ok(())
}

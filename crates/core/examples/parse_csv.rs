use std::collections::BTreeSet;
use std::path::Path;
use std::{env, process};

use app_core::parser;

const SAMPLE_CSV: &str = "crates/core/fixtures/sample.csv";

fn main() {
    let path = env::args().nth(1).unwrap_or(SAMPLE_CSV.to_string());

    let entries = parser::parse_csv(Path::new(&path)).unwrap_or_else(|e| {
        eprintln!("Error parsing {path}: {e}");
        process::exit(1);
    });

    for entry in &entries {
        println!("{entry}");
    }

    let assets: BTreeSet<&str> = entries.iter().map(|e| e.asset.to_str()).collect();

    println!("\n--- Summary ---");
    println!("Total entries: {}", entries.len());
    println!("Unique assets: {} ({:?})", assets.len(), assets);
}

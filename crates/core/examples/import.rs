use std::path::Path;

use app_core::{api, models::Asset};

const SAMPLE_CSV: &str = "crates/core/fixtures/sample.csv";

fn main() {
    let path = std::env::args().nth(1).unwrap_or(SAMPLE_CSV.to_string());
    let summary = api::preview_import(&Asset::Eur, Path::new(&path)).unwrap();
    println!("=== Preview ===");
    println!("{:#?}", summary);
}

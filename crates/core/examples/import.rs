use std::path::Path;

use app_core::{api, context::Context, models::Asset};

const SAMPLE_CSV: &str = "crates/core/fixtures/sample.csv";

fn main() {
    let path = std::env::args().nth(1).unwrap_or(SAMPLE_CSV.to_string());
    let db_path = std::env::temp_dir().join("betc_import_example.db");
    let ctx = Context::open(&db_path, Asset::Eur).unwrap();
    let summary = api::preview_import(&ctx, Path::new(&path)).unwrap();
    println!("=== Preview ===");
    println!("{:#?}", summary);
    let _ = std::fs::remove_file(&db_path);
}

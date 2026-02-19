use std::path::Path;

use app_core::{analysis, parser};

const SAMPLE_CSV: &str = "crates/core/fixtures/sample.csv";

fn main() {
    let path = std::env::args().nth(1).unwrap_or(SAMPLE_CSV.to_string());
    let entries = parser::parse_csv(Path::new(&path)).unwrap();
    let mut trades = analysis::find_trades(&entries);
    trades.sort_by_key(|t| t.date);
    for trade in trades {
        let side = trade.side_for(&parser::Asset::Btc);
        println!("BTC {:?}: {:#?}", side, trade);
    }
}

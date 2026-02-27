use std::path::Path;

use app_core::{
    api,
    models::{AppConfig, Asset},
};

const SAMPLE_CSV: &str = "crates/core/fixtures/sample.csv";

fn main() {
    let path = std::env::args().nth(1).unwrap_or(SAMPLE_CSV.to_string());
    let path = Path::new(&path);

    // ── persist to DB, then query ──────────────────────────
    let cfg = AppConfig {
        db_path: std::env::temp_dir().join("betc_engine_example.db"),
        quote: Asset::Eur,
    };

    // ── trades_summary (via preview_import) ────────────────
    let summary = api::preview_import(&cfg.quote, path).unwrap();
    println!("=== Trades Summary ===");
    println!(
        "{} trades ({} buys, {} sells, {} unknown)",
        summary.total_trades, summary.buys, summary.sells, summary.unknown
    );
    if let Some((earliest, latest)) = summary.date_range {
        println!(
            "Date range: {} → {}",
            earliest.date_naive(),
            latest.date_naive()
        );
    }
    println!(
        "Spent:    {} {}",
        summary.spent.amount(),
        summary.spent.asset()
    );
    println!(
        "Received: {} {}",
        summary.received.amount(),
        summary.received.asset()
    );
    println!(
        "Fees:     {} {}",
        summary.fees.amount(),
        summary.fees.asset()
    );
    api::confirm_import(&cfg, path).unwrap();

    // ── position_summary ───────────────────────────────────
    let pos = api::position_summary(&cfg).unwrap();
    println!("\n=== Position Summary ===");
    println!("Held: {} {}", pos.held.amount(), pos.held.asset());
    println!(
        "Invested: {} {}",
        pos.invested.amount(),
        pos.invested.asset()
    );
    println!(
        "Proceeds: {} {}",
        pos.proceeds.amount(),
        pos.proceeds.asset()
    );
    println!("Fees:     {} {}", pos.fees.amount(), pos.fees.asset());
    println!("Buys: {}, Sells: {}", pos.buys, pos.sells);
    if let Some(bep) = pos.bep {
        println!("Break-even price: {} EUR/BTC", bep);
    } else {
        println!("Break-even price: N/A (no holdings)");
    }

    // ── enriched trades (with BEP timeline) ────────────────
    let trades = api::trades(&cfg).unwrap();
    println!("\n=== Enriched Trades ({}) ===", trades.len());
    for trade in &trades {
        let side_str = trade
            .side
            .as_ref()
            .map_or("—".to_string(), |s| format!("{:?}", s));
        let bep_str = trade.bep.as_ref().map_or("N/A".to_string(), |b| {
            format!("{} {}", b.amount(), b.asset())
        });
        println!(
            "  {} | {:4} | spent {} {} → received {} {} | bep: {}",
            trade.date.date_naive(),
            side_str,
            trade.spent.amount(),
            trade.spent.asset(),
            trade.received.amount(),
            trade.received.asset(),
            bep_str,
        );
    }

    // cleanup
    let _ = std::fs::remove_file(&cfg.db_path);
}

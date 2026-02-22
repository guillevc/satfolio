use std::path::Path;

use app_core::{api, context::Context, models::Asset};

const SAMPLE_CSV: &str = "crates/core/fixtures/sample.csv";

fn main() {
    let path = std::env::args().nth(1).unwrap_or(SAMPLE_CSV.to_string());
    let path = Path::new(&path);

    // ── persist to DB, then query ──────────────────────────
    let db_path = std::env::temp_dir().join("betc_engine_example.db");
    let ctx = Context::open(&db_path, Asset::Eur).unwrap();

    // ── trades_summary (via preview_import) ────────────────
    let summary = api::preview_import(&ctx, path).unwrap();
    println!("=== Trades Summary ===");
    println!(
        "{} trades ({} buys, {} sells, {} unknown)",
        summary.total_trades, summary.buys, summary.sells, summary.unknown
    );
    if let Some((earliest, latest)) = summary.date_range {
        println!("Date range: {} → {}", earliest.date_naive(), latest.date_naive());
    }
    println!("Spent:    {} {}", summary.spent.amount(), summary.spent.asset());
    println!("Received: {} {}", summary.received.amount(), summary.received.asset());
    println!("Fees:     {} {}", summary.fees.amount(), summary.fees.asset());
    api::confirm_import(&ctx, path).unwrap();

    // ── position_summary ───────────────────────────────────
    let pos = api::position_summary(&ctx).unwrap();
    println!("\n=== Position Summary ===");
    println!("Held: {} {}", pos.held.amount(), pos.held.asset());
    println!("Invested: {} {}", pos.invested.amount(), pos.invested.asset());
    println!("Proceeds: {} {}", pos.proceeds.amount(), pos.proceeds.asset());
    println!("Fees:     {} {}", pos.fees.amount(), pos.fees.asset());
    println!("Buys: {}, Sells: {}", pos.buys, pos.sells);
    if let Some(bep) = pos.bep {
        println!("Break-even price: {} EUR/BTC", bep);
    } else {
        println!("Break-even price: N/A (no holdings)");
    }

    // ── bep_snaps ──────────────────────────────────────────
    let snaps = api::bep_snaps(&ctx).unwrap();
    println!("\n=== BEP Timeline ({} snapshots) ===", snaps.len());
    for (date, snap) in &snaps {
        let bep_str = snap.bep.map_or("N/A".to_string(), |b| format!("{}", b));
        println!(
            "  {} | held: {} BTC | bep: {} EUR/BTC",
            date,
            snap.held.amount(),
            bep_str,
        );
    }

    // ── trades list ────────────────────────────────────────
    let trades = api::trades(&ctx).unwrap();
    println!("\n=== All Trades ({}) ===", trades.len());
    for trade in &trades {
        println!(
            "  {} | spent {} {} → received {} {} | fee {} {}",
            trade.date.date_naive(),
            trade.spent.amount(),
            trade.spent.asset(),
            trade.received.amount(),
            trade.received.asset(),
            trade.fee.amount(),
            trade.fee.asset(),
        );
    }

    // cleanup
    let _ = std::fs::remove_file(&db_path);
}

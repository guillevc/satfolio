use std::path::Path;

use crate::engine;
use crate::errors::CoreResult;
use crate::models::{Asset, BepSnapshot, DashboardStats, Trade, TradesSummary};
use crate::parser;

pub fn import_csv(path: &Path) -> CoreResult<TradesSummary> {
    let mut trades = parser::parse_kraken_csv(path)?;
    // TODO: store in db
    trades.sort_by_key(|t| t.date);
    let summary = engine::summarize_trades(&Asset::Btc, &&Asset::Eur, &trades);
    Ok(summary)
}

pub fn bep_series(asset: &Asset, counter: &Asset, trades: &[Trade]) -> Vec<BepSnapshot> {
    engine::compute_bep_series(asset, counter, trades)
}

pub fn dashboard(asset: &Asset, counter: &Asset, trades: &[Trade]) -> DashboardStats {
    engine::compute_dashboard_stats(asset, counter, trades)
}

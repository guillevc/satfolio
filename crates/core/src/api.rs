use std::path::Path;

use crate::errors::CoreResult;
use crate::models::*;
use crate::{context::Context, db, engine, parser};

// ── Import ──────────────────────────────────────────────

pub fn preview_import(path: &Path) -> CoreResult<TradesSummary> {
    let trades = parser::parse_kraken_csv(path)?;
    let summary: TradesSummary = engine::summarize_trades(&Asset::Btc, &Asset::Eur, &trades);
    Ok(summary)
}

pub fn confirm_import(ctx: &Context, path: &Path) -> CoreResult<TradesSummary> {
    let trades = parser::parse_kraken_csv(path)?;
    db::save_trades(&ctx.conn, &trades)?;
    let summary: TradesSummary = engine::summarize_trades(&Asset::Btc, &Asset::Eur, &trades);
    Ok(summary)
}

// ── Dashboard ───────────────────────────────────────────

pub fn dashboard_stats(ctx: &Context) -> CoreResult<DashboardStats> {
    let trades = db::load_trades(&ctx.conn)?;
    let stats = engine::dashboard_stats(&Asset::Btc, &Asset::Eur, &trades);
    Ok(stats)
}

pub fn bep_series(ctx: &Context) -> CoreResult<Vec<BepSnapshot>> {
    let trades = db::load_trades(&ctx.conn)?;
    let series = engine::bep_series(&Asset::Btc, &Asset::Eur, &trades);
    Ok(series)
}

pub fn trades(ctx: &Context) -> CoreResult<Vec<Trade>> {
    Ok(db::load_trades(&ctx.conn)?)
}

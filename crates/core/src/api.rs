use std::collections::BTreeMap;
use std::path::Path;

use chrono::NaiveDate;

use crate::errors::CoreResult;
use crate::models::{Asset, AssetPair, BepSnapshot, PositionSummary, Trade, TradesSummary};
use crate::{context::Context, db, engine, parser};

const BTC_EUR: AssetPair = AssetPair {
    base: Asset::Btc,
    quote: Asset::Eur,
};

pub fn preview_import(path: &Path) -> CoreResult<TradesSummary> {
    let trades = parser::parse_kraken_csv(path)?;
    let summary = engine::trades_summary(&BTC_EUR, &trades)?;
    Ok(summary)
}

pub fn confirm_import(ctx: &Context, path: &Path) -> CoreResult<TradesSummary> {
    let trades = parser::parse_kraken_csv(path)?;
    db::save_trades(&ctx.conn, &trades)?;
    let summary = engine::trades_summary(&BTC_EUR, &trades)?;
    Ok(summary)
}

pub fn position_summary(ctx: &Context) -> CoreResult<PositionSummary> {
    let trades = db::load_trades(&ctx.conn)?;
    let stats = engine::position_summary(&BTC_EUR, &trades)?;
    Ok(stats)
}

pub fn bep_snaps(ctx: &Context) -> CoreResult<BTreeMap<NaiveDate, BepSnapshot>> {
    let trades = db::load_trades(&ctx.conn)?;
    let series = engine::bep_snaps(&BTC_EUR, &trades)?;
    Ok(series)
}

pub fn trades(ctx: &Context) -> CoreResult<Vec<Trade>> {
    let trades = db::load_trades(&ctx.conn)?;
    Ok(trades)
}

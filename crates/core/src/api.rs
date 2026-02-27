use std::path::Path;

use chrono::Utc;

use crate::errors::CoreResult;
use crate::models::{AppConfig, Asset, AssetPair, DashboardStats, EnrichedTrade, TradesSummary};
use crate::{db, engine, parser, price};

fn btc_pair(quote: &Asset) -> AssetPair {
    AssetPair {
        base: Asset::Btc,
        quote: quote.clone(),
    }
}

pub fn preview_import(quote: &Asset, path: &Path) -> CoreResult<TradesSummary> {
    let pair = btc_pair(quote);
    let trades = parser::parse_kraken_csv(path)?;
    let summary = engine::trades_summary(&pair, &trades)?;
    Ok(summary)
}

pub fn confirm_import(cfg: &AppConfig, path: &Path) -> CoreResult<TradesSummary> {
    let conn = db::open(&cfg.db_path)?;
    let pair = btc_pair(&cfg.quote);
    let trades = parser::parse_kraken_csv(path)?;
    db::save_trades(&conn, &trades)?;
    let summary = engine::trades_summary(&pair, &trades)?;
    Ok(summary)
}

pub fn dashboard_stats(cfg: &AppConfig) -> CoreResult<DashboardStats> {
    let conn = db::open(&cfg.db_path)?;
    let pair = btc_pair(&cfg.quote);
    let trades = db::load_trades(&conn)?;
    let summary = engine::position_summary(&pair, &trades)?;

    let candles = db::load_candles(&conn, &cfg.quote)?;
    let current = candles.last().map(|c| c.close).unwrap_or_default();
    let prev = candles.iter().rev().nth(1).map(|c| c.close);

    let mut stats = engine::dashboard_stats(&summary, current, prev);
    stats.candles = candles;
    Ok(stats)
}

pub fn trades(cfg: &AppConfig) -> CoreResult<Vec<EnrichedTrade>> {
    let conn = db::open(&cfg.db_path)?;
    let pair = btc_pair(&cfg.quote);
    let trades = db::load_trades(&conn)?;
    Ok(engine::enrich_trades(&pair, trades)?)
}

/// Gap-fill candles from Kraken API if behind today. Network I/O.
///
/// Opens its own DB connections so no `Connection` is held across the network `.await`.
pub async fn sync_candles(cfg: &AppConfig) -> CoreResult<()> {
    let last_date = {
        let conn = db::open(&cfg.db_path)?;
        let candles = db::load_candles(&conn, &cfg.quote)?;
        match candles.last() {
            Some(last) if last.date < Utc::now().date_naive() => last.date,
            _ => return Ok(()),
        }
    };

    let new = price::fetch_ohlc(&cfg.quote, last_date).await?;

    if !new.is_empty() {
        let conn = db::open(&cfg.db_path)?;
        db::save_candles(&conn, &cfg.quote, &new)?;
    }

    Ok(())
}

/// Open + migrate DB and seed bundled price data if empty. Call once at startup.
pub fn init_db(cfg: &AppConfig, prices_dir: &Path) -> CoreResult<()> {
    let conn = db::open(&cfg.db_path)?;
    let candles = db::load_candles(&conn, &cfg.quote)?;
    if candles.is_empty() {
        let bundled = price::load_bundled_prices(prices_dir, &cfg.quote)?;
        db::save_candles(&conn, &cfg.quote, &bundled)?;
    }
    Ok(())
}

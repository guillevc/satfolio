use std::path::Path;

use chrono::Utc;

use crate::errors::CoreResult;
use crate::models::{
    AppConfig, Asset, AssetPair, Candle, EnrichedTrade, PositionSummary, TradesSummary,
};
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

pub fn position_summary(cfg: &AppConfig) -> CoreResult<PositionSummary> {
    let conn = db::open(&cfg.db_path)?;
    let pair = btc_pair(&cfg.quote);
    let trades = db::load_trades(&conn)?;
    let stats = engine::position_summary(&pair, &trades)?;
    Ok(stats)
}

pub fn trades(cfg: &AppConfig) -> CoreResult<Vec<EnrichedTrade>> {
    let conn = db::open(&cfg.db_path)?;
    let pair = btc_pair(&cfg.quote);
    let trades = db::load_trades(&conn)?;
    Ok(engine::enrich_trades(&pair, trades)?)
}

/// Return candles from DB (or seed from bundled CSV on first run). No network.
pub fn candles(cfg: &AppConfig, prices_dir: &Path) -> CoreResult<Vec<Candle>> {
    let conn = db::open(&cfg.db_path)?;
    let mut candles = db::load_candles(&conn, &cfg.quote)?;

    if candles.is_empty() {
        candles = price::load_bundled_prices(prices_dir, &cfg.quote)?;
        db::save_candles(&conn, &cfg.quote, &candles)?;
    }

    Ok(candles)
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

/// Validate that the DB can be opened and migrated. Call once at startup.
pub fn init_db(db_path: &Path) -> CoreResult<()> {
    db::open(db_path)?;
    Ok(())
}

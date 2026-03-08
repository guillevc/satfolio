use std::path::Path;

use chrono::Utc;

use crate::errors::{CoreError, CoreResult};
use crate::models::{
    AppConfig, Asset, AssetPair, DashboardStats, EnrichedTrade, ImportOutcome, ImportPreview,
    ImportRecord,
};
use crate::{db, engine, hash, parser, price};

/// Construct the BTC/{quote} pair. BTC is always the base asset.
fn trading_pair(quote: &Asset) -> AssetPair {
    AssetPair {
        base: Asset::Btc,
        quote: quote.clone(),
    }
}

/// Auto-detect CSV provider, parse trades, and return preview with dedup info.
pub fn preview_import(cfg: &AppConfig, path: &Path) -> CoreResult<ImportPreview> {
    let provider = parser::detect_provider(path)?;
    let pair = trading_pair(&cfg.quote);
    let trades = parser::parse_csv(&provider, path)?;
    let summary = engine::trades_summary(&pair, &trades)?;

    let file_hash = hash::file_sha256(path)?;
    let conn = db::open(&cfg.db_path)?;
    let exact_file_duplicate = db::find_import_by_hash(&conn, &file_hash)?.is_some();

    let existing = db::existing_trade_hashes(&conn)?;
    let trade_hashes: Vec<String> = trades
        .iter()
        .map(|t| hash::trade_hash(provider.as_str(), t))
        .collect();
    let duplicate_trades = trade_hashes
        .iter()
        .filter(|h| existing.contains(*h))
        .count();

    Ok(ImportPreview {
        provider,
        summary,
        file_hash,
        duplicate_trades,
        exact_file_duplicate,
    })
}

/// Auto-detect CSV provider, parse, dedup, persist trades + import record, and return result.
pub fn confirm_import(cfg: &AppConfig, path: &Path) -> CoreResult<ImportOutcome> {
    let provider = parser::detect_provider(path)?;
    let pair = trading_pair(&cfg.quote);
    let trades = parser::parse_csv(&provider, path)?;
    let summary = engine::trades_summary(&pair, &trades)?;

    let file_hash = hash::file_sha256(path)?;
    let conn = db::open(&cfg.db_path)?;

    if db::find_import_by_hash(&conn, &file_hash)?.is_some() {
        return Err(CoreError::DuplicateFile);
    }

    let existing = db::existing_trade_hashes(&conn)?;
    let trade_hashes: Vec<String> = trades
        .iter()
        .map(|t| hash::trade_hash(provider.as_str(), t))
        .collect();
    let duplicate_count = trade_hashes
        .iter()
        .filter(|h| existing.contains(*h))
        .count();

    if duplicate_count == trades.len() && !trades.is_empty() {
        return Err(CoreError::AllTradesDuplicate(trades.len()));
    }

    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let import = db::save_import_with_trades(
        &conn,
        &provider,
        &filename,
        &file_hash,
        &trades,
        &trade_hashes,
        &existing,
    )?;

    let message = if duplicate_count > 0 {
        Some(format!(
            "{duplicate_count} of {} trades were skipped (already in database)",
            trades.len(),
        ))
    } else {
        None
    };

    Ok(ImportOutcome {
        import,
        summary,
        message,
    })
}

/// List all import records.
pub fn list_imports(cfg: &AppConfig) -> CoreResult<Vec<ImportRecord>> {
    let conn = db::open(&cfg.db_path)?;
    Ok(db::list_imports(&conn)?)
}

/// Remove an import and cascade-delete its trades.
pub fn remove_import(cfg: &AppConfig, import_id: i64) -> CoreResult<()> {
    let conn = db::open(&cfg.db_path)?;
    Ok(db::remove_import(&conn, import_id)?)
}

/// Build dashboard stats from all trades + candle history. Computes position, BEP, and P&L.
pub fn dashboard_stats(cfg: &AppConfig) -> CoreResult<DashboardStats> {
    let conn = db::open(&cfg.db_path)?;
    let pair = trading_pair(&cfg.quote);
    let trades = db::load_trades(&conn)?;
    let summary = engine::position_summary(&pair, &trades)?;

    let candles = db::load_candles(&conn, &cfg.quote)?;
    let current = candles.last().map(|c| c.close).unwrap_or_default();
    let prev = candles.iter().rev().nth(1).map(|c| c.close);

    let mut stats = engine::dashboard_stats(&summary, current, prev);
    stats.candles = candles;
    Ok(stats)
}

/// Load all trades and enrich with running BEP + realized P&L per trade.
pub fn trades(cfg: &AppConfig) -> CoreResult<Vec<EnrichedTrade>> {
    let conn = db::open(&cfg.db_path)?;
    let pair = trading_pair(&cfg.quote);
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
    let conn = db::init(&cfg.db_path)?;
    let candles = db::load_candles(&conn, &cfg.quote)?;
    if candles.is_empty() {
        let bundled = price::load_bundled_prices(prices_dir, &cfg.quote)?;
        db::save_candles(&conn, &cfg.quote, &bundled)?;
    }
    Ok(())
}

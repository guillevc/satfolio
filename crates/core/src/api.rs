use std::path::Path;

use chrono::Utc;

use crate::convert::CrossRateConverter;
use crate::errors::{CoreError, CoreResult};
use crate::models::{
    Asset, AssetPair, DashboardStats, EnrichedTrade, ImportOutcome, ImportPreview, ImportRecord,
    SUPPORTED_FIATS,
};
use crate::{db, engine, hash, parser, price};

/// Construct the BTC/{quote} pair. BTC is always the base asset.
fn trading_pair(quote: &Asset) -> AssetPair {
    AssetPair {
        base: Asset::Btc,
        quote: quote.clone(),
    }
}

/// Build a CrossRateConverter from all close prices in the DB.
fn build_converter(db_path: &Path) -> CoreResult<CrossRateConverter> {
    let conn = db::open(db_path)?;
    let rates = db::load_all_close_prices(&conn)?;
    Ok(CrossRateConverter::new(rates))
}

/// Auto-detect CSV provider, parse trades, and return preview with dedup info.
pub fn preview_import(db_path: &Path, quote: &Asset, path: &Path) -> CoreResult<ImportPreview> {
    let provider = parser::detect_provider(path)?;
    let pair = trading_pair(quote);
    let trades = parser::parse_csv(&provider, path)?;

    let converter = build_converter(db_path)?;
    let normalized: Vec<_> = trades
        .iter()
        .cloned()
        .filter_map(|t| converter.normalize_trade(t, quote))
        .collect();
    let summary = engine::trades_summary(&pair, &normalized)?;

    let file_hash = hash::file_sha256(path)?;
    let conn = db::open(db_path)?;
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
pub fn confirm_import(db_path: &Path, quote: &Asset, path: &Path) -> CoreResult<ImportOutcome> {
    let provider = parser::detect_provider(path)?;
    let pair = trading_pair(quote);
    let trades = parser::parse_csv(&provider, path)?;

    let file_hash = hash::file_sha256(path)?;
    let conn = db::open(db_path)?;

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

    let mut import = db::save_import_with_trades(
        &conn,
        &provider,
        &filename,
        &file_hash,
        &trades,
        &trade_hashes,
        &existing,
    )?;

    // Compute summary from only the newly-inserted trades (excluding duplicates),
    // so trade_count reflects resolved trades actually added by this import.
    let converter = build_converter(db_path)?;
    let new_normalized: Vec<_> = trades
        .iter()
        .zip(trade_hashes.iter())
        .filter(|(_, h)| !existing.contains(*h))
        .filter_map(|(t, _)| converter.normalize_trade(t.clone(), quote))
        .collect();
    let summary = engine::trades_summary(&pair, &new_normalized)?;

    // Update import trade_count to match resolved count (buys + sells).
    let resolved_count = summary.buys + summary.sells;
    if import.trade_count != resolved_count {
        db::update_import_trade_count(&conn, import.id, resolved_count)?;
        import.trade_count = resolved_count;
    }

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
pub fn list_imports(db_path: &Path) -> CoreResult<Vec<ImportRecord>> {
    let conn = db::open(db_path)?;
    Ok(db::list_imports(&conn)?)
}

/// Remove an import and cascade-delete its trades.
pub fn remove_import(db_path: &Path, import_id: i64) -> CoreResult<()> {
    let conn = db::open(db_path)?;
    Ok(db::remove_import(&conn, import_id)?)
}

/// Delete all user data (trades, imports, candles).
pub fn nuke_all_data(db_path: &Path) -> CoreResult<()> {
    let conn = db::open(db_path)?;
    Ok(db::nuke_all_data(&conn)?)
}

/// Build dashboard stats from all trades + candle history. Computes position, BEP, and P&L.
pub fn dashboard_stats(db_path: &Path, quote: &Asset) -> CoreResult<DashboardStats> {
    let conn = db::open(db_path)?;
    let pair = trading_pair(quote);
    let trades = db::load_trades(&conn)?;

    let converter = CrossRateConverter::new(db::load_all_close_prices(&conn)?);
    let normalized: Vec<_> = trades
        .into_iter()
        .filter_map(|t| converter.normalize_trade(t, quote))
        .collect();

    let summary = engine::position_summary(&pair, &normalized)?;

    let candles = db::load_candles(&conn, quote)?;
    let current = candles.last().map(|c| c.close).unwrap_or_default();
    let prev = candles.iter().rev().nth(1).map(|c| c.close);

    let mut stats = engine::dashboard_stats(&summary, current, prev);
    stats.candles = candles;
    Ok(stats)
}

/// Load all trades and enrich with running BEP + realized P&L per trade.
pub fn trades(db_path: &Path, quote: &Asset) -> CoreResult<Vec<EnrichedTrade>> {
    let conn = db::open(db_path)?;
    let pair = trading_pair(quote);
    let trades = db::load_trades(&conn)?;

    let converter = CrossRateConverter::new(db::load_all_close_prices(&conn)?);
    let normalized: Vec<_> = trades
        .into_iter()
        .filter_map(|t| converter.normalize_trade(t, quote))
        .collect();

    Ok(engine::enrich_trades(&pair, normalized)?)
}

/// Gap-fill candles from Kraken API for all supported currencies.
///
/// Opens its own DB connections so no `Connection` is held across the network `.await`.
pub async fn sync_all_candles(db_path: &Path) -> CoreResult<()> {
    for fiat in &SUPPORTED_FIATS {
        let last_date = {
            let conn = db::open(db_path)?;
            let candles = db::load_candles(&conn, fiat)?;
            match candles.last() {
                Some(last) if last.date < Utc::now().date_naive() => last.date,
                _ => continue,
            }
        };

        let new = price::fetch_ohlc(fiat, last_date).await?;

        if !new.is_empty() {
            let conn = db::open(db_path)?;
            db::save_candles(&conn, fiat, &new)?;
        }
    }

    Ok(())
}

/// Open + migrate DB and seed bundled price data for all currencies. Call once at startup.
pub fn init_db(db_path: &Path, prices_dir: &Path) -> CoreResult<()> {
    let conn = db::init(db_path)?;
    for fiat in &SUPPORTED_FIATS {
        let candles = db::load_candles(&conn, fiat)?;
        if candles.is_empty() {
            let bundled = price::load_bundled_prices(prices_dir, fiat)?;
            db::save_candles(&conn, fiat, &bundled)?;
        }
    }
    Ok(())
}

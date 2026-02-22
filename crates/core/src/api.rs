use std::collections::BTreeMap;
use std::path::Path;

use chrono::{NaiveDate, Utc};

use crate::errors::CoreResult;
use crate::models::{Asset, AssetPair, BepSnapshot, Candle, PositionSummary, Trade, TradesSummary};
use crate::{context::Context, db, engine, parser, price};

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

pub fn confirm_import(ctx: &Context, path: &Path) -> CoreResult<TradesSummary> {
    let pair = btc_pair(ctx.quote());
    let trades = parser::parse_kraken_csv(path)?;
    db::save_trades(&ctx.conn, &trades)?;
    let summary = engine::trades_summary(&pair, &trades)?;
    Ok(summary)
}

pub fn position_summary(ctx: &Context) -> CoreResult<PositionSummary> {
    let pair = btc_pair(ctx.quote());
    let trades = db::load_trades(&ctx.conn)?;
    let stats = engine::position_summary(&pair, &trades)?;
    Ok(stats)
}

pub fn bep_snaps(ctx: &Context) -> CoreResult<BTreeMap<NaiveDate, BepSnapshot>> {
    let pair = btc_pair(ctx.quote());
    let trades = db::load_trades(&ctx.conn)?;
    let series = engine::bep_snaps(&pair, &trades)?;
    Ok(series)
}

pub fn trades(ctx: &Context) -> CoreResult<Vec<Trade>> {
    let trades = db::load_trades(&ctx.conn)?;
    Ok(trades)
}

pub fn candles(ctx: &Context, prices_dir: &Path) -> CoreResult<Vec<Candle>> {
    let quote = ctx.quote();
    let mut candles = db::load_candles(&ctx.conn, quote)?;

    if candles.is_empty() {
        candles = price::load_bundled_prices(prices_dir, quote)?;
        db::save_candles(&ctx.conn, quote, &candles)?;
    }

    // Gap-fill: fetch new candles from Kraken if behind today
    if let Some(last) = candles.last() {
        let today = Utc::now().date_naive();
        if last.date < today {
            let new = price::fetch_ohlc(quote, last.date)?;
            if !new.is_empty() {
                db::save_candles(&ctx.conn, quote, &new)?;
                candles = db::load_candles(&ctx.conn, quote)?;
            }
        }
    }

    Ok(candles)
}

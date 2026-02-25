use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

use crate::{
    errors::DbResult,
    models::{Asset, AssetAmount, Candle, Trade},
};

/// Parse a TEXT column value, mapping parse failures to `rusqlite::Error`
/// so they propagate through `DbResult` without panicking.
fn parse_col<T: std::str::FromStr>(val: &str, col: &str) -> Result<T, rusqlite::Error>
where
    T::Err: std::fmt::Display,
{
    val.parse().map_err(|e: T::Err| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::from(format!("{col}: {e}")),
        )
    })
}

pub(crate) fn open(path: &Path) -> DbResult<Connection> {
    let conn = Connection::open(path)?;
    migrate(&conn)?;
    Ok(conn)
}

pub(crate) fn migrate(conn: &Connection) -> DbResult<()> {
    let version: i64 = conn.query_row("SELECT user_version FROM pragma_user_version", [], |r| {
        r.get(0)
    })?;
    if version < 1 {
        conn.execute_batch(
            "
             CREATE TABLE IF NOT EXISTS trades (
                id               INTEGER PRIMARY KEY,
                date             TEXT NOT NULL,
                spent_amount     TEXT NOT NULL,
                spent_asset      TEXT NOT NULL,
                received_amount  TEXT NOT NULL,
                received_asset   TEXT NOT NULL,
                fee_amount       TEXT NOT NULL,
                fee_asset        TEXT NOT NULL
            );
            PRAGMA user_version = 1;
        ",
        )?;
    }
    if version < 2 {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS candles (
                id      INTEGER PRIMARY KEY,
                quote   TEXT NOT NULL,
                date    TEXT NOT NULL,
                open    TEXT NOT NULL,
                high    TEXT NOT NULL,
                low     TEXT NOT NULL,
                close   TEXT NOT NULL,
                volume  TEXT NOT NULL,
                count   INTEGER NOT NULL,
                UNIQUE(quote, date)
            );
            PRAGMA user_version = 2;
        ",
        )?;
    }
    Ok(())
}

pub(crate) fn save_trades(conn: &Connection, trades: &[Trade]) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    let mut stmt = tx.prepare_cached("\
        INSERT INTO trades (date, spent_amount, spent_asset, received_amount, received_asset, fee_amount, fee_asset) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)\
    ")?;
    for trade in trades {
        stmt.execute(params![
            trade.date.to_rfc3339(),
            trade.spent.amount().to_string(),
            trade.spent.asset().as_str(),
            trade.received.amount().to_string(),
            trade.received.asset().as_str(),
            trade.fee.amount().to_string(),
            trade.fee.asset().as_str(),
        ])?;
    }
    drop(stmt);
    tx.commit()?;
    Ok(())
}

pub(crate) fn load_trades(conn: &Connection) -> DbResult<Vec<Trade>> {
    let mut stmt = conn.prepare("\
        SELECT date, spent_amount, spent_asset, received_amount, received_asset, fee_amount, fee_asset \
        FROM trades ORDER BY date ASC\
    ")?;
    let rows = stmt.query_map([], |row| {
        let date = row.get::<_, String>(0)?;
        let spent_amount = row.get::<_, String>(1)?;
        let spent_asset = row.get::<_, String>(2)?;
        let received_amount = row.get::<_, String>(3)?;
        let received_asset = row.get::<_, String>(4)?;
        let fee_amount = row.get::<_, String>(5)?;
        let fee_asset: String = row.get(6)?;
        Ok(Trade {
            date: parse_col::<DateTime<chrono::FixedOffset>>(&date, "trade.date")?
                .with_timezone(&Utc),
            spent: AssetAmount::new(
                parse_col(&spent_amount, "trade.spent_amount")?,
                Asset::from(spent_asset),
            ),
            received: AssetAmount::new(
                parse_col(&received_amount, "trade.received_amount")?,
                Asset::from(received_asset),
            ),
            fee: AssetAmount::new(
                parse_col(&fee_amount, "trade.fee_amount")?,
                Asset::from(fee_asset),
            ),
        })
    })?;
    let trades = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(trades)
}

pub(crate) fn save_candles(conn: &Connection, quote: &Asset, candles: &[Candle]) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    let mut stmt = tx.prepare_cached(
        "INSERT OR REPLACE INTO candles (quote, date, open, high, low, close, volume, count) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;
    let quote_str = quote.as_str();
    for c in candles {
        stmt.execute(params![
            quote_str,
            c.date.to_string(),
            c.open.to_string(),
            c.high.to_string(),
            c.low.to_string(),
            c.close.to_string(),
            c.volume.to_string(),
            c.count,
        ])?;
    }
    drop(stmt);
    tx.commit()?;
    Ok(())
}

pub(crate) fn load_candles(conn: &Connection, quote: &Asset) -> DbResult<Vec<Candle>> {
    let mut stmt = conn.prepare(
        "SELECT date, open, high, low, close, volume, count \
         FROM candles WHERE quote = ?1 ORDER BY date ASC",
    )?;
    let rows = stmt.query_map(params![quote.as_str()], |row| {
        let date: String = row.get(0)?;
        let open: String = row.get(1)?;
        let high: String = row.get(2)?;
        let low: String = row.get(3)?;
        let close: String = row.get(4)?;
        let volume: String = row.get(5)?;
        let count: u32 = row.get(6)?;
        Ok(Candle {
            date: parse_col(&date, "candle.date")?,
            open: parse_col(&open, "candle.open")?,
            high: parse_col(&high, "candle.high")?,
            low: parse_col(&low, "candle.low")?,
            close: parse_col(&close, "candle.close")?,
            volume: parse_col(&volume, "candle.volume")?,
            count,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    fn sample_trade(year: i32, month: u32, day: u32) -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(187.2514), Asset::Eur),
            received: AssetAmount::new(dec!(0.0020104289), Asset::Btc),
            fee: AssetAmount::new(dec!(0.749), Asset::Eur),
        }
    }

    #[test]
    fn migrate_creates_table() {
        let conn = test_conn();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM trades", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = test_conn();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
    }

    #[test]
    fn save_and_load_roundtrip() {
        let conn = test_conn();
        let trades = vec![sample_trade(2024, 1, 15), sample_trade(2024, 3, 20)];
        save_trades(&conn, &trades).unwrap();
        let loaded = load_trades(&conn).unwrap();
        assert_eq!(loaded, trades);
    }

    #[test]
    fn load_empty() {
        let conn = test_conn();
        let trades = load_trades(&conn).unwrap();
        assert!(trades.is_empty());
    }

    #[test]
    fn save_empty() {
        let conn = test_conn();
        save_trades(&conn, &[]).unwrap();
        let trades = load_trades(&conn).unwrap();
        assert!(trades.is_empty());
    }

    // ── Candles ──────────────────────────────────────────

    fn sample_candle(year: i32, month: u32, day: u32, close: Decimal) -> Candle {
        Candle {
            date: NaiveDate::from_ymd_opt(year, month, day).unwrap(),
            open: close,
            high: close,
            low: close,
            close,
            volume: dec!(1.0),
            count: 1,
        }
    }

    #[test]
    fn candles_save_and_load_roundtrip() {
        let conn = test_conn();
        let candles = vec![
            sample_candle(2024, 1, 1, dec!(42000)),
            sample_candle(2024, 1, 2, dec!(43000)),
        ];
        save_candles(&conn, &Asset::Eur, &candles).unwrap();
        let loaded = load_candles(&conn, &Asset::Eur).unwrap();
        assert_eq!(loaded, candles);
    }

    #[test]
    fn candles_filtered_by_quote() {
        let conn = test_conn();
        save_candles(
            &conn,
            &Asset::Eur,
            &[sample_candle(2024, 1, 1, dec!(42000))],
        )
        .unwrap();
        save_candles(
            &conn,
            &Asset::Usd,
            &[sample_candle(2024, 1, 1, dec!(45000))],
        )
        .unwrap();
        let eur = load_candles(&conn, &Asset::Eur).unwrap();
        let usd = load_candles(&conn, &Asset::Usd).unwrap();
        assert_eq!(eur.len(), 1);
        assert_eq!(usd.len(), 1);
        assert_eq!(eur[0].close, dec!(42000));
        assert_eq!(usd[0].close, dec!(45000));
    }

    #[test]
    fn candles_upsert_on_duplicate() {
        let conn = test_conn();
        save_candles(
            &conn,
            &Asset::Eur,
            &[sample_candle(2024, 1, 1, dec!(42000))],
        )
        .unwrap();
        save_candles(
            &conn,
            &Asset::Eur,
            &[sample_candle(2024, 1, 1, dec!(43000))],
        )
        .unwrap();
        let loaded = load_candles(&conn, &Asset::Eur).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].close, dec!(43000));
    }

    #[test]
    fn candles_load_empty() {
        let conn = test_conn();
        let candles = load_candles(&conn, &Asset::Eur).unwrap();
        assert!(candles.is_empty());
    }

    // ── Trades ──────────────────────────────────────────

    #[test]
    fn load_preserves_chronological_order() {
        let conn = test_conn();
        let trades = vec![
            sample_trade(2024, 12, 1),
            sample_trade(2024, 1, 1),
            sample_trade(2024, 6, 15),
        ];
        save_trades(&conn, &trades).unwrap();
        let loaded = load_trades(&conn).unwrap();
        assert_eq!(
            loaded[0].date,
            Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap()
        );
        assert_eq!(
            loaded[1].date,
            Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap()
        );
        assert_eq!(
            loaded[2].date,
            Utc.with_ymd_and_hms(2024, 12, 1, 12, 0, 0).unwrap()
        );
    }
}

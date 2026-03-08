use std::collections::HashSet;
use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

use crate::{
    errors::DbResult,
    models::{Asset, AssetAmount, Candle, ImportRecord, Provider, Trade},
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

/// Open + migrate. Call once at startup.
pub(crate) fn init(path: &Path) -> DbResult<Connection> {
    let conn = Connection::open(path)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Open without migrating. For all post-init usage.
pub(crate) fn open(path: &Path) -> DbResult<Connection> {
    Ok(Connection::open(path)?)
}

pub(crate) fn migrate(conn: &Connection) -> DbResult<()> {
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
            fee_asset        TEXT NOT NULL,
            import_id        INTEGER,
            trade_hash       TEXT
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_trades_hash ON trades(trade_hash);

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

        CREATE TABLE IF NOT EXISTS imports (
            id          INTEGER PRIMARY KEY,
            provider    TEXT NOT NULL,
            filename    TEXT NOT NULL,
            file_hash   TEXT NOT NULL,
            trade_count INTEGER NOT NULL DEFAULT 0,
            date_from   TEXT,
            date_to     TEXT,
            imported_at TEXT NOT NULL
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_imports_file_hash ON imports(file_hash);
        ",
    )?;
    Ok(())
}

#[cfg(test)]
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

/// Load all existing trade hashes for O(1) dedup lookup.
pub(crate) fn existing_trade_hashes(conn: &Connection) -> DbResult<HashSet<String>> {
    let mut stmt = conn.prepare("SELECT trade_hash FROM trades WHERE trade_hash IS NOT NULL")?;
    let hashes = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<HashSet<_>, _>>()?;
    Ok(hashes)
}

/// Find an import record by file hash.
pub(crate) fn find_import_by_hash(
    conn: &Connection,
    file_hash: &str,
) -> DbResult<Option<ImportRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider, filename, file_hash, trade_count, date_from, date_to, imported_at \
         FROM imports WHERE file_hash = ?1",
    )?;
    let mut rows = stmt.query_map(params![file_hash], |row| {
        Ok(ImportRecord {
            id: row.get(0)?,
            provider: parse_col::<Provider>(&row.get::<_, String>(1)?, "imports.provider")?,
            filename: row.get(2)?,
            file_hash: row.get(3)?,
            trade_count: row.get::<_, i64>(4)? as usize,
            date_from: row
                .get::<_, Option<String>>(5)?
                .map(|s| parse_col::<DateTime<chrono::FixedOffset>>(&s, "imports.date_from"))
                .transpose()?
                .map(|d| d.with_timezone(&Utc)),
            date_to: row
                .get::<_, Option<String>>(6)?
                .map(|s| parse_col::<DateTime<chrono::FixedOffset>>(&s, "imports.date_to"))
                .transpose()?
                .map(|d| d.with_timezone(&Utc)),
            imported_at: {
                let s: String = row.get(7)?;
                parse_col::<DateTime<chrono::FixedOffset>>(&s, "imports.imported_at")?
                    .with_timezone(&Utc)
            },
        })
    })?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

/// Insert an import record and its new trades in a single transaction.
/// Skips trades whose hash is in `existing_hashes`. Returns the created ImportRecord.
pub(crate) fn save_import_with_trades(
    conn: &Connection,
    provider: &Provider,
    filename: &str,
    file_hash: &str,
    trades: &[Trade],
    trade_hashes: &[String],
    existing_hashes: &HashSet<String>,
) -> DbResult<ImportRecord> {
    let tx = conn.unchecked_transaction()?;
    let now = Utc::now();

    tx.execute(
        "INSERT INTO imports (provider, filename, file_hash, trade_count, imported_at) VALUES (?1, ?2, ?3, 0, ?4)",
        params![provider.as_str(), filename, file_hash, now.to_rfc3339()],
    )?;
    let import_id = tx.last_insert_rowid();

    let mut stmt = tx.prepare_cached(
        "INSERT INTO trades (date, spent_amount, spent_asset, received_amount, received_asset, \
         fee_amount, fee_asset, import_id, trade_hash) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;

    let mut inserted = 0usize;
    let mut date_from: Option<DateTime<Utc>> = None;
    let mut date_to: Option<DateTime<Utc>> = None;

    for (trade, th) in trades.iter().zip(trade_hashes.iter()) {
        if existing_hashes.contains(th) {
            continue;
        }
        stmt.execute(params![
            trade.date.to_rfc3339(),
            trade.spent.amount().to_string(),
            trade.spent.asset().as_str(),
            trade.received.amount().to_string(),
            trade.received.asset().as_str(),
            trade.fee.amount().to_string(),
            trade.fee.asset().as_str(),
            import_id,
            th,
        ])?;
        inserted += 1;
        date_from = Some(date_from.map_or(trade.date, |d: DateTime<Utc>| d.min(trade.date)));
        date_to = Some(date_to.map_or(trade.date, |d: DateTime<Utc>| d.max(trade.date)));
    }
    drop(stmt);

    tx.execute(
        "UPDATE imports SET trade_count = ?1, date_from = ?2, date_to = ?3 WHERE id = ?4",
        params![
            inserted as i64,
            date_from.map(|d| d.to_rfc3339()),
            date_to.map(|d| d.to_rfc3339()),
            import_id,
        ],
    )?;

    tx.commit()?;

    Ok(ImportRecord {
        id: import_id,
        provider: *provider,
        filename: filename.to_string(),
        file_hash: file_hash.to_string(),
        trade_count: inserted,
        date_from,
        date_to,
        imported_at: now,
    })
}

/// List all imports ordered by most recent first.
pub(crate) fn list_imports(conn: &Connection) -> DbResult<Vec<ImportRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider, filename, file_hash, trade_count, date_from, date_to, imported_at \
         FROM imports ORDER BY imported_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ImportRecord {
            id: row.get(0)?,
            provider: parse_col::<Provider>(&row.get::<_, String>(1)?, "imports.provider")?,
            filename: row.get(2)?,
            file_hash: row.get(3)?,
            trade_count: row.get::<_, i64>(4)? as usize,
            date_from: row
                .get::<_, Option<String>>(5)?
                .map(|s| parse_col::<DateTime<chrono::FixedOffset>>(&s, "imports.date_from"))
                .transpose()?
                .map(|d| d.with_timezone(&Utc)),
            date_to: row
                .get::<_, Option<String>>(6)?
                .map(|s| parse_col::<DateTime<chrono::FixedOffset>>(&s, "imports.date_to"))
                .transpose()?
                .map(|d| d.with_timezone(&Utc)),
            imported_at: {
                let s: String = row.get(7)?;
                parse_col::<DateTime<chrono::FixedOffset>>(&s, "imports.imported_at")?
                    .with_timezone(&Utc)
            },
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Remove an import and cascade-delete its trades.
pub(crate) fn remove_import(conn: &Connection, import_id: i64) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM trades WHERE import_id = ?1",
        params![import_id],
    )?;
    tx.execute("DELETE FROM imports WHERE id = ?1", params![import_id])?;
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
    use crate::hash;
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

    // ── Imports ─────────────────────────────────────────

    fn make_hashes(trades: &[Trade]) -> Vec<String> {
        trades
            .iter()
            .map(|t| hash::trade_hash(Provider::Kraken.as_str(), t))
            .collect()
    }

    #[test]
    fn save_import_inserts_all_trades() {
        let conn = test_conn();
        let trades = vec![sample_trade(2024, 1, 15), sample_trade(2024, 3, 20)];
        let hashes = make_hashes(&trades);
        let existing = HashSet::new();
        let rec = save_import_with_trades(
            &conn,
            &Provider::Kraken,
            "test.csv",
            "abc123",
            &trades,
            &hashes,
            &existing,
        )
        .unwrap();
        assert_eq!(rec.trade_count, 2);
        assert_eq!(rec.filename, "test.csv");
        assert_eq!(rec.provider, Provider::Kraken);
        assert!(rec.date_from.is_some());
        assert!(rec.date_to.is_some());
        let loaded = load_trades(&conn).unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn save_import_skips_duplicates() {
        let conn = test_conn();
        let trades = vec![sample_trade(2024, 1, 15), sample_trade(2024, 3, 20)];
        let hashes = make_hashes(&trades);
        // Mark the first trade as already existing
        let mut existing = HashSet::new();
        existing.insert(hashes[0].clone());
        let rec = save_import_with_trades(
            &conn,
            &Provider::Kraken,
            "test.csv",
            "abc123",
            &trades,
            &hashes,
            &existing,
        )
        .unwrap();
        assert_eq!(rec.trade_count, 1);
        let loaded = load_trades(&conn).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0], trades[1]);
    }

    #[test]
    fn list_imports_roundtrip() {
        let conn = test_conn();
        let trades = vec![sample_trade(2024, 1, 15)];
        let hashes = make_hashes(&trades);
        save_import_with_trades(
            &conn,
            &Provider::Kraken,
            "a.csv",
            "hash_a",
            &trades,
            &hashes,
            &HashSet::new(),
        )
        .unwrap();
        save_import_with_trades(
            &conn,
            &Provider::Kraken,
            "b.csv",
            "hash_b",
            &[],
            &[],
            &HashSet::new(),
        )
        .unwrap();
        let imports = list_imports(&conn).unwrap();
        assert_eq!(imports.len(), 2);
        // Most recent first
        assert_eq!(imports[0].filename, "b.csv");
        assert_eq!(imports[1].filename, "a.csv");
    }

    #[test]
    fn remove_import_cascades_trades() {
        let conn = test_conn();
        let trades = vec![sample_trade(2024, 1, 15), sample_trade(2024, 3, 20)];
        let hashes = make_hashes(&trades);
        let rec = save_import_with_trades(
            &conn,
            &Provider::Kraken,
            "test.csv",
            "abc123",
            &trades,
            &hashes,
            &HashSet::new(),
        )
        .unwrap();
        assert_eq!(load_trades(&conn).unwrap().len(), 2);

        remove_import(&conn, rec.id).unwrap();
        assert_eq!(load_trades(&conn).unwrap().len(), 0);
        assert!(list_imports(&conn).unwrap().is_empty());
    }

    #[test]
    fn find_import_by_hash_found() {
        let conn = test_conn();
        save_import_with_trades(
            &conn,
            &Provider::Kraken,
            "test.csv",
            "abc123",
            &[],
            &[],
            &HashSet::new(),
        )
        .unwrap();
        assert!(find_import_by_hash(&conn, "abc123").unwrap().is_some());
    }

    #[test]
    fn find_import_by_hash_not_found() {
        let conn = test_conn();
        assert!(find_import_by_hash(&conn, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn existing_trade_hashes_returns_all() {
        let conn = test_conn();
        let trades = vec![sample_trade(2024, 1, 15), sample_trade(2024, 3, 20)];
        let hashes = make_hashes(&trades);
        save_import_with_trades(
            &conn,
            &Provider::Kraken,
            "test.csv",
            "abc123",
            &trades,
            &hashes,
            &HashSet::new(),
        )
        .unwrap();
        let existing = existing_trade_hashes(&conn).unwrap();
        assert_eq!(existing.len(), 2);
        assert!(existing.contains(&hashes[0]));
        assert!(existing.contains(&hashes[1]));
    }
}

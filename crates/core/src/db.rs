use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use rust_decimal::Decimal;

use crate::{
    errors::DbResult,
    models::{Asset, AssetAmount, Trade},
};

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
    // if version < 2 {
    //     conn.execute_batch("
    //       ALTER TABLE trades ADD COLUMN todo TEXT;
    //       PRAGMA user_version = 2;
    //   ")?;
    // }
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
            date: DateTime::parse_from_rfc3339(&date)
                .unwrap()
                .with_timezone(&Utc),
            spent: AssetAmount::new(
                Decimal::from_str_exact(&spent_amount).unwrap(),
                Asset::from(spent_asset),
            ),
            received: AssetAmount::new(
                Decimal::from_str_exact(&received_amount).unwrap(),
                Asset::from(received_asset),
            ),
            fee: AssetAmount::new(
                Decimal::from_str_exact(&fee_amount).unwrap(),
                Asset::from(fee_asset),
            ),
        })
    })?;
    let trades = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(trades)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
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
        let trades = vec![
            sample_trade(2024, 1, 15),
            sample_trade(2024, 3, 20),
        ];
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
        assert_eq!(loaded[0].date, Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap());
        assert_eq!(loaded[1].date, Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap());
        assert_eq!(loaded[2].date, Utc.with_ymd_and_hms(2024, 12, 1, 12, 0, 0).unwrap());
    }
}

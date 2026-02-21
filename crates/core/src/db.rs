use std::path::Path;

use rusqlite::Connection;

use crate::{errors::DbResult, models::Trade};

pub(crate) fn open(path: &Path) -> DbResult<Connection> {
    let conn = Connection::open(path)?;
    migrate(&conn)?;
    Ok(conn)
}

pub(crate) fn save_trades(conn: &Connection, trades: &[Trade]) -> DbResult<()> {
    todo!()
}

pub(crate) fn load_trades(conn: &Connection) -> DbResult<Vec<Trade>> {
    todo!()
}

pub(crate) fn migrate(conn: &Connection) -> DbResult<()> {
    // let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    // if version < 1 {
    //     conn.execute_batch("CREATE TABLE IF NOT EXISTS trades ();")?;
    // }
    // Ok(())
    todo!()
}

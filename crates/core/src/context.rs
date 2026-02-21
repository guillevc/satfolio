use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::errors::CoreResult;

pub struct Context {
    pub(crate) conn: Connection,
}

impl Context {
    pub fn open(path: &Path) -> CoreResult<Self> {
        let conn = db::open(path)?;
        Ok(Self { conn })
    }
}

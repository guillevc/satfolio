use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::errors::CoreResult;
use crate::models::Asset;

pub struct Context {
    pub(crate) conn: Connection,
    pub(crate) quote: Asset,
}

impl Context {
    pub fn open(path: &Path, quote: Asset) -> CoreResult<Self> {
        let conn = db::open(path)?;
        Ok(Self { conn, quote })
    }

    pub fn quote(&self) -> &Asset {
        &self.quote
    }
}

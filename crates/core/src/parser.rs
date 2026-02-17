use std::fmt;
use std::path::Path;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

mod datetime_format {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(s, FORMAT)
            .map(|n| n.and_utc())
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Debug)]
pub struct LedgerEntry {
    pub txid: String,
    pub refid: String,
    #[serde(with = "datetime_format")]
    pub time: DateTime<Utc>,
    #[serde(rename = "type")]
    pub type_: String,
    pub subtype: String,
    pub aclass: String,
    pub subclass: String,
    pub asset: String,
    pub wallet: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub balance: Decimal,
}

impl fmt::Display for LedgerEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // {:<20}  → left-align, pad to 20 chars
        // {:>+18.10} → right-align, pad to 18 chars, show +/- sign, 10 decimal places
        write!(
            f,
            "{:<10.10} | {:<10.10} | {:<20} | {:<10.10} | {:<10.10} | {:<5} | {:>+15.10} | {:.10} | {:<15}",
            self.txid,
            self.refid,
            self.time,
            self.type_,
            self.subtype,
            self.asset,
            self.amount,
            self.fee,
            self.wallet,
        )?;

        Ok(())
    }
}

pub fn parse_csv(path: &Path) -> Result<Vec<LedgerEntry>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut entries = Vec::new();

    for result in reader.deserialize() {
        let entry: LedgerEntry = result?;
        entries.push(entry);
    }

    Ok(entries)
}

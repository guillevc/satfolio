use std::fmt;
use std::path::Path;

use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

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
    pub time: String,
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
        let type_label = if self.subtype.is_empty() {
            self.type_.clone()
        } else {
            format!("{}/{}", self.type_, self.subtype)
        };

        // {:<20}  → left-align, pad to 20 chars
        // {:>+18.10} → right-align, pad to 18 chars, show +/- sign, 10 decimal places
        write!(
            f,
            "{} | {:<20} | {:<5} | {:>+18.10} | {:.10} | {:<15}",
            self.time, type_label, self.asset, self.amount, self.fee, self.wallet,
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

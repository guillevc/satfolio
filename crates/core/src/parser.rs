use std::fmt;
use std::path::Path;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize};

use super::error::Result;

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

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Deposit,
    Trade,
    Withdrawal,
    Earn,
    Spend,
    Receive
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            EntryType::Deposit => "deposit",
            EntryType::Trade => "trade",
            EntryType::Withdrawal => "withdrawal",
            EntryType::Earn => "earn",
            EntryType::Spend => "spend",
            EntryType::Receive => "receive",
        })
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(from = "String")]
pub enum Asset {
    Btc,
    Eur,
    Other(String)
}

impl From<String> for Asset {
    fn from(s: String) -> Self {
        match s.as_str() {
            "BTC" | "XBT" => Self::Btc,
            "EUR" => Self::Eur,
            _ => Self::Other(s)
        }
    }
}

impl Asset {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Btc => "BTC",
            Self::Eur => "EUR",
            Self::Other(o) => o
        }
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.to_str())
    }
}

#[derive(Deserialize, Debug)]
pub struct LedgerEntry {
    pub txid: String,
    pub refid: String,
    #[serde(with = "datetime_format")]
    pub time: DateTime<Utc>,
    #[serde(rename = "type")]
    pub type_: EntryType,
    pub subtype: String,
    pub aclass: String,
    pub subclass: String,
    pub asset: Asset,
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

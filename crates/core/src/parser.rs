use std::fmt;
use std::path::Path;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

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
    Receive,
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
    Other(String),
}

impl From<String> for Asset {
    fn from(s: String) -> Self {
        match s.as_str() {
            "BTC" | "XBT" => Self::Btc,
            "EUR" => Self::Eur,
            _ => Self::Other(s),
        }
    }
}

impl Asset {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Btc => "BTC",
            Self::Eur => "EUR",
            Self::Other(o) => o,
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::io::Write;

    // ── Helper ───────────────────────────────────────────────────
    // Writes CSV content to a temp file and returns the path.
    // Use this in parse_csv tests so you don't need external fixtures.
    //
    // Example usage:
    //   let path = csv_tempfile("txid,refid,...\nval1,val2,...\n");
    //   let entries = parse_csv(&path).unwrap();
    //   std::fs::remove_file(&path).ok(); // cleanup
    fn csv_tempfile(content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "betc_test_{}.csv",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    // The CSV header that parse_csv expects (must match LedgerEntry fields).
    // Reuse this constant in all parse_csv tests.
    const CSV_HEADER: &str =
        "txid,refid,time,type,subtype,aclass,subclass,asset,wallet,amount,fee,balance";

    // ── Asset conversion tests ───────────────────────────────────
    // Asset implements From<String> with special cases:
    //   "BTC" | "XBT" → Asset::Btc
    //   "EUR"          → Asset::Eur
    //   anything else  → Asset::Other(s)
    //
    // Test each branch. Asset derives PartialEq so assert_eq! works.
    // Hint: Asset::from("BTC".to_string())

    #[test]
    fn asset_from_btc() {
        todo!("assert Asset::from(\"BTC\".to_string()) == Asset::Btc")
    }

    #[test]
    fn asset_from_xbt_is_btc() {
        // Kraken historically used "XBT" for Bitcoin — both should map to Btc
        todo!("assert Asset::from(\"XBT\".to_string()) == Asset::Btc")
    }

    #[test]
    fn asset_from_eur() {
        todo!("assert Asset::from(\"EUR\".to_string()) == Asset::Eur")
    }

    #[test]
    fn asset_from_unknown() {
        // Any unrecognized ticker falls into Other(String)
        todo!("assert Asset::from(\"ETH\".to_string()) == Asset::Other(\"ETH\".to_string())")
    }

    // ── Asset::to_str roundtrip ──────────────────────────────────
    // to_str() should return the canonical ticker string.
    // For Other, it returns the original string.

    #[test]
    fn asset_to_str_roundtrip() {
        todo!("assert Asset::Btc.to_str() == \"BTC\", etc.")
    }

    // ── EntryType Display ────────────────────────────────────────
    // Each variant should display as its lowercase name.
    // Hint: format!("{}", EntryType::Trade) == "trade"
    // Note: Display uses f.pad(), so format!("{:<10}", val) pads —
    //       use .to_string() or format!("{}") for exact match.

    #[test]
    fn entry_type_display() {
        todo!("assert each EntryType variant displays as lowercase")
    }

    // ── parse_csv happy path ─────────────────────────────────────
    // Write a minimal CSV with one valid row and parse it.
    //
    // Sample row (from real Kraken data):
    //   txid:    "L3M4N5"
    //   refid:   "MECOSFO-GY"
    //   time:    "2024-01-15 12:00:00"   (format: %Y-%m-%d %H:%M:%S)
    //   type:    "trade"
    //   subtype: ""
    //   aclass:  "currency"
    //   subclass: ""
    //   asset:   "EUR"
    //   wallet:  "spot"
    //   amount:  "-187.2514"
    //   fee:     "0.749"
    //   balance: "1000.00"
    //
    // After parsing, assert:
    //   - entries.len() == 1
    //   - entry.asset == Asset::Eur
    //   - entry.amount == dec!(-187.2514)
    //   - entry.type_ == EntryType::Trade
    //
    // Note: LedgerEntry doesn't derive PartialEq, so compare fields individually.

    #[test]
    fn parse_csv_single_row() {
        todo!("write CSV with one row via csv_tempfile, parse, assert fields")
    }

    // ── parse_csv with multiple rows ─────────────────────────────
    // Write 2-3 rows and verify the Vec has the right length
    // and each entry has the expected asset/type.

    #[test]
    fn parse_csv_multiple_rows() {
        todo!("write CSV with multiple rows, assert len and key fields")
    }

    // ── parse_csv error on bad data ──────────────────────────────
    // A malformed row (e.g. missing columns, bad date format)
    // should return Err. Use assert!(result.is_err()).
    //
    // Try: a row with "not-a-date" in the time column.

    #[test]
    fn parse_csv_bad_date_returns_error() {
        todo!("write CSV with bad date, assert parse_csv returns Err")
    }

    // ── parse_csv empty file ─────────────────────────────────────
    // A CSV with only the header (no data rows) should return Ok
    // with an empty Vec.

    #[test]
    fn parse_csv_empty_returns_empty_vec() {
        todo!("write header-only CSV, assert Ok(vec) where vec.is_empty()")
    }
}


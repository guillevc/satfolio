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
    use std::io::Write;

    use rust_decimal_macros::dec;

    use super::{Asset, EntryType, parse_csv};

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

    #[test]
    fn asset_from_btc() {
        assert_eq!(Asset::from("BTC".to_string()), Asset::Btc)
    }

    #[test]
    fn asset_from_xbt_is_btc() {
        assert_eq!(Asset::from("XBT".to_string()), Asset::Btc)
    }

    #[test]
    fn asset_from_eur() {
        assert_eq!(Asset::from("EUR".to_string()), Asset::Eur)
    }

    #[test]
    fn asset_from_unknown() {
        assert_eq!(
            Asset::from("MSC".to_string()),
            Asset::Other("MSC".to_string())
        )
    }

    #[test]
    fn asset_to_str_roundtrip() {
        assert_eq!(Asset::Btc.to_str(), "BTC");
        assert_eq!(Asset::Other("MSC".to_string()).to_str(), "MSC");
    }

    #[test]
    fn entry_type_display() {
        assert_eq!(format!("{}", EntryType::Deposit), "deposit");
        assert_eq!(format!("{}", EntryType::Withdrawal), "withdrawal");
    }

    #[test]
    fn parse_csv_single_row() {
        let csv = format!(
            "{CSV_HEADER}\n\
            L3M4N5,MECOSFO-GY,2024-01-15 12:00:00,trade,,currency,,EUR,spot,-187.2514,0.749,1000.00"
        );
        let path = csv_tempfile(&csv);

        let entries = parse_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert_eq!(entry.asset, Asset::Eur);
        assert_eq!(entry.amount, dec!(-187.2514));
        assert_eq!(entry.type_, EntryType::Trade);
    }

    #[test]
    fn parse_csv_multiple_rows() {
        let csv = format!(
            "{CSV_HEADER}\n\
            TX1,REF-A,2024-01-15 12:00:00,trade,,currency,,EUR,spot,-187.2514,0.749,1000.00\n\
            TX2,REF-A,2024-01-15 12:00:00,trade,,currency,,BTC,spot,0.002,0,0.002\n\
            TX3,REF-B,2024-02-01 09:30:00,deposit,,currency,,EUR,spot,500.00,0,1500.00"
        );
        let path = csv_tempfile(&csv);
        let entries = parse_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].asset, Asset::Eur);
        assert_eq!(entries[1].asset, Asset::Btc);
        assert_eq!(entries[2].type_, EntryType::Deposit);
    }

    #[test]
    fn parse_csv_bad_date_returns_error() {
        let csv = format!(
            "{CSV_HEADER}\n\
            TX1,REF-A,not-a-date,trade,,currency,,EUR,spot,-100,0.5,900"
        );
        let path = csv_tempfile(&csv);
        let result = parse_csv(&path);
        std::fs::remove_file(&path).ok();

        assert!(result.is_err());
    }

    #[test]
    fn parse_csv_empty_returns_empty_vec() {
        let path = csv_tempfile(CSV_HEADER);
        let entries = parse_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert!(entries.is_empty());
    }
}

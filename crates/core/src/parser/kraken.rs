use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::errors::ParseResult;
use crate::models::{Asset, AssetAmount, Provider, Trade};

/// Expected Kraken ledger CSV columns (12 fields).
const KRAKEN_HEADERS: &[&str] = &[
    "txid", "refid", "time", "type", "subtype", "aclass", "subclass", "asset", "wallet", "amount",
    "fee", "balance",
];

/// Returns true if the header row matches a Kraken ledger export.
pub(super) fn matches_headers(headers: &[&str]) -> bool {
    headers == KRAKEN_HEADERS
}

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
enum EntryType {
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

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct LedgerEntry {
    txid: String,
    refid: String,
    #[serde(with = "datetime_format")]
    time: DateTime<Utc>,
    #[serde(rename = "type")]
    type_: EntryType,
    subtype: String,
    aclass: String,
    subclass: String,
    asset: Asset,
    wallet: String,
    amount: Decimal,
    fee: Decimal,
    balance: Decimal,
}

impl fmt::Display for LedgerEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        )
    }
}

fn parse_csv_entries(path: &Path) -> ParseResult<Vec<LedgerEntry>> {
    let mut reader = csv::Reader::from_path(path)?;
    Ok(reader
        .deserialize()
        .collect::<Result<Vec<_>, csv::Error>>()?)
}

fn find_trades(entries: &[LedgerEntry]) -> Vec<Trade> {
    let mut by_refid = HashMap::<&str, Vec<&LedgerEntry>>::new();

    for entry in entries {
        by_refid.entry(&entry.refid).or_default().push(entry);
    }

    by_refid
        .into_iter()
        .filter_map(|(_, entries)| {
            let [left, right] = *entries.as_slice() else {
                return None;
            };
            match (&left.type_, &right.type_) {
                (EntryType::Trade, EntryType::Trade)
                | (EntryType::Spend, EntryType::Receive)
                | (EntryType::Receive, EntryType::Spend) => Some((left, right)),
                _ => None,
            }
        })
        .map(|(left, right)| {
            let (buy, sell) = if left.amount.is_sign_positive() {
                (left, right)
            } else {
                (right, left)
            };
            // Kraken places the fee on either entry — pick whichever has a
            // non-zero fee (first non-zero wins, matching BittyTax behaviour).
            let (fee_amount, fee_asset) = if !sell.fee.is_zero() {
                (sell.fee.abs(), sell.asset.clone())
            } else {
                (buy.fee.abs(), buy.asset.clone())
            };
            Trade {
                date: buy.time,
                spent: AssetAmount::new(sell.amount.abs(), sell.asset.clone()),
                received: AssetAmount::new(buy.amount.abs(), buy.asset.clone()),
                fee: AssetAmount::new(fee_amount, fee_asset),
                provider: Provider::Kraken,
            }
        })
        .collect()
}

pub(super) fn parse(path: &Path) -> ParseResult<Vec<Trade>> {
    let entries = parse_csv_entries(path)?;
    let mut trades = find_trades(&entries);
    trades.sort_by_key(|t| t.date);
    Ok(trades)
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    mod csv {
        use super::*;
        use std::io::Write;
        use tempfile::NamedTempFile;

        fn csv_tempfile(content: &str) -> NamedTempFile {
            let mut f = NamedTempFile::with_suffix(".csv").unwrap();
            f.write_all(content.as_bytes()).unwrap();
            f
        }

        const CSV_HEADER: &str =
            "txid,refid,time,type,subtype,aclass,subclass,asset,wallet,amount,fee,balance";

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
            let f = csv_tempfile(&csv);
            let entries = parse_csv_entries(f.path()).unwrap();
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
            let f = csv_tempfile(&csv);
            let entries = parse_csv_entries(f.path()).unwrap();
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
            let f = csv_tempfile(&csv);
            assert!(parse_csv_entries(f.path()).is_err());
        }

        #[test]
        fn parse_csv_empty_returns_empty_vec() {
            let f = csv_tempfile(CSV_HEADER);
            let entries = parse_csv_entries(f.path()).unwrap();
            assert!(entries.is_empty());
        }
    }

    mod trades {
        use super::*;

        #[test]
        fn trade_trade_pair() {
            let spend = make_entry(
                "MECOSFO-GY",
                EntryType::Trade,
                Asset::Eur,
                dec!(-187.2514),
                dec!(0.749),
            );
            let receive = make_entry(
                "MECOSFO-GY",
                EntryType::Trade,
                Asset::Btc,
                dec!(0.0020104289),
                Decimal::ZERO,
            );
            let result = find_trades(&[spend, receive]);
            assert_eq!(result.len(), 1);
            let trade = result.first().unwrap();
            assert_eq!(trade.spent.amount(), dec!(187.2514));
            assert_eq!(*trade.spent.asset(), Asset::Eur);
            assert_eq!(trade.received.amount(), dec!(0.0020104289));
            assert_eq!(*trade.received.asset(), Asset::Btc);
            assert_eq!(trade.fee.amount(), dec!(0.749));
            assert_eq!(*trade.fee.asset(), Asset::Eur);
        }

        #[test]
        fn spend_receive_pair() {
            let a = make_entry(
                "SPEND-001",
                EntryType::Spend,
                Asset::Eur,
                dec!(-50),
                dec!(0.25),
            );
            let b = make_entry(
                "SPEND-001",
                EntryType::Receive,
                Asset::Btc,
                dec!(0.001),
                Decimal::ZERO,
            );
            let result = find_trades(&[a, b]);
            assert_eq!(result.len(), 1);
            let trade = &result[0];
            assert_eq!(*trade.spent.asset(), Asset::Eur);
            assert_eq!(trade.spent.amount(), dec!(50));
            assert_eq!(*trade.received.asset(), Asset::Btc);
            assert_eq!(trade.received.amount(), dec!(0.001));
            assert_eq!(trade.fee.amount(), dec!(0.25));
        }

        #[test]
        fn earn_entries_excluded() {
            let a = make_entry(
                "EARN-001",
                EntryType::Earn,
                Asset::Btc,
                dec!(-0.001),
                Decimal::ZERO,
            );
            let b = make_entry(
                "EARN-001",
                EntryType::Earn,
                Asset::Btc,
                dec!(0.001),
                Decimal::ZERO,
            );
            let result = find_trades(&[a, b]);
            assert!(result.is_empty());
        }

        #[test]
        fn deposit_only_excluded() {
            let dep = make_entry(
                "DEP-001",
                EntryType::Deposit,
                Asset::Eur,
                dec!(1000),
                Decimal::ZERO,
            );
            let result = find_trades(&[dep]);
            assert!(result.is_empty());
        }

        /// Sell trade: BTC negative, EUR positive, fee on EUR (positive) side.
        #[test]
        fn sell_trade_fee_on_positive_side() {
            let btc = make_entry(
                "SELL-001",
                EntryType::Trade,
                Asset::Btc,
                dec!(-0.001),
                Decimal::ZERO,
            );
            let eur = make_entry(
                "SELL-001",
                EntryType::Trade,
                Asset::Eur,
                dec!(63.124),
                dec!(0.2525),
            );
            let result = find_trades(&[btc, eur]);
            assert_eq!(result.len(), 1);
            let trade = &result[0];
            assert_eq!(*trade.spent.asset(), Asset::Btc);
            assert_eq!(trade.spent.amount(), dec!(0.001));
            assert_eq!(*trade.received.asset(), Asset::Eur);
            assert_eq!(trade.received.amount(), dec!(63.124));
            // Fee is on the EUR (positive) entry — parser must pick it up
            assert_eq!(trade.fee.amount(), dec!(0.2525));
            assert_eq!(*trade.fee.asset(), Asset::Eur);
        }

        /// Buy trade with fee on the BTC (positive) side — matches real
        /// Kraken trade XOI65FD in the fixture.
        #[test]
        fn buy_trade_fee_on_positive_side() {
            let eur = make_entry(
                "BUY-002",
                EntryType::Trade,
                Asset::Eur,
                dec!(-193.9818),
                Decimal::ZERO,
            );
            let btc = make_entry(
                "BUY-002",
                EntryType::Trade,
                Asset::Btc,
                dec!(0.0021553528),
                dec!(0.0000053897),
            );
            let result = find_trades(&[eur, btc]);
            assert_eq!(result.len(), 1);
            let trade = &result[0];
            assert_eq!(*trade.spent.asset(), Asset::Eur);
            assert_eq!(*trade.received.asset(), Asset::Btc);
            // Fee is on BTC (positive/buy) side
            assert_eq!(trade.fee.amount(), dec!(0.0000053897));
            assert_eq!(*trade.fee.asset(), Asset::Btc);
        }
    }

    mod headers {
        use super::*;

        #[test]
        fn matches_kraken_headers() {
            let headers: Vec<&str> =
                "txid,refid,time,type,subtype,aclass,subclass,asset,wallet,amount,fee,balance"
                    .split(',')
                    .collect();
            assert!(matches_headers(&headers));
        }

        #[test]
        fn rejects_non_kraken_headers() {
            let headers = vec!["Timestamp", "Transaction Type", "Asset"];
            assert!(!matches_headers(&headers));
        }
    }

    fn make_entry(
        refid: &str,
        type_: EntryType,
        asset: Asset,
        amount: Decimal,
        fee: Decimal,
    ) -> LedgerEntry {
        use chrono::TimeZone;
        LedgerEntry {
            txid: "TX001".into(),
            refid: refid.into(),
            time: Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
            type_,
            subtype: String::new(),
            aclass: "currency".into(),
            subclass: String::new(),
            asset,
            wallet: String::new(),
            amount,
            fee,
            balance: Decimal::ZERO,
        }
    }
}

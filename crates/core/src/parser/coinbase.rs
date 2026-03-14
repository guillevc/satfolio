use std::path::Path;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::errors::{ParseError, ParseResult};
use crate::models::{Asset, AssetAmount, Trade};

/// Expected Coinbase transaction history CSV columns.
const COINBASE_HEADERS: &[&str] = &[
    "Timestamp",
    "Transaction Type",
    "Asset",
    "Quantity Transacted",
    "Spot Price Currency",
    "Spot Price at Transaction",
    "Subtotal",
    "Total (inclusive of fees and/or spread)",
    "Fees and/or Spread",
    "Notes",
];

/// Returns true if the header row matches a Coinbase transaction export.
pub(super) fn matches_headers(headers: &[&str]) -> bool {
    headers == COINBASE_HEADERS
}

/// Custom CSV deserializers for fields that may be empty in Coinbase exports.
/// The `csv` crate passes empty strings to inner deserializers, which would
/// fail for types like `Decimal`. These helpers map `""` → `None`.
mod csv_option {
    use rust_decimal::Decimal;
    use serde::{self, Deserialize, Deserializer};
    use std::str::FromStr;

    pub fn decimal<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(None)
        } else {
            Decimal::from_str(&s)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }

    pub fn string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s.is_empty() { Ok(None) } else { Ok(Some(s)) }
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct CoinbaseRow {
    #[serde(rename = "Timestamp")]
    timestamp: DateTime<Utc>,
    #[serde(rename = "Transaction Type")]
    transaction_type: String,
    #[serde(rename = "Asset")]
    asset: Asset,
    #[serde(
        rename = "Quantity Transacted",
        deserialize_with = "csv_option::decimal"
    )]
    quantity: Option<Decimal>,
    // Must be Option<String>, NOT Option<Asset>. Asset has #[serde(from = "String")]
    // which converts "" to Asset::Other("") instead of None. We convert to Asset
    // manually in find_trades().
    #[serde(
        rename = "Spot Price Currency",
        deserialize_with = "csv_option::string"
    )]
    spot_price_currency: Option<String>,
    #[serde(
        rename = "Spot Price at Transaction",
        deserialize_with = "csv_option::decimal"
    )]
    spot_price: Option<Decimal>,
    #[serde(rename = "Subtotal", deserialize_with = "csv_option::decimal")]
    subtotal: Option<Decimal>,
    #[serde(
        rename = "Total (inclusive of fees and/or spread)",
        deserialize_with = "csv_option::decimal"
    )]
    total: Option<Decimal>,
    #[serde(
        rename = "Fees and/or Spread",
        deserialize_with = "csv_option::decimal"
    )]
    fees: Option<Decimal>,
    #[serde(rename = "Notes")]
    notes: String,
}

/// Find the byte offset where the Coinbase header row starts.
/// Coinbase CSVs may have preamble metadata lines before the header.
fn find_header_offset(content: &str) -> Option<usize> {
    let mut offset = 0;
    for line in content.split('\n') {
        let trimmed = line.trim_end_matches('\r');
        let headers: Vec<&str> = trimmed
            .split(',')
            .map(|s| s.trim().trim_matches('"'))
            .collect();
        if matches_headers(&headers) {
            return Some(offset);
        }
        offset += line.len() + 1; // +1 for the '\n' delimiter
    }
    None
}

fn parse_csv_rows(path: &Path) -> ParseResult<Vec<CoinbaseRow>> {
    let content = std::fs::read_to_string(path)?;
    let offset = find_header_offset(&content).ok_or_else(|| {
        ParseError::UnrecognizedFormat(content.lines().next().unwrap_or_default().to_string())
    })?;
    let mut reader = csv::Reader::from_reader(&content.as_bytes()[offset..]);
    Ok(reader
        .deserialize()
        .collect::<Result<Vec<_>, csv::Error>>()?)
}

fn find_trades(rows: &[CoinbaseRow]) -> Vec<Trade> {
    rows.iter()
        .filter_map(|row| {
            let is_buy = matches!(
                row.transaction_type.as_str(),
                "Buy" | "Advanced Trade Buy" | "Advance Trade Buy"
            );
            let is_sell = matches!(
                row.transaction_type.as_str(),
                "Sell" | "Advanced Trade Sell" | "Advance Trade Sell"
            );
            if !is_buy && !is_sell {
                return None;
            }

            let subtotal = row.subtotal?;
            let quantity = row.quantity?;
            let fees = row.fees.unwrap_or(Decimal::ZERO);
            let fiat = Asset::from(row.spot_price_currency.clone()?);

            if is_buy {
                Some(Trade {
                    date: row.timestamp,
                    spent: AssetAmount::new(subtotal, fiat.clone()),
                    received: AssetAmount::new(quantity, row.asset.clone()),
                    fee: AssetAmount::new(fees, fiat),
                })
            } else {
                Some(Trade {
                    date: row.timestamp,
                    spent: AssetAmount::new(quantity, row.asset.clone()),
                    received: AssetAmount::new(subtotal, fiat.clone()),
                    fee: AssetAmount::new(fees, fiat),
                })
            }
        })
        .collect()
}

pub(super) fn parse(path: &Path) -> ParseResult<Vec<Trade>> {
    let rows = parse_csv_rows(path)?;
    let mut trades = find_trades(&rows);
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

        const CSV_HEADER: &str = "Timestamp,Transaction Type,Asset,Quantity Transacted,\
            Spot Price Currency,Spot Price at Transaction,Subtotal,\
            Total (inclusive of fees and/or spread),Fees and/or Spread,Notes";

        #[test]
        fn preamble_skipped() {
            let csv = format!(
                "You can use this transaction report\n\
                 \n\
                 {CSV_HEADER}\n\
                 2025-01-15T10:30:00Z,Buy,BTC,0.00150000,EUR,62000.00,93.00,95.79,2.79,Bought BTC"
            );
            let f = csv_tempfile(&csv);
            let rows = parse_csv_rows(f.path()).unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].transaction_type, "Buy");
        }

        #[test]
        fn no_preamble() {
            let csv = format!(
                "{CSV_HEADER}\n\
                 2025-01-15T10:30:00Z,Buy,BTC,0.00150000,EUR,62000.00,93.00,95.79,2.79,Bought BTC"
            );
            let f = csv_tempfile(&csv);
            let rows = parse_csv_rows(f.path()).unwrap();
            assert_eq!(rows.len(), 1);
        }

        #[test]
        fn empty_numeric_fields() {
            let csv = format!(
                "{CSV_HEADER}\n\
                 2025-03-01T08:00:00Z,Receive,BTC,0.00050000,,,,,,Received BTC"
            );
            let f = csv_tempfile(&csv);
            let rows = parse_csv_rows(f.path()).unwrap();
            assert_eq!(rows.len(), 1);
            assert!(rows[0].spot_price_currency.is_none());
            assert!(rows[0].subtotal.is_none());
            assert!(rows[0].total.is_none());
            assert!(rows[0].fees.is_none());
        }

        #[test]
        fn zero_fee_parsed_as_some() {
            let csv = format!(
                "{CSV_HEADER}\n\
                 2025-04-01T10:00:00Z,Learning Reward,GRT,5.00,EUR,0.15,0.75,0.75,0.00,Earned GRT"
            );
            let f = csv_tempfile(&csv);
            let rows = parse_csv_rows(f.path()).unwrap();
            assert_eq!(rows[0].fees, Some(dec!(0.00)));
        }
    }

    mod trades {
        use super::*;
        use chrono::TimeZone;

        fn make_row(
            transaction_type: &str,
            asset: Asset,
            quantity: Option<Decimal>,
            spot_price_currency: Option<&str>,
            subtotal: Option<Decimal>,
            fees: Option<Decimal>,
        ) -> CoinbaseRow {
            CoinbaseRow {
                timestamp: Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).unwrap(),
                transaction_type: transaction_type.to_string(),
                asset,
                quantity,
                spot_price_currency: spot_price_currency.map(String::from),
                spot_price: None,
                subtotal,
                total: None,
                fees,
                notes: String::new(),
            }
        }

        #[test]
        fn buy_mapping() {
            let row = make_row(
                "Buy",
                Asset::Btc,
                Some(dec!(0.0015)),
                Some("EUR"),
                Some(dec!(93.00)),
                Some(dec!(2.79)),
            );
            let trades = find_trades(&[row]);
            assert_eq!(trades.len(), 1);
            let t = &trades[0];
            assert_eq!(t.spent.amount(), dec!(93.00));
            assert_eq!(*t.spent.asset(), Asset::Eur);
            assert_eq!(t.received.amount(), dec!(0.0015));
            assert_eq!(*t.received.asset(), Asset::Btc);
            assert_eq!(t.fee.amount(), dec!(2.79));
            assert_eq!(*t.fee.asset(), Asset::Eur);
        }

        #[test]
        fn sell_mapping() {
            let row = make_row(
                "Sell",
                Asset::Btc,
                Some(dec!(0.001)),
                Some("EUR"),
                Some(dec!(63.00)),
                Some(dec!(1.89)),
            );
            let trades = find_trades(&[row]);
            assert_eq!(trades.len(), 1);
            let t = &trades[0];
            assert_eq!(t.spent.amount(), dec!(0.001));
            assert_eq!(*t.spent.asset(), Asset::Btc);
            assert_eq!(t.received.amount(), dec!(63.00));
            assert_eq!(*t.received.asset(), Asset::Eur);
            assert_eq!(t.fee.amount(), dec!(1.89));
            assert_eq!(*t.fee.asset(), Asset::Eur);
        }

        #[test]
        fn advanced_trade_buy() {
            let row = make_row(
                "Advanced Trade Buy",
                Asset::Btc,
                Some(dec!(0.003)),
                Some("EUR"),
                Some(dec!(180.00)),
                Some(dec!(3.60)),
            );
            let trades = find_trades(&[row]);
            assert_eq!(trades.len(), 1);
            assert_eq!(trades[0].received.amount(), dec!(0.003));
        }

        #[test]
        fn advanced_trade_sell() {
            let row = make_row(
                "Advanced Trade Sell",
                Asset::Btc,
                Some(dec!(0.002)),
                Some("EUR"),
                Some(dec!(128.00)),
                Some(dec!(3.84)),
            );
            let trades = find_trades(&[row]);
            assert_eq!(trades.len(), 1);
            assert_eq!(trades[0].spent.amount(), dec!(0.002));
            assert_eq!(*trades[0].spent.asset(), Asset::Btc);
        }

        #[test]
        fn advance_trade_buy_typo_variant() {
            // Coinbase has a known typo: "Advance Trade Buy" (without the "d")
            let row = make_row(
                "Advance Trade Buy",
                Asset::Btc,
                Some(dec!(0.001)),
                Some("EUR"),
                Some(dec!(65.00)),
                Some(dec!(1.95)),
            );
            let trades = find_trades(&[row]);
            assert_eq!(trades.len(), 1);
            assert_eq!(trades[0].received.amount(), dec!(0.001));
        }

        #[test]
        fn advance_trade_sell_typo_variant() {
            let row = make_row(
                "Advance Trade Sell",
                Asset::Btc,
                Some(dec!(0.001)),
                Some("EUR"),
                Some(dec!(63.00)),
                Some(dec!(1.89)),
            );
            let trades = find_trades(&[row]);
            assert_eq!(trades.len(), 1);
            assert_eq!(trades[0].spent.amount(), dec!(0.001));
            assert_eq!(*trades[0].spent.asset(), Asset::Btc);
        }

        #[test]
        fn non_trade_types_excluded() {
            let rows = vec![
                make_row(
                    "Deposit",
                    Asset::Eur,
                    Some(dec!(500.00)),
                    Some("EUR"),
                    None,
                    None,
                ),
                make_row("Receive", Asset::Btc, Some(dec!(0.0005)), None, None, None),
                make_row("Send", Asset::Btc, Some(dec!(0.001)), None, None, None),
                make_row(
                    "Learning Reward",
                    Asset::Other("GRT".into()),
                    Some(dec!(5.0)),
                    Some("EUR"),
                    Some(dec!(0.75)),
                    Some(dec!(0.00)),
                ),
                make_row(
                    "Staking Income",
                    Asset::Other("ETH".into()),
                    Some(dec!(0.001)),
                    Some("EUR"),
                    Some(dec!(3.20)),
                    Some(dec!(0.00)),
                ),
                make_row(
                    "Convert",
                    Asset::Btc,
                    Some(dec!(0.0005)),
                    Some("EUR"),
                    Some(dec!(32.00)),
                    Some(dec!(0.00)),
                ),
                make_row(
                    "Withdrawal",
                    Asset::Eur,
                    Some(dec!(200.00)),
                    Some("EUR"),
                    None,
                    None,
                ),
            ];
            let trades = find_trades(&rows);
            assert!(trades.is_empty());
        }

        #[test]
        fn missing_subtotal_skipped() {
            // A Buy row with missing subtotal should be safely skipped
            let row = make_row(
                "Buy",
                Asset::Btc,
                Some(dec!(0.001)),
                Some("EUR"),
                None,
                Some(dec!(1.00)),
            );
            let trades = find_trades(&[row]);
            assert!(trades.is_empty());
        }

        #[test]
        fn missing_fees_defaults_to_zero() {
            let row = make_row(
                "Buy",
                Asset::Btc,
                Some(dec!(0.001)),
                Some("EUR"),
                Some(dec!(65.00)),
                None,
            );
            let trades = find_trades(&[row]);
            assert_eq!(trades.len(), 1);
            assert_eq!(trades[0].fee.amount(), Decimal::ZERO);
        }
    }

    mod headers {
        use super::*;

        #[test]
        fn matches_coinbase_headers() {
            let headers: Vec<&str> = vec![
                "Timestamp",
                "Transaction Type",
                "Asset",
                "Quantity Transacted",
                "Spot Price Currency",
                "Spot Price at Transaction",
                "Subtotal",
                "Total (inclusive of fees and/or spread)",
                "Fees and/or Spread",
                "Notes",
            ];
            assert!(matches_headers(&headers));
        }

        #[test]
        fn rejects_non_coinbase_headers() {
            let headers = vec!["txid", "refid", "time"];
            assert!(!matches_headers(&headers));
        }
    }

    mod integration {
        use super::*;

        #[test]
        fn parse_fixture() {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/coinbase_sample.csv");
            let trades = parse(&path).unwrap();
            assert_eq!(trades.len(), 7);

            // Verify chronological order
            for w in trades.windows(2) {
                assert!(w[0].date <= w[1].date);
            }

            // Trade 1: Buy BTC 2025-01-15
            assert_eq!(trades[0].spent.amount(), dec!(93.00));
            assert_eq!(*trades[0].spent.asset(), Asset::Eur);
            assert_eq!(trades[0].received.amount(), dec!(0.00150000));
            assert_eq!(*trades[0].received.asset(), Asset::Btc);
            assert_eq!(trades[0].fee.amount(), dec!(2.79));

            // Trade 2: Buy BTC 2025-02-01
            assert_eq!(trades[1].spent.amount(), dec!(123.00));
            assert_eq!(trades[1].received.amount(), dec!(0.00200000));

            // Trade 3: Sell BTC 2025-02-10
            assert_eq!(trades[2].spent.amount(), dec!(0.00100000));
            assert_eq!(*trades[2].spent.asset(), Asset::Btc);
            assert_eq!(trades[2].received.amount(), dec!(63.00));
            assert_eq!(*trades[2].received.asset(), Asset::Eur);
            assert_eq!(trades[2].fee.amount(), dec!(1.89));

            // Trade 4: Advanced Trade Buy 2025-03-05
            assert_eq!(trades[3].received.amount(), dec!(0.00300000));
            assert_eq!(trades[3].spent.amount(), dec!(180.00));

            // Trade 5: Advanced Trade Sell 2025-03-15
            assert_eq!(trades[4].spent.amount(), dec!(0.00200000));
            assert_eq!(*trades[4].spent.asset(), Asset::Btc);
            assert_eq!(trades[4].received.amount(), dec!(128.00));

            // Trade 6: Advance Trade Buy (typo variant) 2025-04-05
            assert_eq!(trades[5].received.amount(), dec!(0.00100000));
            assert_eq!(trades[5].fee.amount(), dec!(1.95));

            // Trade 7: Buy GRT 2025-05-01 (non-BTC asset → Asset::Other)
            assert_eq!(*trades[6].received.asset(), Asset::Other("GRT".into()));
            assert_eq!(trades[6].received.amount(), dec!(100.00000000));
            assert_eq!(trades[6].spent.amount(), dec!(18.00));
            assert_eq!(trades[6].fee.amount(), dec!(0.54));
        }
    }
}

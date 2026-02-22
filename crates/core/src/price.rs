use std::path::Path;
use std::str::FromStr as _;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::errors::{PriceError, PriceResult};
use crate::models::{Asset, Candle};

mod timestamp_format {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs: i64 = Deserialize::deserialize(deserializer)?;
        DateTime::from_timestamp(secs, 0)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid timestamp: {secs}")))
    }
}

#[derive(Deserialize)]
struct OhlcRow {
    #[serde(with = "timestamp_format")]
    timestamp: DateTime<Utc>,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    count: u32,
}

fn asset_to_filename(quote: &Asset) -> PriceResult<&'static str> {
    match quote {
        Asset::Eur => Ok("XBTEUR_1440.csv"),
        Asset::Gbp => Ok("XBTGBP_1440.csv"),
        Asset::Usd => Ok("XBTUSD_1440.csv"),
        other => Err(PriceError::UnsupportedCurrency(other.to_string())),
    }
}

fn parse_ohlc_csv(rdr: impl std::io::Read) -> PriceResult<Vec<Candle>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(rdr);

    reader
        .deserialize::<OhlcRow>()
        .map(|r| {
            let row = r?;
            Ok(Candle {
                date: row.timestamp.date_naive(),
                open: row.open,
                high: row.high,
                low: row.low,
                close: row.close,
                volume: row.volume,
                count: row.count,
            })
        })
        .collect()
}

pub(crate) fn load_bundled_prices(dir: &Path, quote: &Asset) -> PriceResult<Vec<Candle>> {
    let filename = asset_to_filename(quote)?;
    let path = dir.join(filename);
    let file = std::fs::File::open(&path)?;
    parse_ohlc_csv(file)
}

// ── Kraken OHLC API ────────────────────────────────────────

fn asset_to_kraken_pair(quote: &Asset) -> PriceResult<&'static str> {
    match quote {
        Asset::Eur => Ok("XBTEUR"),
        Asset::Gbp => Ok("XBTGBP"),
        Asset::Usd => Ok("XBTUSD"),
        other => Err(PriceError::UnsupportedCurrency(other.to_string())),
    }
}

fn parse_decimal_field(val: &serde_json::Value, index: usize) -> PriceResult<Decimal> {
    val.get(index)
        .and_then(|v| v.as_str())
        .ok_or_else(|| PriceError::InvalidResponse(format!("missing field at index {index}")))
        .and_then(|s| {
            Decimal::from_str(s)
                .map_err(|e| PriceError::InvalidResponse(format!("bad decimal '{s}': {e}")))
        })
}

pub(crate) fn parse_ohlc_json(body: &[u8]) -> PriceResult<Vec<Candle>> {
    let root: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| PriceError::InvalidResponse(format!("invalid JSON: {e}")))?;

    // Check for API errors
    let errors = root
        .get("error")
        .and_then(|e| e.as_array())
        .ok_or_else(|| PriceError::InvalidResponse("missing error field".into()))?;
    if let Some(err) = errors.first() {
        return Err(PriceError::InvalidResponse(format!(
            "Kraken API error: {}",
            err.as_str().unwrap_or("unknown")
        )));
    }

    // Find the OHLC data key (skip "last")
    let result = root
        .get("result")
        .and_then(|r| r.as_object())
        .ok_or_else(|| PriceError::InvalidResponse("missing result object".into()))?;

    let ohlc_array = result
        .iter()
        .find(|(k, _)| k.as_str() != "last")
        .map(|(_, v)| v)
        .and_then(|v| v.as_array())
        .ok_or_else(|| PriceError::InvalidResponse("no OHLC data found".into()))?;

    // Drop last candle (incomplete current period)
    let count = ohlc_array.len().saturating_sub(1);
    let mut candles = Vec::with_capacity(count);

    // Kraken fields: [time, open, high, low, close, vwap, volume, count]
    for entry in ohlc_array.iter().take(count) {
        let timestamp = entry
            .get(0)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| PriceError::InvalidResponse("missing timestamp".into()))?;
        let date = DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| PriceError::InvalidResponse(format!("bad timestamp: {timestamp}")))?
            .date_naive();

        candles.push(Candle {
            date,
            open: parse_decimal_field(entry, 1)?,
            high: parse_decimal_field(entry, 2)?,
            low: parse_decimal_field(entry, 3)?,
            close: parse_decimal_field(entry, 4)?,
            // index 5 = vwap (skipped)
            volume: parse_decimal_field(entry, 6)?,
            count: entry
                .get(7)
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        });
    }

    Ok(candles)
}

pub(crate) fn fetch_ohlc(quote: &Asset, since: NaiveDate) -> PriceResult<Vec<Candle>> {
    let pair = asset_to_kraken_pair(quote)?;
    let ts = since.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
    let url = format!(
        "https://api.kraken.com/0/public/OHLC?pair={pair}&interval=1440&since={ts}"
    );
    let body = reqwest::blocking::get(&url)?.bytes()?;
    parse_ohlc_json(&body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn csv_bytes(content: &str) -> &[u8] {
        content.as_bytes()
    }

    #[test]
    fn parse_single_candle() {
        let csv = "1381017600,122.0,122.0,122.0,122.0,0.1,1\n";
        let candles = parse_ohlc_csv(csv_bytes(csv)).unwrap();
        assert_eq!(candles.len(), 1);
        let c = &candles[0];
        assert_eq!(c.date, NaiveDate::from_ymd_opt(2013, 10, 6).unwrap());
        assert_eq!(c.close, dec!(122.0));
        assert_eq!(c.open, dec!(122.0));
        assert_eq!(c.volume, dec!(0.1));
        assert_eq!(c.count, 1);
    }

    #[test]
    fn parse_multiple_candles_ordered() {
        let csv = "1381017600,122.0,122.0,122.0,122.0,0.1,1\n\
                   1381104000,123.61,123.61,123.61,123.61,0.1,1\n";
        let candles = parse_ohlc_csv(csv_bytes(csv)).unwrap();
        assert_eq!(candles.len(), 2);
        assert!(candles[0].date < candles[1].date);
    }

    #[test]
    fn parse_empty_csv() {
        let candles = parse_ohlc_csv(csv_bytes("")).unwrap();
        assert!(candles.is_empty());
    }

    #[test]
    fn asset_to_filename_supported() {
        assert_eq!(asset_to_filename(&Asset::Eur).unwrap(), "XBTEUR_1440.csv");
        assert_eq!(asset_to_filename(&Asset::Gbp).unwrap(), "XBTGBP_1440.csv");
        assert_eq!(asset_to_filename(&Asset::Usd).unwrap(), "XBTUSD_1440.csv");
    }

    #[test]
    fn asset_to_filename_unsupported() {
        assert!(asset_to_filename(&Asset::Btc).is_err());
        assert!(asset_to_filename(&Asset::Other("CHF".into())).is_err());
    }

    fn fixtures_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/prices")
    }

    #[test]
    fn load_bundled_eur() {
        let candles = load_bundled_prices(&fixtures_dir(), &Asset::Eur).unwrap();
        assert_eq!(candles.len(), 5);
        assert_eq!(candles[0].date, NaiveDate::from_ymd_opt(2013, 9, 10).unwrap());
        assert_eq!(candles[0].close, dec!(97.0));
        assert_eq!(candles[4].close, dec!(74500.1));
        for w in candles.windows(2) {
            assert!(w[0].date < w[1].date);
        }
    }

    #[test]
    fn load_bundled_gbp() {
        let candles = load_bundled_prices(&fixtures_dir(), &Asset::Gbp).unwrap();
        assert_eq!(candles.len(), 5);
        assert_eq!(candles[0].date, NaiveDate::from_ymd_opt(2014, 11, 6).unwrap());
        assert_eq!(candles[0].close, dec!(213.0));
        assert_eq!(candles[4].close, dec!(64933.2));
        for w in candles.windows(2) {
            assert!(w[0].date < w[1].date);
        }
    }

    #[test]
    fn load_bundled_usd() {
        let candles = load_bundled_prices(&fixtures_dir(), &Asset::Usd).unwrap();
        assert_eq!(candles.len(), 5);
        assert_eq!(candles[0].date, NaiveDate::from_ymd_opt(2013, 10, 6).unwrap());
        assert_eq!(candles[0].close, dec!(122.0));
        assert_eq!(candles[4].close, dec!(87500.1));
        for w in candles.windows(2) {
            assert!(w[0].date < w[1].date);
        }
    }

    // ── Kraken JSON parsing ────────────────────────────────

    #[test]
    fn kraken_pair_mapping() {
        assert_eq!(asset_to_kraken_pair(&Asset::Eur).unwrap(), "XBTEUR");
        assert_eq!(asset_to_kraken_pair(&Asset::Gbp).unwrap(), "XBTGBP");
        assert_eq!(asset_to_kraken_pair(&Asset::Usd).unwrap(), "XBTUSD");
        assert!(asset_to_kraken_pair(&Asset::Btc).is_err());
    }

    #[test]
    fn parse_ohlc_json_valid() {
        // Two candles + one incomplete (last) that should be dropped
        let json = br#"{
            "error": [],
            "result": {
                "XXBTZEUR": [
                    [1609459200, "28900.0", "29000.0", "28800.0", "28950.0", "28925.0", "100.5", 42],
                    [1609545600, "28950.0", "29500.0", "28900.0", "29400.0", "29100.0", "200.3", 85],
                    [1609632000, "29400.0", "29600.0", "29300.0", "29500.0", "29450.0", "50.1", 10]
                ],
                "last": 1609632000
            }
        }"#;
        let candles = parse_ohlc_json(json).unwrap();
        assert_eq!(candles.len(), 2); // last candle dropped
        assert_eq!(candles[0].date, NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        assert_eq!(candles[0].open, dec!(28900.0));
        assert_eq!(candles[0].close, dec!(28950.0));
        assert_eq!(candles[0].volume, dec!(100.5));
        assert_eq!(candles[0].count, 42);
        assert_eq!(candles[1].close, dec!(29400.0));
    }

    #[test]
    fn parse_ohlc_json_api_error() {
        let json = br#"{"error": ["EGeneral:Invalid arguments"], "result": {}}"#;
        let err = parse_ohlc_json(json).unwrap_err();
        assert!(err.to_string().contains("Invalid arguments"));
    }

    #[test]
    fn parse_ohlc_json_single_candle_dropped() {
        // Only one candle = it's the incomplete current period → empty result
        let json = br#"{
            "error": [],
            "result": {
                "XXBTZEUR": [
                    [1609459200, "28900.0", "29000.0", "28800.0", "28950.0", "28925.0", "100.5", 42]
                ],
                "last": 1609459200
            }
        }"#;
        let candles = parse_ohlc_json(json).unwrap();
        assert!(candles.is_empty());
    }
}

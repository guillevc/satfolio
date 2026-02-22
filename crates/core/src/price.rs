use std::path::Path;

use chrono::{DateTime, Utc};
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
                date: row.timestamp,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
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
        assert_eq!(c.date, Utc.with_ymd_and_hms(2013, 10, 6, 0, 0, 0).unwrap());
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
        assert_eq!(candles[0].date, Utc.with_ymd_and_hms(2013, 9, 10, 0, 0, 0).unwrap());
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
        assert_eq!(candles[0].date, Utc.with_ymd_and_hms(2014, 11, 6, 0, 0, 0).unwrap());
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
        assert_eq!(candles[0].date, Utc.with_ymd_and_hms(2013, 10, 6, 0, 0, 0).unwrap());
        assert_eq!(candles[0].close, dec!(122.0));
        assert_eq!(candles[4].close, dec!(87500.1));
        for w in candles.windows(2) {
            assert!(w[0].date < w[1].date);
        }
    }
}

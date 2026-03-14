use std::path::Path;

use sha2::{Digest, Sha256};

use crate::errors::ParseResult;
use crate::models::Trade;

/// SHA-256 hex digest of the entire file contents.
pub(crate) fn file_sha256(path: &Path) -> ParseResult<String> {
    let bytes = std::fs::read(path)?;
    let hash = Sha256::digest(&bytes);
    Ok(format!("{hash:x}"))
}

/// Deterministic trade hash: SHA-256 of `"{source}|{date}|{spent}|{received}|{fee}"`.
///
/// The source prefix (e.g. `"kraken"`) prevents collisions if a second exchange
/// produces trades with identical fields.
pub(crate) fn trade_hash(source: &str, trade: &Trade) -> String {
    let input = format!(
        "{source}|{}|{}|{}|{}|{}|{}|{}",
        trade.date.to_rfc3339(),
        trade.spent.amount(),
        trade.spent.asset(),
        trade.received.amount(),
        trade.received.asset(),
        trade.fee.amount(),
        trade.fee.asset(),
    );
    let hash = Sha256::digest(input.as_bytes());
    format!("{hash:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Asset, AssetAmount, Provider};
    use chrono::{TimeZone, Utc};
    use rust_decimal_macros::dec;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn sample_trade() -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(187.2514), Asset::Eur),
            received: AssetAmount::new(dec!(0.0020104289), Asset::Btc),
            fee: AssetAmount::new(dec!(0.749), Asset::Eur),
            provider: Provider::Kraken,
        }
    }

    #[test]
    fn trade_hash_is_deterministic() {
        let t = sample_trade();
        let h1 = trade_hash("kraken", &t);
        let h2 = trade_hash("kraken", &t);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn trade_hash_differs_by_source() {
        let t = sample_trade();
        let kraken = trade_hash("kraken", &t);
        let other = trade_hash("binance", &t);
        assert_ne!(kraken, other);
    }

    #[test]
    fn trade_hash_differs_by_field() {
        let t1 = sample_trade();
        let mut t2 = sample_trade();
        t2.fee = AssetAmount::new(dec!(1.0), Asset::Eur);
        assert_ne!(trade_hash("kraken", &t1), trade_hash("kraken", &t2));
    }

    #[test]
    fn file_sha256_correctness() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"hello world").unwrap();
        let hash = file_sha256(f.path()).unwrap();
        // Known SHA-256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
}

use std::path::Path;

use crate::errors::{ParseError, ParseResult};
use crate::models::Trade;

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

pub(super) fn parse(_path: &Path) -> ParseResult<Vec<Trade>> {
    Err(ParseError::NotYetSupported("Coinbase".into()))
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn parse_returns_not_yet_supported() {
        let result = parse(Path::new("fake.csv"));
        let err = result.unwrap_err();
        assert!(
            matches!(err, ParseError::NotYetSupported(ref s) if s == "Coinbase"),
            "expected NotYetSupported, got: {err}"
        );
    }
}

mod coinbase;
mod kraken;

use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::errors::{ParseError, ParseResult};
use crate::models::{Provider, Trade};

/// Maximum number of lines to scan for header detection.
/// Coinbase exports have metadata rows before the actual header row.
const MAX_HEADER_SCAN_LINES: usize = 20;

/// Auto-detect which provider produced the CSV by scanning the first N lines
/// for known header signatures.
pub(crate) fn detect_provider(path: &Path) -> ParseResult<Provider> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut first_line = None;
    for line in reader.lines().take(MAX_HEADER_SCAN_LINES) {
        let line = line?;
        if first_line.is_none() {
            first_line = Some(line.clone());
        }
        let headers: Vec<&str> = line
            .split(',')
            .map(|s| s.trim().trim_matches('"'))
            .collect();

        if kraken::matches_headers(&headers) {
            return Ok(Provider::Kraken);
        }
        if coinbase::matches_headers(&headers) {
            return Ok(Provider::Coinbase);
        }
    }

    Err(ParseError::UnrecognizedFormat(
        first_line.unwrap_or_default(),
    ))
}

/// Parse trades from a CSV file using the given provider's parser.
pub(crate) fn parse_csv(provider: &Provider, path: &Path) -> ParseResult<Vec<Trade>> {
    match provider {
        Provider::Kraken => kraken::parse(path),
        Provider::Coinbase => coinbase::parse(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn csv_tempfile(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn detect_kraken_from_headers() {
        let f = csv_tempfile(
            "txid,refid,time,type,subtype,aclass,subclass,asset,wallet,amount,fee,balance\n",
        );
        assert_eq!(detect_provider(f.path()).unwrap(), Provider::Kraken);
    }

    #[test]
    fn detect_kraken_from_quoted_headers() {
        let f = csv_tempfile(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"subclass\",\"asset\",\"wallet\",\"amount\",\"fee\",\"balance\"\n",
        );
        assert_eq!(detect_provider(f.path()).unwrap(), Provider::Kraken);
    }

    #[test]
    fn detect_coinbase_from_headers() {
        let f = csv_tempfile(
            "Timestamp,Transaction Type,Asset,Quantity Transacted,Spot Price Currency,Spot Price at Transaction,Subtotal,Total (inclusive of fees and/or spread),Fees and/or Spread,Notes\n",
        );
        assert_eq!(detect_provider(f.path()).unwrap(), Provider::Coinbase);
    }

    #[test]
    fn detect_coinbase_with_metadata_preamble() {
        // Coinbase exports can have metadata rows before headers
        let f = csv_tempfile(
            "You can use this transaction report to inform your tax return\n\
             \n\
             Timestamp,Transaction Type,Asset,Quantity Transacted,Spot Price Currency,Spot Price at Transaction,Subtotal,Total (inclusive of fees and/or spread),Fees and/or Spread,Notes\n",
        );
        assert_eq!(detect_provider(f.path()).unwrap(), Provider::Coinbase);
    }

    #[test]
    fn detect_unknown_format() {
        let f = csv_tempfile("col_a,col_b,col_c\n1,2,3\n");
        let err = detect_provider(f.path()).unwrap_err();
        assert!(
            matches!(err, ParseError::UnrecognizedFormat(_)),
            "expected UnrecognizedFormat, got: {err}"
        );
    }

    #[test]
    fn parse_csv_coinbase_returns_not_yet_supported() {
        let f = csv_tempfile("fake");
        let err = parse_csv(&Provider::Coinbase, f.path()).unwrap_err();
        assert!(
            matches!(err, ParseError::NotYetSupported(ref s) if s == "Coinbase"),
            "expected NotYetSupported, got: {err}"
        );
    }
}

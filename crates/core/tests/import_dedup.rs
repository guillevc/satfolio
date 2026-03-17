use std::path::Path;

use app_core::api::{confirm_import, init_db, list_imports, preview_import, trades};
use app_core::errors::CoreError;
use app_core::models::Asset;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures");

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(FIXTURES).join(name)
}

/// Create a temp DB, init with bundled prices, and return the guard + path.
/// The `TempDir` must outlive the path to keep the directory alive.
fn test_cfg() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    init_db(&db_path, &fixture("prices")).unwrap();
    (dir, db_path)
}

#[test]
fn confirm_two_overlapping_csvs_deduplicates() {
    let (_dir, db_path) = test_cfg();
    let eur = Asset::Eur;

    let outcome_a = confirm_import(&db_path, &eur, &fixture("kraken/overlap_a.csv")).unwrap();
    assert_eq!(outcome_a.import.trade_count, 22);

    let outcome_b = confirm_import(&db_path, &eur, &fixture("kraken/overlap_b.csv")).unwrap();
    // 28 total in file B, 13 overlap with A → 15 new
    assert_eq!(outcome_b.import.trade_count, 15);
    assert!(
        outcome_b.message.is_some(),
        "should report skipped duplicates"
    );

    let all_trades = trades(&db_path, &eur).unwrap();
    assert_eq!(all_trades.len(), 37);

    let imports = list_imports(&db_path).unwrap();
    assert_eq!(imports.len(), 2);
}

#[test]
fn preview_then_confirm_overlap() {
    let (_dir, db_path) = test_cfg();
    let eur = Asset::Eur;

    // Import file A first
    confirm_import(&db_path, &eur, &fixture("kraken/overlap_a.csv")).unwrap();

    // Preview file B — should detect 13 duplicates
    let preview = preview_import(&db_path, &eur, &fixture("kraken/overlap_b.csv")).unwrap();
    assert_eq!(preview.duplicate_trades, 13);
    assert!(!preview.exact_file_duplicate);

    // Confirm file B
    let outcome_b = confirm_import(&db_path, &eur, &fixture("kraken/overlap_b.csv")).unwrap();
    assert_eq!(outcome_b.import.trade_count, 15);

    let all_trades = trades(&db_path, &eur).unwrap();
    assert_eq!(all_trades.len(), 37);
}

#[test]
fn confirm_same_file_twice_returns_duplicate_file() {
    let (_dir, db_path) = test_cfg();
    let eur = Asset::Eur;
    confirm_import(&db_path, &eur, &fixture("kraken/overlap_a.csv")).unwrap();
    let err = confirm_import(&db_path, &eur, &fixture("kraken/overlap_a.csv")).unwrap_err();
    assert!(
        matches!(err, CoreError::DuplicateFile),
        "expected DuplicateFile, got: {err}"
    );
}

#[test]
fn confirm_subset_returns_all_trades_duplicate() {
    let (_dir, db_path) = test_cfg();
    let eur = Asset::Eur;
    confirm_import(&db_path, &eur, &fixture("kraken/overlap_a.csv")).unwrap();
    let err = confirm_import(&db_path, &eur, &fixture("kraken/overlap_a_subset.csv")).unwrap_err();
    assert!(
        matches!(err, CoreError::AllTradesDuplicate(_)),
        "expected AllTradesDuplicate, got: {err}"
    );
}

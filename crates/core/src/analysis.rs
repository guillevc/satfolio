// analysis.rs — Extract "crypto buy" events from parsed Kraken ledger data.
//
// A "crypto buy" is a transaction where you spent one asset (typically EUR)
// to acquire another (typically BTC). In the Kraken CSV, these appear as
// TWO entries sharing the same `refid`:
//
//   refid: "MECOSFO-GY"
//   ├── EUR  amount: -187.2514   fee: 0.749    (the spend leg)
//   └── BTC  amount: +0.0020104289  fee: 0.0   (the buy leg)
//
// Valid buy patterns (by EntryType):
//   • (Trade, Trade)     — standard market/limit orders
//   • (Spend, Receive)   — instant/simple buy via Kraken app
//
// NOT buys:
//   • (Earn, Earn)       — staking rewards or wallet transfers, skip these
//   • Single entries     — deposits, withdrawals, etc.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::parser::{Asset, EntryType, LedgerEntry};

// TODO(human): Define the CryptoBuy struct
//
// This struct represents a single "buy" event derived from a refid pair.
// Fields you'll need:
//   - date: DateTime<Utc>          — timestamp from the buy leg
//   - asset_bought: Asset           — what you acquired (e.g. BTC)
//   - amount_bought: Decimal        — how much you got (positive)
//   - asset_spent: Asset            — what you paid with (e.g. EUR)
//   - amount_spent: Decimal         — how much you paid (store as POSITIVE using .abs())
//   - fee: Decimal                  — total fee (sum both legs' fees)
//
// Derive Debug so you can println!("{:?}", buy) while developing.
#[derive(Debug)]
pub struct CryptoBuy;

/// Scans ledger entries and extracts crypto buy events.
///
/// Algorithm:
///   1. Group entries by `refid` → HashMap<String, Vec<&LedgerEntry>>
///   2. Keep only groups of exactly 2 entries whose types are
///      (Trade, Trade) or (Spend, Receive)
///   3. In each pair, the leg with negative amount is the "spend",
///      positive is the "buy". Build a CryptoBuy from them.
///
/// Useful methods:
///   - Decimal::is_sign_negative() / is_sign_positive()
///   - Decimal::abs()
///   - HashMap::entry(...).or_insert_with(Vec::new).push(item)
pub fn find_crypto_buys(entries: &[LedgerEntry]) -> Vec<CryptoBuy> {
    // Step 1: Group entries by refid
    // ──────────────────────────────
    // Create a HashMap<String, Vec<&LedgerEntry>>.
    // Iterate over `entries` and group them by their `refid` field.
    // Hint: use entry().or_default().push(&entry)

    // Step 2: Filter to valid buy pairs
    // ──────────────────────────────────
    // Iterate over the groups. Keep only groups where:
    //   - group.len() == 2
    //   - types are (Trade, Trade) or (Spend, Receive) in either order
    // Hint: match on (&group[0].type_, &group[1].type_)

    // Step 3: Build CryptoBuy from each pair
    // ───────────────────────────────────────
    // For each valid pair:
    //   - The entry with negative amount is the spend leg
    //   - The entry with positive amount is the buy leg
    //   - fee = spend_leg.fee + buy_leg.fee
    //   - amount_spent should be stored as positive (use .abs())

    todo!("implement find_crypto_buys")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// Helper: build a LedgerEntry with the given fields, defaults for the rest.
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

    // Test: A Trade/Trade pair produces one CryptoBuy with correct values.
    //
    // Use this real data from the Kraken CSV:
    //   refid: "MECOSFO-GY"
    //   Leg 1: type=Trade, asset=EUR, amount=-187.2514, fee=0.749
    //   Leg 2: type=Trade, asset=BTC, amount=+0.0020104289, fee=0
    //
    // Expected CryptoBuy:
    //   asset_bought = BTC, amount_bought = 0.0020104289
    //   asset_spent  = EUR, amount_spent  = 187.2514  (positive!)
    //   fee = 0.749
    #[test]
    fn trade_trade_pair() {
        todo!("test: Trade/Trade pair → one CryptoBuy")
    }

    // Test: A Spend/Receive pair also produces one CryptoBuy.
    //
    // Example data:
    //   refid: "SPEND-001"
    //   Leg 1: type=Spend,   asset=EUR, amount=-50.00, fee=0.25
    //   Leg 2: type=Receive, asset=BTC, amount=+0.001,  fee=0
    #[test]
    fn spend_receive_pair() {
        todo!("test: Spend/Receive pair → one CryptoBuy")
    }

    // Test: Earn/Earn entries are NOT buys — they should be excluded.
    //
    //   refid: "EARN-001"
    //   Leg 1: type=Earn, asset=BTC, amount=-0.001, fee=0
    //   Leg 2: type=Earn, asset=BTC, amount=+0.001, fee=0
    #[test]
    fn earn_entries_excluded() {
        todo!("test: Earn entries excluded")
    }

    // Test: A single Deposit entry has no matching pair → excluded.
    //
    //   refid: "DEP-001"
    //   Only: type=Deposit, asset=EUR, amount=+1000.0, fee=0
    #[test]
    fn deposit_only_excluded() {
        todo!("test: single Deposit excluded")
    }
}

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::errors::ParseResult;
use crate::models::{Asset, AssetAmount, Provider, Trade};

/// Kraken fee-credit token — has no market value. Fees in KFEE should be zero.
/// Source: <https://support.kraken.com/articles/204799657-What-are-Kraken-fee-credits-KFEE->
fn is_kfee(asset: &Asset) -> bool {
    matches!(asset, Asset::Other(s) if s == "KFEE")
}

/// Returns true if this entry is a staking/earn reward.
///
/// Per Kraken's CSV docs (<https://support.kraken.com/articles/360001169383>):
/// - `type=earn, subtype=reward` — "payouts from on-chain Staking or Opt-In Rewards"
/// - `type=staking` — "primarily used for staking rewards"
fn is_reward(entry: &LedgerEntry) -> bool {
    let is_earn_reward = entry.type_ == EntryType::Earn && entry.subtype == "reward";
    let is_staking = entry.type_ == EntryType::Staking;
    let is_reward_type = entry.type_ == EntryType::Reward;
    (is_earn_reward || is_staking || is_reward_type) && entry.amount.is_sign_positive()
}

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

/// Kraken ledger CSV `type` field values.
///
/// Source: <https://support.kraken.com/articles/360001169383-how-to-interpret-ledger-history-fields>
///
/// Types not in CSV exports (API-only or filter-only) fall into `Other`.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
enum EntryType {
    Trade,
    Earn,
    Spend,
    Receive,
    Staking,
    Reward,
    Deposit,
    Withdrawal,
    Transfer,
    Adjustment,
    #[serde(rename = "margin trade")]
    MarginTrade,
    Margin,
    Rollover,
    Settled,
    #[serde(rename = "invite bonus")]
    InviteBonus,
    #[serde(untagged)]
    Other(String),
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            EntryType::Trade => "trade",
            EntryType::Earn => "earn",
            EntryType::Spend => "spend",
            EntryType::Receive => "receive",
            EntryType::Staking => "staking",
            EntryType::Reward => "reward",
            EntryType::Deposit => "deposit",
            EntryType::Withdrawal => "withdrawal",
            EntryType::Transfer => "transfer",
            EntryType::Adjustment => "adjustment",
            EntryType::MarginTrade => "margin trade",
            EntryType::Margin => "margin",
            EntryType::Rollover => "rollover",
            EntryType::Settled => "settled",
            EntryType::InviteBonus => "invite bonus",
            EntryType::Other(s) => s.as_str(),
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
    let mut trades = Vec::new();

    for entry in entries {
        if entry.refid.is_empty() {
            if is_reward(entry) {
                trades.push(make_reward(entry));
            }
            continue;
        }
        by_refid.entry(&entry.refid).or_default().push(entry);
    }

    for (_, group) in by_refid {
        // Single-entry groups: staking/earn rewards with a refid
        if group.len() == 1 {
            let entry = group[0];
            if is_reward(entry) {
                trades.push(make_reward(entry));
            }
            continue;
        }

        let [left, right] = *group.as_slice() else {
            continue;
        };

        // Failed/cancelled events: same type & subtype, amounts cancel out
        if left.type_ == right.type_
            && left.subtype == right.subtype
            && (left.amount + right.amount).is_zero()
        {
            continue;
        }

        let pair = match (&left.type_, &right.type_) {
            (EntryType::Trade, EntryType::Trade)
            | (EntryType::Spend, EntryType::Receive)
            | (EntryType::Receive, EntryType::Spend) => Some((left, right)),
            _ => None,
        };

        if let Some((left, right)) = pair {
            let (buy, sell) = if left.amount.is_sign_positive() {
                (left, right)
            } else {
                (right, left)
            };
            // Kraken places the fee on either entry — pick whichever has a
            // non-zero fee (first non-zero wins, matching BittyTax behaviour).
            // KFEE tokens have no monetary value — treat as zero fee.
            let (fee_amount, fee_asset) = if !sell.fee.is_zero() && !is_kfee(&sell.asset) {
                (sell.fee.abs(), sell.asset.clone())
            } else if !buy.fee.is_zero() && !is_kfee(&buy.asset) {
                (buy.fee.abs(), buy.asset.clone())
            } else {
                // Both zero or KFEE — use sell asset with zero amount
                (Decimal::ZERO, sell.asset.clone())
            };
            trades.push(Trade {
                date: buy.time,
                spent: AssetAmount::new(sell.amount.abs(), sell.asset.clone()),
                received: AssetAmount::new(buy.amount.abs(), buy.asset.clone()),
                fee: AssetAmount::new(fee_amount, fee_asset),
                provider: Provider::Kraken,
            });
        }
    }

    trades
}

/// Create a reward trade from a single staking/earn entry.
/// Modeled as a zero-cost inflow: spent = 0, received = reward amount.
/// The engine detects this pattern (received = base asset, spent = 0) as `TradeSide::Reward`.
fn make_reward(entry: &LedgerEntry) -> Trade {
    Trade {
        date: entry.time,
        spent: AssetAmount::zero(entry.asset.clone()),
        received: AssetAmount::new(entry.amount.abs(), entry.asset.clone()),
        fee: AssetAmount::zero(entry.asset.clone()),
        provider: Provider::Kraken,
    }
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

        #[test]
        fn parse_csv_transfer_and_staking_types() {
            let csv = format!(
                "{CSV_HEADER}\n\
            TX1,REF-T,2024-03-01 10:00:00,transfer,,currency,,ETH,spot,1.5,0,1.5\n\
            TX2,REF-S,2024-03-02 10:00:00,staking,,currency,,DOT,spot,10.0,0,10.0"
            );
            let f = csv_tempfile(&csv);
            let entries = parse_csv_entries(f.path()).unwrap();
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].type_, EntryType::Transfer);
            assert_eq!(entries[1].type_, EntryType::Staking);
        }

        #[test]
        fn parse_csv_unknown_type_becomes_other() {
            let csv = format!(
                "{CSV_HEADER}\n\
            TX1,REF-X,2024-04-01 10:00:00,something_new,,currency,,BTC,spot,0.1,0,0.1"
            );
            let f = csv_tempfile(&csv);
            let entries = parse_csv_entries(f.path()).unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].type_, EntryType::Other("something_new".into()));
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
        fn transfer_and_unknown_excluded() {
            let transfer = make_entry(
                "XFER-001",
                EntryType::Transfer,
                Asset::Btc,
                dec!(0.5),
                Decimal::ZERO,
            );
            let unknown = make_entry(
                "UNK-001",
                EntryType::Other("mystery".into()),
                Asset::Eur,
                dec!(100),
                Decimal::ZERO,
            );
            let result = find_trades(&[transfer, unknown]);
            assert!(result.is_empty());
        }

        #[test]
        fn staking_reward_parsed() {
            let mut staking = make_entry(
                "STAKE-001",
                EntryType::Staking,
                Asset::Btc,
                dec!(0.00000123),
                Decimal::ZERO,
            );
            // Staking rewards typically have no refid
            staking.refid = String::new();
            let result = find_trades(&[staking]);
            assert_eq!(result.len(), 1);
            let trade = &result[0];
            assert_eq!(trade.received.amount(), dec!(0.00000123));
            assert_eq!(*trade.received.asset(), Asset::Btc);
            assert_eq!(trade.spent.amount(), Decimal::ZERO);
        }

        #[test]
        fn staking_reward_with_refid_parsed() {
            let staking = make_entry(
                "STAKE-002",
                EntryType::Staking,
                Asset::Btc,
                dec!(0.00000456),
                Decimal::ZERO,
            );
            let result = find_trades(&[staking]);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].received.amount(), dec!(0.00000456));
        }

        /// Kraken Earn rewards use type=earn, subtype=reward (current unified system).
        #[test]
        fn earn_reward_parsed() {
            let mut entry = make_entry(
                "EARN-R01",
                EntryType::Earn,
                Asset::Btc,
                dec!(0.00000789),
                Decimal::ZERO,
            );
            entry.subtype = "reward".into();
            let result = find_trades(&[entry]);
            assert_eq!(result.len(), 1);
            let trade = &result[0];
            assert_eq!(trade.received.amount(), dec!(0.00000789));
            assert_eq!(*trade.received.asset(), Asset::Btc);
            assert_eq!(trade.spent.amount(), Decimal::ZERO);
        }

        /// Fiat earn rewards (EUR interest from earn/flexible) are also parsed.
        #[test]
        fn earn_reward_fiat_parsed() {
            let mut entry = make_entry(
                "EARN-R02",
                EntryType::Earn,
                Asset::Eur,
                dec!(0.0179),
                Decimal::ZERO,
            );
            entry.subtype = "reward".into();
            let result = find_trades(&[entry]);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].received.amount(), dec!(0.0179));
            assert_eq!(*result[0].received.asset(), Asset::Eur);
        }

        /// Standalone type=reward entries (reported by CoinTaxman community).
        #[test]
        fn reward_type_parsed() {
            let entry = make_entry(
                "RWD-001",
                EntryType::Reward,
                Asset::Btc,
                dec!(0.00000100),
                Decimal::ZERO,
            );
            let result = find_trades(&[entry]);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].received.amount(), dec!(0.00000100));
            assert_eq!(result[0].spent.amount(), Decimal::ZERO);
        }

        /// Earn allocations (subtype=allocation) should NOT be treated as rewards.
        #[test]
        fn earn_allocation_excluded() {
            let mut alloc = make_entry(
                "EARN-A01",
                EntryType::Earn,
                Asset::Btc,
                dec!(-0.001),
                Decimal::ZERO,
            );
            alloc.subtype = "allocation".into();
            let mut alloc2 = make_entry(
                "EARN-A01",
                EntryType::Earn,
                Asset::Btc,
                dec!(0.001),
                Decimal::ZERO,
            );
            alloc2.subtype = "allocation".into();
            let result = find_trades(&[alloc, alloc2]);
            assert!(result.is_empty());
        }

        #[test]
        fn kfee_fee_on_buy_side_treated_as_zero() {
            let sell_entry = make_entry(
                "KFEE-T1",
                EntryType::Trade,
                Asset::Eur,
                dec!(-100),
                Decimal::ZERO,
            );
            let buy_entry = LedgerEntry {
                fee: dec!(5),
                asset: Asset::Other("KFEE".into()),
                ..make_entry(
                    "KFEE-T1",
                    EntryType::Trade,
                    Asset::Btc,
                    dec!(0.001),
                    Decimal::ZERO,
                )
            };
            let result = find_trades(&[sell_entry, buy_entry]);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].fee.amount(), Decimal::ZERO);
        }

        #[test]
        fn kfee_fee_on_sell_side_treated_as_zero() {
            let sell_entry = LedgerEntry {
                fee: dec!(3),
                asset: Asset::Other("KFEE".into()),
                ..make_entry(
                    "KFEE-T2",
                    EntryType::Trade,
                    Asset::Eur,
                    dec!(-100),
                    Decimal::ZERO,
                )
            };
            let buy_entry = make_entry(
                "KFEE-T2",
                EntryType::Trade,
                Asset::Btc,
                dec!(0.001),
                Decimal::ZERO,
            );
            let result = find_trades(&[sell_entry, buy_entry]);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].fee.amount(), Decimal::ZERO);
        }

        #[test]
        fn failed_event_cancelled() {
            let a = make_entry(
                "FAIL-001",
                EntryType::Withdrawal,
                Asset::Btc,
                dec!(-0.01),
                Decimal::ZERO,
            );
            let b = make_entry(
                "FAIL-001",
                EntryType::Withdrawal,
                Asset::Btc,
                dec!(0.01),
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

    mod integration {
        use super::*;

        #[test]
        fn parse_fixture() {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/kraken/sample.csv");
            let trades = parse(&path).unwrap();

            // 12 trade pairs + 25 spend/receive pairs + 61 earn rewards = 98
            assert_eq!(trades.len(), 98);

            // Verify chronological order
            for w in trades.windows(2) {
                assert!(w[0].date <= w[1].date);
            }

            // First trade is a BTC buy via trade pair (2025-02-14)
            assert_eq!(*trades[0].received.asset(), Asset::Btc);
            assert_eq!(*trades[0].spent.asset(), Asset::Eur);
            assert!(trades[0].spent.amount() > Decimal::ZERO);

            // Earn rewards have zero spent (free BTC/EUR/ETH)
            let rewards: Vec<_> = trades
                .iter()
                .filter(|t| t.spent.amount().is_zero())
                .collect();
            assert_eq!(rewards.len(), 61);
            for r in &rewards {
                assert!(r.received.amount() > Decimal::ZERO);
            }
        }
    }
}

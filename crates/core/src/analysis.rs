use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::parser::{Asset, EntryType, LedgerEntry};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetAmount {
    pub amount: Decimal,
    pub asset: Asset,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Trade {
    pub date: DateTime<Utc>,
    pub spent: AssetAmount,
    pub received: AssetAmount,
    pub fee: AssetAmount,
}

impl Trade {
    pub fn side_for(&self, asset: &Asset) -> Option<TradeSide> {
        match asset {
            a if *a == self.spent.asset => Some(TradeSide::Sell),
            a if *a == self.received.asset => Some(TradeSide::Buy),
            _ => None,
        }
    }
}

pub fn find_trades(entries: &[LedgerEntry]) -> Vec<Trade> {
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
            Trade {
                date: buy.time,
                spent: AssetAmount {
                    amount: sell.amount.abs(),
                    asset: sell.asset.clone(),
                },
                received: AssetAmount {
                    amount: buy.amount.abs(),
                    asset: buy.asset.clone(),
                },
                fee: AssetAmount {
                    amount: sell.fee.abs(),
                    asset: sell.asset.clone(),
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use chrono::TimeZone;
    use rust_decimal_macros::dec;

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

    fn make_trade() -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
            spent: AssetAmount {
                amount: dec!(187.2514),
                asset: Asset::Eur,
            },
            received: AssetAmount {
                amount: dec!(0.0020104289),
                asset: Asset::Btc,
            },
            fee: AssetAmount {
                amount: dec!(0.749),
                asset: Asset::Eur,
            },
        }
    }

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
        assert_eq!(trade.spent.amount, dec!(187.2514));
        assert_eq!(trade.spent.asset, Asset::Eur);
        assert_eq!(trade.received.amount, dec!(0.0020104289));
        assert_eq!(trade.received.asset, Asset::Btc);
        assert_eq!(trade.fee.amount, dec!(0.749));
        assert_eq!(trade.fee.asset, Asset::Eur);
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
        assert_eq!(trade.spent.asset, Asset::Eur);
        assert_eq!(trade.spent.amount, dec!(50));
        assert_eq!(trade.received.asset, Asset::Btc);
        assert_eq!(trade.received.amount, dec!(0.001));
        assert_eq!(trade.fee.amount, dec!(0.25));
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

    #[test]
    fn trade_side_for() {
        let trade = make_trade();
        assert_eq!(trade.side_for(&Asset::Btc), Some(TradeSide::Buy));
        assert_eq!(trade.side_for(&Asset::Eur), Some(TradeSide::Sell));
    }
}

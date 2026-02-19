use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::parser::{Asset, EntryType, LedgerEntry};

#[derive(Debug)]
pub struct AssetAmount {
    pub amount: Decimal,
    pub asset: Asset,
}

#[derive(Debug)]
pub struct CryptoBuy {
    pub date: DateTime<Utc>,
    pub spent: AssetAmount,
    pub received: AssetAmount,
    pub fee: AssetAmount,
}

pub fn find_crypto_buys(entries: &[LedgerEntry]) -> Vec<CryptoBuy> {
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
            CryptoBuy {
                date: buy.time,
                spent: AssetAmount {
                    amount: sell.amount.abs(),
                    asset: sell.asset.clone(),
                },
                received: AssetAmount {
                    amount: buy.amount,
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
        let result = find_crypto_buys(&[spend, receive]);
        assert_eq!(result.len(), 1);
        let buy = result.first().unwrap();
        assert_eq!(buy.spent.amount, dec!(187.2514));
        assert_eq!(buy.spent.asset, Asset::Eur);
        assert_eq!(buy.received.amount, dec!(0.0020104289));
        assert_eq!(buy.received.asset, Asset::Btc);
        assert_eq!(buy.fee.amount, dec!(0.749));
        assert_eq!(buy.fee.asset, Asset::Eur);
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
        let result = find_crypto_buys(&[a, b]);
        assert_eq!(result.len(), 1);
        let buy = &result[0];
        assert_eq!(buy.spent.asset, Asset::Eur);
        assert_eq!(buy.spent.amount, dec!(50));
        assert_eq!(buy.received.asset, Asset::Btc);
        assert_eq!(buy.received.amount, dec!(0.001));
        assert_eq!(buy.fee.amount, dec!(0.25));
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
        let result = find_crypto_buys(&[a, b]);
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
        let result = find_crypto_buys(&[dep]);
        assert!(result.is_empty());
    }
}

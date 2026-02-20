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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradesSummary {
    pub total_trades: usize,
    pub buys: usize,
    pub sells: usize,
    pub unknown: usize,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub spent: AssetAmount,
    pub received: AssetAmount,
    pub fees: AssetAmount,
}

pub fn summarize_trades(
    received_asset: &Asset,
    spent_asset: &Asset,
    trades: &[Trade],
) -> TradesSummary {
    let mut buys = 0usize;
    let mut sells = 0usize;
    let mut unknown = 0usize;
    let mut spent = Decimal::ZERO;
    let mut received = Decimal::ZERO;
    let mut fees = Decimal::ZERO;
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;

    for trade in trades {
        let pair = (&trade.spent.asset, &trade.received.asset);
        if pair == (spent_asset, received_asset) {
            buys += 1;
            spent += trade.spent.amount;
            received += trade.received.amount;
            fees += trade.fee.amount;
        } else if pair == (received_asset, spent_asset) {
            sells += 1;
            spent += trade.received.amount;
            received += trade.spent.amount;
            fees += trade.fee.amount;
        } else {
            unknown += 1;
            continue;
        }

        earliest = Some(earliest.map_or(trade.date, |e| e.min(trade.date)));
        latest = Some(latest.map_or(trade.date, |l| l.max(trade.date)));
    }

    TradesSummary {
        total_trades: trades.len(),
        buys,
        sells,
        unknown,
        date_range: earliest.zip(latest),
        spent: AssetAmount {
            amount: spent,
            asset: spent_asset.clone(),
        },
        received: AssetAmount {
            amount: received,
            asset: received_asset.clone(),
        },
        fees: AssetAmount {
            amount: fees,
            asset: spent_asset.clone(),
        },
    }
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

    fn make_trade_at(
        year: i32,
        month: u32,
        day: u32,
        spent: Decimal,
        received: Decimal,
        fee: Decimal,
    ) -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap(),
            spent: AssetAmount {
                amount: spent,
                asset: Asset::Eur,
            },
            received: AssetAmount {
                amount: received,
                asset: Asset::Btc,
            },
            fee: AssetAmount {
                amount: fee,
                asset: Asset::Eur,
            },
        }
    }

    #[test]
    fn summarize_two_buys() {
        let trades = vec![
            make_trade_at(2025, 2, 14, dec!(187.25), dec!(0.002), dec!(0.75)),
            make_trade_at(2025, 6, 1, dec!(28.37), dec!(0.0003), dec!(0.28)),
        ];
        let summary = summarize_trades(&Asset::Btc, &Asset::Eur, &trades);

        assert_eq!(summary.total_trades, 2);
        assert_eq!(summary.buys, 2);
        assert_eq!(summary.sells, 0);
        assert_eq!(summary.unknown, 0);
        assert_eq!(summary.spent.amount, dec!(215.62));
        assert_eq!(summary.spent.asset, Asset::Eur);
        assert_eq!(summary.received.amount, dec!(0.0023));
        assert_eq!(summary.received.asset, Asset::Btc);
        assert_eq!(summary.fees.amount, dec!(1.03));
        assert_eq!(summary.fees.asset, Asset::Eur);
        let (earliest, latest) = summary.date_range.unwrap();
        assert_eq!(
            earliest,
            Utc.with_ymd_and_hms(2025, 2, 14, 12, 0, 0).unwrap()
        );
        assert_eq!(latest, Utc.with_ymd_and_hms(2025, 6, 1, 12, 0, 0).unwrap());
    }

    #[test]
    fn summarize_empty() {
        let summary = summarize_trades(&Asset::Btc, &Asset::Eur, &[]);
        assert_eq!(summary.total_trades, 0);
        assert_eq!(summary.buys, 0);
        assert!(summary.date_range.is_none());
        assert_eq!(summary.spent.amount, Decimal::ZERO);
    }

    #[test]
    fn summarize_ignores_unrelated_pair() {
        // ETH/USD trade should count as unknown when tracking BTC/EUR
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 3, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount {
                amount: dec!(100),
                asset: Asset::Other("USD".into()),
            },
            received: AssetAmount {
                amount: dec!(0.05),
                asset: Asset::Other("ETH".into()),
            },
            fee: AssetAmount {
                amount: dec!(0.5),
                asset: Asset::Other("USD".into()),
            },
        };
        let summary = summarize_trades(&Asset::Btc, &Asset::Eur, &[trade]);

        assert_eq!(summary.total_trades, 1);
        assert_eq!(summary.buys, 0);
        assert_eq!(summary.unknown, 1);
        assert_eq!(summary.spent.amount, Decimal::ZERO);
        assert_eq!(summary.fees.amount, Decimal::ZERO);
        assert!(summary.date_range.is_none());
    }

    #[test]
    fn summarize_btc_buy_with_wrong_fiat_is_unknown() {
        // BTC/USD trade when tracking BTC/EUR — BTC side matches but EUR doesn't
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 3, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount {
                amount: dec!(200),
                asset: Asset::Other("USD".into()),
            },
            received: AssetAmount {
                amount: dec!(0.002),
                asset: Asset::Btc,
            },
            fee: AssetAmount {
                amount: dec!(1),
                asset: Asset::Other("USD".into()),
            },
        };
        let summary = summarize_trades(&Asset::Btc, &Asset::Eur, &[trade]);

        assert_eq!(summary.buys, 0);
        assert_eq!(summary.unknown, 1);
        assert_eq!(summary.spent.amount, Decimal::ZERO);
    }

    #[test]
    fn summarize_sell_trade() {
        // Selling BTC for EUR: spent=BTC, received=EUR (reverse of buy pair)
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 4, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount {
                amount: dec!(0.005),
                asset: Asset::Btc,
            },
            received: AssetAmount {
                amount: dec!(300),
                asset: Asset::Eur,
            },
            fee: AssetAmount {
                amount: dec!(1.2),
                asset: Asset::Eur,
            },
        };
        let summary = summarize_trades(&Asset::Btc, &Asset::Eur, &[trade]);

        assert_eq!(summary.buys, 0);
        assert_eq!(summary.sells, 1);
        assert_eq!(summary.unknown, 0);
        // TODO(human): assert spent, received, and fees amounts for the sell case
    }
}

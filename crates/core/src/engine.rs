use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::models::{Asset, AssetAmount, BepSnapshot, DashboardStats, Trade, TradesSummary};

pub(crate) fn bep_series(
    received_asset: &Asset,
    spent_asset: &Asset,
    trades: &[Trade],
) -> Vec<BepSnapshot> {
    todo!()
}

pub(crate) fn dashboard_stats(
    received_asset: &Asset,
    spent_asset: &Asset,
    trades: &[Trade],
) -> DashboardStats {
    todo!()
}

pub(crate) fn summarize_trades(
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

    fn make_buy(y: i32, m: u32, d: u32, eur: Decimal, btc: Decimal, fee: Decimal) -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap(),
            spent: AssetAmount {
                amount: eur,
                asset: Asset::Eur,
            },
            received: AssetAmount {
                amount: btc,
                asset: Asset::Btc,
            },
            fee: AssetAmount {
                amount: fee,
                asset: Asset::Eur,
            },
        }
    }

    fn make_sell(y: i32, m: u32, d: u32, btc: Decimal, eur: Decimal, fee: Decimal) -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap(),
            spent: AssetAmount {
                amount: btc,
                asset: Asset::Btc,
            },
            received: AssetAmount {
                amount: eur,
                asset: Asset::Eur,
            },
            fee: AssetAmount {
                amount: fee,
                asset: Asset::Eur,
            },
        }
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

    // ── BEP series ──────────────────────────────────────────

    #[test]
    fn bep_two_buys() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
        ];
        let snaps = bep_series(&Asset::Btc, &Asset::Eur, &trades);
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[0].bep, Some(dec!(100500)));
        assert_eq!(snaps[1].bep, Some(dec!(75375)));
    }

    #[test]
    fn bep_buy_then_sell() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
            make_sell(2025, 3, 1, dec!(0.001), dec!(120), dec!(0.60)),
        ];
        let snaps = bep_series(&Asset::Btc, &Asset::Eur, &trades);
        assert_eq!(snaps.len(), 3);
        assert_eq!(snaps[2].asset_held, dec!(0.003));
        assert_eq!(snaps[2].bep, Some(dec!(60700)));
    }

    #[test]
    fn bep_sell_all_is_none() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_sell(2025, 2, 1, dec!(0.001), dec!(120), dec!(0.60)),
        ];
        let snaps = bep_series(&Asset::Btc, &Asset::Eur, &trades);
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[1].asset_held, Decimal::ZERO);
        assert_eq!(snaps[1].bep, None);
    }

    // ── Dashboard stats ─────────────────────────────────────

    #[test]
    fn dashboard_after_two_buys_and_sell() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
            make_sell(2025, 3, 1, dec!(0.001), dec!(120), dec!(0.60)),
        ];
        let stats = dashboard_stats(&Asset::Btc, &Asset::Eur, &trades);
        assert_eq!(stats.bep, Some(dec!(60700)));
        assert_eq!(stats.asset_held, dec!(0.003));
        assert_eq!(stats.total_spent, dec!(300));
        assert_eq!(stats.total_received, dec!(120));
        assert_eq!(stats.total_fees, dec!(2.10));
        assert_eq!(stats.buys, 2);
        assert_eq!(stats.sells, 1);
    }

    // ── Trade summary ───────────────────────────────────────

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
        assert_eq!(summary.spent.amount, dec!(300));
        assert_eq!(summary.received.amount, dec!(0.005));
        assert_eq!(summary.fees.amount, dec!(1.2));
    }
}

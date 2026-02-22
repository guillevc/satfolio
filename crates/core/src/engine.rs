use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};

use crate::errors::EngineResult;
use crate::models::{
    AssetAmount, AssetPair, BepSnapshot, PositionSummary, Trade, TradeSide, TradesSummary,
};

pub(crate) fn bep_snaps(
    pair: &AssetPair,
    trades: &[Trade],
) -> EngineResult<BTreeMap<NaiveDate, BepSnapshot>> {
    let mut snaps = BTreeMap::new();
    let mut held = AssetAmount::zero(pair.base.clone());
    let mut spent = AssetAmount::zero(pair.quote.clone());
    let mut received = AssetAmount::zero(pair.quote.clone());
    let mut fees = AssetAmount::zero(pair.quote.clone());
    for trade in trades {
        let date = trade.date.date_naive();
        match trade.side_for(pair) {
            Some(TradeSide::Buy) => {
                held = held.checked_add(&trade.received)?;
                spent = spent.checked_add(&trade.spent)?;
                fees = fees.checked_add(&trade.fee)?;
            }
            Some(TradeSide::Sell) => {
                held = held.checked_sub(&trade.spent)?;
                received = received.checked_add(&trade.received)?;
                fees = fees.checked_add(&trade.fee)?;
            }
            None => continue,
        }
        let bep =
            spent.checked_sub(&received)?.checked_add(&fees)?.amount().checked_div(held.amount());
        snaps.insert(
            date,
            BepSnapshot {
                date,
                held: held.clone(),
                spent: spent.clone(),
                received: received.clone(),
                fees: fees.clone(),
                bep,
            },
        );
    }
    Ok(snaps)
}

pub(crate) fn position_summary(
    pair: &AssetPair,
    trades: &[Trade],
) -> EngineResult<PositionSummary> {
    let mut held = AssetAmount::zero(pair.base.clone());
    let mut spent = AssetAmount::zero(pair.quote.clone());
    let mut received = AssetAmount::zero(pair.quote.clone());
    let mut fees = AssetAmount::zero(pair.quote.clone());
    let mut buys = 0;
    let mut sells = 0;

    for trade in trades {
        match trade.side_for(pair) {
            Some(TradeSide::Buy) => {
                buys += 1;
                held = held.checked_add(&trade.received)?;
                spent = spent.checked_add(&trade.spent)?;
                fees = fees.checked_add(&trade.fee)?;
            }
            Some(TradeSide::Sell) => {
                sells += 1;
                held = held.checked_sub(&trade.spent)?;
                received = received.checked_add(&trade.received)?;
                fees = fees.checked_add(&trade.fee)?;
            }
            None => continue,
        }
    }

    let bep = (spent.amount() - received.amount() + fees.amount()).checked_div(held.amount());

    Ok(PositionSummary {
        bep,
        held,
        spent,
        received,
        fees,
        buys,
        sells,
    })
}

pub(crate) fn summarize_trades(
    pair: &AssetPair,
    trades: &[Trade],
) -> EngineResult<TradesSummary> {
    let mut buys = 0;
    let mut sells = 0;
    let mut unknown = 0;
    let mut spent = AssetAmount::zero(pair.quote.clone());
    let mut received = AssetAmount::zero(pair.base.clone());
    let mut fees = AssetAmount::zero(pair.quote.clone());
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;

    for trade in trades {
        match trade.side_for(pair) {
            Some(TradeSide::Buy) => {
                buys += 1;
                spent = spent.checked_add(&trade.spent)?;
                received = received.checked_add(&trade.received)?;
                fees = fees.checked_add(&trade.fee)?;
            }
            Some(TradeSide::Sell) => {
                sells += 1;
                spent = spent.checked_add(&trade.received)?;
                received = received.checked_add(&trade.spent)?;
                fees = fees.checked_add(&trade.fee)?;
            }
            None => {
                unknown += 1;
                continue;
            }
        }

        earliest = Some(earliest.map_or(trade.date, |e| e.min(trade.date)));
        latest = Some(latest.map_or(trade.date, |l| l.max(trade.date)));
    }

    Ok(TradesSummary {
        total_trades: trades.len(),
        buys,
        sells,
        unknown,
        date_range: earliest.zip(latest),
        spent,
        received,
        fees,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Asset;
    use chrono::{NaiveDate, TimeZone};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    const fn btc_eur() -> AssetPair {
        AssetPair {
            base: Asset::Btc,
            quote: Asset::Eur,
        }
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn make_buy(y: i32, m: u32, d: u32, eur: Decimal, btc: Decimal, fee: Decimal) -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(eur, Asset::Eur),
            received: AssetAmount::new(btc, Asset::Btc),
            fee: AssetAmount::new(fee, Asset::Eur),
        }
    }

    fn make_sell(y: i32, m: u32, d: u32, btc: Decimal, eur: Decimal, fee: Decimal) -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(btc, Asset::Btc),
            received: AssetAmount::new(eur, Asset::Eur),
            fee: AssetAmount::new(fee, Asset::Eur),
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
            spent: AssetAmount::new(spent, Asset::Eur),
            received: AssetAmount::new(received, Asset::Btc),
            fee: AssetAmount::new(fee, Asset::Eur),
        }
    }

    // ── BEP series ──────────────────────────────────────────

    #[test]
    fn bep_two_buys() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
        ];
        let snaps = bep_snaps(&btc_eur(), &trades).unwrap();
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[&date(2025, 1, 1)].bep, Some(dec!(100500)));
        assert_eq!(snaps[&date(2025, 2, 1)].bep, Some(dec!(75375)));
    }

    #[test]
    fn bep_buy_then_sell() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
            make_sell(2025, 3, 1, dec!(0.001), dec!(120), dec!(0.60)),
        ];
        let snaps = bep_snaps(&btc_eur(), &trades).unwrap();
        assert_eq!(snaps.len(), 3);
        assert_eq!(snaps[&date(2025, 3, 1)].held.amount(), dec!(0.003));
        assert_eq!(snaps[&date(2025, 3, 1)].bep, Some(dec!(60700)));
    }

    #[test]
    fn bep_sell_all_is_none() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_sell(2025, 2, 1, dec!(0.001), dec!(120), dec!(0.60)),
        ];
        let snaps = bep_snaps(&btc_eur(), &trades).unwrap();
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[&date(2025, 2, 1)].held.amount(), Decimal::ZERO);
        assert_eq!(snaps[&date(2025, 2, 1)].bep, None);
    }

    // ── Position summary ────────────────────────────────────

    #[test]
    fn position_after_two_buys_and_sell() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
            make_sell(2025, 3, 1, dec!(0.001), dec!(120), dec!(0.60)),
        ];
        let pos = position_summary(&btc_eur(), &trades).unwrap();
        assert_eq!(pos.bep, Some(dec!(60700)));
        assert_eq!(pos.held, AssetAmount::new(dec!(0.003), Asset::Btc));
        assert_eq!(pos.spent, AssetAmount::new(dec!(300), Asset::Eur));
        assert_eq!(pos.received, AssetAmount::new(dec!(120), Asset::Eur));
        assert_eq!(pos.fees, AssetAmount::new(dec!(2.10), Asset::Eur));
        assert_eq!(pos.buys, 2);
        assert_eq!(pos.sells, 1);
    }

    // ── Trade summary ───────────────────────────────────────

    #[test]
    fn summarize_two_buys() {
        let trades = vec![
            make_trade_at(2025, 2, 14, dec!(187.25), dec!(0.002), dec!(0.75)),
            make_trade_at(2025, 6, 1, dec!(28.37), dec!(0.0003), dec!(0.28)),
        ];
        let summary = summarize_trades(&btc_eur(), &trades).unwrap();

        assert_eq!(summary.total_trades, 2);
        assert_eq!(summary.buys, 2);
        assert_eq!(summary.sells, 0);
        assert_eq!(summary.unknown, 0);
        assert_eq!(summary.spent.amount(), dec!(215.62));
        assert_eq!(*summary.spent.asset(), Asset::Eur);
        assert_eq!(summary.received.amount(), dec!(0.0023));
        assert_eq!(*summary.received.asset(), Asset::Btc);
        assert_eq!(summary.fees.amount(), dec!(1.03));
        assert_eq!(*summary.fees.asset(), Asset::Eur);
        let (earliest, latest) = summary.date_range.unwrap();
        assert_eq!(
            earliest,
            Utc.with_ymd_and_hms(2025, 2, 14, 12, 0, 0).unwrap()
        );
        assert_eq!(latest, Utc.with_ymd_and_hms(2025, 6, 1, 12, 0, 0).unwrap());
    }

    #[test]
    fn summarize_empty() {
        let summary = summarize_trades(&btc_eur(), &[]).unwrap();
        assert_eq!(summary.total_trades, 0);
        assert_eq!(summary.buys, 0);
        assert!(summary.date_range.is_none());
        assert_eq!(summary.spent.amount(), Decimal::ZERO);
    }

    #[test]
    fn summarize_ignores_unrelated_pair() {
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 3, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(100), Asset::Other("USD".into())),
            received: AssetAmount::new(dec!(0.05), Asset::Other("ETH".into())),
            fee: AssetAmount::new(dec!(0.5), Asset::Other("USD".into())),
        };
        let summary = summarize_trades(&btc_eur(), &[trade]).unwrap();

        assert_eq!(summary.total_trades, 1);
        assert_eq!(summary.buys, 0);
        assert_eq!(summary.unknown, 1);
        assert_eq!(summary.spent.amount(), Decimal::ZERO);
        assert_eq!(summary.fees.amount(), Decimal::ZERO);
        assert!(summary.date_range.is_none());
    }

    #[test]
    fn summarize_btc_buy_with_wrong_fiat_is_unknown() {
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 3, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(200), Asset::Other("USD".into())),
            received: AssetAmount::new(dec!(0.002), Asset::Btc),
            fee: AssetAmount::new(dec!(1), Asset::Other("USD".into())),
        };
        let summary = summarize_trades(&btc_eur(), &[trade]).unwrap();

        assert_eq!(summary.buys, 0);
        assert_eq!(summary.unknown, 1);
        assert_eq!(summary.spent.amount(), Decimal::ZERO);
    }

    #[test]
    fn summarize_sell_trade() {
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 4, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(0.005), Asset::Btc),
            received: AssetAmount::new(dec!(300), Asset::Eur),
            fee: AssetAmount::new(dec!(1.2), Asset::Eur),
        };
        let summary = summarize_trades(&btc_eur(), &[trade]).unwrap();

        assert_eq!(summary.buys, 0);
        assert_eq!(summary.sells, 1);
        assert_eq!(summary.unknown, 0);
        assert_eq!(summary.spent.amount(), dec!(300));
        assert_eq!(summary.received.amount(), dec!(0.005));
        assert_eq!(summary.fees.amount(), dec!(1.2));
    }
}

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::errors::EngineResult;
use crate::models::{
    AssetAmount, AssetPair, EnrichedTrade, PositionSummary, Trade, TradeSide, TradesSummary,
};

impl Trade {
    fn side_for(&self, pair: &AssetPair) -> Option<TradeSide> {
        let trade_pair = (self.spent.asset(), self.received.asset());
        if trade_pair == (&pair.quote, &pair.base) {
            Some(TradeSide::Buy)
        } else if trade_pair == (&pair.base, &pair.quote) {
            Some(TradeSide::Sell)
        } else {
            None
        }
    }
}

struct ResolvedTrade {
    side: TradeSide,
    units: Decimal,
    quote_amount: Decimal,
    fee_quote: Decimal,
}

fn resolve(trade: &Trade, pair: &AssetPair) -> Option<ResolvedTrade> {
    let side = trade.side_for(pair)?;
    let (units, quote_amount) = match side {
        TradeSide::Buy => (trade.received.amount(), trade.spent.amount()),
        TradeSide::Sell => (trade.spent.amount(), trade.received.amount()),
    };
    let price = quote_amount.checked_div(units).unwrap_or(Decimal::ZERO);
    // Normalize fee to quote currency (e.g. BTC fee → EUR via trade price)
    let fee_quote = if trade.fee.asset() == &pair.quote {
        trade.fee.amount()
    } else if trade.fee.asset() == &pair.base {
        trade.fee.amount() * price
    } else {
        Decimal::ZERO
    };
    Some(ResolvedTrade {
        side,
        units,
        quote_amount,
        fee_quote,
    })
}

struct Accumulator {
    held: AssetAmount,
    invested: AssetAmount,
    proceeds: AssetAmount,
    fees: AssetAmount,
    buys: usize,
    sells: usize,
}

impl Accumulator {
    fn new(pair: &AssetPair) -> Self {
        Self {
            held: AssetAmount::zero(pair.base.clone()),
            invested: AssetAmount::zero(pair.quote.clone()),
            proceeds: AssetAmount::zero(pair.quote.clone()),
            fees: AssetAmount::zero(pair.quote.clone()),
            buys: 0,
            sells: 0,
        }
    }

    /// Apply a trade, returning the matched side (or None if unrelated).
    fn apply(&mut self, pair: &AssetPair, trade: &Trade) -> EngineResult<Option<TradeSide>> {
        let r = match resolve(trade, pair) {
            Some(r) => r,
            None => return Ok(None),
        };
        match r.side {
            TradeSide::Buy => {
                self.buys += 1;
                self.held = self.held.checked_add(&trade.received)?;
                self.invested = self.invested.checked_add(&trade.spent)?;
            }
            TradeSide::Sell => {
                self.sells += 1;
                self.held = self.held.checked_sub(&trade.spent)?;
                self.proceeds = self.proceeds.checked_add(&trade.received)?;
            }
        }
        self.fees = self
            .fees
            .checked_add(&AssetAmount::new(r.fee_quote, pair.quote.clone()))?;
        Ok(Some(r.side))
    }

    fn bep(&self) -> EngineResult<Option<Decimal>> {
        Ok(self
            .invested
            .checked_sub(&self.proceeds)?
            .checked_add(&self.fees)?
            .amount()
            .checked_div(self.held.amount()))
    }
}

pub(crate) fn position_summary(
    pair: &AssetPair,
    trades: &[Trade],
) -> EngineResult<PositionSummary> {
    let mut acc = Accumulator::new(pair);
    for trade in trades {
        acc.apply(pair, trade)?;
    }
    Ok(PositionSummary {
        bep: acc.bep()?,
        held: acc.held,
        invested: acc.invested,
        proceeds: acc.proceeds,
        fees: acc.fees,
        buys: acc.buys,
        sells: acc.sells,
    })
}

pub(crate) fn trades_summary(pair: &AssetPair, trades: &[Trade]) -> EngineResult<TradesSummary> {
    let mut buys = 0;
    let mut sells = 0;
    let mut unknown = 0;
    let mut spent = AssetAmount::zero(pair.quote.clone());
    let mut received = AssetAmount::zero(pair.base.clone());
    let mut fees = AssetAmount::zero(pair.quote.clone());
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;

    for trade in trades {
        let r = match resolve(trade, pair) {
            Some(r) => r,
            None => {
                unknown += 1;
                continue;
            }
        };
        match r.side {
            TradeSide::Buy => {
                buys += 1;
                spent = spent.checked_add(&trade.spent)?;
                received = received.checked_add(&trade.received)?;
            }
            TradeSide::Sell => {
                sells += 1;
                spent = spent.checked_add(&trade.received)?;
                received = received.checked_add(&trade.spent)?;
            }
        }
        fees = fees.checked_add(&AssetAmount::new(r.fee_quote, pair.quote.clone()))?;

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

/// Enrich trades with running BEP and realized P&L.
pub(crate) fn enrich_trades(
    pair: &AssetPair,
    trades: Vec<Trade>,
) -> EngineResult<Vec<EnrichedTrade>> {
    let mut held = Decimal::ZERO;
    let mut invested = Decimal::ZERO;
    let mut proceeds = Decimal::ZERO;
    let mut fees_fiat = Decimal::ZERO;

    let mut enriched = Vec::with_capacity(trades.len());

    for trade in trades {
        let r = match resolve(&trade, pair) {
            Some(r) => r,
            None => {
                enriched.push(EnrichedTrade {
                    date: trade.date,
                    spent: trade.spent,
                    received: trade.received,
                    fee: trade.fee,
                    side: None,
                    bep: None,
                    pnl: None,
                });
                continue;
            }
        };

        // Capture BEP *before* applying this trade
        let bep_before = if held.is_zero() {
            None
        } else {
            (invested - proceeds + fees_fiat).checked_div(held)
        };

        let pnl = match r.side {
            TradeSide::Buy => {
                held += r.units;
                invested += r.quote_amount;
                fees_fiat += r.fee_quote;
                None
            }
            TradeSide::Sell => {
                held -= r.units;
                proceeds += r.quote_amount;
                fees_fiat += r.fee_quote;
                bep_before.map(|bep| {
                    AssetAmount::new(
                        r.quote_amount - r.fee_quote - bep * r.units,
                        pair.quote.clone(),
                    )
                })
            }
        };

        let running_bep = if held.is_zero() {
            None
        } else {
            (invested - proceeds + fees_fiat).checked_div(held)
        };

        enriched.push(EnrichedTrade {
            date: trade.date,
            spent: trade.spent,
            received: trade.received,
            fee: trade.fee,
            side: Some(r.side),
            bep: running_bep.map(|v| AssetAmount::new(v, pair.quote.clone())),
            pnl,
        });
    }

    Ok(enriched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Asset;
    use chrono::TimeZone;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    const fn btc_eur() -> AssetPair {
        AssetPair {
            base: Asset::Btc,
            quote: Asset::Eur,
        }
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

    // ── side_for ──────────────────────────────────────────

    #[test]
    fn side_for_buy_sell_none() {
        let trade = make_buy(2024, 1, 15, dec!(187.2514), dec!(0.0020104289), dec!(0.749));
        assert_eq!(trade.side_for(&btc_eur()), Some(TradeSide::Buy));

        let eur_btc = AssetPair {
            base: Asset::Eur,
            quote: Asset::Btc,
        };
        assert_eq!(trade.side_for(&eur_btc), Some(TradeSide::Sell));

        let btc_usd = AssetPair {
            base: Asset::Btc,
            quote: Asset::Usd,
        };
        assert_eq!(trade.side_for(&btc_usd), None);
    }

    // ── BEP via enrich_trades ──────────────────────────────

    #[test]
    fn enrich_two_buys_bep_is_weighted_avg() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
        ];
        let enriched = enrich_trades(&btc_eur(), trades).unwrap();
        assert_eq!(enriched.len(), 2);
        // BEP = (100+0.50)/0.001 = 100_500
        assert_eq!(
            enriched[0].bep,
            Some(AssetAmount::new(dec!(100500), Asset::Eur))
        );
        assert_eq!(enriched[0].side, Some(TradeSide::Buy));
        // BEP = (100+200+0.50+1.00)/0.004 = 75_375
        assert_eq!(
            enriched[1].bep,
            Some(AssetAmount::new(dec!(75375), Asset::Eur))
        );
        assert_eq!(enriched[1].side, Some(TradeSide::Buy));
    }

    #[test]
    fn enrich_buy_sell_bep_adjusts() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
            make_sell(2025, 3, 1, dec!(0.001), dec!(120), dec!(0.60)),
        ];
        let enriched = enrich_trades(&btc_eur(), trades).unwrap();
        assert_eq!(enriched.len(), 3);
        // After sell: BEP = (300-120+2.10)/0.003 = 60_700
        assert_eq!(
            enriched[2].bep,
            Some(AssetAmount::new(dec!(60700), Asset::Eur))
        );
        assert_eq!(enriched[2].side, Some(TradeSide::Sell));
    }

    #[test]
    fn enrich_sell_all_resets_bep() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_sell(2025, 2, 1, dec!(0.001), dec!(120), dec!(0.60)),
        ];
        let enriched = enrich_trades(&btc_eur(), trades).unwrap();
        assert_eq!(enriched.len(), 2);
        assert_eq!(enriched[1].bep, None);
        assert_eq!(enriched[1].side, Some(TradeSide::Sell));
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
        assert_eq!(pos.invested, AssetAmount::new(dec!(300), Asset::Eur));
        assert_eq!(pos.proceeds, AssetAmount::new(dec!(120), Asset::Eur));
        assert_eq!(pos.fees, AssetAmount::new(dec!(2.10), Asset::Eur));
        assert_eq!(pos.buys, 2);
        assert_eq!(pos.sells, 1);
    }

    // ── Trade summary ───────────────────────────────────────

    #[test]
    fn summary_two_buys() {
        let trades = vec![
            make_trade_at(2025, 2, 14, dec!(187.25), dec!(0.002), dec!(0.75)),
            make_trade_at(2025, 6, 1, dec!(28.37), dec!(0.0003), dec!(0.28)),
        ];
        let summary = trades_summary(&btc_eur(), &trades).unwrap();

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
    fn summary_empty() {
        let summary = trades_summary(&btc_eur(), &[]).unwrap();
        assert_eq!(summary.total_trades, 0);
        assert_eq!(summary.buys, 0);
        assert!(summary.date_range.is_none());
        assert_eq!(summary.spent.amount(), Decimal::ZERO);
    }

    #[test]
    fn summary_ignores_unrelated_pair() {
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 3, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(100), Asset::Usd),
            received: AssetAmount::new(dec!(0.05), Asset::Other("ETH".into())),
            fee: AssetAmount::new(dec!(0.5), Asset::Usd),
        };
        let summary = trades_summary(&btc_eur(), &[trade]).unwrap();

        assert_eq!(summary.total_trades, 1);
        assert_eq!(summary.buys, 0);
        assert_eq!(summary.unknown, 1);
        assert_eq!(summary.spent.amount(), Decimal::ZERO);
        assert_eq!(summary.fees.amount(), Decimal::ZERO);
        assert!(summary.date_range.is_none());
    }

    #[test]
    fn summary_btc_buy_with_wrong_fiat_is_unknown() {
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 3, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(200), Asset::Usd),
            received: AssetAmount::new(dec!(0.002), Asset::Btc),
            fee: AssetAmount::new(dec!(1), Asset::Usd),
        };
        let summary = trades_summary(&btc_eur(), &[trade]).unwrap();

        assert_eq!(summary.buys, 0);
        assert_eq!(summary.unknown, 1);
        assert_eq!(summary.spent.amount(), Decimal::ZERO);
    }

    #[test]
    fn summary_sell_trade() {
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 4, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(0.005), Asset::Btc),
            received: AssetAmount::new(dec!(300), Asset::Eur),
            fee: AssetAmount::new(dec!(1.2), Asset::Eur),
        };
        let summary = trades_summary(&btc_eur(), &[trade]).unwrap();

        assert_eq!(summary.buys, 0);
        assert_eq!(summary.sells, 1);
        assert_eq!(summary.unknown, 0);
        assert_eq!(summary.spent.amount(), dec!(300));
        assert_eq!(summary.received.amount(), dec!(0.005));
        assert_eq!(summary.fees.amount(), dec!(1.2));
    }

    // ── Enrich trades ─────────────────────────────────────────

    #[test]
    fn enrich_buy_only() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
        ];
        let enriched = enrich_trades(&btc_eur(), trades).unwrap();
        assert_eq!(enriched.len(), 2);

        // First buy: BEP = (100+0.50)/0.001 = 100_500
        assert_eq!(
            enriched[0].bep,
            Some(AssetAmount::new(dec!(100500), Asset::Eur))
        );
        assert_eq!(enriched[0].pnl, None);

        // Second buy: BEP = (100+200+0.50+1.00)/0.004 = 75_375
        assert_eq!(
            enriched[1].bep,
            Some(AssetAmount::new(dec!(75375), Asset::Eur))
        );
        assert_eq!(enriched[1].pnl, None);
    }

    #[test]
    fn enrich_buy_sell_fiat_fee() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
            make_sell(2025, 3, 1, dec!(0.001), dec!(120), dec!(0.60)),
        ];
        let enriched = enrich_trades(&btc_eur(), trades).unwrap();
        assert_eq!(enriched.len(), 3);

        let sell = &enriched[2];
        // BEP before sell was 75_375
        // P&L = 120 - 0.60 - 75_375 * 0.001 = 120 - 0.60 - 75.375 = 44.025
        assert_eq!(sell.pnl, Some(AssetAmount::new(dec!(44.025), Asset::Eur)));
        // Remaining: held=0.003, invested=300, proceeds=120, fees=2.10
        // BEP = (300-120+2.10)/0.003 = 182.10/0.003 = 60_700
        assert_eq!(sell.bep, Some(AssetAmount::new(dec!(60700), Asset::Eur)));
    }

    #[test]
    fn enrich_sell_with_btc_fee() {
        // Sell with fee denominated in BTC (Kraken edge case)
        let trades = vec![
            make_buy(2025, 1, 1, dec!(1000), dec!(0.01), dec!(5.00)),
            Trade {
                date: Utc.with_ymd_and_hms(2025, 2, 1, 12, 0, 0).unwrap(),
                spent: AssetAmount::new(dec!(0.005), Asset::Btc),
                received: AssetAmount::new(dec!(600), Asset::Eur),
                fee: AssetAmount::new(dec!(0.00005), Asset::Btc), // BTC fee!
            },
        ];
        let enriched = enrich_trades(&btc_eur(), trades).unwrap();
        assert_eq!(enriched.len(), 2);

        let sell = &enriched[1];
        // fee_fiat = 0.00005 * 120_000 = 6.0
        // BEP before sell = (1000+5)/0.01 = 100_500
        // P&L = 600 - 6.0 - 100_500 * 0.005 = 600 - 6 - 502.5 = 91.5
        assert_eq!(sell.pnl, Some(AssetAmount::new(dec!(91.5), Asset::Eur)));
        // After sell: held=0.005, invested=1000, proceeds=600, fees=5+6=11
        // BEP = (1000-600+11)/0.005 = 411/0.005 = 82_200
        assert_eq!(sell.bep, Some(AssetAmount::new(dec!(82200), Asset::Eur)));
    }
}

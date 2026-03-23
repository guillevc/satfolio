use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::errors::EngineResult;
use crate::models::{
    AssetAmount, AssetPair, DashboardStats, EnrichedTrade, PositionSummary, Trade, TradeSide,
    TradesSummary,
};

impl Trade {
    fn side_for(&self, pair: &AssetPair) -> Option<TradeSide> {
        // Reward: received is the base asset, spent is zero (free inflow)
        if self.received.asset() == &pair.base && self.spent.amount().is_zero() {
            return Some(TradeSide::Reward);
        }
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
        TradeSide::Buy | TradeSide::Reward => (trade.received.amount(), trade.spent.amount()),
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
            TradeSide::Reward => {
                self.buys += 1;
                self.held = self.held.checked_add(&trade.received)?;
                // Rewards are free BTC — they add to held but NOT invested,
                // which correctly lowers the break-even price.
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

    fn bep(&self, pair: &AssetPair) -> EngineResult<Option<AssetAmount>> {
        Ok(self
            .invested
            .checked_sub(&self.proceeds)?
            .checked_add(&self.fees)?
            .amount()
            .checked_div(self.held.amount())
            .map(|d| AssetAmount::new(d, pair.quote.clone())))
    }
}

/// Compute running position: BEP, holdings, invested, proceeds, and fees.
pub(crate) fn position_summary(
    pair: &AssetPair,
    trades: &[Trade],
) -> EngineResult<PositionSummary> {
    let mut acc = Accumulator::new(pair);
    for trade in trades {
        acc.apply(pair, trade)?;
    }
    Ok(PositionSummary {
        bep: acc.bep(pair)?,
        held: acc.held,
        invested: acc.invested,
        proceeds: acc.proceeds,
        fees: acc.fees,
        buys: acc.buys,
        sells: acc.sells,
    })
}

/// Aggregate trade counts, volumes, fees, and date range for a set of trades.
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
            TradeSide::Buy | TradeSide::Reward => {
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
        total_trades: buys + sells,
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
    let mut acc = Accumulator::new(pair);
    let mut enriched = Vec::with_capacity(trades.len());

    for trade in trades {
        let r = match resolve(&trade, pair) {
            Some(r) => r,
            None => {
                enriched.push(EnrichedTrade {
                    date: trade.date,
                    provider: trade.provider,
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

        let bep_before = acc.bep(pair)?;
        acc.apply(pair, &trade)?;
        let bep_after = acc.bep(pair)?;

        let pnl = match (&r.side, bep_before.as_ref()) {
            (TradeSide::Sell, Some(bep)) => Some(AssetAmount::new(
                r.quote_amount - r.fee_quote - bep.amount() * r.units,
                pair.quote.clone(),
            )),
            _ => None,
        };

        enriched.push(EnrichedTrade {
            date: trade.date,
            provider: trade.provider,
            spent: trade.spent,
            received: trade.received,
            fee: trade.fee,
            side: Some(r.side),
            bep: bep_after,
            pnl,
        });
    }

    Ok(enriched)
}

/// Build dashboard stats from a position summary and current/previous prices.
pub(crate) fn dashboard_stats(
    summary: &PositionSummary,
    current_price: Decimal,
    prev_price: Option<Decimal>,
) -> DashboardStats {
    // held is in base (BTC), invested/proceeds/fees are in quote (EUR)
    let quote = summary.invested.asset().clone();
    let held_amount = summary.held.amount();

    let position_value = current_price * held_amount;

    let unrealized_pnl = match &summary.bep {
        Some(bep) => (current_price - bep.amount()) * held_amount,
        None => Decimal::ZERO,
    };

    let invested = summary.invested.amount();
    let unrealized_pnl_pct = if invested.is_zero() {
        Decimal::ZERO
    } else {
        unrealized_pnl / invested
    };

    let change_24h_pct = match prev_price {
        Some(prev) if !prev.is_zero() => (current_price - prev) / prev,
        _ => Decimal::ZERO,
    };

    DashboardStats {
        btc_price: AssetAmount::new(current_price, quote.clone()),
        change_24h_pct,
        bep: summary.bep.clone(),
        trade_count: summary.buys + summary.sells,
        held: summary.held.clone(),
        position_value: AssetAmount::new(position_value, quote.clone()),
        unrealized_pnl: AssetAmount::new(unrealized_pnl, quote),
        unrealized_pnl_pct,
        candles: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Asset, Provider};
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
            provider: Provider::Kraken,
        }
    }

    fn make_sell(y: i32, m: u32, d: u32, btc: Decimal, eur: Decimal, fee: Decimal) -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(btc, Asset::Btc),
            received: AssetAmount::new(eur, Asset::Eur),
            fee: AssetAmount::new(fee, Asset::Eur),
            provider: Provider::Kraken,
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
        assert_eq!(pos.bep, Some(AssetAmount::new(dec!(60700), Asset::Eur)));
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
            make_buy(2025, 2, 14, dec!(187.25), dec!(0.002), dec!(0.75)),
            make_buy(2025, 6, 1, dec!(28.37), dec!(0.0003), dec!(0.28)),
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
            provider: Provider::Kraken,
        };
        let summary = trades_summary(&btc_eur(), &[trade]).unwrap();

        assert_eq!(summary.total_trades, 0);
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
            provider: Provider::Kraken,
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
            provider: Provider::Kraken,
        };
        let summary = trades_summary(&btc_eur(), &[trade]).unwrap();

        assert_eq!(summary.buys, 0);
        assert_eq!(summary.sells, 1);
        assert_eq!(summary.unknown, 0);
        assert_eq!(summary.spent.amount(), dec!(300));
        assert_eq!(summary.received.amount(), dec!(0.005));
        assert_eq!(summary.fees.amount(), dec!(1.2));
    }

    #[test]
    fn summary_mixed_known_and_unknown() {
        // 2 buys + 1 sell (BTC/EUR, known) + 2 unknown trades (BTC/USD, ETH/USD)
        // Unknown trades are placed outside the known date range to verify
        // that date_range only spans known trades.
        let known_trades = vec![
            make_buy(2025, 1, 15, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 15, dec!(200), dec!(0.002), dec!(1.00)),
            make_sell(2025, 3, 15, dec!(0.0005), dec!(60), dec!(0.30)),
        ];
        let unknown_btc_usd = Trade {
            date: Utc.with_ymd_and_hms(2024, 12, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(500), Asset::Usd),
            received: AssetAmount::new(dec!(0.005), Asset::Btc),
            fee: AssetAmount::new(dec!(2), Asset::Usd),
            provider: Provider::Kraken,
        };
        let unknown_eth_usd = Trade {
            date: Utc.with_ymd_and_hms(2024, 12, 15, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(300), Asset::Usd),
            received: AssetAmount::new(dec!(0.1), Asset::Other("ETH".into())),
            fee: AssetAmount::new(dec!(1.5), Asset::Usd),
            provider: Provider::Kraken,
        };

        let mut all_trades = vec![unknown_btc_usd, unknown_eth_usd];
        all_trades.extend(known_trades);
        let summary = trades_summary(&btc_eur(), &all_trades).unwrap();

        // total_trades = buys + sells (excludes unknown)
        assert_eq!(summary.buys, 2);
        assert_eq!(summary.sells, 1);
        assert_eq!(summary.unknown, 2);
        assert_eq!(summary.total_trades, 3);

        // Monetary totals reflect only known trades
        // spent = 100 + 200 (buys) + 60 (sell received) = 360
        assert_eq!(summary.spent.amount(), dec!(360));
        assert_eq!(*summary.spent.asset(), Asset::Eur);
        // received = 0.001 + 0.002 (buys) + 0.0005 (sell spent) = 0.0035
        assert_eq!(summary.received.amount(), dec!(0.0035));
        assert_eq!(*summary.received.asset(), Asset::Btc);
        // fees = 0.50 + 1.00 + 0.30 = 1.80
        assert_eq!(summary.fees.amount(), dec!(1.80));
        assert_eq!(*summary.fees.asset(), Asset::Eur);

        // date_range spans only Jan–Mar 2025 (known trades), not Dec 2024 (unknown)
        let (earliest, latest) = summary.date_range.unwrap();
        assert_eq!(
            earliest,
            Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap()
        );
        assert_eq!(latest, Utc.with_ymd_and_hms(2025, 3, 15, 12, 0, 0).unwrap());
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
                provider: Provider::Kraken,
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

    // ── Dashboard stats ─────────────────────────────────────

    fn make_position(
        bep: Option<Decimal>,
        held: Decimal,
        invested: Decimal,
        proceeds: Decimal,
        fees: Decimal,
        buys: usize,
        sells: usize,
    ) -> PositionSummary {
        PositionSummary {
            bep: bep.map(|b| AssetAmount::new(b, Asset::Eur)),
            held: AssetAmount::new(held, Asset::Btc),
            invested: AssetAmount::new(invested, Asset::Eur),
            proceeds: AssetAmount::new(proceeds, Asset::Eur),
            fees: AssetAmount::new(fees, Asset::Eur),
            buys,
            sells,
        }
    }

    #[test]
    fn dashboard_stats_basic() {
        // BEP=60_700, held=0.003, invested=300, proceeds=120, fees=2.10
        let pos = make_position(
            Some(dec!(60700)),
            dec!(0.003),
            dec!(300),
            dec!(120),
            dec!(2.10),
            2,
            1,
        );
        let stats = dashboard_stats(&pos, dec!(90000), Some(dec!(88000)));

        // position_value = 90000 * 0.003 = 270
        assert_eq!(
            stats.position_value,
            AssetAmount::new(dec!(270), Asset::Eur)
        );
        // unrealized_pnl = (90000 - 60700) * 0.003 = 29300 * 0.003 = 87.9
        assert_eq!(
            stats.unrealized_pnl,
            AssetAmount::new(dec!(87.9), Asset::Eur)
        );
        // unrealized_pnl_pct = 87.9 / 300 = 0.293
        assert_eq!(stats.unrealized_pnl_pct, dec!(0.293));
        // change_24h_pct = (90000 - 88000) / 88000 (0–1 fraction)
        let expected_change = dec!(2000) / dec!(88000);
        assert_eq!(stats.change_24h_pct, expected_change);
        assert_eq!(stats.trade_count, 3);
        assert_eq!(stats.btc_price, AssetAmount::new(dec!(90000), Asset::Eur));
    }

    #[test]
    fn dashboard_stats_no_bep() {
        // Position fully closed: no BEP, no held
        let pos = make_position(None, dec!(0), dec!(300), dec!(300), dec!(2), 2, 2);
        let stats = dashboard_stats(&pos, dec!(90000), Some(dec!(85000)));

        assert_eq!(stats.unrealized_pnl, AssetAmount::new(dec!(0), Asset::Eur));
        assert_eq!(stats.unrealized_pnl_pct, Decimal::ZERO);
        assert_eq!(stats.bep, None);
    }

    #[test]
    fn dashboard_stats_no_prev_price() {
        let pos = make_position(
            Some(dec!(50000)),
            dec!(0.01),
            dec!(500),
            dec!(0),
            dec!(1),
            1,
            0,
        );
        let stats = dashboard_stats(&pos, dec!(60000), None);

        assert_eq!(stats.change_24h_pct, Decimal::ZERO);
        // unrealized_pnl = (60000 - 50000) * 0.01 = 100
        assert_eq!(
            stats.unrealized_pnl,
            AssetAmount::new(dec!(100), Asset::Eur)
        );
    }

    #[test]
    fn dashboard_stats_no_investment() {
        // Edge: no invested amount (shouldn't happen normally, but guard against div-by-zero)
        let pos = make_position(None, dec!(0), dec!(0), dec!(0), dec!(0), 0, 0);
        let stats = dashboard_stats(&pos, dec!(90000), Some(dec!(90000)));

        assert_eq!(stats.unrealized_pnl_pct, Decimal::ZERO);
        assert_eq!(stats.change_24h_pct, Decimal::ZERO);
        assert_eq!(stats.trade_count, 0);
    }

    // ── Unrelated trades in enrich ───────────────────────────

    #[test]
    fn enrich_unrelated_trade_passes_through() {
        let eth_usd = Trade {
            date: Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(100), Asset::Usd),
            received: AssetAmount::new(dec!(0.05), Asset::Other("ETH".into())),
            fee: AssetAmount::new(dec!(0.5), Asset::Usd),
            provider: Provider::Kraken,
        };
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            eth_usd,
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
        ];
        let enriched = enrich_trades(&btc_eur(), trades).unwrap();
        assert_eq!(enriched.len(), 3);

        // Unrelated trade gets None for side/bep/pnl
        let unrelated = &enriched[1];
        assert_eq!(unrelated.side, None);
        assert_eq!(unrelated.bep, None);
        assert_eq!(unrelated.pnl, None);

        // BEP after third trade should only reflect BTC/EUR buys, not the ETH/USD trade
        // BEP = (100+200+0.50+1.00)/0.004 = 75_375
        assert_eq!(
            enriched[2].bep,
            Some(AssetAmount::new(dec!(75375), Asset::Eur))
        );
    }

    // ── Reward handling ────────────────────────────────────

    fn make_reward(y: i32, m: u32, d: u32, btc: Decimal) -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap(),
            spent: AssetAmount::zero(Asset::Btc),
            received: AssetAmount::new(btc, Asset::Btc),
            fee: AssetAmount::zero(Asset::Btc),
            provider: Provider::Kraken,
        }
    }

    #[test]
    fn reward_detected_as_reward_side() {
        let reward = make_reward(2025, 4, 1, dec!(0.0001));
        assert_eq!(reward.side_for(&btc_eur()), Some(TradeSide::Reward));
    }

    #[test]
    fn reward_lowers_bep() {
        // Buy 0.001 BTC for 100 EUR (fee 0.50) → BEP = (100+0.50)/0.001 = 100_500
        // Then receive 0.001 BTC reward (free) → held = 0.002, invested still 100
        // BEP = (100 - 0 + 0.50) / 0.002 = 50_250
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_reward(2025, 2, 1, dec!(0.001)),
        ];
        let enriched = enrich_trades(&btc_eur(), trades).unwrap();
        assert_eq!(enriched.len(), 2);

        // First: buy
        assert_eq!(enriched[0].side, Some(TradeSide::Buy));
        assert_eq!(
            enriched[0].bep,
            Some(AssetAmount::new(dec!(100500), Asset::Eur))
        );

        // Second: reward — BEP should halve (double the held, same invested)
        assert_eq!(enriched[1].side, Some(TradeSide::Reward));
        assert_eq!(
            enriched[1].bep,
            Some(AssetAmount::new(dec!(50250), Asset::Eur))
        );
        assert_eq!(enriched[1].pnl, None); // No realized P&L for rewards
    }

    #[test]
    fn non_btc_reward_is_unknown() {
        // ETH reward should not match BTC/EUR pair
        let eth_reward = Trade {
            date: Utc.with_ymd_and_hms(2025, 4, 1, 12, 0, 0).unwrap(),
            spent: AssetAmount::zero(Asset::Other("ETH".into())),
            received: AssetAmount::new(dec!(0.001), Asset::Other("ETH".into())),
            fee: AssetAmount::zero(Asset::Other("ETH".into())),
            provider: Provider::Kraken,
        };
        assert_eq!(eth_reward.side_for(&btc_eur()), None);
    }

    #[test]
    fn sell_after_reward_uses_lowered_bep() {
        // Buy 0.002 BTC for 200 EUR (fee 1) → BEP = 201/0.002 = 100_500
        // Reward 0.001 BTC → held = 0.003, BEP = 201/0.003 = 67_000
        // Sell 0.001 BTC for 80 EUR (fee 0.40)
        // P&L = 80 - 0.40 - 67_000 * 0.001 = 80 - 0.40 - 67 = 12.60
        let trades = vec![
            make_buy(2025, 1, 1, dec!(200), dec!(0.002), dec!(1)),
            make_reward(2025, 2, 1, dec!(0.001)),
            make_sell(2025, 3, 1, dec!(0.001), dec!(80), dec!(0.40)),
        ];
        let enriched = enrich_trades(&btc_eur(), trades).unwrap();
        assert_eq!(enriched.len(), 3);

        let sell = &enriched[2];
        assert_eq!(sell.side, Some(TradeSide::Sell));
        assert_eq!(sell.pnl, Some(AssetAmount::new(dec!(12.60), Asset::Eur)));
    }

    #[test]
    fn summary_mixed_buy_and_sell() {
        let trades = vec![
            make_buy(2025, 1, 1, dec!(100), dec!(0.001), dec!(0.50)),
            make_buy(2025, 2, 1, dec!(200), dec!(0.003), dec!(1.00)),
            make_sell(2025, 3, 1, dec!(0.001), dec!(120), dec!(0.60)),
        ];
        let summary = trades_summary(&btc_eur(), &trades).unwrap();

        assert_eq!(summary.total_trades, 3);
        assert_eq!(summary.buys, 2);
        assert_eq!(summary.sells, 1);
        assert_eq!(summary.unknown, 0);
        // spent accumulates buy spent + sell received (cross-wiring)
        // Buys: 100 + 200 = 300, Sell: 120 → total spent = 420
        assert_eq!(summary.spent.amount(), dec!(420));
        assert_eq!(*summary.spent.asset(), Asset::Eur);
        // received accumulates buy received + sell spent
        // Buys: 0.001 + 0.003 = 0.004, Sell: 0.001 → total received = 0.005
        assert_eq!(summary.received.amount(), dec!(0.005));
        assert_eq!(*summary.received.asset(), Asset::Btc);
        // fees: 0.50 + 1.00 + 0.60 = 2.10
        assert_eq!(summary.fees.amount(), dec!(2.10));
    }
}

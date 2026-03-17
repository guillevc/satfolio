use std::collections::HashMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::models::{Asset, AssetAmount, SUPPORTED_FIATS, Trade};

/// Converts fiat amounts between currencies using BTC-derived cross rates.
///
/// Cross-rate formula: `from→to on date D = BTC_close_to(D) / BTC_close_from(D)`
/// This is an approximation — using daily close prices introduces rounding,
/// but `rust_decimal`'s 28-digit precision makes it negligible.
pub struct CrossRateConverter {
    rates: HashMap<Asset, HashMap<NaiveDate, Decimal>>,
}

impl CrossRateConverter {
    pub fn new(rates: HashMap<Asset, HashMap<NaiveDate, Decimal>>) -> Self {
        Self { rates }
    }

    /// Convert an amount from one fiat to another using BTC cross-rate on the given date.
    /// Falls back to nearest prior close if exact date is missing.
    pub fn convert(
        &self,
        amount: Decimal,
        from: &Asset,
        to: &Asset,
        date: NaiveDate,
    ) -> Option<Decimal> {
        if from == to {
            return Some(amount);
        }
        let btc_from = self.close_on_or_before(from, date)?;
        let btc_to = self.close_on_or_before(to, date)?;
        if btc_from.is_zero() {
            return None;
        }
        // cross_rate = btc_to / btc_from  (how many `to` per `from`)
        Some(amount * btc_to / btc_from)
    }

    /// Normalize a trade's fiat amounts to the target currency.
    /// Returns the trade as-is if already in the target currency.
    /// Returns the converted trade if in a different supported fiat.
    /// Returns None if the trade has no recognizable BTC/fiat pair.
    pub fn normalize_trade(&self, trade: Trade, target: &Asset) -> Option<Trade> {
        let (fiat, is_spent_fiat) = identify_fiat_side(&trade)?;

        if fiat == target {
            return Some(trade);
        }

        let date = trade.date.date_naive();

        if is_spent_fiat {
            // Buy: spent=fiat, received=BTC
            let spent_converted = self.convert(trade.spent.amount(), fiat, target, date)?;
            let fee_converted = if trade.fee.asset() == fiat {
                self.convert(trade.fee.amount(), fiat, target, date)?
            } else {
                // BTC fee — leave as-is (engine converts via trade price)
                trade.fee.amount()
            };
            Some(Trade {
                spent: AssetAmount::new(spent_converted, target.clone()),
                fee: AssetAmount::new(
                    fee_converted,
                    if trade.fee.asset() == fiat {
                        target.clone()
                    } else {
                        trade.fee.asset().clone()
                    },
                ),
                ..trade
            })
        } else {
            // Sell: spent=BTC, received=fiat
            let received_converted = self.convert(trade.received.amount(), fiat, target, date)?;
            let fee_converted = if trade.fee.asset() == fiat {
                self.convert(trade.fee.amount(), fiat, target, date)?
            } else {
                trade.fee.amount()
            };
            Some(Trade {
                received: AssetAmount::new(received_converted, target.clone()),
                fee: AssetAmount::new(
                    fee_converted,
                    if trade.fee.asset() == fiat {
                        target.clone()
                    } else {
                        trade.fee.asset().clone()
                    },
                ),
                ..trade
            })
        }
    }

    /// Look up close price on exact date, or scan backwards up to 7 days.
    fn close_on_or_before(&self, asset: &Asset, date: NaiveDate) -> Option<Decimal> {
        let prices = self.rates.get(asset)?;
        for days_back in 0..=7 {
            if let Some(d) = date.checked_sub_days(chrono::Days::new(days_back))
                && let Some(&close) = prices.get(&d)
            {
                return Some(close);
            }
        }
        None
    }
}

/// Identify which side of the trade is fiat and return (fiat_asset, is_spent_fiat).
fn identify_fiat_side(trade: &Trade) -> Option<(&Asset, bool)> {
    let spent = trade.spent.asset();
    let received = trade.received.asset();

    let spent_is_fiat = SUPPORTED_FIATS.contains(spent);
    let received_is_fiat = SUPPORTED_FIATS.contains(received);

    match (spent_is_fiat, received_is_fiat) {
        (true, false) => Some((spent, true)), // Buy: spent fiat, received BTC
        (false, true) => Some((received, false)), // Sell: spent BTC, received fiat
        _ => None,                            // Both fiat, both non-fiat, or unrelated pair
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    use crate::models::Provider;

    fn make_rates() -> HashMap<Asset, HashMap<NaiveDate, Decimal>> {
        let mut map = HashMap::new();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        // EUR: BTC/EUR = 90_000
        let mut eur = HashMap::new();
        eur.insert(date, dec!(90000));
        map.insert(Asset::Eur, eur);

        // USD: BTC/USD = 100_000
        let mut usd = HashMap::new();
        usd.insert(date, dec!(100000));
        map.insert(Asset::Usd, usd);

        // GBP: BTC/GBP = 80_000
        let mut gbp = HashMap::new();
        gbp.insert(date, dec!(80000));
        map.insert(Asset::Gbp, gbp);

        map
    }

    fn make_buy_trade(
        fiat: Asset,
        fiat_amount: Decimal,
        btc_amount: Decimal,
        fee: Decimal,
    ) -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(fiat_amount, fiat.clone()),
            received: AssetAmount::new(btc_amount, Asset::Btc),
            fee: AssetAmount::new(fee, fiat),
            provider: Provider::Kraken,
        }
    }

    fn make_sell_trade(
        fiat: Asset,
        fiat_amount: Decimal,
        btc_amount: Decimal,
        fee: Decimal,
    ) -> Trade {
        Trade {
            date: Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(btc_amount, Asset::Btc),
            received: AssetAmount::new(fiat_amount, fiat.clone()),
            fee: AssetAmount::new(fee, fiat),
            provider: Provider::Kraken,
        }
    }

    #[test]
    fn same_currency_passthrough() {
        let conv = CrossRateConverter::new(make_rates());
        let trade = make_buy_trade(Asset::Eur, dec!(900), dec!(0.01), dec!(3.6));
        let result = conv.normalize_trade(trade.clone(), &Asset::Eur).unwrap();
        assert_eq!(result, trade);
    }

    #[test]
    fn eur_to_usd_buy_conversion() {
        let conv = CrossRateConverter::new(make_rates());
        // EUR buy: 900 EUR for 0.01 BTC, 3.6 EUR fee
        let trade = make_buy_trade(Asset::Eur, dec!(900), dec!(0.01), dec!(3.6));
        let result = conv.normalize_trade(trade, &Asset::Usd).unwrap();

        // Cross rate EUR→USD = 100_000/90_000 = 10/9
        // 900 * 10/9 = 1000
        assert_eq!(result.spent.amount(), dec!(1000));
        assert_eq!(*result.spent.asset(), Asset::Usd);
        // BTC amount unchanged
        assert_eq!(result.received.amount(), dec!(0.01));
        assert_eq!(*result.received.asset(), Asset::Btc);
        // Fee: 3.6 * 10/9 = 4.0
        assert_eq!(result.fee.amount(), dec!(4.0));
        assert_eq!(*result.fee.asset(), Asset::Usd);
    }

    #[test]
    fn eur_to_usd_sell_conversion() {
        let conv = CrossRateConverter::new(make_rates());
        // EUR sell: 0.01 BTC for 900 EUR, 3.6 EUR fee
        let trade = make_sell_trade(Asset::Eur, dec!(900), dec!(0.01), dec!(3.6));
        let result = conv.normalize_trade(trade, &Asset::Usd).unwrap();

        assert_eq!(result.received.amount(), dec!(1000));
        assert_eq!(*result.received.asset(), Asset::Usd);
        assert_eq!(result.spent.amount(), dec!(0.01));
        assert_eq!(*result.spent.asset(), Asset::Btc);
        assert_eq!(result.fee.amount(), dec!(4.0));
        assert_eq!(*result.fee.asset(), Asset::Usd);
    }

    #[test]
    fn btc_fee_left_as_is() {
        let conv = CrossRateConverter::new(make_rates());
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(900), Asset::Eur),
            received: AssetAmount::new(dec!(0.01), Asset::Btc),
            fee: AssetAmount::new(dec!(0.00001), Asset::Btc), // BTC fee
            provider: Provider::Kraken,
        };
        let result = conv.normalize_trade(trade, &Asset::Usd).unwrap();
        // BTC fee should remain as BTC
        assert_eq!(result.fee.amount(), dec!(0.00001));
        assert_eq!(*result.fee.asset(), Asset::Btc);
    }

    #[test]
    fn unsupported_pair_returns_none() {
        let conv = CrossRateConverter::new(make_rates());
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(100), Asset::Other("ETH".into())),
            received: AssetAmount::new(dec!(0.05), Asset::Other("SOL".into())),
            fee: AssetAmount::new(dec!(0.5), Asset::Other("ETH".into())),
            provider: Provider::Kraken,
        };
        assert!(conv.normalize_trade(trade, &Asset::Usd).is_none());
    }

    #[test]
    fn missing_date_falls_back_to_prior() {
        let mut rates = make_rates();
        // Only have data for Jan 12, query on Jan 15 (3 days later)
        let eur = rates.get_mut(&Asset::Eur).unwrap();
        eur.remove(&NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
        eur.insert(NaiveDate::from_ymd_opt(2025, 1, 12).unwrap(), dec!(90000));
        let usd = rates.get_mut(&Asset::Usd).unwrap();
        usd.remove(&NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
        usd.insert(NaiveDate::from_ymd_opt(2025, 1, 12).unwrap(), dec!(100000));

        let conv = CrossRateConverter::new(rates);
        let trade = make_buy_trade(Asset::Eur, dec!(900), dec!(0.01), dec!(3.6));
        let result = conv.normalize_trade(trade, &Asset::Usd).unwrap();
        assert_eq!(result.spent.amount(), dec!(1000));
    }

    #[test]
    fn convert_same_currency() {
        let conv = CrossRateConverter::new(make_rates());
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        assert_eq!(
            conv.convert(dec!(100), &Asset::Eur, &Asset::Eur, date),
            Some(dec!(100))
        );
    }

    #[test]
    fn convert_eur_to_gbp() {
        let conv = CrossRateConverter::new(make_rates());
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        // EUR→GBP = 80_000/90_000 = 8/9
        // 900 * 8/9 = 800
        let result = conv.convert(dec!(900), &Asset::Eur, &Asset::Gbp, date);
        assert_eq!(result, Some(dec!(800)));
    }
}

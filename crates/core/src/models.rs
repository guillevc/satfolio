use std::fmt;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::errors::AssetMismatch;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(from = "String")]
pub enum Asset {
    Btc,
    Eur,
    Gbp,
    Usd,
    Other(String),
}

impl From<String> for Asset {
    fn from(s: String) -> Self {
        match s.as_str() {
            "BTC" | "XBT" => Self::Btc,
            "EUR" | "ZEUR" => Self::Eur,
            "GBP" | "ZGBP" => Self::Gbp,
            "USD" | "ZUSD" => Self::Usd,
            _ => Self::Other(s),
        }
    }
}

impl Asset {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Btc => "BTC",
            Self::Eur => "EUR",
            Self::Gbp => "GBP",
            Self::Usd => "USD",
            Self::Other(o) => o,
        }
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct AssetPair {
    pub base: Asset,
    pub quote: Asset,
}

impl fmt::Display for AssetPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct AssetAmount {
    amount: Decimal,
    asset: Asset,
}

impl AssetAmount {
    pub fn new(amount: Decimal, asset: Asset) -> Self {
        Self { amount, asset }
    }

    pub fn zero(asset: Asset) -> Self {
        Self::new(Decimal::ZERO, asset)
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn asset(&self) -> &Asset {
        &self.asset
    }

    pub fn checked_add(&self, rhs: &AssetAmount) -> Result<AssetAmount, AssetMismatch> {
        if self.asset != rhs.asset {
            return Err(AssetMismatch {
                expected: self.asset.clone(),
                got: rhs.asset.clone(),
            });
        }
        Ok(Self::new(self.amount + rhs.amount, self.asset.clone()))
    }

    pub fn checked_sub(&self, rhs: &AssetAmount) -> Result<AssetAmount, AssetMismatch> {
        if self.asset != rhs.asset {
            return Err(AssetMismatch {
                expected: self.asset.clone(),
                got: rhs.asset.clone(),
            });
        }
        Ok(Self::new(self.amount - rhs.amount, self.asset.clone()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Trade {
    pub date: DateTime<Utc>,
    pub spent: AssetAmount,
    pub received: AssetAmount,
    pub fee: AssetAmount,
}

impl Trade {
    pub fn side_for(&self, pair: &AssetPair) -> Option<TradeSide> {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
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

// TODO: migrate date to DateTime<Utc> for consistency with Trade and Candle
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BepSnapshot {
    pub date: NaiveDate,
    pub held: AssetAmount,
    pub invested: AssetAmount,
    pub proceeds: AssetAmount,
    pub fees: AssetAmount,
    pub bep: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PositionSummary {
    pub bep: Option<Decimal>,
    pub held: AssetAmount,
    pub invested: AssetAmount,
    pub proceeds: AssetAmount,
    pub fees: AssetAmount,
    pub buys: usize,
    pub sells: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candle {
    pub date: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rust_decimal_macros::dec;

    #[test]
    fn asset_from_btc() {
        assert_eq!(Asset::from("BTC".to_string()), Asset::Btc);
    }

    #[test]
    fn asset_from_xbt_is_btc() {
        assert_eq!(Asset::from("XBT".to_string()), Asset::Btc);
    }

    #[test]
    fn asset_from_eur() {
        assert_eq!(Asset::from("EUR".to_string()), Asset::Eur);
    }

    #[test]
    fn asset_from_gbp() {
        assert_eq!(Asset::from("GBP".to_string()), Asset::Gbp);
    }

    #[test]
    fn asset_from_usd() {
        assert_eq!(Asset::from("USD".to_string()), Asset::Usd);
    }

    #[test]
    fn asset_from_kraken_prefixed() {
        assert_eq!(Asset::from("ZEUR".to_string()), Asset::Eur);
        assert_eq!(Asset::from("ZGBP".to_string()), Asset::Gbp);
        assert_eq!(Asset::from("ZUSD".to_string()), Asset::Usd);
    }

    #[test]
    fn asset_from_unknown() {
        assert_eq!(
            Asset::from("MSC".to_string()),
            Asset::Other("MSC".to_string())
        );
    }

    #[test]
    fn asset_as_str_roundtrip() {
        assert_eq!(Asset::Btc.as_str(), "BTC");
        assert_eq!(Asset::Eur.as_str(), "EUR");
        assert_eq!(Asset::Gbp.as_str(), "GBP");
        assert_eq!(Asset::Usd.as_str(), "USD");
        assert_eq!(Asset::Other("MSC".to_string()).as_str(), "MSC");
    }

    #[test]
    fn trade_side_for() {
        let btc_eur = AssetPair {
            base: Asset::Btc,
            quote: Asset::Eur,
        };
        let trade = Trade {
            date: Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
            spent: AssetAmount::new(dec!(187.2514), Asset::Eur),
            received: AssetAmount::new(dec!(0.0020104289), Asset::Btc),
            fee: AssetAmount::new(dec!(0.749), Asset::Eur),
        };
        assert_eq!(trade.side_for(&btc_eur), Some(TradeSide::Buy));
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
}

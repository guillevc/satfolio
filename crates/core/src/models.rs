use std::fmt;
use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use ts_rs::TS;

use crate::errors::AssetMismatch;

pub struct AppConfig {
    pub db_path: PathBuf,
    pub quote: Asset,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, type = "string"))]
#[serde(from = "String", into = "String")]
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

impl From<Asset> for String {
    fn from(a: Asset) -> Self {
        a.as_str().to_string()
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub struct AssetAmount {
    #[cfg_attr(test, ts(as = "String"))]
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
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Trade {
    pub(crate) date: DateTime<Utc>,
    pub(crate) spent: AssetAmount,
    pub(crate) received: AssetAmount,
    pub(crate) fee: AssetAmount,
}

/// Trade enriched with computed analytics for display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub struct EnrichedTrade {
    pub date: DateTime<Utc>,
    pub spent: AssetAmount,
    pub received: AssetAmount,
    pub fee: AssetAmount,
    /// Trade side relative to the tracked pair. None for unrelated trades.
    pub side: Option<TradeSide>,
    /// Break-even price in quote currency. None if position is fully closed.
    pub bep: Option<AssetAmount>,
    /// Realized P&L in quote currency. None for buys.
    pub pnl: Option<AssetAmount>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub struct PositionSummary {
    #[cfg_attr(test, ts(as = "Option<String>"))]
    pub bep: Option<Decimal>,
    pub held: AssetAmount,
    pub invested: AssetAmount,
    pub proceeds: AssetAmount,
    pub fees: AssetAmount,
    pub buys: usize,
    pub sells: usize,
}

/// Pre-computed dashboard metrics for the frontend stat cards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub struct DashboardStats {
    pub btc_price: AssetAmount,
    #[cfg_attr(test, ts(as = "String"))]
    pub change_24h_pct: Decimal,
    pub bep: Option<AssetAmount>,
    pub trade_count: usize,
    pub held: AssetAmount,
    pub position_value: AssetAmount,
    pub unrealized_pnl: AssetAmount,
    #[cfg_attr(test, ts(as = "String"))]
    pub unrealized_pnl_pct: Decimal,
    pub candles: Vec<Candle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub struct Candle {
    pub date: NaiveDate,
    #[cfg_attr(test, ts(as = "String"))]
    pub open: Decimal,
    #[cfg_attr(test, ts(as = "String"))]
    pub high: Decimal,
    #[cfg_attr(test, ts(as = "String"))]
    pub low: Decimal,
    #[cfg_attr(test, ts(as = "String"))]
    pub close: Decimal,
    #[cfg_attr(test, ts(as = "String"))]
    pub volume: Decimal,
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

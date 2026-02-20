use std::fmt;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(from = "String")]
pub enum Asset {
    Btc,
    Eur,
    Other(String),
}

impl From<String> for Asset {
    fn from(s: String) -> Self {
        match s.as_str() {
            "BTC" | "XBT" => Self::Btc,
            "EUR" => Self::Eur,
            _ => Self::Other(s),
        }
    }
}

impl Asset {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Btc => "BTC",
            Self::Eur => "EUR",
            Self::Other(o) => o,
        }
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.to_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct AssetAmount {
    pub amount: Decimal,
    pub asset: Asset,
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
    pub fn side_for(&self, asset: &Asset) -> Option<TradeSide> {
        match asset {
            a if *a == self.spent.asset => Some(TradeSide::Sell),
            a if *a == self.received.asset => Some(TradeSide::Buy),
            _ => None,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BepSnapshot {
    pub date: DateTime<Utc>,
    pub asset_held: Decimal,
    pub counter_spent: Decimal,
    pub counter_received: Decimal,
    pub fees: Decimal,
    pub bep: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DashboardStats {
    pub bep: Option<Decimal>,
    pub asset_held: Decimal,
    pub total_spent: Decimal,
    pub total_received: Decimal,
    pub total_fees: Decimal,
    pub buys: usize,
    pub sells: usize,
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
    fn asset_from_unknown() {
        assert_eq!(
            Asset::from("MSC".to_string()),
            Asset::Other("MSC".to_string())
        );
    }

    #[test]
    fn asset_to_str_roundtrip() {
        assert_eq!(Asset::Btc.to_str(), "BTC");
        assert_eq!(Asset::Other("MSC".to_string()).to_str(), "MSC");
    }

    #[test]
    fn trade_side_for() {
        let trade = Trade {
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
        };
        assert_eq!(trade.side_for(&Asset::Btc), Some(TradeSide::Buy));
        assert_eq!(trade.side_for(&Asset::Eur), Some(TradeSide::Sell));
    }
}

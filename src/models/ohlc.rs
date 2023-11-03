use crate::Exchange;

use super::price;

#[derive(Debug, Clone, Default, PartialEq)]
///
/// OHLC packet structure
///
pub struct OHLC {
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
}

impl OHLC {
  pub(crate) fn from(value: &[u8], exchange: &Exchange) -> Option<Self> {
    if let Some(bs) = value.get(0..16) {
      Some(OHLC {
        open: price(&bs[0..=3], exchange).unwrap(),
        high: price(&bs[4..=7], exchange).unwrap(),
        low: price(&bs[8..=11], exchange).unwrap(),
        close: price(&bs[12..=15], exchange).unwrap(),
      })
    } else {
      None
    }
  }
}

use std::time::Duration;

use crate::{Depth, Exchange, Mode, OHLC};

use super::{price, value};

#[derive(Debug, Clone, Default)]
///
/// Quote packet structure
///
pub struct Tick {
  pub mode: Mode,
  pub instrument_token: u32,
  pub exchange: Exchange,
  pub is_tradable: bool,
  pub is_index: bool,

  pub last_traded_qty: Option<u32>,
  pub avg_traded_price: Option<f64>,
  pub last_price: Option<f64>,
  pub volume_traded: Option<u32>,
  pub total_buy_qty: Option<u32>,
  pub total_sell_qty: Option<u32>,
  pub ohlc: Option<OHLC>,

  pub last_traded_timestamp: Option<Duration>,
  pub oi: Option<u32>,
  pub oi_day_high: Option<u32>,
  pub oi_day_low: Option<u32>,
  pub exchange_timestamp: Option<Duration>,

  pub net_change: Option<f64>,
  pub depth: Option<Depth>,
}

impl Tick {
  fn set_instrument_token(&mut self, input: &[u8]) -> &mut Self {
    self.instrument_token = value(&input[0..=3]).unwrap();
    self.exchange = ((self.instrument_token & 0xFF) as usize).into();
    self
  }

  fn set_change(&mut self) -> &mut Self {
    self.net_change = self
      .ohlc
      .as_ref()
      .map(|o| o.close)
      .map(|close_price| {
        if let Some(last_price) = self.last_price {
          if close_price == 0_f64 {
            return None;
          } else {
            // Some(((last_price - close_price) * 100.0).div(close_price))
            Some(last_price - close_price)
          }
        } else {
          None
        }
      })
      .unwrap_or_default();
    self
  }
}

impl From<&[u8]> for Tick {
  fn from(input: &[u8]) -> Self {
    let mut tick = Tick::default();

    let parse_ltp = |t: &mut Tick, i: &[u8]| {
      // 0 - 4 bytes : instrument token
      t.set_instrument_token(i);
      // 4 - 8 bytes : ltp
      if let Some(bs) = i.get(4..8) {
        t.mode = Mode::LTP;
        t.last_price = price(bs, &t.exchange);
      }
    };

    let parse_quote = |t: &mut Tick, i: &[u8], is_index: bool| {
      if is_index {
        if let Some(bs) = i.get(8..28) {
          t.mode = Mode::Quote;
          // 8 - 24 bytes : ohlc
          t.ohlc = OHLC::from(&bs[0..16], &t.exchange);
          // 24 - 28 bytes : Price change
          // t.net_change = price(&bs[16..=19], &t.exchange);
          t.set_change();
        }
      } else {
        if let Some(bs) = i.get(8..44) {
          t.mode = Mode::Quote;
          // 8 - 12 bytes : last traded quantity
          t.last_traded_qty = value(&bs[0..4]);
          // 12 - 16 bytes : avg traded price
          t.avg_traded_price = price(&bs[4..8], &t.exchange);
          // 16 - 20 bytes : volume traded today
          t.volume_traded = value(&bs[8..12]);
          // 20 - 24 bytes : total buy quantity
          t.total_buy_qty = value(&bs[12..16]);
          // 24 - 28 bytes : total sell quantity
          t.total_sell_qty = value(&bs[16..20]);
          // 28 - 44 bytes : ohlc
          t.ohlc = OHLC::from(&bs[20..36], &t.exchange);
        }
      }
    };

    let parse_full = |t: &mut Tick, i: &[u8], is_index: bool| {
      if is_index {
        if let Some(bs) = i.get(28..32) {
          t.mode = Mode::Full;
          // 28 - 32 bytes : exchange time
          t.exchange_timestamp =
            value(bs).map(|x| Duration::from_secs(x.into()));
        }
      } else {
        if let Some(bs) = i.get(44..184) {
          t.mode = Mode::Full;
          t.set_change();
          
          // 44 - 48 bytes : last traded timestamp
          t.last_traded_timestamp =
            value(&bs[0..4]).map(|x| Duration::from_secs(x.into()));

          // 48 - 52 bytes : oi
          t.oi = value(&bs[4..8]);
          // 52 - 56 bytes : oi day high
          t.oi_day_high = value(&bs[8..12]);
          // 56 - 60 bytes : oi day low
          t.oi_day_low = value(&bs[12..16]);
          // 60 - 64 bytes : exchange time
          t.exchange_timestamp =
            value(&bs[16..20]).map(|x| Duration::from_secs(x.into()));
          // 64 - 184 bytes : market depth
          t.depth = Depth::from(&bs[20..140], &t.exchange);
        }
      }
    };

    parse_ltp(&mut tick, input);
    if !tick.exchange.is_tradable() {
      tick.is_index = true;
      tick.is_tradable = false;

      parse_quote(&mut tick, input, true);
      parse_full(&mut tick, input, true);
    } else {
      tick.is_index = false;
      tick.is_tradable = true;

      parse_quote(&mut tick, input, false);
      parse_full(&mut tick, input, false);
    }

    tick
  }
}

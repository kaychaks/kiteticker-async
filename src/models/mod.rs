use std::ops::Div;

mod depth;
mod exchange;
mod mode;
mod ohlc;
mod request;
mod text_message;
mod tick;
mod tick_message;
mod ticker_message;
pub use self::depth::{Depth, DepthItem};
pub use self::exchange::Exchange;
pub use self::mode::Mode;
pub use self::ohlc::OHLC;
pub use self::request::Request;
pub use self::text_message::TextMessage;
pub use self::tick::Tick;
pub use self::tick_message::TickMessage;
pub use self::ticker_message::TickerMessage;

fn value(input: &[u8]) -> Option<u32> {
  let value = i32::from_be_bytes(input[0..=3].try_into().unwrap());
  value.try_into().ok()
}

fn value_short(input: &[u8]) -> Option<u16> {
  let value = i16::from_be_bytes(input[0..=1].try_into().unwrap());
  value.try_into().ok()
}

fn price(input: &[u8], exchange: &Exchange) -> Option<f64> {
  let value = i32::from_be_bytes(input[0..4].try_into().unwrap()) as f64;
  if exchange.divisor() > 0_f64 {
    Some(value.div(exchange.divisor()))
  } else {
    None
  }
}

pub(crate) fn packet_length(bs: &[u8]) -> usize {
  i16::from_be_bytes(bs[0..=1].try_into().unwrap()) as usize
}

use serde::{Deserialize, Serialize};
use std::{ops::Div, time::Duration};

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
            Some(((last_price - close_price) * 100.0).div(close_price))
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

          t.set_change();
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

#[derive(Debug, Clone, Default)]
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
  fn from(value: &[u8], exchange: &Exchange) -> Option<Self> {
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

#[derive(Debug, Clone, Default)]
///
/// Market depth packet structure
///
pub struct Depth {
  pub buy: [DepthItem; 5],
  pub sell: [DepthItem; 5],
}

impl Depth {
  fn from(input: &[u8], exchange: &Exchange) -> Option<Self> {
    if let Some(bs) = input.get(0..120) {
      let parse_depth_item = |v: &[u8], start: usize| {
        v.get(start..start + 10)
          .and_then(|xs| DepthItem::from(xs, exchange))
          .unwrap_or_default()
      };
      let mut depth = Depth::default();
      for i in 0..5 {
        let start = i * 12;
        depth.buy[i] = parse_depth_item(bs, start)
      }
      for i in 0..5 {
        let start = 60 + i * 12;
        depth.sell[i] = parse_depth_item(bs, start);
      }

      Some(depth)
    } else {
      None
    }
  }
}

#[derive(Debug, Clone, Default)]
///
/// Structure for each market depth entry
///
pub struct DepthItem {
  pub qty: u32,
  pub price: f64,
  pub orders: u16,
}

impl DepthItem {
  pub fn from(input: &[u8], exchange: &Exchange) -> Option<Self> {
    if let Some(bs) = input.get(0..10) {
      Some(DepthItem {
        qty: value(&bs[0..=3]).unwrap(),
        price: price(&bs[4..=7], exchange).unwrap(),
        orders: value_short(&bs[8..=9]).unwrap(),
      })
    } else {
      None
    }
  }
}

#[derive(Debug, Clone)]
///
/// Parsed message from websocket
///
pub enum TickerMessage {
  /// Quote packets for subscribed tokens
  Tick(Vec<TickMessage>),
  /// Error response
  Error(String),
  /// Order postback
  Order(serde_json::Value),
  /// Messages and alerts from broker
  Message(serde_json::Value),
  /// Websocket closing frame
  ClosingMessage(serde_json::Value),
}

impl From<TextMessage> for TickerMessage {
  fn from(value: TextMessage) -> Self {
    let message_type: TextMessageType = value.message_type.into();
    match message_type {
      TextMessageType::Order => Self::Order(value.data),
      TextMessageType::Error => Self::Error(value.data.to_string()),
      TextMessageType::Message => Self::Message(value.data),
    }
  }
}

#[derive(Debug, Clone, Default)]
///
/// Parsed quote packet
///
pub struct TickMessage {
  pub instrument_token: u32,
  pub content: Tick,
}

impl TickMessage {
  pub(crate) fn new(instrument_token: u32, content: Tick) -> Self {
    Self {
      instrument_token,
      content,
    }
  }
}

#[derive(Debug, Clone, Default)]
///
/// Exchange options
///
pub enum Exchange {
  #[default]
  NSE,
  NFO,
  CDS,
  BSE,
  BFO,
  BCD,
  MCX,
  MCXSX,
  INDICES,
}

impl Exchange {
  fn divisor(&self) -> f64 {
    match self {
      Self::CDS => 100_000_0.0,
      Self::BCD => 100_0.0,
      _ => 100.0,
    }
  }

  fn is_tradable(&self) -> bool {
    match self {
      Self::INDICES => false,
      _ => true,
    }
  }
}

impl From<usize> for Exchange {
  fn from(value: usize) -> Self {
    match value {
      9 => Self::INDICES,
      8 => Self::MCXSX,
      7 => Self::MCX,
      6 => Self::BCD,
      5 => Self::BFO,
      4 => Self::BSE,
      3 => Self::CDS,
      2 => Self::NFO,
      1 => Self::NSE,
      _ => Self::NSE,
    }
  }
}

#[derive(
  Debug, Clone, Deserialize, Serialize, Default, PartialEq, PartialOrd,
)]
#[serde(rename_all = "lowercase")]
///
/// Modes in which packets are streamed
///
pub enum Mode {
  Full,
  #[default]
  Quote,
  LTP,
}

impl TryFrom<usize> for Mode {
  type Error = String;
  fn try_from(value: usize) -> Result<Self, Self::Error> {
    match value {
      8 => Ok(Self::LTP),
      44 => Ok(Self::Quote),
      184 => Ok(Self::Full),
      _ => Err(format!("Invalid packet size: {}", value)),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
///
/// Websocket request actions
///
enum RequestActions {
  Subscribe,
  Unsubscribe,
  Mode,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
///
/// Websocket request data
///
enum RequestData {
  InstrumentTokens(Vec<u32>),
  InstrumentTokensWithMode(Mode, Vec<u32>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
///
/// Websocket request structure
///
pub struct Request {
  a: RequestActions,
  v: RequestData,
}

impl Request {
  fn new(action: RequestActions, value: RequestData) -> Request {
    Request {
      a: action,
      v: value,
    }
  }

  ///
  /// Subscribe to a list of instrument tokens
  ///
  pub fn subscribe(instrument_tokens: Vec<u32>) -> Request {
    Request::new(
      RequestActions::Subscribe,
      RequestData::InstrumentTokens(instrument_tokens),
    )
  }

  ///
  /// Subscribe to a list of instrument tokens with mode
  ///
  pub fn mode(mode: Mode, instrument_tokens: Vec<u32>) -> Request {
    Request::new(
      RequestActions::Mode,
      RequestData::InstrumentTokensWithMode(mode, instrument_tokens),
    )
  }

  ///
  /// Unsubscribe from a list of instrument tokens
  ///
  pub fn unsubscribe(instrument_tokens: Vec<u32>) -> Request {
    Request::new(
      RequestActions::Unsubscribe,
      RequestData::InstrumentTokens(instrument_tokens),
    )
  }
}

impl ToString for Request {
  fn to_string(&self) -> String {
    serde_json::to_string(self)
      .expect("failed to serialize TickerInput to JSON")
  }
}

#[derive(Debug, Clone)]
///
/// Postbacks and non-binary message types
///
enum TextMessageType {
  /// Order postback
  Order,
  /// Error response
  Error,
  /// Messages and alerts from the broker
  Message,
}

impl From<String> for TextMessageType {
  fn from(value: String) -> Self {
    match value.as_str() {
      "order" => Self::Order,
      "error" => Self::Error,
      _ => Self::Message,
    }
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
///
/// Postback and non-binary message structure
///
pub struct TextMessage {
  #[serde(rename = "type")]
  message_type: String,
  data: serde_json::Value,
}

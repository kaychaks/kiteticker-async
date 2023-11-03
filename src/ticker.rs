use crate::models::{
  packet_length, Mode, Request, TextMessage, Tick, TickMessage, TickerMessage,
};
use futures_util::{stream::iter, SinkExt, StreamExt};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
  connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};

#[derive(Debug, Clone)]
///
/// The WebSocket client for connecting to Kite Connect's streaming quotes service.
///
pub struct KiteTickerAsync {
  #[allow(dead_code)]
  api_key: String,
  #[allow(dead_code)]
  access_token: String,
  ws_stream: Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

impl KiteTickerAsync {
  /// Establish a connection with the Kite WebSocket server
  pub async fn connect(
    api_key: &str,
    access_token: &str,
  ) -> Result<Self, String> {
    let socket_url = format!(
      "wss://{}?api_key={}&access_token={}",
      "ws.kite.trade", api_key, access_token
    );
    let url = url::Url::parse(socket_url.as_str()).unwrap();

    let (ws_stream, _) = connect_async(url).await.map_err(|e| e.to_string())?;

    Ok(KiteTickerAsync {
      api_key: api_key.to_string(),
      access_token: access_token.to_string(),
      ws_stream: Arc::new(Mutex::new(ws_stream)),
    })
  }

  /// Subscribes the client to a list of instruments
  pub async fn subscribe(
    mut self,
    instrument_tokens: &[u32],
    mode: Option<Mode>,
  ) -> Result<KiteTickerSubscriber, String> {
    self
      .subscribe_cmd(instrument_tokens, mode.clone())
      .await
      .expect("failed to subscribe");
    let st = instrument_tokens
      .to_vec()
      .iter()
      .map(|t| (t.clone(), mode.to_owned().unwrap_or_default()))
      .collect();

    Ok(KiteTickerSubscriber {
      ticker: self,
      subscribed_tokens: st,
    })
  }

  /// Close the websocket connection
  pub async fn close(&mut self) -> Result<(), String> {
    let mut ws_stream = self.ws_stream.lock().await;
    ws_stream.close(None).await.map_err(|x| x.to_string())?;
    Ok(())
  }

  async fn subscribe_cmd(
    &mut self,
    instrument_tokens: &[u32],
    mode: Option<Mode>,
  ) -> Result<(), String> {
    let mut msgs = iter(vec![
      Ok(Message::Text(
        Request::subscribe(instrument_tokens.to_vec()).to_string(),
      )),
      Ok(Message::Text(
        Request::mode(mode.unwrap_or_default(), instrument_tokens.to_vec())
          .to_string(),
      )),
    ]);

    let mut ws_stream = self.ws_stream.lock().await;

    ws_stream
      .send_all(msgs.by_ref())
      .await
      .expect("failed to send subscription message");

    Ok(())
  }

  async fn unsubscribe_cmd(
    &mut self,
    instrument_tokens: &[u32],
  ) -> Result<(), String> {
    let mut ws_stream = self.ws_stream.lock().await;
    ws_stream
      .send(Message::Text(
        Request::unsubscribe(instrument_tokens.to_vec()).to_string(),
      ))
      .await
      .expect("failed to send unsubscribe message");
    Ok(())
  }

  async fn set_mode_cmd(
    &mut self,
    instrument_tokens: &[u32],
    mode: Mode,
  ) -> Result<(), String> {
    let mut ws_stream = self.ws_stream.lock().await;
    ws_stream
      .send(Message::Text(
        Request::mode(mode, instrument_tokens.to_vec()).to_string(),
      ))
      .await
      .expect("failed to send set mode message");
    Ok(())
  }
}

#[derive(Debug, Clone)]
///
/// The Websocket client that entered in a pub/sub mode once the client subscribed to a list of instruments
///
pub struct KiteTickerSubscriber {
  ticker: KiteTickerAsync,
  subscribed_tokens: HashMap<u32, Mode>,
}

impl KiteTickerSubscriber {
  /// Get the list of subscribed instruments
  pub fn get_subscribed(&self) -> Vec<u32> {
    self
      .subscribed_tokens
      .clone()
      .into_keys()
      .collect::<Vec<_>>()
  }

  /// get all tokens common between subscribed tokens and input tokens
  /// and if the input is empty then all subscribed tokens will be unsubscribed
  fn get_subscribed_or(&self, tokens: &[u32]) -> Vec<u32> {
    if tokens.len() == 0 {
      self.get_subscribed()
    } else {
      tokens
        .iter()
        .filter(|t| self.subscribed_tokens.contains_key(t))
        .map(|t| t.clone())
        .collect::<Vec<_>>()
    }
  }

  /// Subscribe to new tokens
  pub async fn subscribe(
    &mut self,
    tokens: &[u32],
    mode: Option<Mode>,
  ) -> Result<(), String> {
    self.subscribed_tokens.extend(
      tokens
        .iter()
        .map(|t| (t.clone(), mode.clone().unwrap_or_default())),
    );
    let tks = self.get_subscribed();
    self.ticker.subscribe_cmd(tks.as_slice(), None).await?;
    Ok(())
  }

  /// Change the mode of the subscribed instrument tokens
  pub async fn set_mode(
    &mut self,
    instrument_tokens: &[u32],
    mode: Mode,
  ) -> Result<(), String> {
    let tokens = self.get_subscribed_or(instrument_tokens);
    self.ticker.set_mode_cmd(tokens.as_slice(), mode).await
  }

  /// Unsubscribe provided subscribed tokens, if input is empty then all subscribed tokens will unsubscribed
  ///
  /// Tokens in the input which are not part of the subscribed tokens will be ignored.
  pub async fn unsubscribe(
    &mut self,
    instrument_tokens: &[u32],
  ) -> Result<(), String> {
    let tokens = self.get_subscribed_or(instrument_tokens);
    self.ticker.unsubscribe_cmd(tokens.as_slice()).await
  }

  /// Get the next message from the server, waiting if necessary.
  /// If the result is None then server is terminated
  pub async fn next_message(
    &mut self,
  ) -> Result<Option<TickerMessage>, String> {
    let mut ws_stream = self.ticker.ws_stream.lock().await;
    match ws_stream.next().await {
      Some(message) => match message {
        Ok(msg) => Ok(self.process_message(msg)),
        Err(e) => Err(e.to_string()),
      },
      None => Ok(None),
    }
  }

  fn process_message(&self, message: Message) -> Option<TickerMessage> {
    match message {
      Message::Text(text_message) => self.process_text_message(text_message),
      Message::Binary(ref binary_message) => {
        if binary_message.len() < 2 {
          return Some(TickerMessage::Ticks(vec![]));
        } else {
          self.process_binary(binary_message.as_slice())
        }
      }
      Message::Close(closing_message) => closing_message.map(|c| {
        TickerMessage::ClosingMessage(json!({
          "code": c.code.to_string(),
          "reason": c.reason.to_string()
        }))
      }),
      Message::Ping(_) => unimplemented!(),
      Message::Pong(_) => unimplemented!(),
      Message::Frame(_) => unimplemented!(),
    }
  }

  fn process_binary(&self, binary_message: &[u8]) -> Option<TickerMessage> {
    // 0 - 2 : number of packets in the message
    let num_packets =
      i16::from_be_bytes(binary_message[0..=1].try_into().unwrap()) as usize;
    if num_packets > 0 {
      Some(TickerMessage::Ticks(
        [0..num_packets]
          .into_iter()
          .fold((vec![], 2), |(mut acc, start), _| {
            // start - start + 2 : length of the packet
            let packet_len = packet_length(&binary_message[start..start + 2]);
            let next_start = start + 2 + packet_len;
            let tick = Tick::from(&binary_message[start + 2..next_start]);
            acc.push(TickMessage::new(tick.instrument_token, tick));
            (acc, next_start)
          })
          .0,
      ))
    } else {
      None
    }
  }

  fn process_text_message(
    &self,
    text_message: String,
  ) -> Option<TickerMessage> {
    serde_json::from_str::<TextMessage>(&text_message)
      .map(|x| x.into())
      .ok()
  }

  pub async fn close(&mut self) -> Result<(), String> {
    self.ticker.close().await
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use base64::{engine::general_purpose, Engine};

  use crate::{DepthItem, Mode, Tick, OHLC};

  fn load_packet(name: &str) -> Vec<u8> {
    let str =
      std::fs::read_to_string(format!("kiteconnect-mocks/{}.packet", name))
        .map(|s| s.trim().to_string())
        .expect("could not read file");
    let ret = general_purpose::STANDARD
      .decode(str)
      .expect("could not decode");
    ret
  }

  fn setup() -> Vec<(&'static str, Vec<u8>, Tick)> {
    vec![
      (
        "quote packet",
        load_packet("ticker_quote"),
        Tick {
          mode: Mode::Quote,
          exchange: crate::Exchange::NSE,
          instrument_token: 408065,
          is_tradable: true,
          is_index: false,
          last_traded_timestamp: None,
          exchange_timestamp: None,
          last_price: Some(1573.15),
          avg_traded_price: Some(1570.33),
          last_traded_qty: Some(1),
          total_buy_qty: Some(256511),
          total_sell_qty: Some(360503),
          volume_traded: Some(1175986),
          ohlc: Some(OHLC {
            open: 1569.15,
            high: 1575.0,
            low: 1561.05,
            close: 1567.8,
          }),
          oi_day_high: None,
          oi_day_low: None,
          oi: None,
          net_change: None,
          depth: None,
        },
      ),
      (
        "full packet",
        load_packet("ticker_full"),
        Tick {
          mode: Mode::Full,
          exchange: crate::Exchange::NSE,
          instrument_token: 408065,
          is_tradable: true,
          is_index: false,
          last_traded_timestamp: Some(Duration::from_secs(
            chrono::DateTime::parse_from_rfc3339("2021-07-05T10:41:27+05:30")
              .unwrap()
              .timestamp() as u64,
          )),
          exchange_timestamp: Some(Duration::from_secs(
            chrono::DateTime::parse_from_rfc3339("2021-07-05T10:41:27+05:30")
              .unwrap()
              .timestamp() as u64,
          )),
          last_price: Some(1573.7),
          avg_traded_price: Some(1570.37),
          last_traded_qty: Some(7),
          total_buy_qty: Some(256443),
          total_sell_qty: Some(363009),
          volume_traded: Some(1192471),
          ohlc: Some(OHLC {
            open: 1569.15,
            high: 1575.0,
            low: 1561.05,
            close: 1567.8,
          }),
          oi_day_high: Some(0),
          oi_day_low: Some(0),
          oi: Some(0),
          net_change: Some(5.900000000000091),
          depth: Some(crate::Depth {
            buy: [
              DepthItem {
                qty: 5,
                price: 1573.4,
                orders: 1,
              },
              DepthItem {
                qty: 140,
                price: 1573.0,
                orders: 2,
              },
              DepthItem {
                qty: 2,
                price: 1572.95,
                orders: 1,
              },
              DepthItem {
                qty: 219,
                price: 1572.9,
                orders: 7,
              },
              DepthItem {
                qty: 50,
                price: 1572.85,
                orders: 1,
              },
            ],
            sell: [
              DepthItem {
                qty: 172,
                price: 1573.7,
                orders: 3,
              },
              DepthItem {
                qty: 44,
                price: 1573.75,
                orders: 3,
              },
              DepthItem {
                qty: 302,
                price: 1573.85,
                orders: 3,
              },
              DepthItem {
                qty: 141,
                price: 1573.9,
                orders: 2,
              },
              DepthItem {
                qty: 724,
                price: 1573.95,
                orders: 5,
              },
            ],
          }),
        },
      ),
    ]
  }

  #[test]
  fn test_quotes() {
    let data = setup();
    for (name, packet, expected) in data {
      let tick = Tick::try_from(packet.as_slice());
      assert_eq!(tick.is_err(), false);
      assert_eq!(tick.unwrap(), expected, "Testing {}", name);
    }
  }
}

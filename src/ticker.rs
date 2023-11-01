use futures_util::{stream::iter, SinkExt, StreamExt};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
  connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
use crate::models::{
  packet_length, Mode, Request, TextMessage, Tick, TickMessage, TickerMessage,
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
  pub async fn next_message(&mut self) -> Result<Option<TickerMessage>, String> {
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use tokio::select;

  async fn check<F>(
    mode: Mode,
    token: u32,
    sb: &mut KiteTickerSubscriber,
    assertions: Option<F>,
  ) where
    F: Fn(Vec<TickMessage>) -> (),
  {
    loop {
      match sb.next_message().await {
        Ok(message) => match message {
          Some(TickerMessage::Ticks(xs)) => {
            if xs.len() == 0 {
              continue;
            }
            assertions.map(|f| f(xs.clone())).or_else(|| {
              let tick_message = xs.first().unwrap();
              assert!(tick_message.instrument_token == token);
              assert_eq!(tick_message.content.mode, mode);
              Some(())
            });
            break;
          }
          _ => {
            continue;
          }
        },
        _ => {
          assert!(false);
          break;
        }
      }
    }
  }

  #[tokio::test]
  async fn test_ticker() {
    let api_key = std::env::var("KITE_API_KEY").unwrap();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
    let ticker = KiteTickerAsync::connect(&api_key, &access_token).await;

    assert_eq!(ticker.is_ok(), true);

    let ticker = ticker.unwrap();
    let token = 94977; // bata
    let mode = Mode::Full;
    let sb = ticker.subscribe(&[token], Some(mode.clone())).await;
    assert_eq!(sb.is_ok(), true);
    let mut sb = sb.unwrap();
    assert_eq!(sb.subscribed_tokens.len(), 1);
    let mut loop_cnt = 0;
    loop {
      loop_cnt += 1;
      select! {
        Ok(n) = sb.next_message() => {
          match n.to_owned() {
            Some(message) => {
              match message {
                TickerMessage::Ticks(xs) => {
                  if xs.len() == 0 {
                    if loop_cnt > 5 {
                      break;
                    }else {
                      continue;
                    }
                  }
                  assert_eq!(xs.len(), 1);
                  let tick_message = xs.first().unwrap();
                  assert!(tick_message.instrument_token == token);
                  assert_eq!(tick_message.content.mode, mode);
                  if loop_cnt > 5 {
                    break;
                  }
                },
                _ => {
                  if loop_cnt > 5 {
                    break;
                  }
                }
              }
            },
            _ => {
              if loop_cnt > 5 {
                assert!(false);
                break;
              }
            }
          }
        },
        else => {
          assert!(false);
          break;
        }
      }
    }

    sb.ticker.close().await.unwrap();
  }

  #[tokio::test]
  async fn test_unsubscribe() {
    // create a ticker
    let api_key = std::env::var("KITE_API_KEY").unwrap();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
    let ticker = KiteTickerAsync::connect(&api_key, &access_token).await;

    let ticker = ticker.unwrap();
    let token = 94977; // bata
    let mode = Mode::Full;
    let mut sb = ticker
      .subscribe(&[token], Some(mode.clone()))
      .await
      .unwrap();

    let mut loop_cnt = 0;

    loop {
      match sb.next_message().await {
        Ok(message) => match message {
          Some(TickerMessage::Ticks(xs)) => {
            if xs.len() == 0 {
              if loop_cnt > 4 {
                assert!(true);
                break;
              } else {
                loop_cnt += 1;
                continue;
              }
            }
            assert_eq!(xs.len(), 1);
            let tick_message = xs.first().unwrap();
            assert!(tick_message.instrument_token == token);
            sb.unsubscribe(&[]).await.unwrap();
            loop_cnt += 1;
            if loop_cnt > 5 {
              assert!(false);
              break;
            }
          }
          _ => {
            continue;
          }
        },
        _ => {
          assert!(false);
          break;
        }
      }
    }
    sb.ticker.close().await.unwrap();
  }

  async fn create_ticker() -> KiteTickerAsync {
    // create a ticker
    let api_key = std::env::var("KITE_API_KEY").unwrap();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
    let ticker = KiteTickerAsync::connect(&api_key, &access_token).await;
    ticker.expect("failed to create ticker")
  }

  #[tokio::test]
  async fn test_set_mode() {
    let ticker = create_ticker().await;
    let token = 94977; // bata
    let mode = Mode::LTP;
    let new_mode = Mode::Quote;
    let mut sb = ticker
      .subscribe(&[token], Some(mode.clone()))
      .await
      .unwrap();

    let f1: Option<Box<dyn Fn(Vec<TickMessage>) -> ()>> = None;
    let f2: Option<Box<dyn Fn(Vec<TickMessage>) -> ()>> = None;
    check(mode, token, &mut sb, f1).await;
    sb.set_mode(&[], new_mode.clone()).await.unwrap();
    check(new_mode, token, &mut sb, f2).await;

    sb.ticker.close().await.unwrap();
  }

  #[tokio::test]
  async fn test_new_sub() {
    let ticker = create_ticker().await;
    let token = 94977; // bata
    let mode = Mode::LTP;
    let mut sb = ticker
      .subscribe(&[token], Some(mode.clone()))
      .await
      .unwrap();
    tokio::spawn(async move {
      sb.subscribe(&[2953217], None).await.unwrap();
    })
    .await
    .unwrap();
  }
}

mod common;

use kiteticker_async::ticker::*;
use kiteticker_async::*;
use tokio::select;

use crate::common::check;

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
  assert_eq!(sb.get_subscribed().len(), 1);
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

  sb.close().await.unwrap();
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
  sb.close().await.unwrap();
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

  sb.close().await.unwrap();
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

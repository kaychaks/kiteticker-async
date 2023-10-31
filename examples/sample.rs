use kiteticker_async::{KiteTickerAsync, Mode, TickerMessage};

#[tokio::main]
pub async fn main() -> Result<(), String> {
  let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
  let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
  let ticker = KiteTickerAsync::connect(&api_key, &access_token).await?;

  let token = 408065;
  // subscribe to an instrument
  let mut subscriber = ticker.subscribe(&[token], Some(Mode::Full)).await?;

  // await quotes
  loop {
    if let Some(msg) = subscriber.next_message().await? {
      match msg {
        TickerMessage::Ticks(ticks) => {
          let tick = ticks.first().unwrap();
          println!(
            "Received tick for instrument_token {}, {:?}",
            tick.instrument_token, tick
          );
          break;
        }
        _ => {
          println!("Received message from broker {:?}", msg);
          continue;
        }
      }
    }
  }

  Ok(())
}

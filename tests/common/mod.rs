use kiteticker_async::{
  KiteTickerSubscriber, Mode, TickMessage, TickerMessage,
};

pub async fn check<F>(
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

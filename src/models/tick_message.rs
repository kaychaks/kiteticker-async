use crate::Tick;

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

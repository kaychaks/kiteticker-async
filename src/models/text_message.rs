use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
///
/// Postbacks and non-binary message types
///
pub(crate) enum TextMessageType {
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
  pub(crate) message_type: String,
  pub(crate) data: serde_json::Value,
}

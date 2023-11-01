use serde::{Deserialize, Serialize};

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

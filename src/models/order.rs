use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DefaultOnNull};

use crate::Exchange;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderTransactionType {
  Buy,
  Sell,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd)]
pub enum OrderValidity {
  DAY,
  IOC,
  TTL,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
  COMPLETE,
  REJECTED,
  CANCELLED,
  UPDATE,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TimeStamp(i64);

impl From<String> for TimeStamp {
  fn from(value: String) -> Self {
    let secs = NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S")
      .unwrap()
      .timestamp();
    TimeStamp(secs)
  }
}

impl From<TimeStamp> for String {
  fn from(value: TimeStamp) -> Self {
    NaiveDateTime::from_timestamp_opt(value.0, 0)
      .unwrap_or_default()
      .format("%Y-%m-%d %H:%M:%S")
      .to_string()
  }
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Order {
  pub order_id: String,

  #[serde_as(as = "DefaultOnNull")]
  pub exchange_order_id: Option<String>,

  #[serde_as(as = "DefaultOnNull")]
  pub parent_order_id: Option<String>,

  pub placed_by: String,
  pub app_id: u64,

  pub status: OrderStatus,

  #[serde_as(as = "DefaultOnNull")]
  pub status_message: Option<String>,

  #[serde_as(as = "DefaultOnNull")]
  pub status_message_raw: Option<String>,

  pub tradingsymbol: String,
  pub instrument_token: u32,

  #[serde_as(as = "serde_with::FromInto<String>")]
  pub exchange: Exchange,

  pub order_type: String,
  pub transaction_type: OrderTransactionType,

  pub validity: OrderValidity,
  pub variety: String,
  pub product: Option<String>,

  #[serde(default)]
  pub average_price: f64,

  #[serde(default)]
  pub disclosed_quantity: f64,

  pub price: f64,
  pub quantity: u64,
  pub filled_quantity: u64,

  #[serde(default)]
  pub unfilled_quantity: u64,

  #[serde(default)]
  pub pending_quantity: u64,

  #[serde(default)]
  pub cancelled_quantity: u64,

  #[serde(default)]
  pub trigger_price: f64,

  pub user_id: String,

  #[serde_as(as = "serde_with::FromInto<String>")]
  pub order_timestamp: TimeStamp,
  #[serde_as(as = "serde_with::FromInto<String>")]
  pub exchange_timestamp: TimeStamp,
  #[serde_as(as = "serde_with::FromInto<String>")]
  pub exchange_update_timestamp: TimeStamp,

  pub checksum: String,
  #[serde(default)]
  pub meta: Option<serde_json::Map<String, Value>>,

  #[serde_as(as = "DefaultOnNull")]
  #[serde(default)]
  pub tag: Option<String>,
}

#[cfg(test)]
mod tests {

  use sha2::{Digest, Sha256};

  use super::*;

  #[test]
  fn test_order() {
    let postback_json = include_str!("../../kiteconnect-mocks/postback.json");
    let exp_order = Order {
      order_id: "220303000308932".to_string(),
      exchange_order_id: Some("1000000001482421".to_string()),
      parent_order_id: None,
      placed_by: "AB1234".to_string(),
      app_id: 1234,
      status: OrderStatus::COMPLETE,
      status_message: None,
      status_message_raw: None,
      tradingsymbol: "SBIN".to_string(),
      instrument_token: 779521,
      exchange: Exchange::NSE,
      order_type: "MARKET".to_string(),
      transaction_type: OrderTransactionType::Buy,
      validity: OrderValidity::DAY,
      variety: "regular".to_string(),
      product: Some("CNC".to_string()),
      average_price: 470.0,
      disclosed_quantity: 0.0,
      price: 0.0,
      quantity: 1,
      filled_quantity: 1,
      unfilled_quantity: 0,
      pending_quantity: 0,
      cancelled_quantity: 0,
      trigger_price: 0.0,
      user_id: "AB1234".to_string(),
      order_timestamp: TimeStamp(1646299465),
      exchange_timestamp: TimeStamp(1646299465),
      exchange_update_timestamp: TimeStamp(1646299465),
      checksum:
        "2011845d9348bd6795151bf4258102a03431e3bb12a79c0df73fcb4b7fde4b5d"
          .to_string(),
      meta: Some(serde_json::Map::new()),
      tag: None,
    };
    let order = serde_json::from_str::<Order>(postback_json).unwrap();
    assert_eq!(order.clone(), exp_order);

    let mut hasher = Sha256::new();
    hasher.update(order.order_id.as_bytes());
    hasher.update(Into::<String>::into(order.order_timestamp));
    hasher.update(b"0hdv7iw5examplesecret");
    let expected = hasher.finalize();
    let actual = hex::decode(order.checksum).unwrap();
    assert_eq!(expected[..], actual[..]);
  }
}

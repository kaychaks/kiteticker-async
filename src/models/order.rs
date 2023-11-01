use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DefaultOnNull};

use crate::Exchange;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderTransactionType {
  Buy,
  Sell,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum OrderValidity {
  DAY,
  IOC,
  TTL,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
  COMPLETE,
  REJECTED,
  CANCELLED,
  UPDATE,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Order {
  account_id: String,

  order_id: String,

  #[serde_as(as = "DefaultOnNull")]
  exchange_order_id: Option<String>,

  #[serde_as(as = "DefaultOnNull")]
  parent_order_id: Option<String>,

  placed_by: String,
  app_id: u64,

  status: OrderStatus,

  #[serde_as(as = "DefaultOnNull")]
  status_message: Option<String>,

  #[serde_as(as = "DefaultOnNull")]
  status_message_raw: Option<String>,

  tradingsymbol: String,
  instrument_token: u32,

  #[serde_as(as = "serde_with::FromInto<String>")]
  exchange: Exchange,

  order_type: String,
  transaction_type: OrderTransactionType,

  validity: OrderValidity,
  variety: String,
  product: Option<String>,

  #[serde(default)]
  average_price: f64,

  #[serde(default)]
  disclosed_quantity: f64,

  price: f64,
  quantity: u64,
  filled_quantity: u64,

  #[serde(default)]
  unfilled_quantity: u64,

  #[serde(default)]
  pending_quantity: u64,

  #[serde(default)]
  cancelled_quantity: u64,

  #[serde(default)]
  trigger_price: f64,

  user_id: String,

  #[serde_as(as = "serde_with::DurationSeconds<String>")]
  order_timestamp: Duration,
  #[serde_as(as = "serde_with::DurationSeconds<String>")]
  exchange_timestamp: Duration,
  #[serde_as(as = "serde_with::DurationSeconds<String>")]
  exchange_update_timestamp: Duration,

  checksum: String,
  #[serde(default)]
  meta: Option<Value>,

  #[serde_as(as = "DefaultOnNull")]
  #[serde(default)]
  tag: Option<String>
}

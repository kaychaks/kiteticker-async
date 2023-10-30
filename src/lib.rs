#![allow(
  clippy::cognitive_complexity,
  clippy::large_enum_variant,
  clippy::needless_doctest_main
)]
#![warn(
  missing_debug_implementations,
  rust_2018_idioms,
  unreachable_pub
)]
#![doc(test(
  no_crate_inject,
  attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

//! Async implementation of Kite Connect's WebSocket Steaming API
//!
//! This crate provides types to subscribe and receive live quotes for instruments during market hours via WebSockets.
//! The response is parsed and converted into Rust types.
//! The WebSocket connection is managed by the library and reconnected automatically.
//!

mod models;
pub use models::{
  Depth, DepthItem, Exchange, Mode, Request, TextMessage, Tick, TickMessage,
  TickerMessage, OHLC,
};

pub mod ticker;
pub use ticker::{KiteTickerAsync, KiteTickerSubscriber};

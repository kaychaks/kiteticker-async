#![allow(
  clippy::cognitive_complexity,
  clippy::large_enum_variant,
  clippy::needless_doctest_main
)]
#![warn(missing_debug_implementations, rust_2018_idioms, unreachable_pub)]
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
//! # Usage
//! ```
//!
//! use kiteticker_async::{KiteTickerAsync, Mode, TickerMessage};
//!
//! #[tokio::main]
//! pub async fn main() -> Result<(), String> {
//!   let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
//!   let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
//!   let ticker = KiteTickerAsync::connect(&api_key, &access_token).await?;
//!
//!   let token = 408065;
//!   // subscribe to an instrument
//!   let mut subscriber = ticker
//!     .subscribe(&[token], Some(Mode::Full))
//!     .await?;
//!
//!   // await quotes
//!  loop {
//!   if let Some(msg) = subscriber.next_message().await? {
//!     match msg {
//!       TickerMessage::Ticks(ticks) => {
//!         let tick = ticks.first().unwrap();
//!         println!("Received tick for instrument_token {}, {:?}", tick.instrument_token, tick);
//!         break;
//!       },
//!      _ => continue,
//!     }
//!   }
//!  }
//!
//!   Ok(())
//! }
//! ```
mod models;
pub use models::{
  Depth, DepthItem, Exchange, Mode, Request, TextMessage, TickMessage,
  TickerMessage, OHLC, Tick
};

pub mod ticker;
pub use ticker::{KiteTickerAsync, KiteTickerSubscriber};

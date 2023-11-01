# kiteticker-async

Async client for the [Kite Connect WebSocket API](https://kite.trade/docs/connect/v3/websocket/#websocket-streaming).

[![Crates.io][crates-badge]][crates-url]
[![Apache-2.0 Licensed][apache-2-0-badge]][apache-2-0-url]

[crates-badge]: https://img.shields.io/crates/v/kiteticker-async.svg
[crates-url]: https://crates.io/crates/kiteticker-async
[apache-2-0-badge]: https://img.shields.io/badge/license-apache-blue.svg
[apache-2-0-url]: https://github.com/kaychaks/kiteticker-async/blob/master/LICENSE

[Guide](https://kite.trade/docs/connect/v3/websocket/#websocket-streaming) |
[API Docs](https://docs.rs/kiteticker-async/latest/kiteticker-async)

## Overview

The official [kiteconnect-rs](https://crates.io/crates/kiteconnect) is an unmaintained project compared to the Python or Go implementations. As per this [issue](https://github.com/zerodha/kiteconnect-rs/issues/39), it will not get any further updates from the Zerodha Tech team.

Even though the Kite Connect REST APIs are feature-complete, the Ticker APIs are lagging. Here are some of the issues with Ticker API Rust implementation:

- It lacks a few updates, which are present in actively maintained [Python](https://github.com/zerodha/pykiteconnect) & [Go](https://github.com/zerodha/gokiteconnect) implementations.

- It does not parse and serialise quote structure to proper Rust structs and leaves it at an untyped JSON value. This is again a departure from how the same is implemented in libraries of typed languages like [Go](https://github.com/zerodha/gokiteconnect/blob/master/ticker/ticker.go) or [Java](https://github.com/zerodha/javakiteconnect/tree/master/kiteconnect/src/com/zerodhatech/models).

- The design requires the applications to handle the streaming WebSocket messages via callbacks. It is not an idiomatic Rust library design, primarily when the downstream applications rely on modern Rust async concurrency primitives using frameworks like [tokio](https://tokio.rs/).

This crate is an attempt to address the above issues. The primary goal is to have an async-friendly design following Rust's async library design principles championed by [tokio](https://tokio.rs/tokio/tutorial).

## Usage

Add kiteticker-async crate as a dependency in Cargo.toml

```
[dependencies]
kiteticker-async = "0.1.1"
```

## Example

```rust
#[tokio::main]
pub async fn main() -> Result<(), String> {
  let api_key = std::env::var("KITE_API_KEY").unwrap();
  let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
  let ticker = KiteTickerAsync::connect(&api_key, &access_token).await?;

  let token = 408065;
  // subscribe to an instrument
  let mut subscriber = ticker
    .subscribe(&[token], Some(Mode::Full))
    .await?;

  // await quotes
  if let Some(msg) = subscriber.next_message().await? {
    match msg {
      TickerMessage::Tick(ticks) => {
        let tick = ticks.first().unwrap();
        println!("Received tick for instrument_token {}, {}", tick.instrument_token, tick);
      }
    }
  }

  Ok(())
}
```

## Contributing

Use [just](https://github.com/casey/just) to run the development tasks.

```sh
$ just --list
Available recipes:
    build
    check
    doc
    doc-open
    doc-test api_key='' access_token=''
    example api_key access_token
    test api_key='' access_token=''
```

## License

This project is licensed under the [Apache 2.0 License]

[Apache 2.0 license]: https://github.com/kaychaks/kiteticker-async/blob/master/LICENSE

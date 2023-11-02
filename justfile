cargo-doc-command := "cargo doc --no-deps"

check:
  cargo check

build:
  cargo clean --quiet -r
  cargo build --release --quiet

doc:
  cargo clean --doc --quiet
  {{cargo-doc-command}}

doc-open:
  rm -r target/doc
  {{cargo-doc-command}} --open

example api_key access_token:
  KITE_API_KEY={{api_key}} KITE_ACCESS_TOKEN={{access_token}} cargo run --example sample

test:
  cargo test --lib

test-unit: test

test-integration api_key='' access_token='':
  KITE_API_KEY={{api_key}} KITE_ACCESS_TOKEN={{access_token}} cargo test --test '*'

test-doc api_key='' access_token='':
  KITE_API_KEY={{api_key}} KITE_ACCESS_TOKEN={{access_token}}  cargo test --quiet --doc

test-all: test-unit test-integration test-doc

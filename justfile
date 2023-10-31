cargo-doc-command := "cargo doc --no-deps"

check:
  cargo check

test api_key='' access_token='':
  KITE_API_KEY={{api_key}} KITE_ACCESS_TOKEN={{access_token}} cargo test --lib

build:
  cargo clean --quiet -r
  cargo build --release --quiet

doc:
  cargo clean --doc --quiet
  {{cargo-doc-command}}

doc-test api_key='' access_token='':
  KITE_API_KEY={{api_key}} KITE_ACCESS_TOKEN={{access_token}}  cargo test --quiet --doc

doc-open:
  rm -r target/doc
  {{cargo-doc-command}} --open

example api_key access_token:
  KITE_API_KEY={{api_key}} KITE_ACCESS_TOKEN={{access_token}} cargo run --example sample

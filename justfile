cargo-doc-command := "cargo doc --no-deps"

check:
  cargo check

test:
  cargo test

build:
  cargo build

doc:
  rm -r target/doc
  {{cargo-doc-command}}
doc-open:
  rm -r target/doc
  {{cargo-doc-command}} --open

alias b := build-release

build-release:
  cargo fmt
  cargo run -r

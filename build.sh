#!/bin/sh
# Builds the plug-ins and copies the .wasm files to dist/.
set -e
cd "$(dirname "$0")"
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
cargo build --release --target wasm32-unknown-unknown
for p in spiral rose rosette; do
  cp "target/wasm32-unknown-unknown/release/geomancer_$p.wasm" "dist/geomancer-$p.wasm"
done
ls -l dist

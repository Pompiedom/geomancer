#!/bin/sh
# Builds the plug-ins and copies the .wasm files to dist/.
set -e
cd "$(dirname "$0")"
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/geomancer_generators.wasm dist/geomancer.wasm
cp target/wasm32-unknown-unknown/release/geomancer_rosette.wasm dist/geomancer-rosette.wasm
ls -l dist

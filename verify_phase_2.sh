#!/bin/bash
set -e

echo "1. Building Dioxus WASM..."
dx build --platform web --features web --release

echo "2. Copying assets to web/static..."
mkdir -p web/static
cp -r target/dx/rr-ui/release/web/public/* web/static/

echo "3. Building Server Binary (with embedded assets)..."
cargo build --bin rr-ui --features server,web --no-default-features

echo "4. Checking binary size..."
ls -lh target/debug/rr-ui

echo "5. DONE! Run './target/debug/rr-ui' to test."

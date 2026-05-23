#!/bin/bash
set -e

echo "--- Building RR-UI Tactical Panel ---"

# 1. Clean previous artifacts
rm -rf dist/*
mkdir -p dist

# 2. Build WASM with Dioxus CLI
echo "Step 1: Building WASM..."
dx build --platform web --release

# 3. Copy build artifacts to dist/ for RustEmbed
echo "Step 2: Syncing artifacts..."
# Dioxus outputs to target/dx/rr-ui/release/web/public/
cp -r target/dx/rr-ui/release/web/public/* dist/

# 4. Optional: Optimize WASM size if wasm-opt is available
if command -v wasm-opt &> /dev/null
then
    echo "Step 3: Optimizing WASM binary size..."
    wasm-opt -Oz dist/wasm/rr-ui_bg.wasm -o dist/wasm/rr-ui_bg.wasm
fi

echo "--- Build Complete ---"
echo "Run 'cargo run --features server' to start the admin panel."

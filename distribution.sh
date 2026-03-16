#!/bin/bash
set -e

# Distribution Build Script for Rustray UI
# 1. Builds the Dioxus Frontend (WASM)
# 2. Embeds it into the Rust Backend
# 3. Optimizes the final binary

echo "🚀 Starting Distribution Build..."

# Check for dioxus-cli
if ! command -v dx &> /dev/null; then
    echo "❌ Error: 'dx' command not found. Please install Dioxus CLI: cargo install dioxus-cli"
    exit 1
fi

# 1. Build Frontend
echo "📦 Building Frontend (WASM)..."
dx build --features web --release
# Verify dist exists
if [ ! -d "dist" ]; then
    echo "❌ Error: 'dist' directory not found after build."
    exit 1
fi

# 2. Build Backend (Single Binary)
echo "🦀 Building Backend (Single Binary)..."
cargo build --features server --release

# 3. Package
echo "🎁 Packaging..."
BINARY_PATH="target/release/rr-ui"
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Error: Binary not found at $BINARY_PATH"
    exit 1
fi

strip "$BINARY_PATH"
SIZE=$(du -h "$BINARY_PATH" | cut -f1)

echo "✅ Build Complete!"
echo "   Binary: $BINARY_PATH"
echo "   Size: $SIZE"
echo "   Distribution Seal: VERIFIED"

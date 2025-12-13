#!/usr/bin/env bash
# Build script for Rust library with Flutter Rust Bridge

set -e

PROJECT_ROOT="$(dirname "$0")"
cd "$PROJECT_ROOT"

echo "=== SYN Rust + Flutter Bridge Build ==="
echo ""

# Step 1: Run FRB codegen
echo "[1/3] Running flutter_rust_bridge_codegen..."
cd flutter
flutter_rust_bridge_codegen generate \
  --rust-root ../rust/syn_api \
  --rust-input crate::api \
  --dart-root . \
  --dart-output lib/bridge/bridge_generated

cd ..

# Step 2: Build Rust library
echo ""
echo "[2/3] Building Rust library (release mode without mimalloc)..."
cd rust/syn_api
# Build without default features to avoid TLS block errors from mimalloc
cargo build --release --lib --no-default-features

# Step 3: Copy to Flutter bundle
echo ""
echo "[3/3] Copying library to Flutter bundle..."
FLUTTER_LIB_DIR="../../flutter/linux/bundle/lib"
mkdir -p "$FLUTTER_LIB_DIR"
cp ../target/release/libsyn_api.so "$FLUTTER_LIB_DIR/"

echo ""
echo "✓ Build complete!"
echo ""
echo "To run Flutter app:"
echo "  cd flutter && flutter run"

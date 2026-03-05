#!/bin/bash
set -e

DAT_FILE="data/canto_data.dat"
WASM_TARGET="target/wasm32-unknown-unknown/release/rust_canto.wasm"
WASM_OUTPUT="rust_canto.wasm"

# 1. Build the WASM binary
echo "🦀 Compiling Rust to WASM..."
cargo build --release --target wasm32-unknown-unknown

# 2. Optimize the WASM binary
if command -v wasm-opt >/dev/null 2>&1; then
    echo "🚀 Optimizing with wasm-opt..."
    wasm-opt --enable-bulk-memory --enable-sign-ext --disable-reference-types \
        -Oz --strip-debug \
        target/wasm32-unknown-unknown/release/rust_canto.wasm \
        -o rust_canto.wasm

    OLD_SIZE=$(du -h "$WASM_TARGET" | cut -f1)
    NEW_SIZE=$(du -h "$WASM_OUTPUT" | cut -f1)
    echo "✨ Done! Size reduced from $OLD_SIZE to $NEW_SIZE"
else
    echo "⚠️  wasm-opt not found. Copying unoptimized binary."
    echo "👉 Install 'binaryen' to further reduce file size."
    cp "$WASM_TARGET" "$WASM_OUTPUT"
fi

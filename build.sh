#!/bin/bash
set -e

# 1. Clear old artifacts
cargo clean

# 2. Build for WASM
# This triggers build.rs -> generates trie.dat in OUT_DIR -> bakes it into lib.rs
cargo build --release --target wasm32-unknown-unknown

# 3. Optimize (using system wasm-opt to avoid crate bloat)
INFILE="target/wasm32-unknown-unknown/release/rust_canto.wasm"
OUTFILE="rust_canto.wasm"

if [ -f "$INFILE" ]; then
    echo "📦 Optimizing WASM..."
    wasm-opt --enable-bulk-memory --enable-sign-ext --disable-reference-types \
        -Oz --strip-debug "$INFILE" -o "$OUTFILE"
    echo "✅ Success! Final plugin size:"
    du -h "$OUTFILE"
else
    echo "❌ Error: WASM not found at $INFILE"
    exit 1
fi

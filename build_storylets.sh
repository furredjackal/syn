#!/usr/bin/env bash
# Build script for compiling storylets from JSON to binary format
#
# Usage: ./build_storylets.sh [--verbose]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STORYLETS_DIR="$SCRIPT_DIR/storylets"
OUTPUT_DIR="$SCRIPT_DIR/rust/syn_director/data"
OUTPUT_BIN="$OUTPUT_DIR/storylets.bin"

VERBOSE=""
if [[ "$1" == "--verbose" || "$1" == "-v" ]]; then
    VERBOSE="--verbose"
fi

echo "=== SYN Storylet Build Pipeline ==="
echo "Storylets directory: $STORYLETS_DIR"
echo "Output binary: $OUTPUT_BIN"
echo ""

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

# Count storylet files
STORYLET_COUNT=$(find "$STORYLETS_DIR" -name "*.json" 2>/dev/null | wc -l || echo "0")
echo "Found $STORYLET_COUNT storylet JSON files"
echo ""

# Run the compiler
echo "Compiling storylets..."
cd "$SCRIPT_DIR/rust"

cargo run --bin storyletc --release -- \
    --input "$STORYLETS_DIR" \
    --output "$OUTPUT_BIN" \
    $VERBOSE

# Check output
if [[ -f "$OUTPUT_BIN" ]]; then
    SIZE=$(stat -f%z "$OUTPUT_BIN" 2>/dev/null || stat -c%s "$OUTPUT_BIN")
    echo ""
    echo "✓ Compilation successful!"
    echo "  Binary size: $SIZE bytes"
    echo "  Location: $OUTPUT_BIN"
else
    echo ""
    echo "✗ Compilation failed - output file not created"
    exit 1
fi

#!/bin/bash
# Test script for px - builds example data and verifies output

set -e

# Colours for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No colour

echo "=== px test script ==="
echo

# Build the project
echo "Building px..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

PX="./target/release/px"

if [ ! -f "$PX" ]; then
    echo -e "${RED}Build failed - px binary not found${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful${NC}"
echo

# Create output directory
OUTPUT_DIR="./test-output"
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# Find all shape files in examples
SHAPE_FILES=$(find examples -name "*.shape.md" 2>/dev/null)

if [ -z "$SHAPE_FILES" ]; then
    echo -e "${YELLOW}No example shape files found${NC}"
    exit 0
fi

echo "Processing example shape files..."
echo

# Count shapes
TOTAL_SHAPES=0
FAILED=0

for file in $SHAPE_FILES; do
    echo -n "  $file ... "

    # Get relative path for output subdirectory
    dir=$(dirname "$file")
    outdir="$OUTPUT_DIR/$dir"
    mkdir -p "$outdir"

    # Build the shape file
    if $PX build "$file" -o "$outdir" --scale 4 2>/dev/null; then
        # Count PNG files created
        count=$(ls -1 "$outdir"/*.png 2>/dev/null | wc -l | tr -d ' ')
        echo -e "${GREEN}ok${NC} ($count shapes)"
        TOTAL_SHAPES=$((TOTAL_SHAPES + count))
    else
        echo -e "${RED}FAILED${NC}"
        FAILED=$((FAILED + 1))
    fi
done

echo
echo "=== Results ==="

# List generated files
PNG_COUNT=$(find "$OUTPUT_DIR" -name "*.png" 2>/dev/null | wc -l | tr -d ' ')
echo "Generated $PNG_COUNT PNG files in $OUTPUT_DIR/"

if [ "$FAILED" -gt 0 ]; then
    echo -e "${RED}$FAILED file(s) failed to process${NC}"
    exit 1
fi

echo
echo "Output files:"
find "$OUTPUT_DIR" -name "*.png" -exec ls -lh {} \; | awk '{print "  " $NF " (" $5 ")"}'

echo
echo -e "${GREEN}All tests passed!${NC}"

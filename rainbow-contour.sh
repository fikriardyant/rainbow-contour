#!/usr/bin/env bash
echo "======================================================================"
echo "  RAINBOW CONTOUR - Cut & Fill Difference Map Generator v1.0"
echo "  PAMA Mine Engineering Field Launcher"
echo "======================================================================"

mkdir -p output

if [ -f "./target/release/rainbow-contour" ]; then
    BINARY="./target/release/rainbow-contour"
elif [ -f "./target/debug/rainbow-contour" ]; then
    BINARY="./target/debug/rainbow-contour"
else
    echo "Building release binary..."
    cargo build --release
    BINARY="./target/release/rainbow-contour"
fi

"$BINARY" "$@"

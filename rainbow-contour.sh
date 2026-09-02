#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ -f "$SCRIPT_DIR/rainbow-contour" ]; then
    exec "$SCRIPT_DIR/rainbow-contour" "$@"
elif [ -f "$SCRIPT_DIR/target/release/rainbow-contour" ]; then
    exec "$SCRIPT_DIR/target/release/rainbow-contour" "$@"
elif [ -f "$SCRIPT_DIR/target/debug/rainbow-contour" ]; then
    exec "$SCRIPT_DIR/target/debug/rainbow-contour" "$@"
elif command -v cargo &>/dev/null; then
    exec cargo run --release -- "$@"
else
    echo "[ERROR] Executable binary 'rainbow-contour' tidak ditemukan!"
    echo "Silakan download binary release atau install Rust (cargo) untuk meng-compile."
    exit 1
fi

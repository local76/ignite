#!/bin/bash
# build.sh: Build ignite and package it into the dist/ directory.
set -e

# Navigate to project root relative to this script
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== Building ignite in release mode ==="
cargo build --release

# Collect artifacts
DIST_DIR="$PROJECT_ROOT/dist"
BIN_DIR="$DIST_DIR/binaries"
rm -rf "$DIST_DIR"
mkdir -p "$BIN_DIR"

echo "=== Collecting binaries ==="
if [ -f "target/release/ignite" ]; then
    cp "target/release/ignite" "$BIN_DIR/"
    echo "Copied ignite binary to $BIN_DIR/ignite"
fi
if [ -f "target/release/ignite.exe" ]; then
    cp "target/release/ignite.exe" "$BIN_DIR/"
    echo "Copied ignite.exe to $BIN_DIR/ignite.exe"
fi

# Build debian package if cargo-deb is installed
if command -v cargo-deb &> /dev/null; then
    echo "=== Building DEB package ==="
    if cargo deb; then
        cp target/debian/*.deb "$BIN_DIR/" 2>/dev/null || true
        echo "DEB package created and copied to $BIN_DIR"
    else
        echo "Warning: cargo-deb build failed."
    fi
else
    echo "Skipping DEB package build (cargo-deb not installed)."
fi

echo "=== Build completed successfully! Output in: $DIST_DIR ==="

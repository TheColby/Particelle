#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

echo "╔══════════════════════════════════════╗"
echo "║     Particelle — Install Script      ║"
echo "╚══════════════════════════════════════╝"
echo ""

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust toolchain not found."
    echo "   Install it from https://rustup.rs/ and re-run this script."
    exit 1
fi

RUST_VERSION=$(rustc --version | awk '{print $2}')
echo "✓ Rust toolchain found: $RUST_VERSION"

# Build release binary
echo ""
echo "→ Building release binary…"
cargo build --release

BINARY="target/release/particelle"
if [ ! -f "$BINARY" ]; then
    echo "❌ Build failed — binary not found at $BINARY"
    exit 1
fi

echo "✓ Build complete: $BINARY"

# Install
echo ""
echo "→ Installing to $INSTALL_DIR …"
if [ -w "$INSTALL_DIR" ]; then
    cp "$BINARY" "$INSTALL_DIR/particelle"
else
    echo "  (requires sudo)"
    sudo cp "$BINARY" "$INSTALL_DIR/particelle"
fi

echo "✓ Installed: $(which particelle || echo "$INSTALL_DIR/particelle")"

# Create `ptc` shorthand symlink
echo "→ Creating symlink: ptc → particelle"
if [ -w "$INSTALL_DIR" ]; then
    ln -sf "$INSTALL_DIR/particelle" "$INSTALL_DIR/ptc"
else
    sudo ln -sf "$INSTALL_DIR/particelle" "$INSTALL_DIR/ptc"
fi
echo "✓ Symlink created: $(which ptc || echo "$INSTALL_DIR/ptc")"
echo ""

# Verify
particelle --version 2>/dev/null && echo "" || true

echo "Done. Run 'particelle --help' or 'ptc --help' to get started."
echo "Quick start:  ptc init > patch.yaml && ptc validate patch.yaml"

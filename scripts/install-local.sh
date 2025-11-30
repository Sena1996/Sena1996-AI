#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BINARY="$PROJECT_DIR/target/release/sena"
INSTALL_DIR="$HOME/.local/bin"

if [[ ! -d "$INSTALL_DIR" ]]; then
    mkdir -p "$INSTALL_DIR"
fi

if [[ -f "$BINARY" ]]; then
    cp "$BINARY" "$INSTALL_DIR/sena"
    chmod +x "$INSTALL_DIR/sena"
    VERSION=$("$INSTALL_DIR/sena" --version 2>/dev/null | awk '{print $2}')
    echo "Installed sena v$VERSION to $INSTALL_DIR"
else
    echo "Error: Binary not found at $BINARY"
    echo "Run 'cargo build --release' first."
    exit 1
fi

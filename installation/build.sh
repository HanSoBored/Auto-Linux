#!/bin/bash
set -e

BINARY_NAME="autolinux"
BUILD_DIR="build"
INSTALL_DIR="/usr/local/bin"

echo "🐹 Building $BINARY_NAME (Release Mode)..."
mkdir -p "$BUILD_DIR"
go build -ldflags="-s -w" -o "$BUILD_DIR/$BINARY_NAME" ./cmd/autolinux

echo "📦 Installing to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
    cp "$BUILD_DIR/$BINARY_NAME" "$INSTALL_DIR/"
else
    sudo cp "$BUILD_DIR/$BINARY_NAME" "$INSTALL_DIR/"
fi

echo "✅ Success! Installed as: $BINARY_NAME"

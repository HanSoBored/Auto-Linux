#!/bin/bash
set -e

# --- CONFIGURATION ---
REPO_OWNER="HanSoBored"
REPO_NAME="Auto-Linux"
BINARY_BASE_NAME="autolinux"
FINAL_NAME="autolinux"
INSTALL_DIR="/usr/local/bin"

# --- DETECT SYSTEM ---
OS="$(uname -s)"
ARCH="$(uname -m)"

# --- DETECT TERMUX / ADB ---
IS_TERMUX=false
IS_ADB=false
TEMP_DIR="/tmp"

if [ -n "$TERMUX_VERSION" ] || [ -d "/data/data/com.termux" ]; then
    IS_TERMUX=true
    INSTALL_DIR="/data/data/com.termux/files/usr/bin"
    TEMP_DIR="$HOME"
    echo "📱 Termux detected"
elif [ -d "/data/local/tmp" ] && [ ! -d "/data/data/com.termux" ]; then
    IS_ADB=true
    INSTALL_DIR="/data/local/tmp"
    TEMP_DIR="/data/local/tmp"
    echo "🔌 ADB environment detected"
fi

echo "🔍 Detecting system..."
echo "   OS: $OS"
echo "   Arch: $ARCH"

SUFFIX=""

# 1. DETECT OS & MAP ARCHITECTURE
if [ "$OS" = "Linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        SUFFIX="linux-x86_64"
    elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
        SUFFIX="linux-aarch64"
    elif [[ "$ARCH" == armv7* ]] || [ "$ARCH" = "arm" ]; then
        SUFFIX="linux-armv7"
    else
        echo "❌ Unsupported Architecture: $ARCH on Linux"
        exit 1
    fi
elif [ "$OS" = "Darwin" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        SUFFIX="darwin-x86_64"
    elif [ "$ARCH" = "arm64" ]; then
        # macOS returns 'arm64' for M1/M2, but we named the file 'aarch64'
        SUFFIX="darwin-aarch64"
    else
        echo "❌ Unsupported Architecture: $ARCH on macOS"
        exit 1
    fi
else
    echo "❌ Unsupported OS: $OS"
    exit 1
fi

TARGET_FILE="${BINARY_BASE_NAME}-${SUFFIX}"
echo "🎯 Target Release Asset: $TARGET_FILE"

# --- DOWNLOADING ---
echo "⬇️  Downloading latest release..."
DOWNLOAD_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/latest/download/$TARGET_FILE"

# Use curl to download to temp folder
# -L follows redirects
# -f fails silently on server error (404) so we can catch it
if ! curl -f -L -o "$TEMP_DIR/$BINARY_BASE_NAME" "$DOWNLOAD_URL"; then
    echo "❌ Error: Failed to download. The release asset '$TARGET_FILE' might not exist yet."
    exit 1
fi

# --- INSTALLING ---
echo "📦 Installing to $INSTALL_DIR..."
chmod +x "$TEMP_DIR/$BINARY_BASE_NAME"

# Check write permissions
if [ -w "$INSTALL_DIR" ]; then
    mv "$TEMP_DIR/$BINARY_BASE_NAME" "$INSTALL_DIR/$FINAL_NAME"
else
    echo "🔑 Sudo permission required to move binary to $INSTALL_DIR"
    sudo mv "$TEMP_DIR/$BINARY_BASE_NAME" "$INSTALL_DIR/$FINAL_NAME"
fi

echo "✅ Installed successfully!"
echo "   Binary location: $INSTALL_DIR/$FINAL_NAME"
echo "   You can now run it using: $FINAL_NAME"

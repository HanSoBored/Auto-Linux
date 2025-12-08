#!/system/bin/sh

REPO="HanSoBored/Auto-Linux"
BINARY_NAME="autolinux"
RELEASE_FILE="autolinux-aarch64"

SYSTEM_PATH="/data/local/bin"
USER_PATH="$HOME/.local/bin"

GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== AutoLinux Installer ===${NC}"

find_temp_dir() {
    if [ -n "$TMPDIR" ] && [ -w "$TMPDIR" ]; then
        echo "$TMPDIR"
        return
    fi

    if [ -d "/data/data/com.termux/files/usr/tmp" ] && [ -w "/data/data/com.termux/files/usr/tmp" ]; then
        echo "/data/data/com.termux/files/usr/tmp"
        return
    fi

    if [ -w "$HOME" ]; then
        echo "$HOME"
        return
    fi

    if [ -d "/data/local/tmp" ] && [ -w "/data/local/tmp" ]; then
        echo "/data/local/tmp"
        return
    fi

    echo "."
}

TMP_DIR=$(find_temp_dir)
TARGET_TMP="$TMP_DIR/$BINARY_NAME-tmp"

echo "[*] Working directory: $TMP_DIR"

DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$RELEASE_FILE"

echo "[*] Downloading binary..."
if [ -x "$(command -v curl)" ]; then
    curl -L --fail "$DOWNLOAD_URL" -o "$TARGET_TMP"
elif [ -x "$(command -v wget)" ]; then
    wget -O "$TARGET_TMP" "$DOWNLOAD_URL"
else
    echo -e "${RED}[!] Error: curl atau wget tidak ditemukan.${NC}"
    exit 1
fi

if [ ! -f "$TARGET_TMP" ]; then
    echo -e "${RED}[!] Gagal download.${NC}"
    exit 1
fi

chmod +x "$TARGET_TMP"

INSTALLED_PATH=""

if [ "$(id -u)" = "0" ]; then
    echo "[*] Running as Root. Installing to System Path..."
    mkdir -p "$SYSTEM_PATH"
    mv "$TARGET_TMP" "$SYSTEM_PATH/$BINARY_NAME"
    INSTALLED_PATH="$SYSTEM_PATH/$BINARY_NAME"

elif command -v su >/dev/null 2>&1; then
    echo -e "${YELLOW}[?] Root access detected.${NC}"
    echo "    Attempting to install to system path ($SYSTEM_PATH)..."
    echo "    (Please grant Root permission on your device prompt)"
    
    if su -c "mkdir -p $SYSTEM_PATH && cp $TARGET_TMP $SYSTEM_PATH/$BINARY_NAME && chmod 755 $SYSTEM_PATH/$BINARY_NAME"; then
        echo -e "${GREEN}[OK] Successfully installed as Root!${NC}"
        rm "$TARGET_TMP"
        INSTALLED_PATH="$SYSTEM_PATH/$BINARY_NAME"
        
        TERMUX_BIN="/data/data/com.termux/files/usr/bin"
        if [ -d "$TERMUX_BIN" ]; then
            su -c "ln -sf $SYSTEM_PATH/$BINARY_NAME $TERMUX_BIN/$BINARY_NAME"
        fi
    else
        echo -e "${RED}[!] Root permission denied/failed.${NC}"
        echo "    Falling back to User installation..."
    fi
fi

if [ -z "$INSTALLED_PATH" ]; then
    echo "[*] Installing to User Path ($USER_PATH)..."
    mkdir -p "$USER_PATH"
    mv "$TARGET_TMP" "$USER_PATH/$BINARY_NAME"
    INSTALLED_PATH="$USER_PATH/$BINARY_NAME"
    
    case ":$PATH:" in
        *":$USER_PATH:"*) ;;
        *) 
            echo -e "${YELLOW}[!] Warning: $USER_PATH is not in your PATH.${NC}"
            echo "    Add this to your .bashrc/.zshrc:"
            echo "    export PATH=\"\$PATH:$USER_PATH\""
            ;;
    esac
fi

echo ""
echo -e "${GREEN}=== Installation Complete ===${NC}"
echo "Location: $INSTALLED_PATH"
echo "Run using command:"
echo -e "${GREEN}    $BINARY_NAME${NC}"
echo ""
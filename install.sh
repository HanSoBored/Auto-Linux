#!/system/bin/sh

REPO="HanSoBored/Auto-Linux"
BINARY_NAME="autolinux"
INSTALL_DIR="/data/local/bin"
TERMUX_BIN="/data/data/com.termux/files/usr/bin"

GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== AutoLinux Installer ===${NC}"

# if [ "$(id -u)" != "0" ]; then
#     echo -e "${RED}[!] Error: Jalankan installer sebagai root (su).${NC}"
#     exit 1
# fi

if [ ! -d "$INSTALL_DIR" ]; then
    mkdir -p "$INSTALL_DIR"
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/autolinux-aarch64"
echo "[*] Mengunduh ke $INSTALL_DIR..."

if [ -x "$(command -v curl)" ]; then
    curl -L --fail "$DOWNLOAD_URL" -o "$INSTALL_DIR/$BINARY_NAME"
elif [ -x "$(command -v wget)" ]; then
    wget -O "$INSTALL_DIR/$BINARY_NAME" "$DOWNLOAD_URL"
else
    echo -e "${RED}[!] Error: curl/wget tidak ditemukan.${NC}"
    exit 1
fi

if [ ! -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    echo -e "${RED}[!] Download gagal.${NC}"
    exit 1
fi

chmod 755 "$INSTALL_DIR/$BINARY_NAME"

if [ -d "$TERMUX_BIN" ]; then
    echo -e "${YELLOW}[*] Termux terdeteksi! Membuat shortcut...${NC}"
    rm -f "$TERMUX_BIN/$BINARY_NAME"
    ln -s "$INSTALL_DIR/$BINARY_NAME" "$TERMUX_BIN/$BINARY_NAME"
    echo "    Shortcut dibuat di: $TERMUX_BIN/$BINARY_NAME"
fi

echo -e "${GREEN}[SUCCESS] Instalasi Selesai!${NC}"
echo "Sekarang Anda bisa menjalankan perintah ini dari mana saja:"
echo -e "${GREEN}    $BINARY_NAME${NC}"
echo ""
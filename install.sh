#!/bin/sh
# installer untuk Mac/Linux

set -e

# Deteksi OS dan arsitektur
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "$OS" = "darwin" ]; then
    OS="macos"
fi

if [ "$ARCH" = "x86_64" ] || [ "$ARCH" = "amd64" ]; then
    ARCH="x86_64"
elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
    ARCH="aarch64"
else
    echo "Arsitektur tidak didukung: $ARCH"
    exit 1
fi

if [ "$OS" = "macos" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        TARGET="idpp-x86_64-apple-darwin.tar.gz"
    else
        TARGET="idpp-aarch64-apple-darwin.tar.gz"
    fi
elif [ "$OS" = "linux" ]; then
    TARGET="idpp-x86_64-unknown-linux-gnu.tar.gz"
else
    echo "Sistem operasi tidak didukung: $OS"
    exit 1
fi

# URL rilis
URL="https://github.com/rillToMe/idpp/releases/latest/download/$TARGET"

echo "Mengunduh ID++ dan Rak untuk $OS ($ARCH)..."
curl -sL -o /tmp/idpp_temp.tar.gz "$URL"

echo "Mengekstrak dan memasang ke /usr/local/bin/ (mungkin membutuhkan akses sudo)..."
sudo tar -xzf /tmp/idpp_temp.tar.gz -C /usr/local/bin/
sudo chmod +x /usr/local/bin/idpp /usr/local/bin/rak

# Bersihkan file temp
rm /tmp/idpp_temp.tar.gz

echo "Instalasi selesai! Coba jalankan:"
echo "  idpp --versi"
echo "  rak --help"

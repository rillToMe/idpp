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
        TARGET="idpp-x86_64-apple-darwin"
    else
        TARGET="idpp-aarch64-apple-darwin"
    fi
elif [ "$OS" = "linux" ]; then
    TARGET="idpp-x86_64-unknown-linux-gnu"
else
    echo "Sistem operasi tidak didukung: $OS"
    exit 1
fi

# URL rilis (sesuaikan dengan repository kamu)
URL="https://github.com/rillToMe/idpp/releases/latest/download/$TARGET"

echo "Mengunduh ID++ untuk $OS ($ARCH)..."
curl -sL -o /tmp/idpp "$URL"

echo "Memasang ke /usr/local/bin/idpp (mungkin membutuhkan akses sudo)..."
sudo mv /tmp/idpp /usr/local/bin/idpp
sudo chmod +x /usr/local/bin/idpp

echo "Instalasi selesai! Coba jalankan: idpp --versi"
idpp --versi

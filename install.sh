#!/bin/bash
set -euo pipefail

REPO="ricardosalcedo/agentpack"
INSTALL_DIR="/usr/local/bin"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64) ARCH="amd64" ;;
  aarch64|arm64) ARCH="arm64" ;;
  *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
esac

BINARY="agentpack-${OS}-${ARCH}"
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"v//' | sed 's/".*//')
URL="https://github.com/${REPO}/releases/download/v${LATEST}/${BINARY}"

echo "Installing agentpack v${LATEST} (${OS}/${ARCH})..."
curl -fsSL "$URL" -o /tmp/agentpack
chmod +x /tmp/agentpack
sudo mv /tmp/agentpack "${INSTALL_DIR}/agentpack"
echo "✓ Installed to ${INSTALL_DIR}/agentpack"
agentpack --version

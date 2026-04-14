#!/usr/bin/env bash
# One-liner installer for ave-cloud-rs-cli
#
# Usage: curl -fsSL https://raw.githubusercontent.com/mantis-box/ave-cloud-rs-cli/main/scripts/install.sh | sh
#
# Or download specific version:
# curl -fsSL https://raw.githubusercontent.com/mantis-box/ave-cloud-rs-cli/v0.1.0/scripts/install.sh | sh

set -e

VERSION=${VERSION:-latest}
INSTALL_DIR=${INSTALL_DIR:-/usr/local/bin}
REPO="mantis-box/ave-cloud-rs-cli"

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    aarch64|arm64)
        TARGET="aarch64-unknown-linux-gnu"
        ;;
    armv7l)
        TARGET="armv7-unknown-linux-gnueabihf"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH" >&2
        echo "Supported: x86_64, aarch64, armv7l" >&2
        exit 1
        ;;
esac

echo "Detected architecture: $ARCH (target: $TARGET)"

# Get release info
if [ "$VERSION" = "latest" ]; then
    echo "Fetching latest release..."
    RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"
    # Fetch the actual tag name from the API
    VERSION=$(curl -s "$RELEASE_URL" | grep -o '"tag_name": *"[^"]*"' | cut -d'"' -f4)
    echo "Latest version is $VERSION"
else
    echo "Fetching release $VERSION..."
    RELEASE_URL="https://api.github.com/repos/${REPO}/releases/tags/$VERSION"
fi

# Download and verify
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

BINARY_NAME="ave-cloud-rs-cli-${TARGET}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}"

echo "Downloading from $DOWNLOAD_URL..."
cd "$TEMP_DIR"

if ! curl -fsSL "$DOWNLOAD_URL" -o "$BINARY_NAME"; then
    echo "Error: Failed to download binary" >&2
    echo "Check if release exists: $RELEASE_URL" >&2
    exit 1
fi

# Make executable
chmod +x "$BINARY_NAME"

# Install
if [ -w "$INSTALL_DIR" ]; then
    mv "$BINARY_NAME" "$INSTALL_DIR/ave-cloud-rs-cli"
    echo "Installed to $INSTALL_DIR/ave-cloud-rs-cli"
else
    echo "Note: $INSTALL_DIR is not writable, using sudo..."
    sudo mv "$BINARY_NAME" "$INSTALL_DIR/ave-cloud-rs-cli"
    sudo chmod +x "$INSTALL_DIR/ave-cloud-rs-cli"
    echo "Installed to $INSTALL_DIR/ave-cloud-rs-cli"
fi

echo ""
echo "Installation complete!"
echo ""
echo "Next steps:"
echo "  1. Set environment variables:"
echo "     export AVE_API_KEY=your_api_key"
echo "     export API_PLAN=free"
echo ""
echo "  2. Try it out:"
echo "     ave-cloud-rs-cli price BSC 0x1234..."
echo ""

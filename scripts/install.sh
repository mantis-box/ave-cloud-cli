#!/usr/bin/env bash
# One-liner installer for ave-cloud-cli
#
# Usage: curl -fsSL https://raw.githubusercontent.com/mantis-box/ave-cloud-cli/main/scripts/install.sh | sh
#
# Or download specific version:
# curl -fsSL https://raw.githubusercontent.com/mantis-box/ave-cloud-cli/v0.1.0/scripts/install.sh | sh

set -e

VERSION=${VERSION:-latest}
INSTALL_DIR=${INSTALL_DIR:-/usr/local/bin}
REPO="mantis-box/ave-cloud-cli"

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
    VERSION=$(curl -s "$RELEASE_URL" | grep -o '"tag_name": *"[^"]*"' | cut -d'"' -f4)
    echo "Latest version is $VERSION"
else
    echo "Fetching release $VERSION..."
    RELEASE_URL="https://api.github.com/repos/${REPO}/releases/tags/$VERSION"
fi

# Download and verify
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

BINARY_NAME="ave-cloud-cli-${TARGET}"
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
    mv "$BINARY_NAME" "$INSTALL_DIR/ave-cloud-cli"
    echo "Installed to $INSTALL_DIR/ave-cloud-cli"
else
    echo "Note: $INSTALL_DIR is not writable, using sudo..."
    sudo mv "$BINARY_NAME" "$INSTALL_DIR/ave-cloud-cli"
    sudo chmod +x "$INSTALL_DIR/ave-cloud-cli"
    echo "Installed to $INSTALL_DIR/ave-cloud-cli"
fi

echo ""
echo "Installation complete!"
echo ""
echo "Next steps:"
echo "  1. Set environment variables (add to ~/.bashrc or ~/.zshrc):"
echo ""
echo "     # Required — get your API key from https://cloud.ave.ai"
echo "     export AVE_API_KEY=your_api_key"
echo ""
echo "     # Optional — API plan: free | normal | pro (default: free)"
echo "     export API_PLAN=free"
echo ""
echo "     # Optional — required for proxy wallet commands (market-order, limit-order, list-wallets, etc.)"
echo "     # Get your secret key from https://cloud.ave.ai/account"
echo "     export AVE_SECRET_KEY=your_secret_key"
echo ""
echo "     # Optional — for one-step EVM self-custody swaps (swap-evm)"
echo "     export AVE_EVM_PRIVATE_KEY=your_evm_private_key"
echo ""
echo "     # Optional — for one-step Solana self-custody swaps (swap-solana)"
echo "     export AVE_SOLANA_PRIVATE_KEY=your_solana_private_key"
echo ""
echo "     # Optional — override default public RPC endpoints"
echo "     # export AVE_BSC_RPC_URL=https://bsc.publicnode.com"
echo "     # export AVE_ETH_RPC_URL=https://ethereum.publicnode.com"
echo "     # export AVE_BASE_RPC_URL=https://base.publicnode.com"
echo ""
echo "  2. Reload your shell:"
echo "     source ~/.bashrc   # or source ~/.zshrc"
echo ""
echo "  3. Try it out:"
echo "     ave-cloud-cli search btc"
echo "     ave-cloud-cli chains"
echo "     ave-cloud-cli gas-tip"
echo ""
echo "  4. Proxy wallet commands (requires AVE_SECRET_KEY):"
echo "     ave-cloud-cli list-wallets"
echo "     ave-cloud-cli create-wallet --name my-wallet"
echo ""
echo "  Full command reference: ave-cloud-cli --help"
echo ""

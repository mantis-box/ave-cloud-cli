#!/usr/bin/env bash
# Cross-compile ave-cloud-cli for Raspberry Pi
#
# Usage: ./scripts/build-rpi.sh [aarch64|armv7]
#
# Targets:
#   aarch64 - Raspberry Pi 4/5 (64-bit)
#   armv7   - Raspberry Pi 3/Zero 2 (32-bit)

set -e

TARGET_ARCH=${1:-aarch64}

case $TARGET_ARCH in
    aarch64)
        TARGET="aarch64-unknown-linux-gnu"
        echo "Building for Raspberry Pi 4/5 (aarch64)..."
        ;;
    armv7)
        TARGET="armv7-unknown-linux-gnueabihf"
        echo "Building for Raspberry Pi 3/Zero 2 (armv7)..."
        ;;
    *)
        echo "Usage: $0 [aarch64|armv7]"
        exit 1
        ;;
esac

# Check if target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Installing Rust target $TARGET..."
    rustup target add "$TARGET"
fi

# Check for cross-compiler
CROSS_COMPILER=""
if [ "$TARGET_ARCH" = "aarch64" ]; then
    CROSS_COMPILER="aarch64-linux-gnu-gcc"
else
    CROSS_COMPILER="arm-linux-gnueabihf-gcc"
fi

if ! command -v "$CROSS_COMPILER" &> /dev/null; then
    echo "Installing cross-compiler..."
    sudo apt-get update
    sudo apt-get install -y "gcc-$TARGET_ARCH-linux-gnu"
fi

# Configure Cargo for cross-compilation
mkdir -p .cargo
cat > .cargo/config.toml << EOF
[target.$TARGET]
linker = "$CROSS_COMPILER"
EOF

# Build
echo "Building..."
cargo build --release --target "$TARGET" --features rustls

# Report
OUTPUT="target/$TARGET/release/ave-cloud-cli"
if [ -f "$OUTPUT" ]; then
    SIZE=$(du -h "$OUTPUT" | cut -f1)
    echo ""
    echo "Build complete!"
    echo "  Binary: $OUTPUT"
    echo "  Size: $SIZE"
    echo ""
    echo "To deploy:"
    echo "  scp $OUTPUT pi@raspberrypi:/usr/local/bin/"
else
    echo "Build failed!"
    exit 1
fi

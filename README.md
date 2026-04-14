# ave-cloud-rs-cli

Rust-based CLI for Ave Cloud API that replicates server-side functionality including REST API queries, WebSocket streams, and DEX trading.

## Features

- **REST API**: Token prices, kline data, holders, risk scores, swap transactions
- **WebSocket**: Real-time price/tx/kline streams, order status push
- **Trade**: Chain wallet (self-custody EVM) + proxy wallet (server-managed)
- **Cross-platform**: x86_64, aarch64, armv7 (Raspberry Pi)

## Installation

### Pre-built binaries

```bash
curl -fsSL https://raw.githubusercontent.com/owner/ave-cloud-rs-cli/main/scripts/install.sh | sh
```

### Build from source

```bash
cargo build --release
```

### Docker

```bash
docker pull ghcr.io/owner/ave-cloud-rs-cli:latest
docker run -e AVE_API_KEY=your_key ghcr.io/owner/ave-cloud-rs-cli
```

## Quick Start

```bash
# Set environment
export AVE_API_KEY=your_api_key
export API_PLAN=free

# Get token price
ave-cloud-rs-cli price BSC 0x1234...

# Get token info
ave-cloud-rs-cli info ETH 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2

# Risk check
ave-cloud-rs-cli risk BSC 0x1234...

# Search tokens
ave-cloud-rs-cli search "pepe" --chain BSC
```

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| AVE_API_KEY | Yes | - | API key from cloud.ave.ai |
| API_PLAN | No | free | free, normal, pro |
| AVE_SECRET_KEY | For proxy trading | - | HMAC secret |
| AVE_EVM_PRIVATE_KEY | For chain wallet | - | Hex private key |
| AVE_MNEMONIC | For chain wallet | - | Seed phrase |
| AVE_BSC_RPC_URL | No | publicnode.com | BSC RPC |
| AVE_ETH_RPC_URL | No | publicnode.com | ETH RPC |
| AVE_BASE_RPC_URL | No | publicnode.com | Base RPC |

## CLI Commands

### Data REST (Free+)
```
ave-cloud-rs-cli price <chain> <address>     Token price
ave-cloud-rs-cli info <chain> <address>      Full token info
ave-cloud-rs-cli kline <chain> <address>     OHLCV data
ave-cloud-rs-cli risk <chain> <address>       Risk/honeypot check
ave-cloud-rs-cli search <query>             Token search
```

### Data WebSocket (Pro)
```
ave-cloud-rs-cli stream price <chain> <addr>  Real-time prices
ave-cloud-rs-cli stream tx <chain> <addr>     Real-time transactions
ave-cloud-rs-cli daemon --skill data-wss     Background daemon
```

### Trading (Normal/Pro)
```
ave-cloud-rs-cli buy <chain> <token> <usd>   Market buy
ave-cloud-rs-cli sell <chain> <token> <usd>  Market sell
ave-cloud-rs-cli orders                       List orders
```

## Raspberry Pi

```bash
# Cross-compile for Pi 4
./scripts/build-rpi.sh aarch64

# Deploy
scp target/aarch64-unknown-linux-gnu/release/ave-cloud-rs-cli pi@pi:/usr/local/bin/
```

## Development

```bash
# Build
cargo build

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy -- -D warnings
```

## License

MIT

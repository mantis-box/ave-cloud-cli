# ave-cloud-cli

Hyper-optimized, statically linked Rust binary (~5MB) for Ave Cloud API — REST queries, WebSocket streams, and DEX trading. Cross-compiled for `x86_64`, `aarch64`, and `armv7` (Raspberry Pi).

## Features

- **REST API**: Token prices, kline data, holders, risk scores, trending, smart wallets, swap transactions
- **WebSocket**: Real-time price/tx/kline streams, order status push
- **Trade**: Chain wallet (self-custody EVM + k256 signing) + proxy wallet (server-managed)
- **Cross-platform**: x86_64, aarch64, armv7 (Raspberry Pi)
- **Dependencies**: tokio, reqwest, tokio-tungstenite, k256, clap, zeroize

## Installation

### Pre-built binaries

```bash
curl -fsSL https://raw.githubusercontent.com/mantis-box/ave-cloud-cli/main/scripts/install.sh | sh
```

### Build from source

```bash
cargo build --release
```

### Docker

```bash
docker pull ghcr.io/owner/ave-cloud-cli:latest
docker run -e AVE_API_KEY=your_key ghcr.io/owner/ave-cloud-cli
```

## Quick Start

```bash
# Set environment
export AVE_API_KEY=your_api_key
export API_PLAN=free

# Get token price
ave-cloud-cli price BSC 0x1234...

# Get token info
ave-cloud-cli info ETH 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2

# Risk check
ave-cloud-cli risk BSC 0x1234...

# Search tokens
ave-cloud-cli search "pepe" --chain BSC
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
ave-cloud-cli price <chain> <address>     Token price
ave-cloud-cli info <chain> <address>      Full token info (holders, market cap, supply)
ave-cloud-cli kline <chain> <address>     OHLCV data
ave-cloud-cli risk <chain> <address>       Risk/honeypot check
ave-cloud-cli search <query>             Token search
ave-cloud-cli trending <chain>           Trending tokens (bsc/solana)
ave-cloud-cli ranks --topic <hot|gainer|loser|meme>  Token rankings
ave-cloud-cli signals --chain <chain>     Trading signals (solana)
ave-cloud-cli smart-wallets --chain <chain> --keyword <query>  Whale activity
```

### Data WebSocket (Pro)
```
ave-cloud-cli stream price <chain> <addr>  Real-time prices
ave-cloud-cli stream tx <chain> <addr>     Real-time transactions
ave-cloud-cli stream kline <chain> <addr>  Real-time OHLCV kline updates
ave-cloud-cli wss-repl                     Interactive WSS read-eval-print loop
ave-cloud-cli daemon --skill data-wss     Background daemon
```

### Trading (Normal/Pro)
```
ave-cloud-cli buy <chain> <token> <usd>   Market buy
ave-cloud-cli sell <chain> <token> <usd>  Market sell
ave-cloud-cli orders                       List orders
```

## Raspberry Pi

```bash
# Cross-compile for Pi 4
./scripts/build-rpi.sh aarch64

# Deploy
scp target/aarch64-unknown-linux-gnu/release/ave-cloud-cli pi@pi:/usr/local/bin/
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

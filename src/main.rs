use clap::{Parser, Subcommand};
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod error;
mod rest;
mod trade;
mod types;
mod ws;

use crate::config::Config;
use crate::rest::AveClient;

// ============================================================
// CLI Definition
// ============================================================

#[derive(Parser)]
#[command(name = "ave-cloud-cli")]
#[command(version = "0.1.0")]
#[command(about = "Ave Cloud API CLI — ZeroClaw skill binary", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[arg(short, long, help = "Pretty-print JSON output (default: raw API response)")]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // ============================================================
    // Data REST v2 — Token Queries
    // ============================================================
    /// Search tokens by keyword
    Search {
        query: String,
        #[arg(long)]
        chain: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        orderby: Option<String>,
    },
    /// Batch search token details by address-chain list (up to 50)
    SearchDetails {
        #[arg(long, required = true, num_args = 1..)]
        tokens: Vec<String>,
    },
    /// Get tokens by platform/launchpad tag
    PlatformTokens {
        #[arg(long, required = true)]
        platform: String,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        orderby: Option<String>,
    },
    /// Get token detail by address and chain
    Token {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
    },
    /// Batch get prices for up to 200 tokens
    BatchPrice {
        #[arg(long, required = true, num_args = 1..)]
        tokens: Vec<String>,
        #[arg(long)]
        tvl_min: Option<f64>,
        #[arg(long)]
        volume_min: Option<f64>,
    },
    /// Get a single token price (shorthand: POST /tokens/price)
    Price {
        chain: String,
        address: String,
    },
    /// Get full token info
    Info {
        chain: String,
        address: String,
    },
    /// Get top 100 tokens for a pair
    Top100 {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
    },
    /// Get token holders
    Holders {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        sort_by: Option<String>,
        #[arg(long)]
        order: Option<String>,
    },

    // ============================================================
    // Data REST v2 — Kline
    // ============================================================
    /// Get kline data by token address
    KlineToken {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, default_value = "60")]
        interval: u32,
        #[arg(long, default_value = "24")]
        limit: u32,
    },
    /// Get kline data by pair address
    KlinePair {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, default_value = "60")]
        interval: u32,
        #[arg(long, default_value = "24")]
        limit: u32,
    },
    /// Get Ondo-mapped kline data
    KlineOndo {
        #[arg(long, required = true)]
        pair: String,
        #[arg(long, default_value = "60")]
        interval: u32,
        #[arg(long, default_value = "24")]
        limit: u32,
        #[arg(long)]
        from_time: Option<i64>,
        #[arg(long)]
        to_time: Option<i64>,
    },
    /// Get kline/candlestick data (legacy alias → kline-token)
    Kline {
        chain: String,
        address: String,
        #[arg(long, default_value = "60")]
        interval: u32,
        #[arg(long, default_value = "100")]
        limit: u32,
    },

    // ============================================================
    // Data REST v2 — Market / Rankings
    // ============================================================
    /// Get trending tokens
    Trending {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, default_value = "1")]
        page: u32,
        #[arg(long, default_value = "20")]
        page_size: u32,
    },
    /// List available rank topics
    RankTopics,
    /// Get token rankings by topic
    Ranks {
        #[arg(long, required = true)]
        topic: String,
    },
    /// Get contract risk/security report
    Risk {
        chain: String,
        address: String,
    },
    /// List all supported chains
    Chains,
    /// Get main tokens for a chain
    MainTokens {
        #[arg(long, required = true)]
        chain: String,
    },

    // ============================================================
    // Data REST v2 — Wallet / Address
    // ============================================================
    /// Get wallet swap transaction history
    AddressTxs {
        #[arg(long, required = true)]
        wallet: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long)]
        token: Option<String>,
        #[arg(long)]
        from_time: Option<i64>,
        #[arg(long)]
        last_time: Option<String>,
        #[arg(long)]
        last_id: Option<String>,
        #[arg(long)]
        page_size: Option<u32>,
    },
    /// Get wallet PnL for a specific token
    AddressPnl {
        #[arg(long, required = true)]
        wallet: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        token: String,
    },
    /// Get token holdings for a wallet
    WalletTokens {
        #[arg(long, required = true)]
        wallet: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long)]
        sort: Option<String>,
        #[arg(long)]
        sort_dir: Option<String>,
        #[arg(long)]
        page_size: Option<u32>,
        #[arg(long)]
        page_no: Option<u32>,
        #[arg(long)]
        hide_sold: bool,
        #[arg(long)]
        hide_small: Option<f64>,
        #[arg(long)]
        blue_chips: bool,
    },
    /// Get wallet overview and stats
    WalletInfo {
        #[arg(long, required = true)]
        wallet: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long)]
        self_address: Option<String>,
    },
    /// List smart wallets with profit filters
    SmartWallets {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long)]
        keyword: Option<String>,
        #[arg(long)]
        sort: Option<String>,
        #[arg(long)]
        sort_dir: Option<String>,
    },
    /// Get public trading signals
    Signals {
        #[arg(long)]
        chain: Option<String>,
        #[arg(long)]
        page_size: Option<u32>,
        #[arg(long)]
        page_no: Option<u32>,
    },

    // ============================================================
    // Data REST v2 — Transactions
    // ============================================================
    /// Get swap transactions for a pair
    Txs {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
    },
    /// Get liquidity transactions for a pair
    LiqTxs {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long)]
        type_: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        from_time: Option<i64>,
        #[arg(long)]
        to_time: Option<i64>,
        #[arg(long)]
        sort: Option<String>,
    },
    /// Get transaction detail by hash
    TxDetail {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        account: String,
        #[arg(long, required = true)]
        tx_hash: String,
        #[arg(long)]
        start_from: Option<i64>,
        #[arg(long)]
        end_at: Option<i64>,
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Get trading pair detail
    Pair {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
    },

    // ============================================================
    // Trade Chain Wallet
    // ============================================================
    /// Get swap quote (estimated output)
    Quote {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        in_amount: String,
        #[arg(long, required = true)]
        in_token: String,
        #[arg(long, required = true)]
        out_token: String,
        #[arg(long, required = true)]
        swap_type: String,
    },
    /// Query recommended slippage for a token
    AutoSlippage {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        token: String,
        #[arg(long)]
        use_mev: bool,
    },
    /// Query recommended gas prices per chain
    GasTip,
    /// Approve ERC-20 token for chain wallet swap router
    ApproveChain {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        token: String,
        #[arg(long)]
        in_amount: Option<String>,
        #[arg(long)]
        rpc_url: Option<String>,
    },
    /// Create unsigned EVM swap transaction
    CreateEvmTx {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        creator_address: String,
        #[arg(long, required = true)]
        in_amount: String,
        #[arg(long, required = true)]
        in_token: String,
        #[arg(long, required = true)]
        out_token: String,
        #[arg(long, required = true)]
        swap_type: String,
        #[arg(long, required = true)]
        slippage: String,
        #[arg(long)]
        fee_recipient: Option<String>,
        #[arg(long)]
        fee_recipient_rate: Option<String>,
        #[arg(long)]
        auto_slippage: bool,
    },
    /// Submit signed EVM transaction
    SendEvmTx {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        request_tx_id: String,
        #[arg(long, required = true)]
        signed_tx: String,
        #[arg(long)]
        use_mev: bool,
    },
    /// Create unsigned Solana swap transaction
    CreateSolanaTx {
        #[arg(long, required = true)]
        creator_address: String,
        #[arg(long, required = true)]
        in_amount: String,
        #[arg(long, required = true)]
        in_token: String,
        #[arg(long, required = true)]
        out_token: String,
        #[arg(long, required = true)]
        swap_type: String,
        #[arg(long, required = true)]
        slippage: String,
        #[arg(long, required = true)]
        fee: String,
        #[arg(long)]
        use_mev: bool,
        #[arg(long)]
        fee_recipient: Option<String>,
        #[arg(long)]
        fee_recipient_rate: Option<String>,
        #[arg(long)]
        auto_slippage: bool,
    },
    /// Submit signed Solana transaction
    SendSolanaTx {
        #[arg(long, required = true)]
        request_tx_id: String,
        #[arg(long, required = true)]
        signed_tx: String,
        #[arg(long)]
        use_mev: bool,
    },
    /// One-step EVM swap: create + sign + send (requires AVE_EVM_PRIVATE_KEY)
    SwapEvm {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        in_amount: String,
        #[arg(long, required = true)]
        in_token: String,
        #[arg(long, required = true)]
        out_token: String,
        #[arg(long, required = true)]
        swap_type: String,
        #[arg(long, required = true)]
        slippage: String,
        #[arg(long)]
        fee_recipient: Option<String>,
        #[arg(long)]
        fee_recipient_rate: Option<String>,
        #[arg(long)]
        auto_slippage: bool,
        #[arg(long)]
        use_mev: bool,
        #[arg(long)]
        rpc_url: Option<String>,
    },
    /// One-step Solana swap: create + sign + send (requires AVE_SOLANA_PRIVATE_KEY)
    SwapSolana {
        #[arg(long, required = true)]
        in_amount: String,
        #[arg(long, required = true)]
        in_token: String,
        #[arg(long, required = true)]
        out_token: String,
        #[arg(long, required = true)]
        swap_type: String,
        #[arg(long, required = true)]
        slippage: String,
        #[arg(long, required = true)]
        fee: String,
        #[arg(long)]
        fee_recipient: Option<String>,
        #[arg(long)]
        fee_recipient_rate: Option<String>,
        #[arg(long)]
        auto_slippage: bool,
        #[arg(long)]
        use_mev: bool,
    },

    // ============================================================
    // Trade Proxy Wallet
    // ============================================================
    /// List proxy wallets
    ListWallets {
        #[arg(long)]
        assets_ids: Option<String>,
    },
    /// Create a delegate proxy wallet
    CreateWallet {
        #[arg(long, required = true)]
        name: String,
        #[arg(long)]
        return_mnemonic: bool,
    },
    /// Delete delegate proxy wallets
    DeleteWallet {
        #[arg(long, required = true, num_args = 1..)]
        assets_ids: Vec<String>,
    },
    /// Place a market swap order (proxy wallet)
    MarketOrder {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        assets_id: String,
        #[arg(long, required = true)]
        in_token: String,
        #[arg(long, required = true)]
        out_token: String,
        #[arg(long, required = true)]
        in_amount: String,
        #[arg(long, required = true)]
        swap_type: String,
        #[arg(long, required = true)]
        slippage: String,
        #[arg(long)]
        use_mev: bool,
        #[arg(long)]
        gas: Option<String>,
        #[arg(long)]
        extra_gas: Option<String>,
        #[arg(long)]
        auto_slippage: bool,
        #[arg(long)]
        auto_gas: Option<String>,
        #[arg(long)]
        auto_sell: Option<Vec<String>>,
    },
    /// Place a limit order (proxy wallet)
    LimitOrder {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        assets_id: String,
        #[arg(long, required = true)]
        in_token: String,
        #[arg(long, required = true)]
        out_token: String,
        #[arg(long, required = true)]
        in_amount: String,
        #[arg(long, required = true)]
        swap_type: String,
        #[arg(long, required = true)]
        slippage: String,
        #[arg(long, required = true)]
        limit_price: String,
        #[arg(long)]
        use_mev: bool,
        #[arg(long)]
        gas: Option<String>,
        #[arg(long)]
        extra_gas: Option<String>,
        #[arg(long)]
        expire_time: Option<i64>,
        #[arg(long)]
        auto_slippage: bool,
        #[arg(long)]
        auto_gas: Option<String>,
    },
    /// Query market swap orders by IDs
    GetSwapOrders {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        ids: String,
    },
    /// Query limit orders (paginated)
    GetLimitOrders {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        assets_id: String,
        #[arg(long, required = true)]
        page_size: u32,
        #[arg(long, required = true)]
        page_no: u32,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        token: Option<String>,
    },
    /// Cancel pending limit orders
    CancelLimitOrder {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true, num_args = 1..)]
        ids: Vec<String>,
    },
    /// Approve token for EVM proxy wallet trading
    ApproveToken {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        assets_id: String,
        #[arg(long, required = true)]
        token_address: String,
    },
    /// Query token approval status
    GetApproval {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        ids: String,
    },
    /// Transfer tokens from a delegate proxy wallet
    Transfer {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        assets_id: String,
        #[arg(long, required = true)]
        from_address: String,
        #[arg(long, required = true)]
        to_address: String,
        #[arg(long, required = true)]
        token_address: String,
        #[arg(long, required = true)]
        amount: String,
        #[arg(long)]
        gas: Option<String>,
        #[arg(long)]
        extra_gas: Option<String>,
    },
    /// Query transfer status
    GetTransfer {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        ids: String,
    },
    /// Simple buy via proxy wallet (requires Normal/Pro plan)
    Buy {
        chain: String,
        token: String,
        amount_usd: f64,
        #[arg(long, help = "Perform honeypot check before buying")]
        safe: bool,
        #[arg(long, help = "Take profit price")]
        tp: Option<f64>,
        #[arg(long, help = "Stop loss price")]
        sl: Option<f64>,
    },
    /// Simple sell via proxy wallet (requires Normal/Pro plan)
    Sell {
        chain: String,
        token: String,
        amount_usd: f64,
    },
    /// List proxy orders (market swap orders)
    Orders {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true, help = "Your proxy wallet assetsId")]
        assets_id: String,
        #[arg(long)]
        status: Option<String>,
    },

    // ============================================================
    // Data WSS — WebSocket Streams (requires Pro plan)
    // ============================================================
    /// Interactive WebSocket REPL (pro plan)
    WssRepl,
    /// Stream live price updates for tokens (pro plan)
    WatchPrice {
        #[arg(long, required = true, num_args = 1..)]
        tokens: Vec<String>,
    },
    /// Stream live kline updates (pro plan)
    WatchKline {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, default_value = "k60")]
        interval: String,
        #[arg(long)]
        format: Option<String>,
    },
    /// Stream live swap/liquidity events (pro plan)
    WatchTx {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, default_value = "tx")]
        topic: String,
    },
    /// Stream WebSocket (daemon mode)
    Daemon {
        #[arg(long, default_value = "data-wss")]
        skill: String,
    },
    /// Stream real-time price updates (pro plan, legacy alias)
    StreamPrice {
        chain: String,
        address: String,
    },
    /// Stream real-time transactions (pro plan, legacy alias)
    StreamTx {
        chain: String,
        address: String,
    },
}

// ============================================================
// Main
// ============================================================

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let log_level = match cli.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    // All logs go to stderr — stdout is reserved for data (matches Python)
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_writer(std::io::stderr)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = execute_command(cli, config).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

// ============================================================
// Command Execution
// All data commands output raw API JSON to stdout (matches Python)
// ============================================================

async fn execute_command(cli: Cli, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let client = AveClient::new(config.clone())?;

    // Helper: print raw API response exactly as Python does
    // Python: print(json.dumps(resp.json(), indent=2))
    macro_rules! out {
        ($val:expr) => {
            println!("{}", serde_json::to_string_pretty($val)?)
        };
    }

    match cli.command {
        // ============================================================
        // Data REST v2 — Token Queries
        // ============================================================
        Commands::Search { query, chain, limit, orderby } => {
            let mut params: Vec<(String, String)> = vec![
                ("keyword".to_string(), query),
                ("limit".to_string(), limit.to_string()),
            ];
            if let Some(c) = chain {
                params.push(("chain".to_string(), c));
            }
            if let Some(o) = orderby {
                params.push(("orderby".to_string(), o));
            }
            let raw = client.get_v2_raw("/tokens", &params).await?;
            out!(&raw);
        }

        Commands::SearchDetails { tokens } => {
            let body = serde_json::json!({ "token_ids": tokens });
            let raw = client.post_v2_raw("/tokens/search", body).await?;
            out!(&raw);
        }

        Commands::PlatformTokens { platform, limit, orderby } => {
            let mut params: Vec<(String, String)> = vec![("tag".to_string(), platform)];
            if let Some(l) = limit {
                params.push(("limit".to_string(), l.to_string()));
            }
            if let Some(o) = orderby {
                params.push(("orderby".to_string(), o));
            }
            let raw = client.get_v2_raw("/tokens/platform", &params).await?;
            out!(&raw);
        }

        Commands::Token { address, chain } => {
            let path = format!("/tokens/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let raw = client.get_v2_raw(&path, &[]).await?;
            out!(&raw);
        }

        Commands::BatchPrice { tokens, tvl_min, volume_min } => {
            let mut body = serde_json::json!({ "token_ids": tokens });
            if let Some(tvl) = tvl_min {
                body["tvl_min"] = serde_json::json!(tvl);
            }
            if let Some(vol) = volume_min {
                body["tx_24h_volume_min"] = serde_json::json!(vol);
            }
            let raw = client.post_v2_raw("/tokens/price", body).await?;
            out!(&raw);
        }

        // shorthand: price <chain> <address>
        Commands::Price { chain, address } => {
            let body = serde_json::json!({
                "token_ids": [format!("{}-{}", address.to_lowercase(), chain.to_lowercase())]
            });
            let raw = client.post_v2_raw("/tokens/price", body).await?;
            out!(&raw);
        }

        // shorthand: info <chain> <address>
        Commands::Info { chain, address } => {
            let path = format!("/tokens/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let raw = client.get_v2_raw(&path, &[]).await?;
            out!(&raw);
        }

        Commands::Top100 { address, chain } => {
            let path = format!("/tokens/top100/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let raw = client.get_v2_raw(&path, &[]).await?;
            out!(&raw);
        }

        Commands::Holders { address, chain, limit, sort_by, order } => {
            let mut params: Vec<(String, String)> = vec![];
            if let Some(l) = limit {
                params.push(("limit".to_string(), l.to_string()));
            }
            if let Some(s) = sort_by {
                params.push(("sort_by".to_string(), s));
            }
            if let Some(o) = order {
                params.push(("order".to_string(), o));
            }
            let path = format!("/tokens/holders/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let raw = client.get_v2_raw(&path, &params).await?;
            out!(&raw);
        }

        // ============================================================
        // Kline
        // ============================================================
        Commands::KlineToken { address, chain, interval, limit } => {
            let path = format!("/klines/token/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let params = vec![
                ("interval".to_string(), interval.to_string()),
                ("limit".to_string(), limit.to_string()),
            ];
            let raw = client.get_v2_raw(&path, &params).await?;
            out!(&raw);
        }

        Commands::KlinePair { address, chain, interval, limit } => {
            let path = format!("/klines/pair/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let params = vec![
                ("interval".to_string(), interval.to_string()),
                ("limit".to_string(), limit.to_string()),
            ];
            let raw = client.get_v2_raw(&path, &params).await?;
            out!(&raw);
        }

        Commands::KlineOndo { pair, interval, limit, from_time, to_time } => {
            let mut params = vec![
                ("interval".to_string(), interval.to_string()),
                ("limit".to_string(), limit.to_string()),
            ];
            if let Some(t) = from_time {
                params.push(("from_time".to_string(), t.to_string()));
            }
            if let Some(t) = to_time {
                params.push(("to_time".to_string(), t.to_string()));
            }
            let path = format!("/klines/pair/ondo/{}", pair);
            let raw = client.get_v2_raw(&path, &params).await?;
            out!(&raw);
        }

        Commands::Kline { chain, address, interval, limit } => {
            let path = format!("/klines/token/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let params = vec![
                ("interval".to_string(), interval.to_string()),
                ("limit".to_string(), limit.to_string()),
            ];
            let raw = client.get_v2_raw(&path, &params).await?;
            out!(&raw);
        }

        // ============================================================
        // Market / Rankings
        // ============================================================
        Commands::Trending { chain, page, page_size } => {
            let params = vec![
                ("chain".to_string(), chain),
                ("current_page".to_string(), page.to_string()),
                ("page_size".to_string(), page_size.to_string()),
            ];
            let raw = client.get_v2_raw("/tokens/trending", &params).await?;
            out!(&raw);
        }

        Commands::RankTopics => {
            let raw = client.get_v2_raw("/ranks/topics", &[]).await?;
            out!(&raw);
        }

        Commands::Ranks { topic } => {
            let params = vec![("topic".to_string(), topic)];
            let raw = client.get_v2_raw("/ranks", &params).await?;
            out!(&raw);
        }

        Commands::Risk { chain, address } => {
            let path = format!("/contracts/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let raw = client.get_v2_raw(&path, &[]).await?;
            out!(&raw);
        }

        Commands::Chains => {
            let raw = client.get_v2_raw("/supported_chains", &[]).await?;
            out!(&raw);
        }

        Commands::MainTokens { chain } => {
            let params = vec![("chain".to_string(), chain)];
            let raw = client.get_v2_raw("/tokens/main", &params).await?;
            out!(&raw);
        }

        // ============================================================
        // Wallet / Address
        // ============================================================
        Commands::AddressTxs { wallet, chain, token, from_time, last_time, last_id, page_size } => {
            let mut params = vec![
                ("wallet_address".to_string(), wallet),
                ("chain".to_string(), chain),
            ];
            if let Some(t) = token {
                params.push(("token_address".to_string(), t));
            }
            if let Some(t) = from_time {
                params.push(("from_time".to_string(), t.to_string()));
            }
            if let Some(t) = last_time {
                params.push(("last_time".to_string(), t));
            }
            if let Some(id) = last_id {
                params.push(("last_id".to_string(), id));
            }
            if let Some(s) = page_size {
                params.push(("page_size".to_string(), s.to_string()));
            }
            let raw = client.get_v2_raw("/address/tx", &params).await?;
            out!(&raw);
        }

        Commands::AddressPnl { wallet, chain, token } => {
            let params = vec![
                ("wallet_address".to_string(), wallet),
                ("chain".to_string(), chain),
                ("token_address".to_string(), token),
            ];
            let raw = client.get_v2_raw("/address/pnl", &params).await?;
            out!(&raw);
        }

        Commands::WalletTokens {
            wallet, chain, sort, sort_dir, page_size, page_no, hide_sold, hide_small, blue_chips,
        } => {
            let mut params = vec![
                ("wallet_address".to_string(), wallet),
                ("chain".to_string(), chain),
            ];
            if let Some(s) = sort { params.push(("sort".to_string(), s)); }
            if let Some(d) = sort_dir { params.push(("sort_dir".to_string(), d)); }
            if let Some(s) = page_size { params.push(("pageSize".to_string(), s.to_string())); }
            if let Some(n) = page_no { params.push(("pageNO".to_string(), n.to_string())); }
            if hide_sold { params.push(("hide_sold".to_string(), "1".to_string())); }
            if let Some(h) = hide_small { params.push(("hide_small".to_string(), h.to_string())); }
            if blue_chips { params.push(("blue_chips".to_string(), "1".to_string())); }
            let raw = client.get_v2_raw("/address/walletinfo/tokens", &params).await?;
            out!(&raw);
        }

        Commands::WalletInfo { wallet, chain, self_address } => {
            let mut params = vec![
                ("wallet_address".to_string(), wallet),
                ("chain".to_string(), chain),
            ];
            if let Some(a) = self_address { params.push(("self_address".to_string(), a)); }
            let raw = client.get_v2_raw("/address/walletinfo", &params).await?;
            out!(&raw);
        }

        Commands::SmartWallets { chain, keyword, sort, sort_dir } => {
            let mut params = vec![("chain".to_string(), chain)];
            if let Some(k) = keyword { params.push(("keyword".to_string(), k)); }
            if let Some(s) = sort { params.push(("sort".to_string(), s)); }
            if let Some(d) = sort_dir { params.push(("sort_dir".to_string(), d)); }
            let raw = client.get_v2_raw("/address/smart_wallet/list", &params).await?;
            out!(&raw);
        }

        Commands::Signals { chain, page_size, page_no } => {
            let mut params = vec![
                ("chain".to_string(), chain.unwrap_or_else(|| "solana".to_string())),
            ];
            if let Some(s) = page_size { params.push(("pageSize".to_string(), s.to_string())); }
            if let Some(n) = page_no { params.push(("pageNO".to_string(), n.to_string())); }
            let raw = client.get_v2_raw("/signals/public/list", &params).await?;
            out!(&raw);
        }

        // ============================================================
        // Transactions
        // ============================================================
        Commands::Txs { address, chain } => {
            let path = format!("/txs/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let raw = client.get_v2_raw(&path, &[]).await?;
            out!(&raw);
        }

        Commands::LiqTxs { address, chain, type_, limit, from_time, to_time, sort } => {
            let mut params: Vec<(String, String)> = vec![];
            if let Some(t) = type_ { params.push(("type".to_string(), t)); }
            if let Some(l) = limit { params.push(("limit".to_string(), l.to_string())); }
            if let Some(t) = from_time { params.push(("from_time".to_string(), t.to_string())); }
            if let Some(t) = to_time { params.push(("to_time".to_string(), t.to_string())); }
            if let Some(s) = sort { params.push(("sort".to_string(), s)); }
            let path = format!("/txs/liq/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let raw = client.get_v2_raw(&path, &params).await?;
            out!(&raw);
        }

        Commands::TxDetail { chain, account, tx_hash, start_from, end_at, limit } => {
            let mut params = vec![
                ("chain".to_string(), chain),
                ("account_address".to_string(), account),
                ("tx_hash".to_string(), tx_hash),
            ];
            if let Some(t) = start_from { params.push(("start_from".to_string(), t.to_string())); }
            if let Some(t) = end_at { params.push(("end_at".to_string(), t.to_string())); }
            if let Some(l) = limit { params.push(("limit".to_string(), l.to_string())); }
            let raw = client.get_v2_raw("/txs/detail", &params).await?;
            out!(&raw);
        }

        Commands::Pair { address, chain } => {
            let path = format!("/pairs/{}-{}", address.to_lowercase(), chain.to_lowercase());
            let raw = client.get_v2_raw(&path, &[]).await?;
            out!(&raw);
        }

        // ============================================================
        // Trade Chain Wallet
        // ============================================================
        Commands::Quote { chain, in_amount, in_token, out_token, swap_type } => {
            let body = serde_json::json!({
                "chain": chain,
                "inAmount": in_amount,
                "inTokenAddress": in_token,
                "outTokenAddress": out_token,
                "swapType": swap_type,
            });
            let raw = client.trade_post_raw("/v1/thirdParty/chainWallet/getAmountOut", body, false).await?;
            out!(&raw);
        }

        Commands::AutoSlippage { chain, token, use_mev } => {
            let body = serde_json::json!({
                "chain": chain,
                "tokenAddress": token,
                "useMev": use_mev,
            });
            let raw = client.trade_post_raw("/v1/thirdParty/chainWallet/getAutoSlippage", body, false).await?;
            out!(&raw);
        }

        Commands::GasTip => {
            let raw = client.trade_get_raw("/v1/thirdParty/chainWallet/getGasTip", &[], false).await?;
            out!(&raw);
        }

        Commands::ApproveChain { chain, token, in_amount, rpc_url: _ } => {
            // Returns the spender address from a quote so the user can approve it
            let body = serde_json::json!({
                "chain": chain,
                "inAmount": in_amount.unwrap_or_else(|| "1000000000000000000".to_string()),
                "inTokenAddress": token,
                "outTokenAddress": "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                "swapType": "sell",
            });
            let raw = client.trade_post_raw("/v1/thirdParty/chainWallet/getAmountOut", body, false).await?;
            out!(&raw);
        }

        Commands::CreateEvmTx {
            chain, creator_address, in_amount, in_token, out_token,
            swap_type, slippage, fee_recipient, fee_recipient_rate, auto_slippage,
        } => {
            let mut body = serde_json::json!({
                "chain": chain,
                "creatorAddress": creator_address,
                "inAmount": in_amount,
                "inTokenAddress": in_token,
                "outTokenAddress": out_token,
                "swapType": swap_type,
                "slippage": slippage,
            });
            if let Some(r) = fee_recipient { body["feeRecipient"] = serde_json::json!(r); }
            if let Some(r) = fee_recipient_rate { body["feeRecipientRate"] = serde_json::json!(r); }
            if auto_slippage { body["autoSlippage"] = serde_json::json!(true); }
            let raw = client.trade_post_raw("/v1/thirdParty/chainWallet/createEvmTx", body, false).await?;
            out!(&raw);
        }

        Commands::SendEvmTx { chain, request_tx_id, signed_tx, use_mev } => {
            let mut body = serde_json::json!({
                "chain": chain,
                "requestTxId": request_tx_id,
                "signedTx": signed_tx,
            });
            if use_mev { body["useMev"] = serde_json::json!(true); }
            let raw = client.trade_post_raw("/v1/thirdParty/chainWallet/sendSignedEvmTx", body, false).await?;
            out!(&raw);
        }

        Commands::CreateSolanaTx {
            creator_address, in_amount, in_token, out_token, swap_type,
            slippage, fee, use_mev, fee_recipient, fee_recipient_rate, auto_slippage,
        } => {
            let mut body = serde_json::json!({
                "creatorAddress": creator_address,
                "inAmount": in_amount,
                "inTokenAddress": in_token,
                "outTokenAddress": out_token,
                "swapType": swap_type,
                "slippage": slippage,
                "fee": fee,
            });
            if use_mev { body["useMev"] = serde_json::json!(true); }
            if let Some(r) = fee_recipient { body["feeRecipient"] = serde_json::json!(r); }
            if let Some(r) = fee_recipient_rate { body["feeRecipientRate"] = serde_json::json!(r); }
            if auto_slippage { body["autoSlippage"] = serde_json::json!(true); }
            let raw = client.trade_post_raw("/v1/thirdParty/chainWallet/createSolanaTx", body, false).await?;
            out!(&raw);
        }

        Commands::SendSolanaTx { request_tx_id, signed_tx, use_mev } => {
            let mut body = serde_json::json!({
                "requestTxId": request_tx_id,
                "signedTx": signed_tx,
            });
            if use_mev { body["useMev"] = serde_json::json!(true); }
            let raw = client.trade_post_raw("/v1/thirdParty/chainWallet/sendSignedSolanaTx", body, false).await?;
            out!(&raw);
        }

        Commands::SwapEvm { .. } => {
            return Err("swap-evm requires local EVM private key (AVE_EVM_PRIVATE_KEY). Use create-evm-tx then sign-and-send-evm-tx separately.".into());
        }

        Commands::SwapSolana { .. } => {
            return Err("swap-solana requires local Solana private key (AVE_SOLANA_PRIVATE_KEY). Use create-solana-tx then sign-and-send-solana-tx separately.".into());
        }

        // ============================================================
        // Trade Proxy Wallet
        // ============================================================
        Commands::ListWallets { assets_ids } => {
            let params: Vec<(String, String)> = if let Some(ids) = assets_ids {
                vec![("assetsIds".to_string(), ids)]
            } else {
                vec![]
            };
            let raw = client.trade_get_raw("/v1/thirdParty/user/getUserByAssetsId", &params, true).await?;
            out!(&raw);
        }

        Commands::CreateWallet { name, return_mnemonic } => {
            let mut body = serde_json::json!({ "assetsName": name });
            if return_mnemonic { body["returnMnemonic"] = serde_json::json!(true); }
            let raw = client.trade_post_raw("/v1/thirdParty/user/generateWallet", body, true).await?;
            out!(&raw);
        }

        Commands::DeleteWallet { assets_ids } => {
            let body = serde_json::json!({ "assetsIds": assets_ids });
            let raw = client.trade_post_raw("/v1/thirdParty/user/deleteWallet", body, true).await?;
            out!(&raw);
        }

        Commands::MarketOrder {
            chain, assets_id, in_token, out_token, in_amount, swap_type,
            slippage, use_mev, gas, extra_gas, auto_slippage, auto_gas, auto_sell,
        } => {
            let mut body = serde_json::json!({
                "chain": chain,
                "assetsId": assets_id,
                "inTokenAddress": in_token,
                "outTokenAddress": out_token,
                "inAmount": in_amount,
                "swapType": swap_type,
                "slippage": slippage,
                "useMev": use_mev,
            });
            if let Some(g) = gas { body["gas"] = serde_json::json!(g); }
            if let Some(g) = extra_gas { body["extraGas"] = serde_json::json!(g); }
            if auto_slippage { body["autoSlippage"] = serde_json::json!(true); }
            if let Some(a) = auto_gas { body["autoGas"] = serde_json::json!(a); }
            if let Some(c) = auto_sell {
                let parsed: Vec<serde_json::Value> = c.iter()
                    .filter_map(|s| serde_json::from_str(s).ok())
                    .collect();
                body["autoSellConfig"] = serde_json::json!(parsed);
            }
            let raw = client.trade_post_raw("/v1/thirdParty/tx/sendSwapOrder", body, true).await?;
            out!(&raw);
        }

        Commands::LimitOrder {
            chain, assets_id, in_token, out_token, in_amount, swap_type,
            slippage, limit_price, use_mev, gas, extra_gas, expire_time, auto_slippage, auto_gas,
        } => {
            let mut body = serde_json::json!({
                "chain": chain,
                "assetsId": assets_id,
                "inTokenAddress": in_token,
                "outTokenAddress": out_token,
                "inAmount": in_amount,
                "swapType": swap_type,
                "slippage": slippage,
                "limitPrice": limit_price,
                "useMev": use_mev,
            });
            if let Some(g) = gas { body["gas"] = serde_json::json!(g); }
            if let Some(g) = extra_gas { body["extraGas"] = serde_json::json!(g); }
            if let Some(e) = expire_time { body["expireTime"] = serde_json::json!(e); }
            if auto_slippage { body["autoSlippage"] = serde_json::json!(true); }
            if let Some(a) = auto_gas { body["autoGas"] = serde_json::json!(a); }
            let raw = client.trade_post_raw("/v1/thirdParty/tx/sendLimitOrder", body, true).await?;
            out!(&raw);
        }

        Commands::GetSwapOrders { chain, ids } => {
            let params = vec![
                ("chain".to_string(), chain),
                ("ids".to_string(), ids),
            ];
            let raw = client.trade_get_raw("/v1/thirdParty/tx/getSwapOrder", &params, true).await?;
            out!(&raw);
        }

        Commands::GetLimitOrders { chain, assets_id, page_size, page_no, status, token } => {
            let mut params = vec![
                ("chain".to_string(), chain),
                ("assetsId".to_string(), assets_id),
                ("pageSize".to_string(), page_size.to_string()),
                ("pageNo".to_string(), page_no.to_string()),
            ];
            if let Some(s) = status { params.push(("status".to_string(), s)); }
            if let Some(t) = token { params.push(("token".to_string(), t)); }
            let raw = client.trade_get_raw("/v1/thirdParty/tx/getLimitOrder", &params, true).await?;
            out!(&raw);
        }

        Commands::CancelLimitOrder { chain, ids } => {
            let body = serde_json::json!({ "chain": chain, "ids": ids });
            let raw = client.trade_post_raw("/v1/thirdParty/tx/cancelLimitOrder", body, true).await?;
            out!(&raw);
        }

        Commands::ApproveToken { chain, assets_id, token_address } => {
            let body = serde_json::json!({
                "chain": chain,
                "assetsId": assets_id,
                "tokenAddress": token_address,
            });
            let raw = client.trade_post_raw("/v1/thirdParty/tx/approve", body, true).await?;
            out!(&raw);
        }

        Commands::GetApproval { chain, ids } => {
            let params = vec![
                ("chain".to_string(), chain),
                ("ids".to_string(), ids),
            ];
            let raw = client.trade_get_raw("/v1/thirdParty/tx/getApprove", &params, true).await?;
            out!(&raw);
        }

        Commands::Transfer { chain, assets_id, from_address, to_address, token_address, amount, gas, extra_gas } => {
            let mut body = serde_json::json!({
                "chain": chain,
                "assetsId": assets_id,
                "fromAddress": from_address,
                "toAddress": to_address,
                "tokenAddress": token_address,
                "amount": amount,
            });
            if let Some(g) = gas { body["gas"] = serde_json::json!(g); }
            if let Some(g) = extra_gas { body["extraGas"] = serde_json::json!(g); }
            let raw = client.trade_post_raw("/v1/thirdParty/tx/transfer", body, true).await?;
            out!(&raw);
        }

        Commands::GetTransfer { chain, ids } => {
            let params = vec![
                ("chain".to_string(), chain),
                ("ids".to_string(), ids),
            ];
            let raw = client.trade_get_raw("/v1/thirdParty/tx/getTransfer", &params, true).await?;
            out!(&raw);
        }

        // ============================================================
        // Simple High-Level Proxy Wallet (Buy/Sell/Orders)
        // ============================================================
        Commands::Buy { chain, token, amount_usd, safe, tp, sl } => {
            if safe {
                let path = format!("/contracts/{}-{}", token.to_lowercase(), chain.to_lowercase());
                let raw = client.get_v2_raw(&path, &[]).await?;
                if let Some(data) = raw.get("data") {
                    let is_honeypot = data.get("is_honeypot")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if is_honeypot {
                        warn!("Token {} is flagged as HONEYPOT!", token);
                        return Err("Refusing to buy honeypot token".into());
                    }
                }
                info!("Risk check passed, placing buy order...");
            }

            let mut body = serde_json::json!({
                "chain": chain,
                "inTokenAddress": "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                "outTokenAddress": token,
                "inAmount": (amount_usd * 1e18) as u64,
                "swapType": "buy",
                "slippage": "100",
            });
            if let Some(tp_price) = tp { body["tpPrice"] = serde_json::json!(tp_price); }
            if let Some(sl_price) = sl { body["slPrice"] = serde_json::json!(sl_price); }
            let raw = client.trade_post_raw("/v1/thirdParty/tx/sendSwapOrder", body, true).await?;
            out!(&raw);
        }

        Commands::Sell { chain, token, amount_usd } => {
            let body = serde_json::json!({
                "chain": chain,
                "inTokenAddress": token,
                "outTokenAddress": "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                "inAmount": (amount_usd * 1e18) as u64,
                "swapType": "sell",
                "slippage": "100",
            });
            let raw = client.trade_post_raw("/v1/thirdParty/tx/sendSwapOrder", body, true).await?;
            out!(&raw);
        }

        Commands::Orders { chain, assets_id, status } => {
            let mut params: Vec<(String, String)> = vec![
                ("chain".to_string(), chain),
                ("assetsId".to_string(), assets_id),
                ("pageSize".to_string(), "20".to_string()),
                ("pageNo".to_string(), "1".to_string()),
            ];
            if let Some(s) = status {
                params.push(("status".to_string(), s));
            }
            let raw = client.trade_get_raw("/v1/thirdParty/tx/getLimitOrder", &params, true).await?;
            out!(&raw);
        }

        // ============================================================
        // WebSocket Commands
        // ============================================================
        Commands::WssRepl => {
            ws::run_repl(&config).await?;
        }

        Commands::WatchPrice { tokens } => {
            ws::watch_price(&config, tokens).await?;
        }

        Commands::WatchKline { address, chain, interval, format } => {
            let format_markdown = format.as_deref() == Some("markdown");
            ws::watch_kline(&config, &address, &chain, &interval, format_markdown).await?;
        }

        Commands::WatchTx { address, chain, topic } => {
            ws::watch_tx(&config, &address, &chain, &topic).await?;
        }

        Commands::Daemon { skill } => {
            info!("Starting daemon for skill: {}", skill);
            if skill == "data-wss" {
                let wss = ws::DataWss::new(&config)?;
                wss.daemon().await?;
            } else if skill == "trade-wss" {
                let wss = ws::TradeWss::new(&config)?;
                let mut rx = wss.watch_orders().await?;
                while let Some(order) = rx.recv().await {
                    println!("{}", serde_json::to_string_pretty(&order)?);
                }
            } else {
                return Err(format!("Unknown skill: {}", skill).into());
            }
        }

        Commands::StreamPrice { chain, address } => {
            info!("Starting price stream for {} on {}", address, chain);
            let mut wss = ws::DataWss::new(&config)?;
            wss.subscribe_price(&chain, &address);
            let mut rx = wss.stream_typed().await?;
            while let Some(event) = rx.recv().await {
                if let ws::WsEvent::Price(p) = event {
                    println!("{}", serde_json::to_string_pretty(&p)?);
                }
            }
        }

        Commands::StreamTx { chain, address } => {
            info!("Starting tx stream for {} on {}", address, chain);
            let mut wss = ws::DataWss::new(&config)?;
            wss.subscribe_txs(&chain, &address);
            let mut rx = wss.stream_typed().await?;
            while let Some(event) = rx.recv().await {
                if let ws::WsEvent::Tx(p) = event {
                    println!("{}", serde_json::to_string_pretty(&p)?);
                }
            }
        }
    }

    Ok(())
}

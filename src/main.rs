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
use crate::rest::data::DataApiV2;
use crate::rest::trade::{ProxyWalletApi, TradeRestApi};
use crate::rest::AveClient;

#[derive(Parser)]
#[command(name = "ave-cloud-skills")]
#[command(version = "0.1.0")]
#[command(about = "ZeroClaw skill suite for Ave Cloud API", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[arg(short, long, help = "Output as JSON")]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // ============================================================
    // Data REST v1 Commands (existing)
    // ============================================================
    /// Get token price
    Price { chain: String, address: String },
    /// Get full token info
    Info { chain: String, address: String },
    /// Get kline/candlestick data
    Kline {
        chain: String,
        address: String,
        #[arg(long, default_value = "1h")]
        interval: String,
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Get risk score for a token
    Risk { chain: String, address: String },
    /// Search for tokens
    Search {
        query: String,
        #[arg(long)]
        chain: Option<String>,
    },
    /// Stream real-time price updates (requires Pro plan)
    StreamPrice { chain: String, address: String },
    /// Stream real-time transactions (requires Pro plan)
    StreamTx { chain: String, address: String },
    /// Run as daemon for WebSocket streams (requires Pro plan)
    Daemon {
        #[arg(long, default_value = "data-wss")]
        skill: String,
    },
    /// Place a market buy order (requires Normal/Pro plan)
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
    /// Place a market sell order (requires Normal/Pro plan)
    Sell {
        chain: String,
        token: String,
        amount_usd: f64,
    },
    /// List orders
    Orders {
        #[arg(long)]
        status: Option<String>,
    },

    // ============================================================
    // Data REST v2 Commands (Phase 1)
    // ============================================================
    /// Batch get prices for tokens (v2)
    BatchPrice {
        #[arg(long, required = true)]
        tokens: Vec<String>,
        #[arg(long)]
        tvl_min: Option<f64>,
        #[arg(long)]
        volume_min: Option<f64>,
    },
    /// Batch search token details by address-chain (v2)
    SearchDetails {
        #[arg(long, required = true)]
        tokens: Vec<String>,
    },
    /// Get kline data by token address (v2)
    KlineToken {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, default_value = "60")]
        interval: u32,
        #[arg(long, default_value = "24")]
        size: u32,
    },
    /// Get kline data by pair address (v2)
    KlinePair {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, default_value = "60")]
        interval: u32,
        #[arg(long, default_value = "24")]
        size: u32,
    },
    /// Get Ondo-mapped kline data (v2)
    KlineOndo {
        #[arg(long, required = true)]
        pair: String,
        #[arg(long, default_value = "60")]
        interval: u32,
        #[arg(long, default_value = "24")]
        size: u32,
        #[arg(long)]
        from_time: Option<i64>,
        #[arg(long)]
        to_time: Option<i64>,
    },
    /// Get token detail (v2)
    Token {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
    },
    /// Get token holders (v2)
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
    /// Get trending tokens (v2)
    Trending {
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, default_value = "0")]
        page: u32,
        #[arg(long, default_value = "20")]
        page_size: u32,
    },
    /// List rank topics (v2)
    RankTopics,
    /// Get token rankings by topic (v2)
    Ranks {
        #[arg(long, required = true)]
        topic: String,
    },
    /// Get chains list (v2)
    Chains,
    /// Get main tokens for chain (v2)
    MainTokens {
        #[arg(long, required = true)]
        chain: String,
    },
    /// Get platform tokens (v2)
    PlatformTokens {
        #[arg(long, required = true)]
        platform: String,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        orderby: Option<String>,
    },
    /// Get wallet swap history (v2)
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
    /// Get wallet PnL (v2)
    AddressPnl {
        #[arg(long, required = true)]
        wallet: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long, required = true)]
        token: String,
    },
    /// Get wallet token holdings (v2)
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
    /// Get wallet info (v2)
    WalletInfo {
        #[arg(long, required = true)]
        wallet: String,
        #[arg(long, required = true)]
        chain: String,
        #[arg(long)]
        self_address: Option<String>,
    },
    /// List smart wallets (v2)
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
    /// Get public trading signals (v2)
    Signals {
        #[arg(long)]
        chain: Option<String>,
        #[arg(long)]
        page_size: Option<u32>,
        #[arg(long)]
        page_no: Option<u32>,
    },
    /// Get swap transactions for pair (v2)
    Txs {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
    },
    /// Get liquidity transactions (v2)
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
    /// Get transaction detail (v2)
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
    /// Get trading pair detail (v2)
    Pair {
        #[arg(long, required = true)]
        address: String,
        #[arg(long, required = true)]
        chain: String,
    },

    // ============================================================
    // Trade Chain Wallet Commands (Phase 3)
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
    /// One-step EVM swap: create + sign + send
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
    /// One-step Solana swap: create + sign + send
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
    // Trade Proxy Wallet Commands (Phase 4)
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
        #[arg(long, required = true)]
        assets_ids: Vec<String>,
    },
    /// Place a market swap order
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
    /// Place a limit order
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
        #[arg(long, required = true)]
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

    // ============================================================
    // Data WSS Commands (Phase 2)
    // ============================================================
    /// Interactive WebSocket REPL (pro plan)
    WssRepl,
    /// Stream live price updates (pro plan)
    WatchPrice {
        #[arg(long, required = true)]
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Setup logging
    let log_level = match cli.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // Load config
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Execute command
    if let Err(e) = execute_command(cli, config).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn execute_command(cli: Cli, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let client = AveClient::new(config.clone())?;
    let data_v2 = DataApiV2::new(&client);
    let trade_api = TradeRestApi::new(&client);
    let proxy_api = ProxyWalletApi::new(&client);

    match cli.command {
        // ============================================================
        // Data REST v1 Commands
        // ============================================================
        Commands::Price { chain, address } => {
            let price = client.data().token_price(&chain, &address).await?;
            if cli.json {
                println!("{{\"price\": {}}}", price);
            } else {
                println!("Token: {} on {}", address, chain);
                println!("Price: ${}", price);
            }
        }
        Commands::Info { chain, address } => {
            let info = client.data().token_info(&chain, &address).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&info)?);
            } else {
                println!("Token: {} ({})", info.name, info.symbol);
                println!("Chain: {}", info.chain);
                if let Some(price) = info.price_usd {
                    println!("Price: ${}", price);
                }
                if let Some(mcap) = info.market_cap {
                    println!("Market Cap: ${}", mcap);
                }
                if let Some(vol) = info.volume_24h {
                    println!("24h Volume: ${}", vol);
                }
                if let Some(h) = info.holder_count {
                    println!("Holders: {}", h);
                }
            }
        }
        Commands::Kline {
            chain,
            address,
            interval,
            limit,
        } => {
            let data = client
                .data()
                .kline(&chain, &address, &interval, limit)
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                for k in data {
                    println!(
                        "{} | O:{:.6} H:{:.6} L:{:.6} C:{:.6} V:{:.2}",
                        k.timestamp, k.open, k.high, k.low, k.close, k.volume
                    );
                }
            }
        }
        Commands::Risk { chain, address } => {
            let risk = client.data().risk_check(&chain, &address).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&risk)?);
            } else {
                println!("Token: {}", address);
                println!("Honeypot: {}", if risk.is_honeypot { "YES" } else { "NO" });
                println!("Risk Level: {}/100", risk.risk_level);
                println!("Can Buy: {}", risk.can_buy);
                println!("Can Sell: {}", risk.can_sell);
                println!("Buy Tax: {}%", risk.buy_tax);
                println!("Sell Tax: {}%", risk.sell_tax);
                if !risk.flags.is_empty() {
                    println!("Flags: {:?}", risk.flags);
                }
            }
        }
        Commands::Search { query, chain } => {
            let results = client.data().search(&query, chain.as_deref()).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                for token in results {
                    println!(
                        "{} ({}): {} - {}",
                        token.symbol, token.chain, token.address, token.name
                    );
                }
            }
        }
        Commands::StreamPrice { chain, address } => {
            info!("Starting price stream for {} on {}", address, chain);
            let mut wss = ws::DataWss::new(&config)?;
            wss.subscribe_price(&chain, &address);
            let mut rx = wss.stream_typed().await?;
            while let Some(event) = rx.recv().await {
                if let ws::WsEvent::Price(p) = event {
                    println!("{}: ${}", p.token, p.price);
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
                    println!(
                        "Tx: {} - {:?} {} for {} USD",
                        p.tx.tx_hash, p.tx.side, p.tx.amount_token, p.tx.amount_usd
                    );
                }
            }
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
                    println!("Order update: {} - {:?}", order.order_id, order.status);
                }
            } else {
                return Err(format!("Unknown skill: {}", skill).into());
            }
        }
        Commands::Buy {
            chain,
            token,
            amount_usd,
            safe,
            tp,
            sl,
        } => {
            let wallet = trade::ProxyWallet::new(&client)?;
            if safe {
                let risk = client.data().risk_check(&chain, &token).await?;
                if risk.is_honeypot {
                    warn!("Token {} is flagged as HONEYPOT!", token);
                    return Err("Refusing to buy honeypot token".into());
                }
                if risk.risk_level > 70 {
                    warn!("Token {} has high risk level: {}", token, risk.risk_level);
                }
                info!("Risk check passed, placing buy order...");
            }
            let order = if tp.is_some() || sl.is_some() {
                wallet
                    .buy_with_tp_sl(
                        &chain,
                        &token,
                        amount_usd,
                        tp.unwrap_or(0.0),
                        sl.unwrap_or(0.0),
                    )
                    .await?
            } else {
                wallet.market_buy(&chain, &token, amount_usd).await?
            };
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&order)?);
            } else {
                println!("Order placed: {}", order.order_id);
                println!("Status: {:?}", order.status);
            }
        }
        Commands::Sell {
            chain,
            token,
            amount_usd,
        } => {
            let wallet = trade::ProxyWallet::new(&client)?;
            let order = wallet.market_sell(&chain, &token, amount_usd).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&order)?);
            } else {
                println!("Order placed: {}", order.order_id);
                println!("Status: {:?}", order.status);
            }
        }
        Commands::Orders { status } => {
            let wallet = trade::ProxyWallet::new(&client)?;
            let status_filter = match status.as_deref() {
                Some("pending") => Some(types::OrderStatus::Pending),
                Some("filled") => Some(types::OrderStatus::Filled),
                Some("cancelled") => Some(types::OrderStatus::Cancelled),
                Some("failed") => Some(types::OrderStatus::Failed),
                _ => None,
            };
            let orders = wallet.list_orders(status_filter).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&orders)?);
            } else {
                for order in orders {
                    println!(
                        "{} | {:?} | {:?} {} | ${} | {}",
                        order.order_id,
                        order.status,
                        order.side,
                        order.token_address,
                        order.amount_usd,
                        order.created_at
                    );
                }
            }
        }

        // ============================================================
        // Data REST v2 Commands (Phase 1)
        // ============================================================
        Commands::BatchPrice {
            tokens,
            tvl_min,
            volume_min,
        } => {
            let token_refs: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
            let result = data_v2.batch_price(token_refs, tvl_min, volume_min).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Batch Price Results ({} tokens):", result.tokens.len());
                for item in result.tokens {
                    println!("{}: ${:?}", item.id, item.price_usd);
                }
            }
        }
        Commands::SearchDetails { tokens } => {
            let token_refs: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
            let result = data_v2.search_details(token_refs).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for token in result {
                    println!(
                        "{} ({}): {} - {}",
                        token.symbol, token.chain, token.address, token.name
                    );
                }
            }
        }
        Commands::KlineToken {
            address,
            chain,
            interval,
            size,
        } => {
            let result = data_v2
                .kline_token(&chain, &address, interval, size, None, None, None)
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for k in result.points {
                    println!(
                        "{} | O:{:.6} H:{:.6} L:{:.6} C:{:.6} V:{:.2}",
                        k.timestamp, k.open, k.high, k.low, k.close, k.volume
                    );
                }
            }
        }
        Commands::KlinePair {
            address,
            chain,
            interval,
            size,
        } => {
            let result = data_v2
                .kline_pair(&chain, &address, interval, size, None, None, None)
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for k in result.points {
                    println!(
                        "{} | O:{:.6} H:{:.6} L:{:.6} C:{:.6} V:{:.2}",
                        k.timestamp, k.open, k.high, k.low, k.close, k.volume
                    );
                }
            }
        }
        Commands::KlineOndo {
            pair,
            interval,
            size,
            from_time,
            to_time,
        } => {
            let result = data_v2
                .kline_ondo(&pair, interval, size, from_time, to_time)
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for k in result.points {
                    println!(
                        "{} | O:{:.6} H:{:.6} L:{:.6} C:{:.6} V:{:.2}",
                        k.timestamp, k.open, k.high, k.low, k.close, k.volume
                    );
                }
            }
        }
        Commands::Token { address, chain } => {
            let result = data_v2.token(&chain, &address).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Token: {} ({})", result.name, result.symbol);
                println!("Chain: {}", result.chain);
                println!("Address: {}", result.address);
                if let Some(price) = result.price_usd {
                    println!("Price: ${}", price);
                }
                if let Some(mcap) = result.market_cap {
                    println!("Market Cap: ${}", mcap);
                }
            }
        }
        Commands::Holders {
            address,
            chain,
            limit,
            sort_by,
            order,
        } => {
            let result = data_v2
                .holders(
                    &chain,
                    &address,
                    limit,
                    sort_by.as_deref(),
                    order.as_deref(),
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Holders for {} on {}:", address, chain);
                for holder in result.iter().take(20) {
                    println!("{:?}", holder);
                }
            }
        }
        Commands::Trending {
            chain,
            page,
            page_size,
        } => {
            let result = data_v2.trending(&chain, page, page_size).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for token in result {
                    println!("{} ({}): ${:?}", token.symbol, token.chain, token.price_usd);
                }
            }
        }
        Commands::RankTopics => {
            let result = data_v2.rank_topics().await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Available Rank Topics:");
                for topic in result.topics {
                    println!("{} - {:?}", topic.id, topic.description);
                }
            }
        }
        Commands::Ranks { topic } => {
            let result = data_v2.ranks(&topic).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for token in result {
                    println!(
                        "#{:?} {} ({}): ${:?}",
                        token.rank, token.token.symbol, token.token.chain, token.token.price_usd
                    );
                }
            }
        }
        Commands::Chains => {
            let result = data_v2.chains().await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for chain in result {
                    println!("{} ({})", chain.name, chain.symbol);
                }
            }
        }
        Commands::MainTokens { chain } => {
            let result = data_v2.main_tokens(&chain).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for token in result {
                    println!("{} ({}) - ${:?}", token.symbol, token.name, token.price_usd);
                }
            }
        }
        Commands::PlatformTokens {
            platform,
            limit,
            orderby,
        } => {
            let result = data_v2
                .platform_tokens(&platform, limit, orderby.as_deref())
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for token in result {
                    println!(
                        "{} ({}) - ${:?}",
                        token.symbol, token.chain, token.price_usd
                    );
                }
            }
        }
        Commands::AddressTxs {
            wallet,
            chain,
            token,
            from_time,
            last_time,
            last_id,
            page_size,
        } => {
            let result = data_v2
                .address_txs(
                    &wallet,
                    &chain,
                    token.as_deref(),
                    from_time,
                    last_time.as_deref(),
                    last_id.as_deref(),
                    page_size,
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for tx in result {
                    println!(
                        "{} | {:?} | {} {} | ${}",
                        tx.tx_hash, tx.side, tx.amount_token, tx.token_address, tx.amount_usd
                    );
                }
            }
        }
        Commands::AddressPnl {
            wallet,
            chain,
            token,
        } => {
            let result = data_v2.address_pnl(&wallet, &chain, &token).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Wallet PnL for {} on {}", wallet, chain);
                println!(
                    "Total Bought: {} ${}",
                    result.total_bought, result.total_bought_usd
                );
                println!(
                    "Total Sold: {} ${}",
                    result.total_sold, result.total_sold_usd
                );
                println!("PnL: ${} ({:.2}%)", result.pnl_usd, result.pnl_percent);
            }
        }
        Commands::WalletTokens {
            wallet,
            chain,
            sort,
            sort_dir,
            page_size,
            page_no,
            hide_sold,
            hide_small,
            blue_chips,
        } => {
            let result = data_v2
                .wallet_tokens(
                    &wallet,
                    &chain,
                    sort.as_deref(),
                    sort_dir.as_deref(),
                    page_size,
                    page_no,
                    hide_sold,
                    hide_small,
                    blue_chips,
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Wallet {} Holdings on {}:", wallet, chain);
                for token in result {
                    println!(
                        "{} ({}): ${:?}",
                        token.symbol, token.chain, token.balance_usd
                    );
                }
            }
        }
        Commands::WalletInfo {
            wallet,
            chain,
            self_address,
        } => {
            let result = data_v2
                .wallet_info(&wallet, &chain, self_address.as_deref())
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Wallet Info for {} on {}", wallet, chain);
                println!("Total Tokens: {:?}", result.total_tokens);
                println!("Total Value: ${:?}", result.total_value_usd);
                println!(
                    "Total PnL: ${:?} ({:?}%)",
                    result.total_pnl_usd, result.total_pnl_percent
                );
            }
        }
        Commands::SmartWallets {
            chain,
            keyword,
            sort,
            sort_dir,
        } => {
            let result = data_v2
                .smart_wallets(
                    &chain,
                    keyword.as_deref(),
                    sort.as_deref(),
                    sort_dir.as_deref(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Smart Wallets on {}:", chain);
                for wallet in result {
                    println!("{}", wallet.address);
                }
            }
        }
        Commands::Signals {
            chain,
            page_size,
            page_no,
        } => {
            let result = data_v2
                .signals(chain.as_deref(), page_size, page_no)
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Trading Signals:");
                for signal in result {
                    println!(
                        "{} on {} - {:?} | Entry: {:?} | Target: {:?}",
                        signal.token_address,
                        signal.chain,
                        signal.signal_type,
                        signal.entry_price,
                        signal.target_price
                    );
                }
            }
        }
        Commands::Txs { address, chain } => {
            let result = data_v2.txs(&chain, &address).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for tx in result {
                    println!(
                        "{} | {:?} | {} for ${}",
                        tx.tx_hash, tx.side, tx.amount_token, tx.amount_usd
                    );
                }
            }
        }
        Commands::LiqTxs {
            address,
            chain,
            type_,
            limit,
            from_time,
            to_time,
            sort,
        } => {
            let result = data_v2
                .liq_txs(
                    &chain,
                    &address,
                    type_.as_deref(),
                    limit,
                    from_time,
                    to_time,
                    sort.as_deref(),
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for tx in result {
                    println!(
                        "{} | {:?} | {} {}",
                        tx.tx_hash, tx.type_, tx.token_a_address, tx.amount_a
                    );
                }
            }
        }
        Commands::TxDetail {
            chain,
            account,
            tx_hash,
            start_from,
            end_at,
            limit,
        } => {
            let result = data_v2
                .tx_detail(&chain, &account, &tx_hash, start_from, end_at, limit)
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Transaction: {}", result.tx_hash);
                println!("Chain: {}", result.chain);
                println!("From: {}", result.from);
                println!("To: {:?}", result.to);
                println!("Status: {:?}", result.status);
            }
        }
        Commands::Pair { address, chain } => {
            let result = data_v2.pair(&chain, &address).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Pair: {}", result.pair_address);
                println!("Chain: {}", result.chain);
                println!(
                    "Base: {} ({})",
                    result.base_token.symbol, result.base_token.name
                );
                println!(
                    "Quote: {} ({})",
                    result.quote_token.symbol, result.quote_token.name
                );
                println!("Price: ${:?}", result.price_usd);
                println!("Liquidity: ${:?}", result.liquidity_usd);
            }
        }

        // ============================================================
        // Trade Chain Wallet Commands (Phase 3)
        // ============================================================
        Commands::Quote {
            chain,
            in_amount,
            in_token,
            out_token,
            swap_type,
        } => {
            let result = trade_api
                .quote(&chain, &in_amount, &in_token, &out_token, &swap_type)
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Quote for {} {} -> {}", in_amount, in_token, out_token);
                println!("Out Amount: {}", result.out_amount);
                println!("Out Amount Min: {}", result.out_amount_min);
                println!("Price Impact: {}%", result.price_impact);
            }
        }
        Commands::AutoSlippage {
            chain,
            token,
            use_mev,
        } => {
            let result = trade_api.auto_slippage(&chain, &token, use_mev).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!(
                    "Auto Slippage for {} on {}: {} bps (MEV: {})",
                    token, chain, result.slippage_bps, result.use_mev
                );
            }
        }
        Commands::GasTip => {
            let result = trade_api.gas_tip().await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for tip in result.tips {
                    println!(
                        "{}: Low={} | Avg={} | High={} {}",
                        tip.chain, tip.low, tip.average, tip.high, tip.unit
                    );
                }
            }
        }
        Commands::ApproveChain {
            chain,
            token,
            in_amount,
            rpc_url: _,
        } => {
            // This requires local signing - for now just get the spender from quote
            let quote = trade_api
                .quote(
                    &chain,
                    in_amount.as_deref().unwrap_or("1000000000000000000"),
                    &token,
                    "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                    "sell",
                )
                .await?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &serde_json::json!({"spender": quote.spender, "token": token, "chain": chain})
                    )?
                );
            } else {
                println!("Token: {}", token);
                println!("Chain: {}", chain);
                println!("Spender: {:?}", quote.spender);
                println!("Note: Use your EVM wallet to approve this spender");
            }
        }
        Commands::CreateEvmTx {
            chain,
            creator_address,
            in_amount,
            in_token,
            out_token,
            swap_type,
            slippage,
            fee_recipient,
            fee_recipient_rate,
            auto_slippage,
        } => {
            let result = trade_api
                .create_evm_tx(
                    &chain,
                    &creator_address,
                    &in_amount,
                    &in_token,
                    &out_token,
                    &swap_type,
                    &slippage,
                    fee_recipient.as_deref(),
                    fee_recipient_rate.as_deref(),
                    auto_slippage,
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("EVM Transaction Created:");
                println!("Request Tx ID: {}", result.request_tx_id);
                println!("Gas Limit: {}", result.gas_limit);
                println!("To: {}", result.tx_content.to);
            }
        }
        Commands::SendEvmTx {
            chain,
            request_tx_id,
            signed_tx,
            use_mev,
        } => {
            let result = trade_api
                .send_evm_tx(&chain, &request_tx_id, &signed_tx, use_mev)
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Transaction Broadcast:");
                println!("Tx Hash: {}", result.tx_hash);
            }
        }
        Commands::CreateSolanaTx {
            creator_address,
            in_amount,
            in_token,
            out_token,
            swap_type,
            slippage,
            fee,
            use_mev,
            fee_recipient,
            fee_recipient_rate,
            auto_slippage,
        } => {
            let result = trade_api
                .create_solana_tx(
                    &creator_address,
                    &in_amount,
                    &in_token,
                    &out_token,
                    &swap_type,
                    &slippage,
                    &fee,
                    use_mev,
                    fee_recipient.as_deref(),
                    fee_recipient_rate.as_deref(),
                    auto_slippage,
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Solana Transaction Created:");
                println!("Request Tx ID: {}", result.request_tx_id);
            }
        }
        Commands::SendSolanaTx {
            request_tx_id,
            signed_tx,
            use_mev,
        } => {
            let result = trade_api
                .send_solana_tx(&request_tx_id, &signed_tx, use_mev)
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Transaction Broadcast:");
                println!("Tx Hash: {}", result.tx_hash);
            }
        }
        Commands::SwapEvm {
            chain: _,
            in_amount: _,
            in_token: _,
            out_token: _,
            swap_type: _,
            slippage: _,
            fee_recipient: _,
            fee_recipient_rate: _,
            auto_slippage: _,
            use_mev: _,
            rpc_url: _,
        } => {
            return Err(
                "swap-evm requires local EVM signing. Use create-evm-tx + send-evm-tx instead."
                    .into(),
            );
        }
        Commands::SwapSolana {
            in_amount: _,
            in_token: _,
            out_token: _,
            swap_type: _,
            slippage: _,
            fee: _,
            fee_recipient: _,
            fee_recipient_rate: _,
            auto_slippage: _,
            use_mev: _,
        } => {
            return Err("swap-solana requires local Solana signing. Use create-solana-tx + send-solana-tx instead.".into());
        }

        // ============================================================
        // Trade Proxy Wallet Commands (Phase 4)
        // ============================================================
        Commands::ListWallets { assets_ids } => {
            let result = proxy_api.list_wallets(assets_ids.as_deref()).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for wallet in result {
                    println!(
                        "{} | {} | {} | {:?}",
                        wallet.assets_id, wallet.assets_name, wallet.address, wallet.status
                    );
                }
            }
        }
        Commands::CreateWallet {
            name,
            return_mnemonic,
        } => {
            let result = proxy_api.create_wallet(&name, return_mnemonic).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Wallet Created:");
                println!("Assets ID: {}", result.assets_id);
                println!("Address: {}", result.address);
                println!("Name: {}", result.assets_name);
                if let Some(mnemonic) = result.mnemonic {
                    println!("Mnemonic: {}", mnemonic);
                }
            }
        }
        Commands::DeleteWallet { assets_ids } => {
            let asset_refs: Vec<&str> = assets_ids.iter().map(|s| s.as_str()).collect();
            let result = proxy_api.delete_wallet(asset_refs).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Wallet(s) deleted");
            }
        }
        Commands::MarketOrder {
            chain,
            assets_id,
            in_token,
            out_token,
            in_amount,
            swap_type,
            slippage,
            use_mev,
            gas,
            extra_gas,
            auto_slippage,
            auto_gas,
            auto_sell,
        } => {
            let auto_sell_config: Option<Vec<serde_json::Value>> = auto_sell.map(|v| {
                v.iter()
                    .filter_map(|s| serde_json::from_str(s).ok())
                    .collect()
            });
            let result = proxy_api
                .market_order(
                    &chain,
                    &assets_id,
                    &in_token,
                    &out_token,
                    &in_amount,
                    &swap_type,
                    &slippage,
                    use_mev,
                    gas.as_deref(),
                    extra_gas.as_deref(),
                    auto_slippage,
                    auto_gas.as_deref(),
                    auto_sell_config,
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Market Order Placed:");
                println!("Order ID: {}", result.order_id);
                println!("Status: {}", result.status);
            }
        }
        Commands::LimitOrder {
            chain,
            assets_id,
            in_token,
            out_token,
            in_amount,
            swap_type,
            slippage,
            limit_price,
            use_mev,
            gas,
            extra_gas,
            expire_time,
            auto_slippage,
            auto_gas,
        } => {
            let result = proxy_api
                .limit_order(
                    &chain,
                    &assets_id,
                    &in_token,
                    &out_token,
                    &in_amount,
                    &swap_type,
                    &slippage,
                    &limit_price,
                    use_mev,
                    gas.as_deref(),
                    extra_gas.as_deref(),
                    expire_time,
                    auto_slippage,
                    auto_gas.as_deref(),
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Limit Order Placed:");
                println!("Order ID: {}", result.order_id);
                println!("Status: {}", result.status);
                println!("Trigger Price: {}", result.trigger_price);
            }
        }
        Commands::GetSwapOrders { chain, ids } => {
            let result = proxy_api.get_swap_orders(&chain, &ids).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for order in result.orders {
                    println!(
                        "{} | {:?} | {} -> {}",
                        order.order_id, order.status, order.in_token, order.out_token
                    );
                }
            }
        }
        Commands::GetLimitOrders {
            chain,
            assets_id,
            page_size,
            page_no,
            status,
            token,
        } => {
            let result = proxy_api
                .get_limit_orders(
                    &chain,
                    &assets_id,
                    page_size,
                    page_no,
                    status.as_deref(),
                    token.as_deref(),
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for order in result.orders {
                    println!(
                        "{} | {:?} | {} -> {} | Limit: {}",
                        order.order_id,
                        order.status,
                        order.in_token,
                        order.out_token,
                        order.limit_price
                    );
                }
            }
        }
        Commands::CancelLimitOrder { chain, ids } => {
            let id_refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
            let result = proxy_api.cancel_limit_order(&chain, id_refs).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Limit order(s) cancelled");
            }
        }
        Commands::ApproveToken {
            chain,
            assets_id,
            token_address,
        } => {
            let result = proxy_api
                .approve_token(&chain, &assets_id, &token_address)
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Approval Created:");
                println!("Approval ID: {}", result.approval_id);
                println!("Status: {}", result.status);
                println!("Spender: {}", result.spender);
            }
        }
        Commands::GetApproval { chain, ids } => {
            let result = proxy_api.get_approval(&chain, &ids).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for approval in result {
                    println!(
                        "{} | {:?} | {} | {}",
                        approval.approval_id,
                        approval.status,
                        approval.token_address,
                        approval.amount
                    );
                }
            }
        }
        Commands::Transfer {
            chain,
            assets_id,
            from_address,
            to_address,
            token_address,
            amount,
            gas,
            extra_gas,
        } => {
            let result = proxy_api
                .transfer(
                    &chain,
                    &assets_id,
                    &from_address,
                    &to_address,
                    &token_address,
                    &amount,
                    gas.as_deref(),
                    extra_gas.as_deref(),
                )
                .await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Transfer Initiated:");
                println!("Transfer ID: {}", result.transfer_id);
                println!("Status: {}", result.status);
            }
        }
        Commands::GetTransfer { chain, ids } => {
            let result = proxy_api.get_transfer(&chain, &ids).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for transfer in result {
                    println!(
                        "{} | {:?} | {} -> {}",
                        transfer.transfer_id, transfer.status, transfer.from, transfer.to
                    );
                }
            }
        }
        // ============================================================
        // Data WSS Commands
        // ============================================================
        Commands::WssRepl => {
            ws::run_repl(&config).await?;
        }
        Commands::WatchPrice { tokens } => {
            ws::watch_price(&config, tokens).await?;
        }
        Commands::WatchKline {
            address,
            chain,
            interval,
            format,
        } => {
            let format_markdown = format.as_deref() == Some("markdown");
            ws::watch_kline(&config, &address, &chain, &interval, format_markdown).await?;
        }
        Commands::WatchTx {
            address,
            chain,
            topic,
        } => {
            ws::watch_tx(&config, &address, &chain, &topic).await?;
        }
    }

    Ok(())
}

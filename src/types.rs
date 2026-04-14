use serde::{Deserialize, Serialize};

// ============================================================
// Common API Response Wrapper
// ============================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct AveResponse<T> {
    pub status: i32,
    pub msg: String,
    #[serde(default)]
    pub data_type: Option<i32>,
    #[serde(default)]
    pub data: Option<T>,
}

/// Raw API response with untyped data field for initial parsing
#[derive(Debug, Clone, Deserialize)]
pub struct RawApiResponse {
    pub status: i32,
    pub msg: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub data_type: Option<i32>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

// ============================================================
// Data REST Types
// ============================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    #[serde(default)]
    pub decimals: Option<u8>,
    pub chain: String,
    #[serde(default)]
    pub price_usd: Option<f64>,
    #[serde(default)]
    pub price_change_24h: Option<f64>,
    #[serde(default)]
    pub market_cap: Option<f64>,
    #[serde(default)]
    pub volume_24h: Option<f64>,
    #[serde(default)]
    pub liquidity_usd: Option<f64>,
    #[serde(default)]
    pub holder_count: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Kline {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiskScore {
    pub address: String,
    #[serde(rename = "is_honeypot")]
    pub is_honeypot: bool,
    #[serde(rename = "risk_level")]
    pub risk_level: u8,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(rename = "can_buy")]
    pub can_buy: bool,
    #[serde(rename = "can_sell")]
    pub can_sell: bool,
    #[serde(rename = "buy_tax")]
    pub buy_tax: f64,
    #[serde(rename = "sell_tax")]
    pub sell_tax: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwapTx {
    #[serde(rename = "tx_hash")]
    pub tx_hash: String,
    pub timestamp: i64,
    pub side: TradeSide,
    #[serde(rename = "token_address")]
    pub token_address: String,
    #[serde(rename = "amount_token")]
    pub amount_token: String,
    #[serde(rename = "amount_usd")]
    pub amount_usd: f64,
    pub wallet: String,
}

// ============================================================
// Trade Types
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwapQuote {
    pub chain: String,
    #[serde(rename = "token_in")]
    pub token_in: String,
    #[serde(rename = "token_out")]
    pub token_out: String,
    #[serde(rename = "amount_in")]
    pub amount_in: String,
    #[serde(rename = "amount_out_min")]
    pub amount_out_min: String,
    #[serde(rename = "price_impact")]
    pub price_impact: f64,
    #[serde(rename = "gas_estimate")]
    pub gas_estimate: u64,
    #[serde(default)]
    pub route: Vec<String>,
    #[serde(default)]
    pub calldata: Option<String>,
    pub to: String,
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Order {
    #[serde(rename = "order_id")]
    pub order_id: String,
    #[serde(rename = "order_type")]
    pub order_type: OrderType,
    pub side: TradeSide,
    #[serde(rename = "token_address")]
    pub token_address: String,
    pub chain: String,
    pub status: OrderStatus,
    #[serde(rename = "created_at")]
    pub created_at: i64,
    #[serde(rename = "updated_at")]
    pub updated_at: i64,
    #[serde(rename = "amount_usd")]
    pub amount_usd: f64,
    #[serde(rename = "trigger_price")]
    pub trigger_price: Option<f64>,
    #[serde(rename = "tp_price")]
    pub tp_price: Option<f64>,
    #[serde(rename = "sl_price")]
    pub sl_price: Option<f64>,
}

// ============================================================
// WebSocket Frame Types
// ============================================================

/// JSON-RPC 2.0 request format for v2 WebSocket protocol
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct WssJsonRpcRequest {
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    pub id: u64,
}

impl WssJsonRpcRequest {
    #[allow(dead_code)]
    pub fn subscribe(method: &str, params: Vec<&str>, id: u64) -> Self {
        WssJsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: serde_json::json!(params),
            id,
        }
    }
}

/// WebSocket subscription message (v1 format)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WsSubscribeMsg {
    pub action: String,
    pub channel: String,
    pub token: String,
    pub chain: String,
    #[serde(default)]
    pub interval: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WsPriceUpdate {
    pub channel: String,
    pub token: String,
    pub chain: String,
    pub price: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WsTxUpdate {
    pub channel: String,
    pub tx: SwapTx,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WsOrderUpdate {
    #[serde(rename = "order_id")]
    pub order_id: String,
    pub status: OrderStatus,
    #[serde(rename = "filled_amount")]
    pub filled_amount: Option<f64>,
    #[serde(rename = "tx_hash")]
    pub tx_hash: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WsKlineUpdate {
    pub channel: String,
    pub token: String,
    pub chain: String,
    pub candle: Kline,
}

/// v2 WebSocket kline event (JSON-RPC 2.0 response format)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WssKlineEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub pair: Option<String>,
    #[serde(default)]
    pub chain: Option<String>,
    #[serde(default)]
    pub interval: Option<String>,
    #[serde(default)]
    pub time: Option<i64>,
    #[serde(default)]
    pub open: Option<f64>,
    #[serde(default)]
    pub high: Option<f64>,
    #[serde(default)]
    pub low: Option<f64>,
    #[serde(default)]
    pub close: Option<f64>,
    #[serde(default)]
    pub volume: Option<f64>,
}

/// v2 WebSocket tx event (JSON-RPC 2.0 response format)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct WssTxEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub pair: Option<String>,
    #[serde(default)]
    pub chain: Option<String>,
    #[serde(default)]
    pub tx_hash: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub amount_token: Option<String>,
    #[serde(default)]
    pub amount_usd: Option<f64>,
    #[serde(default)]
    pub wallet: Option<String>,
    #[serde(default)]
    pub token_address: Option<String>,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

/// Kline formatter for markdown output (stores recent closes for mini-chart)
#[derive(Debug, Clone)]
pub struct KlineFormatter {
    history: Vec<f64>,
    max_history: usize,
}

impl KlineFormatter {
    pub fn new(max_history: usize) -> Self {
        KlineFormatter {
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    pub fn render(&mut self, event: &WssKlineEvent) -> String {
        if let (Some(time), Some(open), Some(high), Some(low), Some(close)) =
            (event.time, event.open, event.high, event.low, event.close)
        {
            // Update history
            self.history.push(close);
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }

            // Simple markdown table row with timestamp and OHLC
            format!(
                "| {} | O:{:.4} H:{:.4} L:{:.4} C:{:.4} |",
                time, open, high, low, close
            )
        } else {
            format!("{:#?}", event)
        }
    }
}

// ============================================================
// Proxy Order Params (for creating orders)
// ============================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyOrderParams {
    #[serde(rename = "token_address")]
    pub token_address: String,
    pub chain: String,
    pub side: TradeSide,
    #[serde(rename = "order_type")]
    pub order_type: OrderType,
    #[serde(rename = "amount_usd")]
    pub amount_usd: f64,
    #[serde(rename = "trigger_price")]
    pub trigger_price: Option<f64>,
    #[serde(rename = "tp_price")]
    pub tp_price: Option<f64>,
    #[serde(rename = "sl_price")]
    pub sl_price: Option<f64>,
}

// ============================================================
// Data REST v2 Types (Phase 1)
// ============================================================

/// Batch price response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchPriceResponse {
    #[serde(default)]
    pub tokens: Vec<TokenPriceItem>,
}

/// Individual token price item in batch response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenPriceItem {
    pub id: String,
    #[serde(default)]
    pub price_usd: Option<f64>,
    #[serde(default)]
    pub price_raw: Option<String>,
    #[serde(default)]
    pub token_address: Option<String>,
    #[serde(default)]
    pub chain: Option<String>,
}

/// Kline data with v2 format
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KlineV2 {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    #[serde(default)]
    pub confirm: Option<bool>,
    #[serde(default)]
    pub quote_volume: Option<f64>,
}

/// Kline response wrapper for v2
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KlineResponse {
    pub points: Vec<KlineV2>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub total_count: Option<u32>,
}

/// Pair info response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PairInfo {
    pub pair_address: String,
    pub chain: String,
    pub base_token: TokenInfo,
    pub quote_token: TokenInfo,
    #[serde(default)]
    pub price_usd: Option<f64>,
    #[serde(default)]
    pub liquidity_usd: Option<f64>,
    #[serde(default)]
    pub volume_24h: Option<f64>,
    #[serde(default)]
    pub price_change_24h: Option<f64>,
    #[serde(default)]
    pub tx_count_24h: Option<u64>,
}

/// Address transaction
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressTx {
    #[serde(rename = "tx_hash")]
    pub tx_hash: String,
    pub timestamp: i64,
    pub block: Option<u64>,
    pub side: TradeSide,
    #[serde(rename = "pair_address")]
    pub pair_address: String,
    #[serde(rename = "token_address")]
    pub token_address: String,
    #[serde(rename = "amount_token")]
    pub amount_token: String,
    #[serde(rename = "amount_usd")]
    pub amount_usd: f64,
    #[serde(rename = "wallet_address")]
    pub wallet_address: String,
    #[serde(default)]
    pub is_buy: Option<bool>,
    #[serde(default)]
    pub gas_used: Option<String>,
    #[serde(default)]
    pub gas_price: Option<String>,
}

/// Address PnL response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressPnl {
    #[serde(rename = "token_address")]
    pub token_address: String,
    pub chain: String,
    #[serde(rename = "total_bought")]
    pub total_bought: String,
    #[serde(rename = "total_sold")]
    pub total_sold: String,
    #[serde(rename = "total_bought_usd")]
    pub total_bought_usd: f64,
    #[serde(rename = "total_sold_usd")]
    pub total_sold_usd: f64,
    #[serde(rename = "pnl_usd")]
    pub pnl_usd: f64,
    #[serde(rename = "pnl_percent")]
    pub pnl_percent: f64,
    #[serde(default)]
    pub avg_buy_price: Option<f64>,
    #[serde(default)]
    pub avg_sell_price: Option<f64>,
    #[serde(default)]
    pub realized_pnl: Option<f64>,
    #[serde(default)]
    pub unrealized_pnl: Option<f64>,
}

/// Wallet info response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletInfo {
    #[serde(rename = "wallet_address")]
    pub wallet_address: String,
    pub chain: String,
    #[serde(default)]
    pub total_tokens: Option<u32>,
    #[serde(default)]
    pub total_value_usd: Option<f64>,
    #[serde(default)]
    pub total_pnl_usd: Option<f64>,
    #[serde(default)]
    pub total_pnl_percent: Option<f64>,
    #[serde(default)]
    pub top_token: Option<WalletToken>,
    #[serde(default)]
    pub blue_chip_count: Option<u32>,
    #[serde(default)]
    pub last_txn_time: Option<i64>,
}

/// Wallet token holding
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletToken {
    #[serde(rename = "token_address")]
    pub token_address: String,
    pub symbol: String,
    pub name: String,
    pub chain: String,
    #[serde(default)]
    pub balance: Option<String>,
    #[serde(default)]
    pub balance_usd: Option<f64>,
    #[serde(default)]
    pub price_usd: Option<f64>,
    #[serde(default)]
    pub change_24h: Option<f64>,
    #[serde(default)]
    pub is_blue_chip: Option<bool>,
}

/// Smart wallet info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmartWallet {
    pub address: String,
    pub chain: String,
    #[serde(default)]
    pub profit_900: Option<f64>,
    #[serde(default)]
    pub profit_300_900: Option<f64>,
    #[serde(default)]
    pub profit_100_300: Option<f64>,
    #[serde(default)]
    pub profit_10_100: Option<f64>,
    #[serde(default)]
    pub profit_neg10_10: Option<f64>,
    #[serde(default)]
    pub profit_neg50_neg10: Option<f64>,
    #[serde(default)]
    pub profit_neg100_neg50: Option<f64>,
    #[serde(default)]
    pub last_trade_time: Option<i64>,
    #[serde(default)]
    pub total_trades: Option<u64>,
}

/// Trading signal
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Signal {
    pub id: String,
    pub chain: String,
    #[serde(rename = "pair_address")]
    pub pair_address: String,
    #[serde(rename = "token_address")]
    pub token_address: String,
    #[serde(rename = "signal_type")]
    pub signal_type: String,
    #[serde(default)]
    pub entry_price: Option<f64>,
    #[serde(default)]
    pub target_price: Option<f64>,
    #[serde(default)]
    pub stop_loss: Option<f64>,
    #[serde(default)]
    pub leverage: Option<f64>,
    #[serde(default)]
    pub confidence: Option<f64>,
    pub timestamp: i64,
}

/// Rank topics response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RankTopics {
    pub topics: Vec<RankTopic>,
}

/// Rank topic item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RankTopic {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// Ranked token
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RankedToken {
    pub rank: u32,
    pub token: TokenInfo,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub change_24h: Option<f64>,
}

/// Platform token
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlatformToken {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub chain: String,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub price_usd: Option<f64>,
    #[serde(default)]
    pub volume_24h: Option<f64>,
    #[serde(default)]
    pub market_cap: Option<f64>,
}

/// Chain info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainInfo {
    pub id: String,
    pub name: String,
    pub symbol: String,
    #[serde(default)]
    pub explorer_url: Option<String>,
    #[serde(default)]
    pub rpc_url: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

/// Transaction detail
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TxDetail {
    #[serde(rename = "tx_hash")]
    pub tx_hash: String,
    pub chain: String,
    pub block: Option<u64>,
    pub timestamp: i64,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub gas_used: Option<String>,
    #[serde(default)]
    pub gas_price: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub logs: Option<Vec<TxLog>>,
    #[serde(default)]
    pub call_data: Option<CallData>,
}

/// Transaction log
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TxLog {
    pub address: String,
    #[serde(default)]
    pub topics: Option<Vec<String>>,
    pub data: String,
}

/// Call data for swap transactions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CallData {
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub params: Option<Vec<serde_json::Value>>,
    #[serde(rename = "token_address")]
    pub token_address: Option<String>,
    #[serde(rename = "amount_in")]
    pub amount_in: Option<String>,
    #[serde(rename = "amount_out")]
    pub amount_out: Option<String>,
    #[serde(default)]
    pub path: Option<Vec<String>>,
}

/// Liquidity transaction
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiqTx {
    #[serde(rename = "tx_hash")]
    pub tx_hash: String,
    pub timestamp: i64,
    #[serde(rename = "pair_address")]
    pub pair_address: String,
    pub type_: String,
    #[serde(rename = "token_a_address")]
    pub token_a_address: String,
    #[serde(rename = "token_b_address")]
    pub token_b_address: String,
    #[serde(rename = "amount_a")]
    pub amount_a: String,
    #[serde(rename = "amount_b")]
    pub amount_b: String,
    #[serde(rename = "amount_a_usd")]
    pub amount_a_usd: Option<f64>,
    #[serde(rename = "amount_b_usd")]
    pub amount_b_usd: Option<f64>,
    pub sender: String,
    #[serde(default)]
    pub is_bot: Option<bool>,
}

/// Main token (chain native/primary tokens)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MainToken {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub chain: String,
    #[serde(default)]
    pub price_usd: Option<f64>,
    #[serde(default)]
    pub change_24h: Option<f64>,
    #[serde(default)]
    pub market_cap: Option<f64>,
}

// ============================================================
// Trade Chain Types (Phase 3)
// ============================================================

/// Chain quote response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainQuote {
    pub chain: String,
    #[serde(rename = "in_token")]
    pub in_token: String,
    #[serde(rename = "out_token")]
    pub out_token: String,
    #[serde(rename = "in_amount")]
    pub in_amount: String,
    #[serde(rename = "out_amount")]
    pub out_amount: String,
    #[serde(rename = "out_amount_min")]
    pub out_amount_min: String,
    #[serde(rename = "price_impact")]
    pub price_impact: f64,
    #[serde(default)]
    pub route: Vec<String>,
    #[serde(default)]
    pub spender: Option<String>,
    #[serde(default)]
    pub gas_estimate: Option<String>,
}

/// Auto slippage response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AutoSlippageResponse {
    #[serde(rename = "slippage_bps")]
    pub slippage_bps: u32,
    #[serde(rename = "use_mev")]
    pub use_mev: bool,
}

/// Gas tip response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GasTip {
    pub chain: String,
    pub low: f64,
    pub average: f64,
    pub high: f64,
    pub unit: String,
}

/// Gas tips response wrapper
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GasTipResponse {
    pub tips: Vec<GasTip>,
}

/// EVM transaction creation response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvmTxResponse {
    #[serde(rename = "request_tx_id")]
    pub request_tx_id: String,
    #[serde(rename = "tx_content")]
    pub tx_content: EvmTxContent,
    #[serde(rename = "gas_limit")]
    pub gas_limit: String,
}

/// EVM transaction content
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvmTxContent {
    pub to: String,
    pub data: String,
    pub value: String,
    #[serde(default)]
    pub gas: Option<String>,
    #[serde(default)]
    pub gas_price: Option<String>,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub chain_id: Option<u64>,
}

/// Solana transaction creation response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SolanaTxResponse {
    #[serde(rename = "request_tx_id")]
    pub request_tx_id: String,
    #[serde(rename = "tx_content")]
    pub tx_content: SolanaTxContent,
}

/// Solana transaction content
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SolanaTxContent {
    #[serde(default)]
    pub instructions: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub recent_blockhash: Option<String>,
    #[serde(default)]
    pub fee_payer: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

/// Broadcast transaction response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadcastResponse {
    #[serde(rename = "tx_hash")]
    pub tx_hash: String,
}

// ============================================================
// Trade Proxy Types (Phase 4)
// ============================================================

/// Proxy wallet info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyWallet {
    #[serde(rename = "assets_id")]
    pub assets_id: String,
    #[serde(rename = "assets_name")]
    pub assets_name: String,
    pub address: String,
    pub chain: String,
    pub status: String,
    #[serde(default)]
    pub mnemonic: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
}

/// Market order response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketOrderResponse {
    #[serde(rename = "order_id")]
    pub order_id: String,
    pub status: String,
    #[serde(rename = "created_at")]
    pub created_at: i64,
    #[serde(default)]
    pub tx_hash: Option<String>,
    #[serde(default)]
    pub in_amount: Option<String>,
    #[serde(default)]
    pub out_amount: Option<String>,
}

/// Limit order response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitOrderResponse {
    #[serde(rename = "order_id")]
    pub order_id: String,
    pub status: String,
    #[serde(rename = "trigger_price")]
    pub trigger_price: f64,
    #[serde(rename = "created_at")]
    pub created_at: i64,
    #[serde(default)]
    pub expire_time: Option<i64>,
    #[serde(default)]
    pub tx_hash: Option<String>,
}

/// Approval response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApprovalResponse {
    #[serde(rename = "approval_id")]
    pub approval_id: String,
    pub status: String,
    #[serde(rename = "token_address")]
    pub token_address: String,
    pub spender: String,
    #[serde(rename = "amount")]
    pub amount: String,
    #[serde(rename = "chain")]
    pub chain: String,
    #[serde(rename = "created_at")]
    pub created_at: i64,
}

/// Transfer response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransferResponse {
    #[serde(rename = "transfer_id")]
    pub transfer_id: String,
    pub status: String,
    pub from: String,
    pub to: String,
    #[serde(rename = "token_address")]
    pub token_address: String,
    pub amount: String,
    pub chain: String,
    #[serde(rename = "created_at")]
    pub created_at: i64,
    #[serde(default)]
    pub tx_hash: Option<String>,
}

/// Swap orders response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwapOrdersResponse {
    pub orders: Vec<SwapOrderItem>,
}

/// Swap order item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwapOrderItem {
    #[serde(rename = "order_id")]
    pub order_id: String,
    pub status: String,
    pub chain: String,
    #[serde(rename = "in_token")]
    pub in_token: String,
    #[serde(rename = "out_token")]
    pub out_token: String,
    #[serde(rename = "in_amount")]
    pub in_amount: String,
    #[serde(rename = "out_amount")]
    pub out_amount: Option<String>,
    #[serde(rename = "slippage")]
    pub slippage: f64,
    #[serde(rename = "created_at")]
    pub created_at: i64,
    #[serde(default)]
    pub tx_hash: Option<String>,
}

/// Limit orders response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitOrdersResponse {
    pub orders: Vec<LimitOrderItem>,
    #[serde(default)]
    pub total_count: Option<u32>,
}

/// Limit order item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitOrderItem {
    #[serde(rename = "order_id")]
    pub order_id: String,
    pub status: String,
    pub chain: String,
    #[serde(rename = "in_token")]
    pub in_token: String,
    #[serde(rename = "out_token")]
    pub out_token: String,
    #[serde(rename = "in_amount")]
    pub in_amount: String,
    #[serde(rename = "limit_price")]
    pub limit_price: f64,
    #[serde(rename = "filled_amount")]
    pub filled_amount: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: i64,
    #[serde(default)]
    pub expire_time: Option<i64>,
}

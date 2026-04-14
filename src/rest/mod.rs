// REST API module
// - client.rs: AveClient, DataApi, TradeApi implementations
// - data.rs: DataApi v2 methods
// - trade.rs: TradeRestApi for chain wallet and proxy wallet operations

pub mod client;
pub mod data;
pub mod trade;

#[allow(unused)]
pub use client::{
    AveClient, DataApi, TradeApi, BASE_URL, DATA_V2_BASE_URL, TRADE_API_BASE_URL, WSS_BASE_URL,
};

// WebSocket module
// - manager.rs: WsManager for WebSocket connection management
// - data.rs: DataWss for data streams (price, txs, kline)
// - trade.rs: TradeWss for order status updates
// - repl.rs: Interactive REPL and stream commands (watch-price, watch-kline, watch-tx)

pub mod data;
#[allow(unused)]
pub mod manager;
pub mod repl;
pub mod trade;

pub use data::{DataWss, WsEvent};
#[allow(unused)]
pub use manager::WsManager;
pub use repl::{run_repl, watch_kline, watch_price, watch_tx};
pub use trade::TradeWss;

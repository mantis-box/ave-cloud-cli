use tokio::sync::mpsc;

use crate::config::{ApiPlan, Config};
use crate::error::ZeroClawError;
use crate::types::{WsKlineUpdate, WsOrderUpdate, WsPriceUpdate, WsSubscribeMsg, WsTxUpdate};
use crate::ws::manager::WsManager;

const DATA_WS_URL: &str = "wss://wss.ave-api.xyz";

pub struct DataWss {
    manager: WsManager,
    subscriptions: Vec<WsSubscribeMsg>,
}

impl DataWss {
    pub fn new(config: &Config) -> Result<Self, ZeroClawError> {
        // Require Pro plan for WebSocket streaming
        if !config.can_stream() {
            return Err(ZeroClawError::PlanRequired {
                required: "pro".to_string(),
                current: match config.api_plan {
                    ApiPlan::Free => "free".to_string(),
                    ApiPlan::Normal => "normal".to_string(),
                    ApiPlan::Pro => "pro".to_string(),
                },
            });
        }

        let manager = WsManager::new(DATA_WS_URL, config)?;

        Ok(DataWss {
            manager,
            subscriptions: Vec::new(),
        })
    }

    pub fn subscribe_price(&mut self, chain: &str, token: &str) {
        self.subscriptions.push(WsSubscribeMsg {
            action: "subscribe".to_string(),
            channel: "price".to_string(),
            token: token.to_string(),
            chain: chain.to_string(),
            interval: None,
        });
    }

    pub fn subscribe_txs(&mut self, chain: &str, token: &str) {
        self.subscriptions.push(WsSubscribeMsg {
            action: "subscribe".to_string(),
            channel: "tx".to_string(),
            token: token.to_string(),
            chain: chain.to_string(),
            interval: None,
        });
    }

    #[allow(dead_code)]
    pub fn subscribe_kline(&mut self, chain: &str, token: &str, interval: &str) {
        self.subscriptions.push(WsSubscribeMsg {
            action: "subscribe".to_string(),
            channel: "kline".to_string(),
            token: token.to_string(),
            chain: chain.to_string(),
            interval: Some(interval.to_string()),
        });
    }

    #[allow(dead_code)]
    pub async fn stream(self) -> Result<mpsc::Receiver<serde_json::Value>, ZeroClawError> {
        let (tx, rx) = mpsc::channel::<serde_json::Value>(256);

        let subscriptions = self.subscriptions.clone();
        let mut manager = self.manager;

        tokio::spawn(async move {
            let on_connect = move || -> Vec<String> {
                subscriptions
                    .iter()
                    .map(|s| serde_json::to_string(s).unwrap_or_default())
                    .collect()
            };

            let tx_clone = tx.clone();
            let on_message = move |msg: String| {
                let tx_inner = tx_clone.clone();
                async move {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&msg) {
                        if tx_inner.send(value).await.is_err() {
                            // Receiver dropped, stop the stream
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
            };

            if let Err(e) = manager.run(on_connect, on_message).await {
                tracing::error!("WebSocket stream error: {}", e);
            }
        });

        Ok(rx)
    }

    pub async fn daemon(self) -> Result<(), ZeroClawError> {
        let mut manager = self.manager;

        let subscriptions = self.subscriptions.clone();
        let on_connect = move || -> Vec<String> {
            subscriptions
                .iter()
                .map(|s| serde_json::to_string(s).unwrap_or_default())
                .collect()
        };

        let on_message = |msg: String| {
            async move {
                // Print data to stdout, log to stderr
                eprintln!("{}", msg);
                Ok(true)
            }
        };

        manager.run(on_connect, on_message).await
    }

    pub async fn stream_typed(self) -> Result<mpsc::Receiver<WsEvent>, ZeroClawError> {
        let (tx, rx) = mpsc::channel::<WsEvent>(256);

        let subscriptions = self.subscriptions.clone();
        let mut manager = self.manager;

        tokio::spawn(async move {
            let on_connect = move || -> Vec<String> {
                subscriptions
                    .iter()
                    .map(|s| serde_json::to_string(s).unwrap_or_default())
                    .collect()
            };

            let tx_clone = tx.clone();
            let on_message = move |msg: String| {
                let tx_inner = tx_clone.clone();
                async move {
                    let event = parse_ws_event(&msg);
                    if tx_inner.send(event).await.is_err() {
                        return Ok(false);
                    }
                    Ok(true)
                }
            };

            if let Err(e) = manager.run(on_connect, on_message).await {
                tracing::error!("WebSocket stream error: {}", e);
            }
        });

        Ok(rx)
    }
}

#[allow(dead_code)]
pub enum WsEvent {
    Price(WsPriceUpdate),
    Tx(WsTxUpdate),
    Kline(WsKlineUpdate),
    Order(WsOrderUpdate),
    Unknown(serde_json::Value),
}

fn parse_ws_event(msg: &str) -> WsEvent {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(msg) {
        let channel = value.get("channel").and_then(|c| c.as_str()).unwrap_or("");

        match channel {
            "price" => {
                if let Ok(update) = serde_json::from_value::<WsPriceUpdate>(value.clone()) {
                    return WsEvent::Price(update);
                }
            }
            "tx" => {
                if let Ok(update) = serde_json::from_value::<WsTxUpdate>(value.clone()) {
                    return WsEvent::Tx(update);
                }
            }
            "kline" => {
                if let Ok(update) = serde_json::from_value::<WsKlineUpdate>(value.clone()) {
                    return WsEvent::Kline(update);
                }
            }
            "order" => {
                if let Ok(update) = serde_json::from_value::<WsOrderUpdate>(value.clone()) {
                    return WsEvent::Order(update);
                }
            }
            _ => {}
        }
    }
    WsEvent::Unknown(serde_json::Value::Null)
}

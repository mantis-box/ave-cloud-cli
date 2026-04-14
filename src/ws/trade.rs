use std::time::Duration;

use tokio::sync::mpsc;
use tracing;

use crate::config::{ApiPlan, Config};
use crate::error::ZeroClawError;
use crate::types::WsOrderUpdate;
use crate::ws::manager::WsManager;

const TRADE_WS_URL: &str = "wss://bot-api.ave.ai/thirdws";

pub struct TradeWss {
    manager: WsManager,
}

impl TradeWss {
    pub fn new(config: &Config) -> Result<Self, ZeroClawError> {
        // Require Normal or Pro plan
        if !config.can_trade_proxy() {
            return Err(ZeroClawError::PlanRequired {
                required: "normal".to_string(),
                current: match config.api_plan {
                    ApiPlan::Free => "free".to_string(),
                    ApiPlan::Normal => "normal".to_string(),
                    ApiPlan::Pro => "pro".to_string(),
                },
            });
        }

        // Require secret_key for signature
        if config.secret_key.is_none() {
            return Err(ZeroClawError::Config(
                "AVE_SECRET_KEY required for trade WebSocket".to_string(),
            ));
        }

        let manager = WsManager::new(TRADE_WS_URL, config)?;

        Ok(TradeWss { manager })
    }

    pub async fn watch_orders(self) -> Result<mpsc::Receiver<WsOrderUpdate>, ZeroClawError> {
        let (tx, rx) = mpsc::channel::<WsOrderUpdate>(128);

        let mut manager = self.manager;

        tokio::spawn(async move {
            let on_connect = || -> Vec<String> {
                vec![serde_json::json!({
                    "action": "subscribe",
                    "channel": "orders"
                })
                .to_string()]
            };

            let tx_clone = tx;
            let on_message = move |msg: String| {
                let tx_inner = tx_clone.clone();
                async move {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&msg) {
                        if let Ok(update) = serde_json::from_value::<WsOrderUpdate>(value) {
                            if tx_inner.send(update).await.is_err() {
                                return Ok(false);
                            }
                        }
                    }
                    Ok(true)
                }
            };

            if let Err(e) = manager.run(on_connect, on_message).await {
                tracing::error!("Trade WebSocket error: {}", e);
            }
        });

        Ok(rx)
    }

    #[allow(dead_code)]
    pub async fn watch_order(
        self,
        order_id: &str,
    ) -> Result<mpsc::Receiver<WsOrderUpdate>, ZeroClawError> {
        let (tx, rx) = mpsc::channel::<WsOrderUpdate>(128);

        let mut manager = self.manager;
        let order_id = order_id.to_string();

        tokio::spawn(async move {
            let on_connect = move || -> Vec<String> {
                vec![serde_json::json!({
                    "action": "subscribe",
                    "channel": "orders",
                    "order_id": order_id
                })
                .to_string()]
            };

            let tx_clone = tx;
            let on_message = move |msg: String| {
                let tx_inner = tx_clone.clone();
                async move {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&msg) {
                        if let Ok(update) = serde_json::from_value::<WsOrderUpdate>(value) {
                            if tx_inner.send(update).await.is_err() {
                                return Ok(false);
                            }
                        }
                    }
                    Ok(true)
                }
            };

            if let Err(e) = manager.run(on_connect, on_message).await {
                tracing::error!("Trade WebSocket error: {}", e);
            }
        });

        Ok(rx)
    }

    #[allow(dead_code)]
    pub async fn await_order_completion(
        mut rx: mpsc::Receiver<WsOrderUpdate>,
        order_id: &str,
        timeout: Duration,
    ) -> Result<WsOrderUpdate, ZeroClawError> {
        let order_id = order_id.to_string();
        let result = tokio::time::timeout(timeout, async move {
            while let Some(update) = rx.recv().await {
                if update.order_id == order_id {
                    let terminal = matches!(
                        update.status,
                        crate::types::OrderStatus::Filled
                            | crate::types::OrderStatus::Cancelled
                            | crate::types::OrderStatus::Failed
                    );
                    if terminal {
                        return Ok(update);
                    }
                }
            }
            Err(ZeroClawError::WsDisconnected)
        })
        .await;

        match result {
            Ok(inner_result) => inner_result,
            Err(_) => Err(ZeroClawError::Config(
                "Timeout waiting for order completion".to_string(),
            )),
        }
    }
}

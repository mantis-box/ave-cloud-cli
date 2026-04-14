use std::time::Duration;
use url::Url;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::error::ZeroClawError;

const PING_INTERVAL: Duration = Duration::from_secs(30);
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(60);

pub struct WsManager {
    url: Url,
    #[allow(dead_code)]
    api_key: String,
    reconnect_delay: Duration,
}

impl WsManager {
    pub fn new(url: &str, config: &Config) -> Result<Self, ZeroClawError> {
        // Add API key as query param - but don't log the full URL
        let url_with_key = format!("{}?api_key={}", url, config.api_key);
        let url = Url::parse(&url_with_key)
            .map_err(|e| ZeroClawError::Config(format!("Invalid WebSocket URL: {}", e)))?;

        // Mask API key in debug output
        let masked_key = format!(
            "...{}",
            &config.api_key[config.api_key.len().saturating_sub(4)..]
        );
        debug!(
            "WebSocket URL: {}?api_key={}",
            url.host_str().unwrap_or("unknown"),
            masked_key
        );

        Ok(WsManager {
            url,
            api_key: config.api_key.clone(),
            reconnect_delay: Duration::from_secs(1),
        })
    }

    pub async fn run<F, Fut>(
        &mut self,
        on_connect: impl Fn() -> Vec<String> + Clone,
        on_message: F,
    ) -> Result<(), ZeroClawError>
    where
        F: Fn(String) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<bool, ZeroClawError>> + Send,
    {
        loop {
            match self
                .connect_and_run(on_connect.clone(), on_message.clone())
                .await
            {
                Ok(()) => return Ok(()),
                Err(e) => {
                    warn!(
                        "WebSocket error: {}. Reconnecting in {:?}",
                        e, self.reconnect_delay
                    );
                    tokio::time::sleep(self.reconnect_delay).await;
                    // Exponential backoff
                    self.reconnect_delay = (self.reconnect_delay * 2).min(MAX_RECONNECT_DELAY);
                }
            }
        }
    }

    async fn connect_and_run<F, Fut>(
        &mut self,
        on_connect: impl Fn() -> Vec<String> + Clone,
        on_message: F,
    ) -> Result<(), ZeroClawError>
    where
        F: Fn(String) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<bool, ZeroClawError>> + Send,
    {
        info!("Connecting to WebSocket...");
        let (ws_stream, _) = connect_async(&self.url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Reset reconnect delay on successful connect
        self.reconnect_delay = Duration::from_secs(1);

        // Send subscription messages
        let subscribe_msgs = on_connect();
        for msg in subscribe_msgs {
            debug!("Sending: {}", msg);
            write.send(Message::Text(msg)).await?;
        }

        // Create ping interval
        let mut ping_interval = tokio::time::interval(PING_INTERVAL);
        ping_interval.tick().await; // Skip first tick

        loop {
            tokio::select! {
                // Incoming message
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            let should_continue = on_message(text.to_string()).await?;
                            if !should_continue {
                                return Ok(());
                            }
                        }
                        Some(Ok(Message::Binary(data))) => {
                            if let Ok(text) = String::from_utf8(data) {
                                let should_continue = on_message(text).await?;
                                if !should_continue {
                                    return Ok(());
                                }
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            debug!("Received ping, sending pong");
                            write.send(Message::Pong(data)).await?;
                        }
                        Some(Ok(Message::Pong(_))) => {
                            debug!("Received pong");
                        }
                        Some(Ok(Message::Close(reason))) => {
                            info!("Server closed connection: {:?}", reason);
                            return Err(ZeroClawError::WsDisconnected);
                        }
                        Some(Ok(Message::Frame(_))) => {
                            // Ignore frame messages - they're internal to the protocol
                        }
                        Some(Err(e)) => {
                            warn!("WebSocket read error: {}", e);
                            return Err(ZeroClawError::WebSocket(Box::new(e)));
                        }
                        None => {
                            return Err(ZeroClawError::WsDisconnected);
                        }
                    }
                }
                // Ping tick
                _ = ping_interval.tick() => {
                    debug!("Sending ping");
                    match tokio::time::timeout(PING_TIMEOUT, write.send(Message::Ping(vec![]))).await {
                        Ok(Ok(())) => {
                            debug!("Ping sent successfully");
                        }
                        Ok(Err(e)) => {
                            warn!("Failed to send ping: {}", e);
                            return Err(ZeroClawError::WebSocket(Box::new(e)));
                        }
                        Err(_) => {
                            warn!("Ping timeout");
                            return Err(ZeroClawError::WsDisconnected);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_manager_creation() {
        let config = crate::config::Config {
            api_key: "test_key_12345".to_string(),
            api_plan: crate::config::ApiPlan::Pro,
            secret_key: None,
            evm_private_key: None,
            solana_private_key: None,
            mnemonic: None,
            bsc_rpc_url: "https://bsc.publicnode.com".to_string(),
            eth_rpc_url: "https://eth.publicnode.com".to_string(),
            base_rpc_url: "https://base.publicnode.com".to_string(),
        };

        let manager = WsManager::new("wss://ave.ai/ws", &config);
        assert!(manager.is_ok());
    }
}

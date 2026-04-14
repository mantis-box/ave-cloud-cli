//! Interactive WebSocket REPL for Ave Cloud Data WSS API.
//!
//! Supports JSON-RPC 2.0 protocol with commands:
//! - subscribe price <addr-chain>...
//! - subscribe tx <pair> <chain> [tx|multi_tx|liq]
//! - subscribe kline <pair> <chain> [interval]
//! - unsubscribe
//! - quit

use std::io::{self, Write};

use futures_util::{FutureExt, SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info};
use url::Url;

use crate::config::Config;
use crate::error::ZeroClawError;

const DATA_WSS_URL: &str = "wss://wss.ave-api.xyz";

/// Run interactive WebSocket REPL.
pub async fn run_repl(config: &Config) -> Result<(), ZeroClawError> {
    if !config.can_stream() {
        return Err(ZeroClawError::PlanRequired {
            required: "pro".to_string(),
            current: match config.api_plan {
                crate::config::ApiPlan::Free => "free".to_string(),
                crate::config::ApiPlan::Normal => "normal".to_string(),
                crate::config::ApiPlan::Pro => "pro".to_string(),
            },
        });
    }

    let url = format!("{}?api_key={}", DATA_WSS_URL, config.api_key);
    let ws_url = Url::parse(&url)
        .map_err(|e| ZeroClawError::Config(format!("Invalid WebSocket URL: {}", e)))?;

    info!("Connecting to {}", ws_url.host_str().unwrap_or("unknown"));

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;

    let (mut write, mut read) = ws_stream.split();

    eprintln!("Connected. Type 'help' for commands.");

    let mut msg_id: u64 = 0;

    loop {
        // Print prompt
        print!("\n> ");
        io::stdout()
            .flush()
            .map_err(|e| ZeroClawError::Io(e.to_string()))?;

        // Read line from stdin (blocking)
        let mut line = String::new();
        let stdin = io::stdin();
        let bytes = stdin
            .read_line(&mut line)
            .map_err(|e| ZeroClawError::Io(e.to_string()))?;
        if bytes == 0 {
            break; // EOF
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let cmd = parts[0].to_lowercase();
        match cmd.as_str() {
            "quit" | "exit" | "q" => {
                break;
            }
            "help" => {
                eprintln!(
                    "Commands:\n  subscribe price <addr-chain> [...]\n  subscribe tx <pair> <chain> [tx|multi_tx|liq]\n  subscribe kline <pair> <chain> [interval]\n  unsubscribe\n  quit"
                );
            }
            "subscribe" if parts.len() >= 2 => {
                msg_id += 1;
                let topic = parts[1];
                match topic {
                    "price" if parts.len() >= 3 => {
                        let tokens: Vec<&str> = parts[2..].into();
                        let msg = serde_json::json!({
                            "jsonrpc": "2.0",
                            "method": "subscribe",
                            "params": ["price", tokens],
                            "id": msg_id
                        });
                        write
                            .send(Message::Text(msg.to_string()))
                            .await
                            .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;
                        eprintln!("Subscribed to price for {} token(s)", tokens.len());
                    }
                    "tx" | "multi_tx" | "liq" if parts.len() >= 4 => {
                        let pair = parts[2];
                        let chain = parts[3];
                        let topic_name = if parts.len() > 4 { parts[4] } else { topic };
                        let msg = serde_json::json!({
                            "jsonrpc": "2.0",
                            "method": "subscribe",
                            "params": [topic_name, pair, chain],
                            "id": msg_id
                        });
                        write
                            .send(Message::Text(msg.to_string()))
                            .await
                            .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;
                        eprintln!("Subscribed to {} for {} on {}", topic_name, pair, chain);
                    }
                    "kline" if parts.len() >= 4 => {
                        let pair = parts[2];
                        let chain = parts[3];
                        let interval = if parts.len() > 4 { parts[4] } else { "k60" };
                        let msg = serde_json::json!({
                            "jsonrpc": "2.0",
                            "method": "subscribe",
                            "params": ["kline", pair, interval, chain],
                            "id": msg_id
                        });
                        write
                            .send(Message::Text(msg.to_string()))
                            .await
                            .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;
                        eprintln!(
                            "Subscribed to kline for {} on {} ({})",
                            pair, chain, interval
                        );
                    }
                    _ => {
                        eprintln!(
                            "Unknown topic: {}. Topics: price, tx, multi_tx, liq, kline",
                            topic
                        );
                    }
                }
            }
            "unsubscribe" => {
                msg_id += 1;
                let msg = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "unsubscribe",
                    "params": [],
                    "id": msg_id
                });
                write
                    .send(Message::Text(msg.to_string()))
                    .await
                    .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;
                eprintln!("Unsubscribed");
            }
            _ => {
                eprintln!("Unknown command: {}. Type 'help'.", cmd);
            }
        }

        // Drain any pending messages that arrived before we got back to reading
        loop {
            match read.next().now_or_never() {
                Some(Some(Ok(msg))) => match msg {
                    Message::Text(text) => {
                        println!("{}", text);
                        println!("---");
                    }
                    Message::Binary(data) => {
                        if let Ok(text) = String::from_utf8(data) {
                            println!("{}", text);
                            println!("---");
                        }
                    }
                    Message::Ping(data) => {
                        write
                            .send(Message::Pong(data))
                            .await
                            .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;
                    }
                    Message::Close(reason) => {
                        eprintln!("Connection closed: {:?}", reason);
                        return Ok(());
                    }
                    _ => {}
                },
                Some(None) => break, // Stream ended
                None => break,       // No more messages right now
                Some(Some(Err(e))) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Stream price updates for multiple tokens.
pub async fn watch_price(config: &Config, tokens: Vec<String>) -> Result<(), ZeroClawError> {
    if !config.can_stream() {
        return Err(ZeroClawError::PlanRequired {
            required: "pro".to_string(),
            current: match config.api_plan {
                crate::config::ApiPlan::Free => "free".to_string(),
                crate::config::ApiPlan::Normal => "normal".to_string(),
                crate::config::ApiPlan::Pro => "pro".to_string(),
            },
        });
    }

    let url = format!("{}?api_key={}", DATA_WSS_URL, config.api_key);
    let ws_url = Url::parse(&url)
        .map_err(|e| ZeroClawError::Config(format!("Invalid WebSocket URL: {}", e)))?;

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;

    let (mut write, mut read) = ws_stream.split();

    // Send subscription
    let msg = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "subscribe",
        "params": ["price", tokens],
        "id": 1
    });
    write
        .send(Message::Text(msg.to_string()))
        .await
        .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;

    eprintln!(
        "Subscribed to price for {} token(s). Press Ctrl+C to exit.",
        tokens.len()
    );

    // Read messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("{}", text);
                println!("---");
            }
            Ok(Message::Close(reason)) => {
                eprintln!("Connection closed: {:?}", reason);
                break;
            }
            Ok(Message::Ping(data)) => {
                write
                    .send(Message::Pong(data))
                    .await
                    .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Stream kline updates for a pair.
pub async fn watch_kline(
    config: &Config,
    address: &str,
    chain: &str,
    interval: &str,
    format_markdown: bool,
) -> Result<(), ZeroClawError> {
    if !config.can_stream() {
        return Err(ZeroClawError::PlanRequired {
            required: "pro".to_string(),
            current: match config.api_plan {
                crate::config::ApiPlan::Free => "free".to_string(),
                crate::config::ApiPlan::Normal => "normal".to_string(),
                crate::config::ApiPlan::Pro => "pro".to_string(),
            },
        });
    }

    let url = format!("{}?api_key={}", DATA_WSS_URL, config.api_key);
    let ws_url = Url::parse(&url)
        .map_err(|e| ZeroClawError::Config(format!("Invalid WebSocket URL: {}", e)))?;

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;

    let (mut write, mut read) = ws_stream.split();

    // Send subscription
    let msg = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "subscribe",
        "params": ["kline", address, interval, chain],
        "id": 1
    });
    write
        .send(Message::Text(msg.to_string()))
        .await
        .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;

    eprintln!(
        "Subscribed to kline for {} on {} ({}). Press Ctrl+C to exit.",
        address, chain, interval
    );

    let mut formatter = if format_markdown {
        Some(crate::types::KlineFormatter::new(20))
    } else {
        None
    };

    // Read messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Some(ref mut fmt) = formatter {
                    // Try to extract kline event and format
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(event) = extract_kline_event(&value) {
                            println!("{}", fmt.render(&event));
                            println!("---");
                            continue;
                        }
                    }
                }
                println!("{}", text);
                println!("---");
            }
            Ok(Message::Close(reason)) => {
                eprintln!("Connection closed: {:?}", reason);
                break;
            }
            Ok(Message::Ping(data)) => {
                write
                    .send(Message::Pong(data))
                    .await
                    .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Stream tx updates for a pair.
pub async fn watch_tx(
    config: &Config,
    address: &str,
    chain: &str,
    topic: &str,
) -> Result<(), ZeroClawError> {
    if !config.can_stream() {
        return Err(ZeroClawError::PlanRequired {
            required: "pro".to_string(),
            current: match config.api_plan {
                crate::config::ApiPlan::Free => "free".to_string(),
                crate::config::ApiPlan::Normal => "normal".to_string(),
                crate::config::ApiPlan::Pro => "pro".to_string(),
            },
        });
    }

    let url = format!("{}?api_key={}", DATA_WSS_URL, config.api_key);
    let ws_url = Url::parse(&url)
        .map_err(|e| ZeroClawError::Config(format!("Invalid WebSocket URL: {}", e)))?;

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;

    let (mut write, mut read) = ws_stream.split();

    // Send subscription
    let msg = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "subscribe",
        "params": [topic, address, chain],
        "id": 1
    });
    write
        .send(Message::Text(msg.to_string()))
        .await
        .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;

    eprintln!(
        "Subscribed to {} for {} on {}. Press Ctrl+C to exit.",
        topic, address, chain
    );

    // Read messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("{}", text);
                println!("---");
            }
            Ok(Message::Close(reason)) => {
                eprintln!("Connection closed: {:?}", reason);
                break;
            }
            Ok(Message::Ping(data)) => {
                write
                    .send(Message::Pong(data))
                    .await
                    .map_err(|e| ZeroClawError::WebSocket(Box::new(e)))?;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Extract kline event from JSON-RPC 2.0 response.
fn extract_kline_event(value: &serde_json::Value) -> Option<crate::types::WssKlineEvent> {
    // Try various nested locations for kline data
    if let Some(result) = value.get("result") {
        if let Some(kline) = result.get("kline") {
            let source = kline.get("usd").or(kline.get("eth"))?;
            let id = result.get("id").and_then(|v| v.as_str()).unwrap_or("pair");
            let (pair, chain) = if let Some(idx) = id.rfind('-') {
                (&id[..idx], &id[idx + 1..])
            } else {
                (id, "?")
            };
            return Some(crate::types::WssKlineEvent {
                event_type: "kline".to_string(),
                pair: Some(pair.to_string()),
                chain: Some(chain.to_string()),
                interval: result
                    .get("interval")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                time: source.get("time").and_then(|v| v.as_i64()),
                open: source.get("open").and_then(|v| v.as_f64()),
                high: source.get("high").and_then(|v| v.as_f64()),
                low: source.get("low").and_then(|v| v.as_f64()),
                close: source.get("close").and_then(|v| v.as_f64()),
                volume: source.get("volume").and_then(|v| v.as_f64()),
            });
        }
    }
    None
}

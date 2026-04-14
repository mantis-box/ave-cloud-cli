//! Proxy Wallet - high-level wrapper for server-managed trading
//!
//! This module provides a simplified interface for trading using the Ave Cloud
//! proxy wallet service. Orders are created via REST API and tracked via WebSocket.

use tracing::warn;

use crate::error::ZeroClawError;
use crate::rest::{AveClient, DataApi};
use crate::types::{Order, OrderStatus, OrderType, ProxyOrderParams, RiskScore, TradeSide};

pub struct ProxyWallet<'a> {
    client: &'a AveClient,
}

#[allow(dead_code)]
impl<'a> ProxyWallet<'a> {
    pub fn new(client: &'a AveClient) -> Result<Self, ZeroClawError> {
        let config = client.config();
        if !config.can_trade_proxy() {
            return Err(ZeroClawError::PlanRequired {
                required: "normal".to_string(),
                current: match config.api_plan {
                    crate::config::ApiPlan::Free => "free".to_string(),
                    crate::config::ApiPlan::Normal => "normal".to_string(),
                    crate::config::ApiPlan::Pro => "pro".to_string(),
                },
            });
        }
        Ok(ProxyWallet { client })
    }

    /// Place a market buy order
    pub async fn market_buy(
        &self,
        chain: &str,
        token: &str,
        amount_usd: f64,
    ) -> Result<Order, ZeroClawError> {
        let params = ProxyOrderParams {
            token_address: token.to_string(),
            chain: chain.to_string(),
            side: TradeSide::Buy,
            order_type: OrderType::Market,
            amount_usd,
            trigger_price: None,
            tp_price: None,
            sl_price: None,
        };

        self.client.trade().create_proxy_order(params).await
    }

    /// Place a market sell order
    pub async fn market_sell(
        &self,
        chain: &str,
        token: &str,
        amount_usd: f64,
    ) -> Result<Order, ZeroClawError> {
        let params = ProxyOrderParams {
            token_address: token.to_string(),
            chain: chain.to_string(),
            side: TradeSide::Sell,
            order_type: OrderType::Market,
            amount_usd,
            trigger_price: None,
            tp_price: None,
            sl_price: None,
        };

        self.client.trade().create_proxy_order(params).await
    }

    /// Place a limit buy order
    pub async fn limit_buy(
        &self,
        chain: &str,
        token: &str,
        amount_usd: f64,
        trigger_price: f64,
    ) -> Result<Order, ZeroClawError> {
        let params = ProxyOrderParams {
            token_address: token.to_string(),
            chain: chain.to_string(),
            side: TradeSide::Buy,
            order_type: OrderType::Limit,
            amount_usd,
            trigger_price: Some(trigger_price),
            tp_price: None,
            sl_price: None,
        };

        self.client.trade().create_proxy_order(params).await
    }

    /// Place a limit sell order
    pub async fn limit_sell(
        &self,
        chain: &str,
        token: &str,
        amount_usd: f64,
        trigger_price: f64,
    ) -> Result<Order, ZeroClawError> {
        let params = ProxyOrderParams {
            token_address: token.to_string(),
            chain: chain.to_string(),
            side: TradeSide::Sell,
            order_type: OrderType::Limit,
            amount_usd,
            trigger_price: Some(trigger_price),
            tp_price: None,
            sl_price: None,
        };

        self.client.trade().create_proxy_order(params).await
    }

    /// Place a market buy with take-profit and stop-loss attached
    pub async fn buy_with_tp_sl(
        &self,
        chain: &str,
        token: &str,
        amount_usd: f64,
        tp_price: f64,
        sl_price: f64,
    ) -> Result<Order, ZeroClawError> {
        let params = ProxyOrderParams {
            token_address: token.to_string(),
            chain: chain.to_string(),
            side: TradeSide::Buy,
            order_type: OrderType::Market,
            amount_usd,
            trigger_price: None,
            tp_price: Some(tp_price),
            sl_price: Some(sl_price),
        };

        self.client.trade().create_proxy_order(params).await
    }

    /// Cancel an existing order
    pub async fn cancel(&self, order_id: &str) -> Result<(), ZeroClawError> {
        self.client.trade().cancel_proxy_order(order_id).await
    }

    /// List all open orders
    pub async fn list_open(&self) -> Result<Vec<Order>, ZeroClawError> {
        self.client
            .trade()
            .list_proxy_orders(Some(OrderStatus::Pending))
            .await
    }

    /// List all orders, optionally filtered by status
    pub async fn list_orders(
        &self,
        status: Option<OrderStatus>,
    ) -> Result<Vec<Order>, ZeroClawError> {
        self.client.trade().list_proxy_orders(status).await
    }

    /// Get a specific order by ID
    pub async fn get_order(&self, order_id: &str) -> Result<Order, ZeroClawError> {
        self.client.trade().get_proxy_order(order_id).await
    }

    /// Safe market buy - performs honeypot check before executing
    ///
    /// # Safety
    /// Warns or errors on:
    /// - Honeypot tokens (is_honeypot = true)
    /// - High risk scores (>70)
    /// - High buy tax (>10%)
    pub async fn safe_market_buy(
        &self,
        chain: &str,
        token: &str,
        amount_usd: f64,
        data_api: &DataApi<'_>,
        max_risk_level: u8,
        fail_on_honeypot: bool,
    ) -> Result<Order, ZeroClawError> {
        // Step 1: Perform risk check
        let risk: RiskScore = data_api.risk_check(chain, token).await?;

        // Step 2: Honeypot check
        if risk.is_honeypot {
            if fail_on_honeypot {
                return Err(ZeroClawError::Config(
                    "Token is flagged as honeypot. Refusing to trade.".to_string(),
                ));
            } else {
                warn!(
                    "Token {} on {} is flagged as HONEYPOT. Trading anyway.",
                    token, chain
                );
            }
        }

        // Step 3: Risk level check
        if risk.risk_level > max_risk_level {
            if fail_on_honeypot {
                return Err(ZeroClawError::Config(format!(
                    "Token risk level {} exceeds maximum {}. Refusing to trade.",
                    risk.risk_level, max_risk_level
                )));
            } else {
                warn!(
                    "Token {} risk level {} exceeds threshold {}. Trading anyway.",
                    token, risk.risk_level, max_risk_level
                );
            }
        }

        // Step 4: Buy tax check
        if risk.buy_tax > 10.0 {
            warn!(
                "Token {} has high buy tax: {}%. Consider the cost.",
                token, risk.buy_tax
            );
        }

        // Step 5: Execute the buy
        self.market_buy(chain, token, amount_usd).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_order_params_serialization() {
        let params = ProxyOrderParams {
            token_address: "0x123".to_string(),
            chain: "BSC".to_string(),
            side: TradeSide::Buy,
            order_type: OrderType::Market,
            amount_usd: 100.0,
            trigger_price: None,
            tp_price: Some(200.0),
            sl_price: Some(50.0),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"token_address\":\"0x123\""));
        assert!(json.contains("\"side\":\"buy\""));
        assert!(json.contains("\"tp_price\":200"));
    }
}

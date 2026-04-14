use std::fmt::Write;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::Semaphore;
use tracing::debug;

use crate::config::{ApiPlan, Config};
use crate::error::ZeroClawError;

// ============================================================
// Header Constants (aligned with Python reference)
// ============================================================
pub const HEADER_DATA_API_KEY: &str = "X-API-KEY";
pub const HEADER_TRADE_ACCESS_KEY: &str = "AVE-ACCESS-KEY";
pub const HEADER_TRADE_ACCESS_TIMESTAMP: &str = "AVE-ACCESS-TIMESTAMP";
pub const HEADER_TRADE_ACCESS_SIGN: &str = "AVE-ACCESS-SIGN";

pub const BASE_URL: &str = "https://ave.ai/api";
pub const WSS_BASE_URL: &str = "wss://wss.ave-api.xyz";
pub const DATA_V2_BASE_URL: &str = "https://data.ave-api.xyz/v2";
pub const TRADE_API_BASE_URL: &str = "https://bot-api.ave.ai";
#[allow(dead_code)]
pub const TRADE_WSS_BASE_URL: &str = "wss://bot-api.ave.ai/thirdws";

pub struct AveClient {
    http: Client,
    config: Config,
    #[allow(dead_code)]
    limiter: Arc<Semaphore>,
    #[allow(dead_code)]
    rpm: u32,
}

impl AveClient {
    pub fn new(config: Config) -> Result<Self, ZeroClawError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .gzip(true)
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| ZeroClawError::Config(format!("Failed to create HTTP client: {}", e)))?;

        let rpm = config.api_plan.rpm();
        let limiter = Arc::new(Semaphore::new(rpm as usize));

        debug!("AveClient created with {} RPM", rpm);

        Ok(AveClient {
            http,
            config,
            limiter,
            rpm,
        })
    }

    pub fn data(&self) -> DataApi<'_> {
        DataApi { client: self }
    }

    pub fn trade(&self) -> TradeApi<'_> {
        TradeApi { client: self }
    }

    #[allow(dead_code)]
    pub fn wss_base_url(&self) -> &str {
        WSS_BASE_URL
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T, ZeroClawError> {
        self.wait_for_quota().await;

        let url = format!("{}{}", BASE_URL, path);
        debug!("GET {}", path);

        let resp = self
            .http
            .get(&url)
            .header(HEADER_DATA_API_KEY, &self.config.api_key)
            .query(params)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ZeroClawError::RateLimit);
        }

        let text = resp.text().await?;
        let parsed: crate::types::RawApiResponse = serde_json::from_str(&text)?;

        if parsed.status != 1 && parsed.status != 200 {
            return Err(ZeroClawError::Api {
                code: parsed.status,
                message: parsed.msg,
            });
        }

        let data: T = serde_json::from_value(parsed.data.unwrap_or(serde_json::Value::Null))?;

        Ok(data)
    }

    pub(crate) async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ZeroClawError> {
        self.wait_for_quota().await;

        let url = format!("{}{}", BASE_URL, path);
        debug!("POST {}", path);

        let resp = self
            .http
            .post(&url)
            .header(HEADER_DATA_API_KEY, &self.config.api_key)
            .json(body)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ZeroClawError::RateLimit);
        }

        let text = resp.text().await?;
        let parsed: crate::types::RawApiResponse = serde_json::from_str(&text)?;

        if parsed.status != 1 && parsed.status != 200 {
            return Err(ZeroClawError::Api {
                code: parsed.status,
                message: parsed.msg,
            });
        }

        let data: T = serde_json::from_value(parsed.data.unwrap_or(serde_json::Value::Null))?;

        Ok(data)
    }

    async fn wait_for_quota(&self) {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // ============================================================
    // V2 Data API Methods (Phase 1)
    // ============================================================

    pub(crate) async fn get_v2<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(String, String)],
    ) -> Result<T, ZeroClawError> {
        self.wait_for_quota().await;

        let url = format!("{}{}", DATA_V2_BASE_URL, path);
        debug!("GET v2 {}", path);

        let resp = self
            .http
            .get(&url)
            .header(HEADER_DATA_API_KEY, &self.config.api_key)
            .query(&params)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ZeroClawError::RateLimit);
        }

        let text = resp.text().await?;
        let parsed: crate::types::RawApiResponse = serde_json::from_str(&text)?;

        if parsed.status != 1 && parsed.status != 200 {
            return Err(ZeroClawError::Api {
                code: parsed.status,
                message: parsed.msg,
            });
        }

        let data: T = serde_json::from_value(parsed.data.unwrap_or(serde_json::Value::Null))?;

        Ok(data)
    }

    pub(crate) async fn post_v2<T: DeserializeOwned>(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<T, ZeroClawError> {
        self.wait_for_quota().await;

        let url = format!("{}{}", DATA_V2_BASE_URL, path);
        debug!("POST v2 {}", path);

        let resp = self
            .http
            .post(&url)
            .header(HEADER_DATA_API_KEY, &self.config.api_key)
            .json(&body)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ZeroClawError::RateLimit);
        }

        let text = resp.text().await?;
        let parsed: crate::types::RawApiResponse = serde_json::from_str(&text)?;

        if parsed.status != 1 && parsed.status != 200 {
            return Err(ZeroClawError::Api {
                code: parsed.status,
                message: parsed.msg,
            });
        }

        let data: T = serde_json::from_value(parsed.data.unwrap_or(serde_json::Value::Null))?;

        Ok(data)
    }

    // ============================================================
    // Trade API V2 Methods (Phase 3 & 4)
    // ============================================================

    pub(crate) async fn trade_get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(String, String)],
        proxy: bool,
    ) -> Result<T, ZeroClawError> {
        self.wait_for_quota().await;

        let url = format!("{}{}", TRADE_API_BASE_URL, path);
        debug!("GET trade {} (proxy={})", path, proxy);

        let mut req = self.http.get(&url);
        req = req.header(HEADER_TRADE_ACCESS_KEY, &self.config.api_key);

        if proxy {
            if let Some(ref _secret) = self.config.secret_key {
                let (timestamp, signature) = self.sign_proxy_request("GET", path, "");
                req = req
                    .header(HEADER_TRADE_ACCESS_TIMESTAMP, &timestamp)
                    .header(HEADER_TRADE_ACCESS_SIGN, &signature);
            }
        }

        let resp = req.query(&params).send().await?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ZeroClawError::RateLimit);
        }

        let text = resp.text().await?;
        let parsed: crate::types::RawApiResponse = serde_json::from_str(&text)?;

        if parsed.status != 1 && parsed.status != 200 {
            return Err(ZeroClawError::Api {
                code: parsed.status,
                message: parsed.msg,
            });
        }

        let data: T = serde_json::from_value(parsed.data.unwrap_or(serde_json::Value::Null))?;

        Ok(data)
    }

    pub(crate) async fn trade_post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: serde_json::Value,
        proxy: bool,
    ) -> Result<T, ZeroClawError> {
        self.wait_for_quota().await;

        let url = format!("{}{}", TRADE_API_BASE_URL, path);
        debug!("POST trade {} (proxy={})", path, proxy);

        let mut req = self.http.post(&url);
        req = req.header(HEADER_TRADE_ACCESS_KEY, &self.config.api_key);

        if proxy {
            if let Some(ref _secret) = self.config.secret_key {
                let body_str = AveClient::serialize_sorted(&body);
                let (timestamp, signature) = self.sign_proxy_request("POST", path, &body_str);
                req = req
                    .header(HEADER_TRADE_ACCESS_TIMESTAMP, &timestamp)
                    .header(HEADER_TRADE_ACCESS_SIGN, &signature);
            }
        }

        let resp = req.json(&body).send().await?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ZeroClawError::RateLimit);
        }

        let text = resp.text().await?;
        let parsed: crate::types::RawApiResponse = serde_json::from_str(&text)?;

        if parsed.status != 1 && parsed.status != 200 {
            return Err(ZeroClawError::Api {
                code: parsed.status,
                message: parsed.msg,
            });
        }

        let data: T = serde_json::from_value(parsed.data.unwrap_or(serde_json::Value::Null))?;

        Ok(data)
    }

    /// Sign request for proxy wallet using the Python reference algorithm:
    /// message = timestamp + method + path + body
    /// signature = base64(hmac_sha256(secret, message))
    fn sign_proxy_request(&self, method: &str, path: &str, body: &str) -> (String, String) {
        use base64::Engine;
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let secret = match &self.config.secret_key {
            Some(s) => s,
            None => return (String::new(), String::new()),
        };

        // Timestamp in ISO 8601 format (Python reference: second precision, no microseconds)
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // Message: timestamp + method + path + body (body serialized as JSON with sorted keys)
        let mut message = format!("{}{}{}", timestamp, method.to_uppercase(), path);
        if !body.is_empty() {
            // Python reference: json.dumps(body, sort_keys=True, separators=(",", ":"))
            message.push_str(body);
        }

        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(message.as_bytes());

        let signature =
            base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        (timestamp, signature)
    }

    /// Serialize JSON with sorted keys and no spaces (matching Python json.dumps with sort_keys=True)
    pub fn serialize_sorted(body: &serde_json::Value) -> String {
        fn write_value(v: &serde_json::Value, s: &mut String) {
            match v {
                serde_json::Value::Null => s.push_str("null"),
                serde_json::Value::Bool(b) => s.push_str(if *b { "true" } else { "false" }),
                serde_json::Value::Number(n) => s.push_str(&n.to_string()),
                serde_json::Value::String(s2) => write!(s, "\"{}\"", s2).unwrap(),
                serde_json::Value::Array(arr) => {
                    s.push('[');
                    for (i, item) in arr.iter().enumerate() {
                        if i > 0 {
                            s.push(',');
                        }
                        write_value(item, s);
                    }
                    s.push(']');
                }
                serde_json::Value::Object(obj) => {
                    s.push('{');
                    let mut entries: Vec<_> = obj.iter().collect();
                    entries.sort_by(|a, b| a.0.cmp(b.0));
                    for (i, (k, v)) in entries.iter().enumerate() {
                        if i > 0 {
                            s.push(',');
                        }
                        write!(s, "\"{}\":", k).unwrap();
                        write_value(v, s);
                    }
                    s.push('}');
                }
            }
        }
        let mut result = String::new();
        write_value(body, &mut result);
        result
    }

    pub(crate) fn require_plan(
        &self,
        required: ApiPlan,
        _feature: &str,
    ) -> Result<(), ZeroClawError> {
        let current_plan = match self.config.api_plan {
            ApiPlan::Free => "free",
            ApiPlan::Normal => "normal",
            ApiPlan::Pro => "pro",
        };

        let meets_requirement = matches!(
            (&required, &self.config.api_plan),
            (ApiPlan::Free, _)
                | (ApiPlan::Normal, ApiPlan::Normal | ApiPlan::Pro)
                | (ApiPlan::Pro, ApiPlan::Pro)
        );

        if !meets_requirement {
            let required_plan = match required {
                ApiPlan::Free => "free",
                ApiPlan::Normal => "normal",
                ApiPlan::Pro => "pro",
            };

            return Err(ZeroClawError::PlanRequired {
                required: required_plan.to_string(),
                current: current_plan.to_string(),
            });
        }

        Ok(())
    }
}

// ============================================================
// Data API
// ============================================================

#[allow(dead_code)]
pub struct DataApi<'a> {
    client: &'a AveClient,
}

#[allow(dead_code)]
impl<'a> DataApi<'a> {
    pub async fn token_price(&self, chain: &str, address: &str) -> Result<f64, ZeroClawError> {
        #[derive(serde::Deserialize)]
        struct PriceResponse {
            #[serde(rename = "price_usd")]
            price_usd: f64,
        }

        let body = serde_json::json!({
            "token_ids": [format!("{}-{}", address.to_lowercase(), chain.to_lowercase())]
        });

        let data: PriceResponse = self
            .client
            .post_v2("/tokens/price", body)
            .await?;

        Ok(data.price_usd)
    }

    pub async fn token_info(
        &self,
        chain: &str,
        address: &str,
    ) -> Result<crate::types::TokenInfo, ZeroClawError> {
        let path = format!(
            "/tokens/{}-{}",
            address.to_lowercase(),
            chain.to_lowercase()
        );
        self.client.get_v2(&path, &[]).await
    }

    pub async fn search(
        &self,
        query: &str,
        chain: Option<&str>,
    ) -> Result<Vec<crate::types::TokenInfo>, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![("keyword".to_string(), query.to_string())];
        if let Some(c) = chain {
            params.push(("chain".to_string(), c.to_string()));
        }
        self.client.get_v2("/tokens", &params).await
    }

    pub async fn kline(
        &self,
        chain: &str,
        address: &str,
        interval: &str,
        limit: u32,
    ) -> Result<Vec<crate::types::Kline>, ZeroClawError> {
        #[derive(serde::Deserialize)]
        struct KlineResponse {
            points: Vec<crate::types::Kline>,
        }

        let path = format!("/klines/token/{}-{}", address.to_lowercase(), chain.to_lowercase());
        let params: Vec<(String, String)> = vec![
            ("interval".to_string(), interval.to_string()),
            ("size".to_string(), limit.to_string()),
        ];
        let resp: KlineResponse = self.client.get_v2(&path, &params).await?;
        Ok(resp.points)
    }

    pub async fn holders(
        &self,
        chain: &str,
        address: &str,
        page: u32,
    ) -> Result<Vec<serde_json::Value>, ZeroClawError> {
        let path = format!(
            "/tokens/holders/{}-{}",
            address.to_lowercase(),
            chain.to_lowercase()
        );
        let params: Vec<(String, String)> = vec![("page".to_string(), page.to_string())];
        self.client.get_v2(&path, &params).await
    }

    pub async fn swap_txs(
        &self,
        chain: &str,
        address: &str,
        _limit: u32,
    ) -> Result<Vec<crate::types::SwapTx>, ZeroClawError> {
        let path = format!("/txs/{}-{}", address.to_lowercase(), chain.to_lowercase());
        self.client.get_v2(&path, &[]).await
    }

    pub async fn trending(
        &self,
        chain: Option<&str>,
    ) -> Result<Vec<crate::types::TokenInfo>, ZeroClawError> {
        let params: Vec<(String, String)> = match chain {
            Some(c) => vec![
                ("chain".to_string(), c.to_string()),
                ("current_page".to_string(), "1".to_string()),
                ("page_size".to_string(), "20".to_string()),
            ],
            None => vec![
                ("current_page".to_string(), "1".to_string()),
                ("page_size".to_string(), "20".to_string()),
            ],
        };
        self.client.get_v2("/tokens/trending", &params).await
    }

    pub async fn risk_check(
        &self,
        chain: &str,
        address: &str,
    ) -> Result<crate::types::RiskScore, ZeroClawError> {
        let path = format!(
            "/contracts/{}-{}",
            address.to_lowercase(),
            chain.to_lowercase()
        );
        self.client.get_v2(&path, &[]).await
    }
}

// ============================================================
// Trade API
// ============================================================

#[allow(dead_code)]
pub struct TradeApi<'a> {
    client: &'a AveClient,
}

#[allow(dead_code)]
impl<'a> TradeApi<'a> {
    pub async fn swap_quote(
        &self,
        chain: &str,
        token_in: &str,
        token_out: &str,
        amount_in: &str,
        slippage_bps: u32,
    ) -> Result<crate::types::SwapQuote, ZeroClawError> {
        let params = vec![
            ("chain".to_string(), chain.to_string()),
            ("token_in".to_string(), token_in.to_string()),
            ("token_out".to_string(), token_out.to_string()),
            ("in_amount".to_string(), amount_in.to_string()),
            ("slippage".to_string(), slippage_bps.to_string()),
        ];
        self.client.trade_get("/quote", &params, false).await
    }

    pub async fn build_chain_tx(
        &self,
        chain: &str,
        quote: &crate::types::SwapQuote,
        wallet_address: &str,
    ) -> Result<serde_json::Value, ZeroClawError> {
        #[derive(serde::Serialize)]
        struct BuildTxRequest<'a> {
            chain: &'a str,
            quote: &'a crate::types::SwapQuote,
            #[serde(rename = "wallet_address")]
            wallet_address: &'a str,
        }

        let body = serde_json::to_value(&BuildTxRequest {
            chain,
            quote,
            wallet_address,
        })
        .map_err(|e| ZeroClawError::Config(format!("Failed to serialize request: {}", e)))?;

        self.client.trade_post("/build", body, false).await
    }

    pub async fn broadcast_tx(
        &self,
        chain: &str,
        signed_tx_hex: &str,
    ) -> Result<String, ZeroClawError> {
        #[derive(serde::Serialize)]
        struct BroadcastRequest<'a> {
            chain: &'a str,
            #[serde(rename = "signed_tx")]
            signed_tx: &'a str,
        }

        #[derive(serde::Deserialize)]
        struct BroadcastResponse {
            #[serde(rename = "tx_hash")]
            tx_hash: String,
        }

        let body = serde_json::to_value(&BroadcastRequest {
            chain,
            signed_tx: signed_tx_hex,
        })
        .map_err(|e| ZeroClawError::Config(format!("Failed to serialize request: {}", e)))?;

        let resp: BroadcastResponse = self.client.trade_post("/broadcast", body, false).await?;
        Ok(resp.tx_hash)
    }

    pub async fn create_proxy_order(
        &self,
        params: crate::types::ProxyOrderParams,
    ) -> Result<crate::types::Order, ZeroClawError> {
        self.client.require_plan(ApiPlan::Normal, "proxy wallet")?;

        let body = serde_json::to_value(&params)
            .map_err(|e| ZeroClawError::Config(format!("Failed to serialize order: {}", e)))?;

        self.client.trade_post("/proxy/order", body, true).await
    }

    pub async fn cancel_proxy_order(&self, order_id: &str) -> Result<(), ZeroClawError> {
        self.client.require_plan(ApiPlan::Normal, "proxy wallet")?;

        let path = format!("/proxy/order/{}", order_id);
        let (timestamp, signature) = self.client.sign_proxy_request("DELETE", &path, "");

        let url = format!("{}{}", TRADE_API_BASE_URL, path);
        let resp = self
            .client
            .http
            .delete(&url)
            .header(HEADER_TRADE_ACCESS_KEY, &self.client.config.api_key)
            .header(HEADER_TRADE_ACCESS_TIMESTAMP, &timestamp)
            .header(HEADER_TRADE_ACCESS_SIGN, &signature)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ZeroClawError::RateLimit);
        }

        let text = resp.text().await?;
        let parsed: crate::types::RawApiResponse = serde_json::from_str(&text)?;

        if parsed.status != 1 && parsed.status != 200 {
            return Err(ZeroClawError::Api {
                code: parsed.status,
                message: parsed.msg,
            });
        }

        Ok(())
    }

    pub async fn list_proxy_orders(
        &self,
        status: Option<crate::types::OrderStatus>,
    ) -> Result<Vec<crate::types::Order>, ZeroClawError> {
        self.client.require_plan(ApiPlan::Normal, "proxy wallet")?;

        let params: Vec<(String, String)> = match status {
            Some(s) => {
                let status_str = match s {
                    crate::types::OrderStatus::Pending => "Pending",
                    crate::types::OrderStatus::Filled => "Filled",
                    crate::types::OrderStatus::PartiallyFilled => "PartiallyFilled",
                    crate::types::OrderStatus::Cancelled => "Cancelled",
                    crate::types::OrderStatus::Failed => "Failed",
                };
                vec![("status".to_string(), status_str.to_string())]
            }
            None => vec![],
        };
        self.client.trade_get("/proxy/orders", &params, true).await
    }

    pub async fn get_proxy_order(
        &self,
        order_id: &str,
    ) -> Result<crate::types::Order, ZeroClawError> {
        self.client.require_plan(ApiPlan::Normal, "proxy wallet")?;

        let params = vec![("order_id".to_string(), order_id.to_string())];
        self.client.trade_get("/proxy/order", &params, true).await
    }
}

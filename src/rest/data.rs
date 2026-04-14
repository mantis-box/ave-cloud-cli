//! Data REST API v2 implementation
//!
//! This module implements the v2 Data REST API endpoints for Ave Cloud.
//! Reference: https://github.com/ave-cloud-skill/scripts/ave/data/

use crate::error::ZeroClawError;
use crate::rest::AveClient;
use crate::types::*;

// ============================================================
// DataApi v2 - Token Operations
// ============================================================

#[allow(unused)]
pub struct DataApiV2<'a> {
    client: &'a AveClient,
}

#[allow(unused)]
impl<'a> DataApiV2<'a> {
    pub fn new(client: &'a AveClient) -> Self {
        DataApiV2 { client }
    }

    /// Search tokens by keyword
    pub async fn search(
        &self,
        keyword: &str,
        chain: Option<&str>,
        limit: Option<u32>,
        orderby: Option<&str>,
    ) -> Result<Vec<TokenInfo>, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![("keyword".to_string(), keyword.to_string())];
        if let Some(c) = chain {
            params.push(("chain".to_string(), c.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit".to_string(), l.to_string()));
        }
        if let Some(o) = orderby {
            params.push(("orderby".to_string(), o.to_string()));
        }

        self.client.get_v2("/tokens", &params).await
    }

    /// Batch search token details by address-chain list
    pub async fn search_details(&self, tokens: Vec<&str>) -> Result<Vec<TokenInfo>, ZeroClawError> {
        let body = serde_json::json!({
            "token_ids": tokens
        });
        self.client.post_v2("/tokens/search", body).await
    }

    /// Get tokens by platform/launchpad tag
    pub async fn platform_tokens(
        &self,
        platform: &str,
        limit: Option<u32>,
        orderby: Option<&str>,
    ) -> Result<Vec<PlatformToken>, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![("tag".to_string(), platform.to_string())];
        if let Some(l) = limit {
            params.push(("limit".to_string(), l.to_string()));
        }
        if let Some(o) = orderby {
            params.push(("orderby".to_string(), o.to_string()));
        }

        self.client.get_v2("/tokens/platform", &params).await
    }

    /// Get token detail by address and chain
    pub async fn token(&self, chain: &str, address: &str) -> Result<TokenInfo, ZeroClawError> {
        let path = format!(
            "/tokens/{}-{}",
            address.to_lowercase(),
            chain.to_lowercase()
        );
        self.client.get_v2(&path, &[]).await
    }

    /// Batch get prices for up to 200 tokens
    pub async fn batch_price(
        &self,
        tokens: Vec<&str>,
        tvl_min: Option<f64>,
        volume_min: Option<f64>,
    ) -> Result<BatchPriceResponse, ZeroClawError> {
        let mut body = serde_json::json!({
            "token_ids": tokens
        });

        if let Some(tvl) = tvl_min {
            body["tvl_min"] = serde_json::json!(tvl);
        }
        if let Some(vol) = volume_min {
            body["tx_24h_volume_min"] = serde_json::json!(vol);
        }

        self.client.post_v2("/tokens/price", body).await
    }

    /// Get token holders with sort/order
    pub async fn holders(
        &self,
        chain: &str,
        address: &str,
        limit: Option<u32>,
        sort_by: Option<&str>,
        order: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![];
        if let Some(l) = limit {
            params.push(("limit".to_string(), l.to_string()));
        }
        if let Some(s) = sort_by {
            params.push(("sort_by".to_string(), s.to_string()));
        }
        if let Some(o) = order {
            params.push(("order".to_string(), o.to_string()));
        }

        let path = format!(
            "/tokens/holders/{}-{}",
            address.to_lowercase(),
            chain.to_lowercase()
        );
        self.client.get_v2(&path, &params).await
    }
}

// ============================================================
// DataApi v2 - Kline Operations
// ============================================================

#[allow(unused)]
impl<'a> DataApiV2<'a> {
    /// Get kline data by token address
    pub async fn kline_token(
        &self,
        chain: &str,
        address: &str,
        interval: u32,
        size: u32,
    ) -> Result<KlineResponse, ZeroClawError> {
        let path = format!("/klines/token/{}-{}", address, chain);
        let params: Vec<(String, String)> = vec![
            ("interval".to_string(), interval.to_string()),
            ("size".to_string(), size.to_string()),
        ];
        self.client.get_v2(&path, &params).await
    }

    /// Get kline data by pair address
    pub async fn kline_pair(
        &self,
        chain: &str,
        address: &str,
        interval: u32,
        size: u32,
    ) -> Result<KlineResponse, ZeroClawError> {
        let path = format!("/klines/pair/{}-{}", address, chain);
        let params: Vec<(String, String)> = vec![
            ("interval".to_string(), interval.to_string()),
            ("size".to_string(), size.to_string()),
        ];
        self.client.get_v2(&path, &params).await
    }

    /// Get Ondo-mapped kline data by pair address or ticker
    pub async fn kline_ondo(
        &self,
        pair: &str,
        interval: u32,
        size: u32,
        from_time: Option<i64>,
        to_time: Option<i64>,
    ) -> Result<KlineResponse, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![
            ("interval".to_string(), interval.to_string()),
            ("size".to_string(), size.to_string()),
        ];
        if let Some(t) = from_time {
            params.push(("from_time".to_string(), t.to_string()));
        }
        if let Some(t) = to_time {
            params.push(("to_time".to_string(), t.to_string()));
        }

        let path = format!("/klines/pair/ondo/{}", pair);
        self.client.get_v2(&path, &params).await
    }
}

// ============================================================
// DataApi v2 - Market Operations
// ============================================================

#[allow(unused)]
impl<'a> DataApiV2<'a> {
    /// Get trending tokens on a chain
    pub async fn trending(
        &self,
        chain: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<TokenInfo>, ZeroClawError> {
        let params: Vec<(String, String)> = vec![
            ("chain".to_string(), chain.to_string()),
            ("current_page".to_string(), page.to_string()),
            ("page_size".to_string(), page_size.to_string()),
        ];
        self.client.get_v2("/tokens/trending", &params).await
    }

    /// List available rank topics
    pub async fn rank_topics(&self) -> Result<RankTopics, ZeroClawError> {
        self.client.get_v2("/ranks/topics", &[]).await
    }

    /// Get token rankings by topic
    pub async fn ranks(&self, topic: &str) -> Result<Vec<RankedToken>, ZeroClawError> {
        let params: Vec<(String, String)> = vec![("topic".to_string(), topic.to_string())];
        self.client.get_v2("/ranks", &params).await
    }

    /// Get contract risk/security report
    pub async fn risk(&self, chain: &str, address: &str) -> Result<RiskScore, ZeroClawError> {
        let path = format!(
            "/contracts/{}-{}",
            address.to_lowercase(),
            chain.to_lowercase()
        );
        self.client.get_v2(&path, &[]).await
    }

    /// List all supported chains
    pub async fn chains(&self) -> Result<Vec<ChainInfo>, ZeroClawError> {
        self.client.get_v2("/supported_chains", &[]).await
    }

    /// Get main tokens for a chain
    pub async fn main_tokens(&self, chain: &str) -> Result<Vec<MainToken>, ZeroClawError> {
        let params: Vec<(String, String)> = vec![("chain".to_string(), chain.to_string())];
        self.client.get_v2("/tokens/main", &params).await
    }
}

// ============================================================
// DataApi v2 - Wallet Operations
// ============================================================

#[allow(unused)]
impl<'a> DataApiV2<'a> {
    /// Get wallet swap transaction history
    #[allow(clippy::too_many_arguments)]
    pub async fn address_txs(
        &self,
        wallet: &str,
        chain: &str,
        token: Option<&str>,
        from_time: Option<i64>,
        last_time: Option<&str>,
        last_id: Option<&str>,
        page_size: Option<u32>,
    ) -> Result<Vec<AddressTx>, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![
            ("wallet_address".to_string(), wallet.to_string()),
            ("chain".to_string(), chain.to_string()),
        ];
        if let Some(t) = token {
            params.push(("token_address".to_string(), t.to_string()));
        }
        if let Some(t) = from_time {
            params.push(("from_time".to_string(), t.to_string()));
        }
        if let Some(t) = last_time {
            params.push(("last_time".to_string(), t.to_string()));
        }
        if let Some(id) = last_id {
            params.push(("last_id".to_string(), id.to_string()));
        }
        if let Some(s) = page_size {
            params.push(("page_size".to_string(), s.to_string()));
        }

        self.client.get_v2("/address/tx", &params).await
    }

    /// Get wallet PnL for a specific token
    pub async fn address_pnl(
        &self,
        wallet: &str,
        chain: &str,
        token: &str,
    ) -> Result<AddressPnl, ZeroClawError> {
        let params: Vec<(String, String)> = vec![
            ("wallet_address".to_string(), wallet.to_string()),
            ("chain".to_string(), chain.to_string()),
            ("token_address".to_string(), token.to_string()),
        ];
        self.client.get_v2("/address/pnl", &params).await
    }

    /// Get token holdings for a wallet
    #[allow(clippy::too_many_arguments)]
    pub async fn wallet_tokens(
        &self,
        wallet: &str,
        chain: &str,
        sort: Option<&str>,
        sort_dir: Option<&str>,
        page_size: Option<u32>,
        page_no: Option<u32>,
        hide_sold: bool,
        hide_small: Option<f64>,
        blue_chips: bool,
    ) -> Result<Vec<WalletToken>, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![
            ("wallet_address".to_string(), wallet.to_string()),
            ("chain".to_string(), chain.to_string()),
        ];
        if let Some(s) = sort {
            params.push(("sort".to_string(), s.to_string()));
        }
        if let Some(d) = sort_dir {
            params.push(("sort_dir".to_string(), d.to_string()));
        }
        if let Some(s) = page_size {
            params.push(("pageSize".to_string(), s.to_string()));
        }
        if let Some(n) = page_no {
            params.push(("pageNO".to_string(), n.to_string()));
        }
        if hide_sold {
            params.push(("hide_sold".to_string(), "1".to_string()));
        }
        if let Some(h) = hide_small {
            params.push(("hide_small".to_string(), h.to_string()));
        }
        if blue_chips {
            params.push(("blue_chips".to_string(), "1".to_string()));
        }

        self.client
            .get_v2("/address/walletinfo/tokens", &params)
            .await
    }

    /// Get wallet overview and stats
    pub async fn wallet_info(
        &self,
        wallet: &str,
        chain: &str,
        self_address: Option<&str>,
    ) -> Result<WalletInfo, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![
            ("wallet_address".to_string(), wallet.to_string()),
            ("chain".to_string(), chain.to_string()),
        ];
        if let Some(a) = self_address {
            params.push(("self_address".to_string(), a.to_string()));
        }

        self.client.get_v2("/address/walletinfo", &params).await
    }

    /// List smart wallets with profit filters
    #[allow(clippy::too_many_arguments)]
    pub async fn smart_wallets(
        &self,
        chain: &str,
        keyword: Option<&str>,
        sort: Option<&str>,
        sort_dir: Option<&str>,
        profit_above_900_min: Option<f64>,
        profit_above_900_max: Option<f64>,
        profit_300_900_min: Option<f64>,
        profit_300_900_max: Option<f64>,
        profit_100_300_min: Option<f64>,
        profit_100_300_max: Option<f64>,
        profit_10_100_min: Option<f64>,
        profit_10_100_max: Option<f64>,
        profit_neg10_10_min: Option<f64>,
        profit_neg10_10_max: Option<f64>,
        profit_neg50_neg10_min: Option<f64>,
        profit_neg50_neg10_max: Option<f64>,
        profit_neg100_neg50_min: Option<f64>,
        profit_neg100_neg50_max: Option<f64>,
        last_trade_time_min: Option<f64>,
        last_trade_time_max: Option<f64>,
    ) -> Result<Vec<SmartWallet>, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![("chain".to_string(), chain.to_string())];

        if let Some(k) = keyword {
            params.push(("keyword".to_string(), k.to_string()));
        }
        if let Some(s) = sort {
            params.push(("sort".to_string(), s.to_string()));
        }
        if let Some(d) = sort_dir {
            params.push(("sort_dir".to_string(), d.to_string()));
        }

        // Add profit filters
        if let Some(v) = profit_above_900_min {
            params.push((
                "profit_above_900_percent_num_min".to_string(),
                v.to_string(),
            ));
        }
        if let Some(v) = profit_above_900_max {
            params.push((
                "profit_above_900_percent_num_max".to_string(),
                v.to_string(),
            ));
        }
        if let Some(v) = profit_300_900_min {
            params.push(("profit_300_900_percent_num_min".to_string(), v.to_string()));
        }
        if let Some(v) = profit_300_900_max {
            params.push(("profit_300_900_percent_num_max".to_string(), v.to_string()));
        }
        if let Some(v) = profit_100_300_min {
            params.push(("profit_100_300_percent_num_min".to_string(), v.to_string()));
        }
        if let Some(v) = profit_100_300_max {
            params.push(("profit_100_300_percent_num_max".to_string(), v.to_string()));
        }
        if let Some(v) = profit_10_100_min {
            params.push(("profit_10_100_percent_num_min".to_string(), v.to_string()));
        }
        if let Some(v) = profit_10_100_max {
            params.push(("profit_10_100_percent_num_max".to_string(), v.to_string()));
        }
        if let Some(v) = profit_neg10_10_min {
            params.push(("profit_neg10_10_percent_num_min".to_string(), v.to_string()));
        }
        if let Some(v) = profit_neg10_10_max {
            params.push(("profit_neg10_10_percent_num_max".to_string(), v.to_string()));
        }
        if let Some(v) = profit_neg50_neg10_min {
            params.push((
                "profit_neg50_neg10_percent_num_min".to_string(),
                v.to_string(),
            ));
        }
        if let Some(v) = profit_neg50_neg10_max {
            params.push((
                "profit_neg50_neg10_percent_num_max".to_string(),
                v.to_string(),
            ));
        }
        if let Some(v) = profit_neg100_neg50_min {
            params.push((
                "profit_neg100_neg50_percent_num_min".to_string(),
                v.to_string(),
            ));
        }
        if let Some(v) = profit_neg100_neg50_max {
            params.push((
                "profit_neg100_neg50_percent_num_max".to_string(),
                v.to_string(),
            ));
        }
        if let Some(v) = last_trade_time_min {
            params.push(("last_trade_time_min".to_string(), v.to_string()));
        }
        if let Some(v) = last_trade_time_max {
            params.push(("last_trade_time_max".to_string(), v.to_string()));
        }

        self.client
            .get_v2("/address/smart_wallet/list", &params)
            .await
    }
}

// ============================================================
// DataApi v2 - Transaction Operations
// ============================================================

#[allow(unused)]
impl<'a> DataApiV2<'a> {
    /// Get swap transactions for a pair
    pub async fn txs(&self, chain: &str, address: &str) -> Result<Vec<SwapTx>, ZeroClawError> {
        let path = format!("/txs/{}-{}", address.to_lowercase(), chain.to_lowercase());
        self.client.get_v2(&path, &[]).await
    }

    /// Get liquidity transactions for a pair
    #[allow(clippy::too_many_arguments)]
    pub async fn liq_txs(
        &self,
        chain: &str,
        address: &str,
        type_: Option<&str>,
        limit: Option<u32>,
        from_time: Option<i64>,
        to_time: Option<i64>,
        sort: Option<&str>,
    ) -> Result<Vec<LiqTx>, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![];
        if let Some(t) = type_ {
            params.push(("type".to_string(), t.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit".to_string(), l.to_string()));
        }
        if let Some(t) = from_time {
            params.push(("from_time".to_string(), t.to_string()));
        }
        if let Some(t) = to_time {
            params.push(("to_time".to_string(), t.to_string()));
        }
        if let Some(s) = sort {
            params.push(("sort".to_string(), s.to_string()));
        }

        let path = format!(
            "/txs/liq/{}-{}",
            address.to_lowercase(),
            chain.to_lowercase()
        );
        self.client.get_v2(&path, &params).await
    }

    /// Get transaction detail by hash
    pub async fn tx_detail(
        &self,
        chain: &str,
        account: &str,
        tx_hash: &str,
        start_from: Option<i64>,
        end_at: Option<i64>,
        limit: Option<u32>,
    ) -> Result<TxDetail, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![
            ("chain".to_string(), chain.to_string()),
            ("account_address".to_string(), account.to_string()),
            ("tx_hash".to_string(), tx_hash.to_string()),
        ];
        if let Some(t) = start_from {
            params.push(("start_from".to_string(), t.to_string()));
        }
        if let Some(t) = end_at {
            params.push(("end_at".to_string(), t.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit".to_string(), l.to_string()));
        }

        self.client.get_v2("/txs/detail", &params).await
    }

    /// Get trading pair detail
    pub async fn pair(&self, chain: &str, address: &str) -> Result<PairInfo, ZeroClawError> {
        let path = format!("/pairs/{}-{}", address.to_lowercase(), chain.to_lowercase());
        self.client.get_v2(&path, &[]).await
    }

    /// Get public trading signals
    pub async fn signals(
        &self,
        chain: Option<&str>,
        page_size: Option<u32>,
        page_no: Option<u32>,
    ) -> Result<Vec<Signal>, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![];
        if let Some(c) = chain {
            params.push(("chain".to_string(), c.to_string()));
        }
        if let Some(s) = page_size {
            params.push(("pageSize".to_string(), s.to_string()));
        }
        if let Some(n) = page_no {
            params.push(("pageNO".to_string(), n.to_string()));
        }

        self.client.get_v2("/signals/public/list", &params).await
    }
}

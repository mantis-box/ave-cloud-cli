//! Trade REST API implementation
//!
//! This module implements the Trade REST API endpoints for Ave Cloud Bot API.
//! Reference: https://github.com/ave-cloud-skill/scripts/ave/trade/

use crate::error::ZeroClawError;
use crate::rest::AveClient;
use crate::types::*;

// ============================================================
// TradeRestApi - Chain Wallet Operations (Phase 3)
// ============================================================

#[allow(unused)]
pub struct TradeRestApi<'a> {
    client: &'a AveClient,
}

#[allow(unused)]
impl<'a> TradeRestApi<'a> {
    pub fn new(client: &'a AveClient) -> Self {
        TradeRestApi { client }
    }

    /// Get swap quote (estimated output)
    pub async fn quote(
        &self,
        chain: &str,
        in_amount: &str,
        in_token: &str,
        out_token: &str,
        swap_type: &str,
    ) -> Result<ChainQuote, ZeroClawError> {
        let body = serde_json::json!({
            "chain": chain,
            "inAmount": in_amount,
            "inTokenAddress": in_token,
            "outTokenAddress": out_token,
            "swapType": swap_type,
        });
        self.client
            .trade_post("/v1/thirdParty/chainWallet/getAmountOut", body, false)
            .await
    }

    /// Query recommended slippage for a token
    pub async fn auto_slippage(
        &self,
        chain: &str,
        token: &str,
        use_mev: bool,
    ) -> Result<AutoSlippageResponse, ZeroClawError> {
        let body = serde_json::json!({
            "chain": chain,
            "tokenAddress": token,
            "useMev": use_mev,
        });
        self.client
            .trade_post("/v1/thirdParty/chainWallet/getAutoSlippage", body, false)
            .await
    }

    /// Query recommended gas prices per chain
    pub async fn gas_tip(&self) -> Result<GasTipResponse, ZeroClawError> {
        self.client
            .trade_get("/v1/thirdParty/chainWallet/getGasTip", &[], false)
            .await
    }

    /// Create unsigned EVM swap transaction
    #[allow(clippy::too_many_arguments)]
    pub async fn create_evm_tx(
        &self,
        chain: &str,
        creator_address: &str,
        in_amount: &str,
        in_token: &str,
        out_token: &str,
        swap_type: &str,
        slippage: &str,
        fee_recipient: Option<&str>,
        fee_recipient_rate: Option<&str>,
        auto_slippage: bool,
    ) -> Result<EvmTxResponse, ZeroClawError> {
        let mut body = serde_json::json!({
            "chain": chain,
            "creatorAddress": creator_address,
            "inAmount": in_amount,
            "inTokenAddress": in_token,
            "outTokenAddress": out_token,
            "swapType": swap_type,
            "slippage": slippage,
        });

        if let Some(r) = fee_recipient {
            body["feeRecipient"] = serde_json::json!(r);
        }
        if let Some(r) = fee_recipient_rate {
            body["feeRecipientRate"] = serde_json::json!(r);
        }
        if auto_slippage {
            body["autoSlippage"] = serde_json::json!(true);
        }

        self.client
            .trade_post("/v1/thirdParty/chainWallet/createEvmTx", body, false)
            .await
    }

    /// Submit signed EVM transaction
    pub async fn send_evm_tx(
        &self,
        chain: &str,
        request_tx_id: &str,
        signed_tx: &str,
        use_mev: bool,
    ) -> Result<BroadcastResponse, ZeroClawError> {
        let mut body = serde_json::json!({
            "chain": chain,
            "requestTxId": request_tx_id,
            "signedTx": signed_tx,
        });

        if use_mev {
            body["useMev"] = serde_json::json!(true);
        }

        self.client
            .trade_post("/v1/thirdParty/chainWallet/sendSignedEvmTx", body, false)
            .await
    }

    /// Create unsigned Solana swap transaction
    #[allow(clippy::too_many_arguments)]
    pub async fn create_solana_tx(
        &self,
        creator_address: &str,
        in_amount: &str,
        in_token: &str,
        out_token: &str,
        swap_type: &str,
        slippage: &str,
        fee: &str,
        use_mev: bool,
        fee_recipient: Option<&str>,
        fee_recipient_rate: Option<&str>,
        auto_slippage: bool,
    ) -> Result<SolanaTxResponse, ZeroClawError> {
        let mut body = serde_json::json!({
            "creatorAddress": creator_address,
            "inAmount": in_amount,
            "inTokenAddress": in_token,
            "outTokenAddress": out_token,
            "swapType": swap_type,
            "slippage": slippage,
            "fee": fee,
        });

        if use_mev {
            body["useMev"] = serde_json::json!(true);
        }
        if let Some(r) = fee_recipient {
            body["feeRecipient"] = serde_json::json!(r);
        }
        if let Some(r) = fee_recipient_rate {
            body["feeRecipientRate"] = serde_json::json!(r);
        }
        if auto_slippage {
            body["autoSlippage"] = serde_json::json!(true);
        }

        self.client
            .trade_post("/v1/thirdParty/chainWallet/createSolanaTx", body, false)
            .await
    }

    /// Submit signed Solana transaction
    pub async fn send_solana_tx(
        &self,
        request_tx_id: &str,
        signed_tx: &str,
        use_mev: bool,
    ) -> Result<BroadcastResponse, ZeroClawError> {
        let mut body = serde_json::json!({
            "requestTxId": request_tx_id,
            "signedTx": signed_tx,
        });

        if use_mev {
            body["useMev"] = serde_json::json!(true);
        }

        self.client
            .trade_post("/v1/thirdParty/chainWallet/sendSignedSolanaTx", body, false)
            .await
    }
}

// ============================================================
// ProxyWalletApi - Proxy Wallet Operations (Phase 4)
// ============================================================

#[allow(unused)]
pub struct ProxyWalletApi<'a> {
    client: &'a AveClient,
}

#[allow(unused)]
impl<'a> ProxyWalletApi<'a> {
    pub fn new(client: &'a AveClient) -> Self {
        ProxyWalletApi { client }
    }

    /// List proxy wallets
    pub async fn list_wallets(
        &self,
        assets_ids: Option<&str>,
    ) -> Result<Vec<ProxyWallet>, ZeroClawError> {
        let params: Vec<(String, String)> = if let Some(ids) = assets_ids {
            vec![("assetsIds".to_string(), ids.to_string())]
        } else {
            vec![]
        };

        self.client
            .trade_get("/v1/thirdParty/user/getUserByAssetsId", &params, true)
            .await
    }

    /// Create a delegate proxy wallet
    pub async fn create_wallet(
        &self,
        name: &str,
        return_mnemonic: bool,
    ) -> Result<ProxyWallet, ZeroClawError> {
        let mut body = serde_json::json!({
            "assetsName": name,
        });

        if return_mnemonic {
            body["returnMnemonic"] = serde_json::json!(true);
        }

        self.client
            .trade_post("/v1/thirdParty/user/generateWallet", body, true)
            .await
    }

    /// Delete delegate proxy wallets
    pub async fn delete_wallet(
        &self,
        assets_ids: Vec<&str>,
    ) -> Result<serde_json::Value, ZeroClawError> {
        let body = serde_json::json!({
            "assetsIds": assets_ids,
        });

        self.client
            .trade_post("/v1/thirdParty/user/deleteWallet", body, true)
            .await
    }

    /// Place a market swap order
    #[allow(clippy::too_many_arguments)]
    pub async fn market_order(
        &self,
        chain: &str,
        assets_id: &str,
        in_token: &str,
        out_token: &str,
        in_amount: &str,
        swap_type: &str,
        slippage: &str,
        use_mev: bool,
        gas: Option<&str>,
        extra_gas: Option<&str>,
        auto_slippage: bool,
        auto_gas: Option<&str>,
        auto_sell_config: Option<Vec<serde_json::Value>>,
    ) -> Result<MarketOrderResponse, ZeroClawError> {
        let mut body = serde_json::json!({
            "chain": chain,
            "assetsId": assets_id,
            "inTokenAddress": in_token,
            "outTokenAddress": out_token,
            "inAmount": in_amount,
            "swapType": swap_type,
            "slippage": slippage,
            "useMev": use_mev,
        });

        if let Some(g) = gas {
            body["gas"] = serde_json::json!(g);
        }
        if let Some(g) = extra_gas {
            body["extraGas"] = serde_json::json!(g);
        }
        if auto_slippage {
            body["autoSlippage"] = serde_json::json!(true);
        }
        if let Some(a) = auto_gas {
            body["autoGas"] = serde_json::json!(a);
        }
        if let Some(c) = auto_sell_config {
            body["autoSellConfig"] = serde_json::json!(c);
        }

        self.client
            .trade_post("/v1/thirdParty/tx/sendSwapOrder", body, true)
            .await
    }

    /// Place a limit order
    #[allow(clippy::too_many_arguments)]
    pub async fn limit_order(
        &self,
        chain: &str,
        assets_id: &str,
        in_token: &str,
        out_token: &str,
        in_amount: &str,
        swap_type: &str,
        slippage: &str,
        limit_price: &str,
        use_mev: bool,
        gas: Option<&str>,
        extra_gas: Option<&str>,
        expire_time: Option<i64>,
        auto_slippage: bool,
        auto_gas: Option<&str>,
    ) -> Result<LimitOrderResponse, ZeroClawError> {
        let mut body = serde_json::json!({
            "chain": chain,
            "assetsId": assets_id,
            "inTokenAddress": in_token,
            "outTokenAddress": out_token,
            "inAmount": in_amount,
            "swapType": swap_type,
            "slippage": slippage,
            "limitPrice": limit_price,
            "useMev": use_mev,
        });

        if let Some(g) = gas {
            body["gas"] = serde_json::json!(g);
        }
        if let Some(g) = extra_gas {
            body["extraGas"] = serde_json::json!(g);
        }
        if let Some(e) = expire_time {
            body["expireTime"] = serde_json::json!(e);
        }
        if auto_slippage {
            body["autoSlippage"] = serde_json::json!(true);
        }
        if let Some(a) = auto_gas {
            body["autoGas"] = serde_json::json!(a);
        }

        self.client
            .trade_post("/v1/thirdParty/tx/sendLimitOrder", body, true)
            .await
    }

    /// Query market swap orders by IDs
    pub async fn get_swap_orders(
        &self,
        chain: &str,
        ids: &str,
    ) -> Result<SwapOrdersResponse, ZeroClawError> {
        let params: Vec<(String, String)> = vec![
            ("chain".to_string(), chain.to_string()),
            ("ids".to_string(), ids.to_string()),
        ];

        self.client
            .trade_get("/v1/thirdParty/tx/getSwapOrder", &params, true)
            .await
    }

    /// Query limit orders (paginated)
    pub async fn get_limit_orders(
        &self,
        chain: &str,
        assets_id: &str,
        page_size: u32,
        page_no: u32,
        status: Option<&str>,
        token: Option<&str>,
    ) -> Result<LimitOrdersResponse, ZeroClawError> {
        let mut params: Vec<(String, String)> = vec![
            ("chain".to_string(), chain.to_string()),
            ("assetsId".to_string(), assets_id.to_string()),
            ("pageSize".to_string(), page_size.to_string()),
            ("pageNo".to_string(), page_no.to_string()),
        ];

        if let Some(s) = status {
            params.push(("status".to_string(), s.to_string()));
        }
        if let Some(t) = token {
            params.push(("token".to_string(), t.to_string()));
        }

        self.client
            .trade_get("/v1/thirdParty/tx/getLimitOrder", &params, true)
            .await
    }

    /// Cancel pending limit orders
    pub async fn cancel_limit_order(
        &self,
        chain: &str,
        ids: Vec<&str>,
    ) -> Result<serde_json::Value, ZeroClawError> {
        let body = serde_json::json!({
            "chain": chain,
            "ids": ids,
        });

        self.client
            .trade_post("/v1/thirdParty/tx/cancelLimitOrder", body, true)
            .await
    }

    /// Approve token for EVM proxy wallet trading
    pub async fn approve_token(
        &self,
        chain: &str,
        assets_id: &str,
        token_address: &str,
    ) -> Result<ApprovalResponse, ZeroClawError> {
        let body = serde_json::json!({
            "chain": chain,
            "assetsId": assets_id,
            "tokenAddress": token_address,
        });

        self.client
            .trade_post("/v1/thirdParty/tx/approve", body, true)
            .await
    }

    /// Query token approval status
    pub async fn get_approval(
        &self,
        chain: &str,
        ids: &str,
    ) -> Result<Vec<ApprovalResponse>, ZeroClawError> {
        let params: Vec<(String, String)> = vec![
            ("chain".to_string(), chain.to_string()),
            ("ids".to_string(), ids.to_string()),
        ];

        self.client
            .trade_get("/v1/thirdParty/tx/getApprove", &params, true)
            .await
    }

    /// Transfer tokens from a delegate proxy wallet
    #[allow(clippy::too_many_arguments)]
    pub async fn transfer(
        &self,
        chain: &str,
        assets_id: &str,
        from_address: &str,
        to_address: &str,
        token_address: &str,
        amount: &str,
        gas: Option<&str>,
        extra_gas: Option<&str>,
    ) -> Result<TransferResponse, ZeroClawError> {
        let mut body = serde_json::json!({
            "chain": chain,
            "assetsId": assets_id,
            "fromAddress": from_address,
            "toAddress": to_address,
            "tokenAddress": token_address,
            "amount": amount,
        });

        if let Some(g) = gas {
            body["gas"] = serde_json::json!(g);
        }
        if let Some(g) = extra_gas {
            body["extraGas"] = serde_json::json!(g);
        }

        self.client
            .trade_post("/v1/thirdParty/tx/transfer", body, true)
            .await
    }

    /// Query transfer status
    pub async fn get_transfer(
        &self,
        chain: &str,
        ids: &str,
    ) -> Result<Vec<TransferResponse>, ZeroClawError> {
        let params: Vec<(String, String)> = vec![
            ("chain".to_string(), chain.to_string()),
            ("ids".to_string(), ids.to_string()),
        ];

        self.client
            .trade_get("/v1/thirdParty/tx/getTransfer", &params, true)
            .await
    }
}

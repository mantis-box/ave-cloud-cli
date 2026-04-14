use std::env;
use std::str::FromStr;

use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiPlan {
    Free,
    Normal,
    Pro,
}

impl ApiPlan {
    pub fn rpm(&self) -> u32 {
        match self {
            ApiPlan::Free => 1,
            ApiPlan::Normal => 5,
            ApiPlan::Pro => 20,
        }
    }

    pub fn supports_websocket(&self) -> bool {
        matches!(self, ApiPlan::Pro)
    }

    pub fn supports_proxy_wallet(&self) -> bool {
        matches!(self, ApiPlan::Normal | ApiPlan::Pro)
    }
}

impl FromStr for ApiPlan {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(ApiPlan::Free),
            "normal" => Ok(ApiPlan::Normal),
            "pro" => Ok(ApiPlan::Pro),
            _ => Err(format!(
                "Invalid API plan: {}. Valid values: free, normal, pro",
                s
            )),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub api_key: String,
    pub api_plan: ApiPlan,
    pub secret_key: Option<String>,
    pub evm_private_key: Option<String>,
    pub solana_private_key: Option<String>,
    pub mnemonic: Option<String>,
    pub bsc_rpc_url: String,
    pub eth_rpc_url: String,
    pub base_rpc_url: String,
}

// ============================================================
// Chain Constants (aligned with Python reference)
// ============================================================

/// Chain IDs for EVM chains
#[allow(dead_code)]
pub const CHAIN_ID_BSC: u64 = 56;
#[allow(dead_code)]
pub const CHAIN_ID_ETH: u64 = 1;
#[allow(dead_code)]
pub const CHAIN_ID_BASE: u64 = 8453;

/// Supported EVM chains
#[allow(dead_code)]
pub const EVM_CHAINS: &[&str] = &["bsc", "eth", "base"];

/// All supported chains
#[allow(dead_code)]
pub const ALL_CHAINS: &[&str] = &["bsc", "eth", "base", "solana"];

/// Valid WebSocket intervals
#[allow(dead_code)]
pub const VALID_WSS_INTERVALS: &[&str] = &[
    "s1", "k1", "k5", "k15", "k30", "k60", "k120", "k240", "k1440", "k10080",
];

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let api_key = env::var("AVE_API_KEY").map_err(|_| {
            "AVE_API_KEY environment variable is required. Get your API key from https://cloud.ave.ai"
        })?;

        let api_plan_str = env::var("API_PLAN").unwrap_or_else(|_| "free".to_string());
        let api_plan = ApiPlan::from_str(&api_plan_str)?;

        let secret_key = env::var("AVE_SECRET_KEY").ok();
        let evm_private_key = env::var("AVE_EVM_PRIVATE_KEY").ok();
        let solana_private_key = env::var("AVE_SOLANA_PRIVATE_KEY").ok();
        let mnemonic = env::var("AVE_MNEMONIC").ok();

        let bsc_rpc_url = env::var("AVE_BSC_RPC_URL")
            .unwrap_or_else(|_| "https://bsc.publicnode.com".to_string());
        let eth_rpc_url = env::var("AVE_ETH_RPC_URL")
            .unwrap_or_else(|_| "https://ethereum.publicnode.com".to_string());
        let base_rpc_url = env::var("AVE_BASE_RPC_URL")
            .unwrap_or_else(|_| "https://base.publicnode.com".to_string());

        debug!("Config loaded");

        Ok(Config {
            api_key,
            api_plan,
            secret_key,
            evm_private_key,
            solana_private_key,
            mnemonic,
            bsc_rpc_url,
            eth_rpc_url,
            base_rpc_url,
        })
    }

    pub fn can_trade_proxy(&self) -> bool {
        self.api_plan.supports_proxy_wallet()
    }

    pub fn can_stream(&self) -> bool {
        self.api_plan.supports_websocket()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_plan_rpm() {
        assert_eq!(ApiPlan::Free.rpm(), 1);
        assert_eq!(ApiPlan::Normal.rpm(), 5);
        assert_eq!(ApiPlan::Pro.rpm(), 20);
    }

    #[test]
    fn test_api_plan_supports_websocket() {
        assert!(!ApiPlan::Free.supports_websocket());
        assert!(!ApiPlan::Normal.supports_websocket());
        assert!(ApiPlan::Pro.supports_websocket());
    }

    #[test]
    fn test_api_plan_supports_proxy_wallet() {
        assert!(!ApiPlan::Free.supports_proxy_wallet());
        assert!(ApiPlan::Normal.supports_proxy_wallet());
        assert!(ApiPlan::Pro.supports_proxy_wallet());
    }

    #[test]
    fn test_api_plan_from_str() {
        assert_eq!(ApiPlan::from_str("free").unwrap(), ApiPlan::Free);
        assert_eq!(ApiPlan::from_str("FREE").unwrap(), ApiPlan::Free);
        assert_eq!(ApiPlan::from_str("Normal").unwrap(), ApiPlan::Normal);
        assert_eq!(ApiPlan::from_str("pro").unwrap(), ApiPlan::Pro);
        assert!(ApiPlan::from_str("invalid").is_err());
    }
}

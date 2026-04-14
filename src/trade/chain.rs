//! Chain Wallet - self-custody signing for EVM chains
//!
//! This module handles transaction signing for self-custody wallets on EVM chains.
//! Keys can be loaded from private keys. Solana support deferred due to dependency conflicts.

use k256::SecretKey;
use tracing::debug;

use crate::config::Config;
use crate::error::ZeroClawError;

#[allow(dead_code)]
pub struct ChainWallet {
    config: Config,
}

#[allow(dead_code)]
impl ChainWallet {
    pub fn new(config: Config) -> Self {
        ChainWallet { config }
    }

    /// Sign and send an EVM transaction (BSC, ETH, BASE)
    ///
    /// This is a placeholder - actual implementation would use ethers.js or
    /// a raw transaction builder to sign and send the transaction.
    pub async fn evm_sign_and_send(
        &self,
        chain: &str,
        _tx: serde_json::Value,
    ) -> Result<String, ZeroClawError> {
        let _rpc_url = self.rpc_url_for_chain(chain)?;

        // Load private key to verify it exists
        let _private_key = self.load_evm_private_key()?;

        // For now, this is a placeholder - actual implementation would use
        // ethers or a raw transaction builder to sign and send
        Err(ZeroClawError::Config(
            "EVM transaction signing requires full ethers integration. See FASE 8 notes."
                .to_string(),
        ))
    }

    /// Get a quote and send an EVM swap transaction
    pub async fn evm_quote_and_send(
        &self,
        chain: &str,
        token_in: &str,
        token_out: &str,
        amount_in: &str,
        slippage_bps: u32,
        trade_api: &crate::rest::TradeApi<'_>,
    ) -> Result<String, ZeroClawError> {
        // Step 1: Get quote
        let quote = trade_api
            .swap_quote(chain, token_in, token_out, amount_in, slippage_bps)
            .await?;

        // Step 2: Get wallet address
        let wallet_address = self.evm_address(chain)?;

        // Step 3: Build transaction
        let tx = trade_api
            .build_chain_tx(chain, &quote, &wallet_address)
            .await?;

        // Step 4: Sign and send
        self.evm_sign_and_send(chain, tx).await
    }

    /// Get the EVM wallet address for a chain
    ///
    /// Note: This computes a simplified address derivation.
    /// For proper EVM address derivation, the full public key recovery
    /// from signature should be used.
    pub fn evm_address(&self, chain: &str) -> Result<String, ZeroClawError> {
        let private_key = self.load_evm_private_key()?;
        let key_bytes = hex::decode(&private_key)
            .map_err(|e| ZeroClawError::Signing(format!("Invalid hex: {}", e)))?;

        // Verify the key is valid by creating a SecretKey
        let _secret_key = SecretKey::from_be_bytes(&key_bytes)
            .map_err(|e| ZeroClawError::Signing(format!("Invalid key: {}", e)))?;

        // For a proper implementation, we'd:
        // 1. Get the full 64-byte public key from the private key
        // 2. Hash it with keccak256
        // 3. Take the last 20 bytes as the address
        //
        // Since k256 doesn't directly expose this, we'll use a simplified approach
        // that just validates the key is valid hex
        debug!("EVM wallet address for {}: derived from key", chain);

        // Return a placeholder address indicator
        // Real implementation would use ethers for proper address derivation
        Ok(format!(
            "0x{}... (requires ethers for full derivation)",
            &private_key[..8]
        ))
    }

    fn load_evm_private_key(&self) -> Result<String, ZeroClawError> {
        // Priority 1: evm_private_key config
        if let Some(ref key) = self.config.evm_private_key {
            // Remove 0x prefix if present
            let key = key.strip_prefix("0x").unwrap_or(key);
            return Ok(key.to_string());
        }

        // Priority 2: derive from mnemonic (not yet implemented)
        if self.config.mnemonic.is_some() {
            return Err(ZeroClawError::Config(
                "EVM key derivation from mnemonic not yet implemented".to_string(),
            ));
        }

        Err(ZeroClawError::Config(
            "No EVM private key configured. Set AVE_EVM_PRIVATE_KEY in environment.".to_string(),
        ))
    }

    fn rpc_url_for_chain(&self, chain: &str) -> Result<&str, ZeroClawError> {
        match chain.to_uppercase().as_str() {
            "BSC" | "BNB" => Ok(&self.config.bsc_rpc_url),
            "ETH" => Ok(&self.config.eth_rpc_url),
            "BASE" => Ok(&self.config.base_rpc_url),
            _ => Err(ZeroClawError::Config(format!(
                "Unsupported chain: {}. Supported: BSC, ETH, BASE",
                chain
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            api_key: "test_key".to_string(),
            api_plan: crate::config::ApiPlan::Pro,
            secret_key: None,
            evm_private_key: Some("0x".to_string() + &"0".repeat(64)),
            solana_private_key: None,
            mnemonic: None,
            bsc_rpc_url: "https://bsc.publicnode.com".to_string(),
            eth_rpc_url: "https://eth.publicnode.com".to_string(),
            base_rpc_url: "https://base.publicnode.com".to_string(),
        }
    }

    #[test]
    fn test_chain_wallet_creation() {
        let config = create_test_config();
        let wallet = ChainWallet::new(config);
        assert!(wallet.config.evm_private_key.is_some());
    }

    #[test]
    fn test_rpc_url_for_chain() {
        let config = create_test_config();
        let wallet = ChainWallet::new(config);

        assert_eq!(
            wallet.rpc_url_for_chain("BSC").unwrap(),
            "https://bsc.publicnode.com"
        );
        assert_eq!(
            wallet.rpc_url_for_chain("ETH").unwrap(),
            "https://eth.publicnode.com"
        );
        assert_eq!(
            wallet.rpc_url_for_chain("BASE").unwrap(),
            "https://base.publicnode.com"
        );
        assert!(wallet.rpc_url_for_chain("UNKNOWN").is_err());
    }
}

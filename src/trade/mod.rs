// Trade module
// - chain.rs: ChainWallet for self-custody signing (EVM only, Solana deferred)
// - proxy.rs: ProxyWallet for server-managed trading

#[allow(unused)]
pub mod chain;
pub mod proxy;

#[allow(unused)]
pub use chain::ChainWallet;
pub use proxy::ProxyWallet;

use clap::Parser;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Solana Token Holder Monitoring Bot
/// Monitors token holder count changes in real-time
#[derive(Parser, Debug)]
#[command(name = "solana-holder-bot")]
#[command(about = "Monitor Solana token holders in real-time", long_about = None)]
pub struct Cli {
    /// Token mint address to monitor
    #[arg(value_name = "MINT_ADDRESS")]
    pub mint_address: String,

    /// RPC endpoint URL
    #[arg(long = "rpc-url", default_value = "https://api.mainnet-beta.solana.com")]
    pub rpc_url: String,

    /// Polling interval in seconds
    #[arg(long = "interval", default_value = "30")]
    pub interval: u64,

    /// Enable JSON logging output
    #[arg(long = "json-log")]
    pub json_log: bool,

    /// Maximum number of RPC retries
    #[arg(long = "max-retries", default_value = "3")]
    pub max_retries: u32,

    /// RPC request timeout in seconds
    #[arg(long = "timeout", default_value = "30")]
    pub timeout: u64,

    /// Enable API server
    #[arg(long = "api")]
    pub api_server: bool,

    /// API server port
    #[arg(long = "api-port", default_value = "56789")]
    pub api_port: u16,

    /// Cache TTL in seconds for API
    #[arg(long = "cache-ttl", default_value = "30")]
    pub cache_ttl: u64,
}

impl Cli {
    /// Parse and validate mint address
    pub fn parse_mint(&self) -> anyhow::Result<Pubkey> {
        Pubkey::from_str(&self.mint_address)
            .map_err(|e| anyhow::anyhow!("Invalid mint address '{}': {}", self.mint_address, e))
    }

    /// Validate CLI arguments
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.interval == 0 {
            return Err(anyhow::anyhow!("Interval must be greater than 0"));
        }
        if self.max_retries == 0 {
            return Err(anyhow::anyhow!("Max retries must be greater than 0"));
        }
        Ok(())
    }
}


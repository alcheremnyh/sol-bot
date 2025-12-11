use anyhow::{Context, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, RpcFilterType};
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// RPC client wrapper with retry logic and health checks
pub struct SolanaRpcClient {
    client: RpcClient,
    max_retries: u32,
    timeout: Duration,
}

impl SolanaRpcClient {
    /// Create new RPC client
    pub fn new(rpc_url: String, max_retries: u32, timeout_secs: u64) -> Self {
        let client = RpcClient::new_with_commitment(
            rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );
        
        info!("Initialized RPC client: {}", rpc_url);
        
        Self {
            client,
            max_retries,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Check RPC connection health
    pub async fn health_check(&self) -> Result<()> {
        self.client
            .get_slot()
            .await
            .context("RPC health check failed")?;
        Ok(())
    }

    /// Get token accounts by mint with retry logic
    pub async fn get_token_accounts_by_mint(
        &self,
        mint: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>> {
        let mut last_error = None;
        
        for attempt in 0..self.max_retries {
            match self._get_token_accounts_by_mint(mint).await {
                Ok(accounts) => {
                    if attempt > 0 {
                        info!("Successfully retrieved accounts after {} retries", attempt);
                    }
                    return Ok(accounts);
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    last_error = Some(e);
                    warn!(
                        "RPC request failed (attempt {}/{}): {}",
                        attempt + 1,
                        self.max_retries,
                        error_msg
                    );
                    if attempt < self.max_retries - 1 {
                        let delay = Self::exponential_backoff(attempt);
                        warn!("Retrying in {:?}...", delay);
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap().context("Failed to get token accounts after all retries"))
    }

    /// Internal method to fetch token accounts with pagination
    async fn _get_token_accounts_by_mint(
        &self,
        mint: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>> {
        // Try getProgramAccounts first (works on private RPCs)
        match self._try_get_program_accounts(mint).await {
            Ok(accounts) if !accounts.is_empty() => {
                info!("Successfully fetched {} accounts using getProgramAccounts", accounts.len());
                return Ok(accounts);
            }
            Ok(_) => {
                warn!("getProgramAccounts returned empty result");
            }
            Err(e) => {
                let error_str = format!("{}", e);
                // Check if it's the known public RPC limitation
                if error_str.contains("excluded from account secondary indexes") 
                    || error_str.contains("this RPC method unavailable") {
                    return Err(anyhow::anyhow!(
                        "Public RPC endpoint '{}' does not support getProgramAccounts for Token Program.\n\
                        This is a known limitation of public RPC endpoints.\n\n\
                        SOLUTIONS:\n\
                        1. Use a private RPC endpoint:\n\
                           - Helius: https://mainnet.helius-rpc.com/?api-key=YOUR_KEY\n\
                           - QuickNode: https://your-endpoint.solana-mainnet.quiknode.pro/YOUR_KEY/\n\
                           - Alchemy: https://solana-mainnet.g.alchemy.com/v2/YOUR_KEY\n\
                        2. Or try alternative public RPCs:\n\
                           - https://rpc.ankr.com/solana\n\
                           - https://solana-api.projectserum.com\n\n\
                        Example: cargo run -- {} --rpc-url https://rpc.ankr.com/solana --interval 30",
                        self.client.url(),
                        mint
                    ));
                }
                warn!("getProgramAccounts failed: {}", e);
            }
        }

        // If we get here, return error - we can't use alternative methods reliably
        Err(anyhow::anyhow!(
            "Unable to fetch token accounts. Please use a private RPC endpoint that supports getProgramAccounts."
        ))
    }

    /// Try to get accounts using getProgramAccounts with optimized filters
    async fn _try_get_program_accounts(
        &self,
        mint: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>> {
        let token_program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
            .context("Failed to parse Token Program ID")?;

        let mint_bytes = mint.as_ref();
        
        // Use DataSize filter (165 bytes = standard SPL Token account size)
        // and Memcmp filter for mint address at offset 0
        let filters = vec![
            RpcFilterType::DataSize(165),
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(0, mint_bytes.to_vec())),
        ];

        let config = RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                commitment: Some(CommitmentConfig::confirmed()),
                data_slice: None, // Load full data to parse amount
                min_context_slot: None,
            },
            with_context: None,
            sort_results: None,
        };

        // Rate limiting: small delay before request
        sleep(Duration::from_millis(100)).await;

        debug!("Fetching token accounts for mint: {}", mint);
        debug!("Using token program ID: {}", token_program_id);
        debug!("RPC URL: {}", self.client.url());

        let accounts = self
            .client
            .get_program_accounts_with_config(&token_program_id, config)
            .await
            .with_context(|| {
                format!(
                    "Failed to fetch program accounts from RPC {} for mint {}",
                    self.client.url(),
                    mint
                )
            })?;

        debug!("Fetched {} accounts from RPC", accounts.len());

        let mut all_accounts = Vec::new();

        // Convert UiAccount to Account format
        for (pubkey, account_data) in accounts {
            // account_data.data is already Vec<u8> when using Base64 encoding
            // The RPC client automatically decodes it
            let data = account_data.data.clone();

            let account = Account {
                lamports: account_data.lamports,
                data,
                owner: account_data.owner,
                executable: account_data.executable,
                rent_epoch: account_data.rent_epoch,
            };
            all_accounts.push((pubkey, account));
        }

        info!("Total token accounts found: {}", all_accounts.len());
        Ok(all_accounts)
    }


    /// Exponential backoff delay
    fn exponential_backoff(attempt: u32) -> Duration {
        let base_delay_ms = 1000u64;
        let delay_ms = base_delay_ms * 2u64.pow(attempt);
        Duration::from_millis(delay_ms.min(10000)) // Cap at 10 seconds
    }

    /// Get RPC URL
    pub fn rpc_url(&self) -> String {
        self.client.url().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires RPC connection
    async fn test_health_check() {
        let client = SolanaRpcClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
            3,
            30,
        );
        let result = client.health_check().await;
        assert!(result.is_ok());
    }
}


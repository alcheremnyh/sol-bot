use anyhow::{Context, Result};
use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use solana_program::pubkey::Pubkey;
use crate::rpc_client::SolanaRpcClient;
use crate::token_monitor::extract_holders;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{error, info};

/// Cache entry for holder count
#[derive(Debug, Clone)]
pub struct HolderCacheEntry {
    count: usize,
    timestamp: u64,
    #[allow(dead_code)]
    mint: Pubkey,
}

/// Cache for holder counts with automatic refresh
pub struct HolderCache {
    cache: Arc<RwLock<HashMap<String, HolderCacheEntry>>>,
    rpc_client: Arc<SolanaRpcClient>,
    refresh_interval: Duration,
}

impl HolderCache {
    pub fn new(rpc_client: Arc<SolanaRpcClient>, refresh_interval_secs: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            rpc_client,
            refresh_interval: Duration::from_secs(refresh_interval_secs),
        }
    }

    /// Start background task to refresh cache
    pub fn start_refresh_task(&self) {
        let cache = self.cache.clone();
        let rpc_client = self.rpc_client.clone();
        let interval_duration = self.refresh_interval;
        let mut mints_to_refresh = Vec::new();

        tokio::spawn(async move {
            let mut refresh_timer = interval(interval_duration);

            loop {
                refresh_timer.tick().await;

                // Collect all mints that need refresh
                {
                    let cache_read = cache.read().await;
                    let keys: Vec<String> = cache_read.keys().cloned().collect();
                    mints_to_refresh = keys;
                }

                // Refresh each mint
                for mint_str in &mints_to_refresh {
                    match Self::fetch_holder_count(&rpc_client, mint_str).await {
                        Ok(count) => {
                            let mint = match Pubkey::from_str(mint_str) {
                                Ok(m) => m,
                                Err(_) => continue,
                            };

                            let entry = HolderCacheEntry {
                                count,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                mint,
                            };

                            let mut cache_write = cache.write().await;
                            cache_write.insert(mint_str.clone(), entry);
                            info!("Refreshed cache for mint {}: {} holders", mint_str, count);
                        }
                        Err(e) => {
                            error!("Failed to refresh cache for {}: {}", mint_str, e);
                        }
                    }
                }
            }
        });
    }

    /// Get holder count from cache or fetch if not cached
    pub async fn get_holder_count(&self, mint_str: &str) -> Result<HolderCacheEntry> {
        // Check cache first
        {
            let cache_read = self.cache.read().await;
            if let Some(entry) = cache_read.get(mint_str) {
                return Ok(entry.clone());
            }
        }

        // Not in cache, fetch it
        info!("Cache miss for {}, fetching...", mint_str);
        let count = Self::fetch_holder_count(&self.rpc_client, mint_str).await?;
        let mint = Pubkey::from_str(mint_str)
            .context("Invalid mint address")?;

        let entry = HolderCacheEntry {
            count,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            mint,
        };

        // Store in cache
        {
            let mut cache_write = self.cache.write().await;
            cache_write.insert(mint_str.to_string(), entry.clone());
        }

        Ok(entry)
    }

    /// Fetch holder count from RPC
    async fn fetch_holder_count(
        rpc_client: &SolanaRpcClient,
        mint_str: &str,
    ) -> Result<usize> {
        let mint = Pubkey::from_str(mint_str)
            .context("Invalid mint address")?;

        let accounts = rpc_client
            .get_token_accounts_by_mint(&mint)
            .await
            .context("Failed to fetch token accounts")?;

        let holders = extract_holders(&accounts)
            .context("Failed to extract holders")?;

        Ok(holders.len())
    }
}

/// API response structure
#[derive(serde::Serialize)]
struct HolderResponse {
    mint: String,
    holders: usize,
    timestamp: u64,
    cached: bool,
}

/// Get holder count endpoint
async fn get_holders(
    Path(mint_str): Path<String>,
    axum::extract::State(cache): axum::extract::State<Arc<HolderCache>>,
) -> Result<Json<HolderResponse>, StatusCode> {
    // Validate mint address format
    if Pubkey::from_str(&mint_str).is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    match cache.get_holder_count(&mint_str).await {
        Ok(entry) => Ok(Json(HolderResponse {
            mint: mint_str,
            holders: entry.count,
            timestamp: entry.timestamp,
            cached: true, // Always cached after first fetch
        })),
        Err(e) => {
            error!("Error getting holder count: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "solana-holder-bot-api"
    }))
}

/// Create API router
pub fn create_api_router(cache: Arc<HolderCache>) -> Router {
    Router::new()
        .route("/holders/:mint", get(get_holders))
        .route("/health", get(health_check))
        .with_state(cache)
        .layer(tower_http::cors::CorsLayer::permissive())
}

/// Start API server
pub async fn start_api_server(
    cache: Arc<HolderCache>,
    port: u16,
) -> Result<()> {
    let app = create_api_router(cache);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .context("Failed to bind to port")?;

    info!("API server started on http://0.0.0.0:{}", port);
    info!("Endpoints:");
    info!("  GET /holders/:mint - Get holder count for token");
    info!("  GET /health - Health check");

    axum::serve(listener, app)
        .await
        .context("API server error")?;

    Ok(())
}


use anyhow::{Context, Result};
use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::{extract_holders, SolanaRpcClient};

/// Cached holder count result
#[derive(Debug, Clone)]
struct CachedResult {
    count: usize,
    timestamp: Instant,
}

/// Cache entry with TTL
#[derive(Debug)]
struct CacheEntry {
    result: CachedResult,
    ttl: Duration,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.result.timestamp.elapsed() >= self.ttl
    }

    fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// Cache for holder counts
type HolderCache = Arc<RwLock<HashMap<String, CacheEntry>>>;

/// API server state
#[derive(Clone)]
pub struct ApiState {
    rpc_client: Arc<SolanaRpcClient>,
    cache: HolderCache,
    cache_ttl: Duration,
}

/// Response structure for holder count API
#[derive(Serialize, Deserialize)]
pub struct HolderCountResponse {
    pub mint: String,
    pub holders: usize,
    pub cached: bool,
    pub timestamp: u64,
}

impl ApiState {
    pub fn new(rpc_client: Arc<SolanaRpcClient>, cache_ttl_secs: u64) -> Self {
        Self {
            rpc_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
        }
    }

    /// Get holder count for a mint (with caching)
    async fn get_holder_count(&self, mint: &str) -> Result<HolderCountResponse> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(mint) {
                if entry.is_valid() {
                    info!("Cache hit for mint: {}", mint);
                    return Ok(HolderCountResponse {
                        mint: mint.to_string(),
                        holders: entry.result.count,
                        cached: true,
                        timestamp: entry.result.timestamp.elapsed().as_secs(),
                    });
                }
            }
        }

        // Cache miss or expired - fetch from RPC
        info!("Cache miss for mint: {}, fetching from RPC...", mint);
        let count = self.fetch_holder_count(mint).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                mint.to_string(),
                CacheEntry {
                    result: CachedResult {
                        count,
                        timestamp: Instant::now(),
                    },
                    ttl: self.cache_ttl,
                },
            );
        }

        Ok(HolderCountResponse {
            mint: mint.to_string(),
            holders: count,
            cached: false,
            timestamp: 0,
        })
    }

    /// Fetch holder count from RPC
    async fn fetch_holder_count(&self, mint_str: &str) -> Result<usize> {
        let mint = Pubkey::from_str(mint_str)
            .context(format!("Invalid mint address: {}", mint_str))?;

        // Fetch token accounts
        let accounts = self
            .rpc_client
            .get_token_accounts_by_mint(&mint)
            .await
            .context("Failed to fetch token accounts")?;

        // Extract unique holders
        let holders = extract_holders(&accounts)
            .context("Failed to extract holders from accounts")?;

        Ok(holders.len())
    }

    /// Background task to refresh cache periodically
    pub async fn start_cache_refresher(&self, mints: Vec<String>) {
        let state = self.clone();
        tokio::spawn(async move {
            loop {
                info!("Starting cache refresh cycle for {} mints", mints.len());
                
                for mint in &mints {
                    match state.fetch_holder_count(mint).await {
                        Ok(count) => {
                            let mut cache = state.cache.write().await;
                            cache.insert(
                                mint.clone(),
                                CacheEntry {
                                    result: CachedResult {
                                        count,
                                        timestamp: Instant::now(),
                                    },
                                    ttl: state.cache_ttl,
                                },
                            );
                            info!("Refreshed cache for {}: {} holders", mint, count);
                        }
                        Err(e) => {
                            error!("Failed to refresh cache for {}: {}", mint, e);
                        }
                    }
                    
                    // Small delay between requests
                    sleep(Duration::from_millis(500)).await;
                }

                // Wait for cache TTL before next refresh
                sleep(state.cache_ttl).await;
            }
        });
    }
}

/// GET /holders/:mint - Get holder count for a token
async fn get_holders_handler(
    Path(mint): Path<String>,
    axum::extract::State(state): axum::extract::State<ApiState>,
) -> Result<Json<HolderCountResponse>, (StatusCode, String)> {
    match state.get_holder_count(&mint).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Error getting holder count for {}: {}", mint, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get holder count: {}", e),
            ))
        }
    }
}

/// GET /health - Health check endpoint
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "solana-holder-bot-api"
    }))
}

/// Create and configure the API router
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/holders/:mint", get(get_holders_handler))
        .route("/health", get(health_handler))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}

/// Start the API server
pub async fn start_api_server(
    rpc_client: Arc<SolanaRpcClient>,
    port: u16,
    cache_ttl_secs: u64,
) -> Result<()> {
    let state = ApiState::new(rpc_client, cache_ttl_secs);
    
    let app = create_router(state.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .context(format!("Failed to bind to port {}", port))?;

    info!("🚀 API server started on http://0.0.0.0:{}", port);
    info!("📊 Endpoints:");
    info!("   GET /holders/:mint - Get holder count for a token");
    info!("   GET /health - Health check");

    axum::serve(listener, app)
        .await
        .context("API server error")?;

    Ok(())
}


use anyhow::{Context, Result};
use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use solana_program::pubkey::Pubkey;
use crate::rpc_client::SolanaRpcClient;
use crate::token_monitor::extract_holders;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

/// Cache entry for holder count
#[derive(Debug, Clone)]
pub struct HolderCacheEntry {
    count: usize,
    timestamp: u64,
    #[allow(dead_code)]
    mint: Pubkey,
    request_count: u64,  // Количество запросов для этого токена
    first_seen: u64,      // Когда токен был впервые запрошен
}

/// Cache for holder counts with automatic refresh
/// Limited to 2 tokens maximum - oldest token is removed when adding a third
pub struct HolderCache {
    cache: Arc<RwLock<HashMap<String, HolderCacheEntry>>>,
    rpc_client: Arc<SolanaRpcClient>,
    refresh_interval: Duration,
    max_tokens: usize,  // Максимальное количество токенов в кэше
    api_timeout: Duration,  // Таймаут для API запросов (короче чем RPC timeout)
}

impl HolderCache {
    pub fn new(rpc_client: Arc<SolanaRpcClient>, refresh_interval_secs: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            rpc_client,
            refresh_interval: Duration::from_secs(refresh_interval_secs),
            max_tokens: 2,  // Ограничение: максимум 2 токена
            api_timeout: Duration::from_secs(5),  // API таймаут: 30 секунд (быстрее чем RPC timeout)
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
                    mints_to_refresh = cache_read.keys().cloned().collect();
                }

                // Refresh each mint
                for mint_str in &mints_to_refresh {
                    // Use longer timeout for background refresh (no user waiting)
                    let refresh_timeout = Duration::from_secs(90);
                    match Self::fetch_holder_count(&rpc_client, mint_str, refresh_timeout).await {
                        Ok(count) => {
                            let mint = match Pubkey::from_str(mint_str) {
                                Ok(m) => m,
                                Err(_) => continue,
                            };

                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            
                            // Сохраняем существующие данные если есть
                            let (request_count, first_seen) = {
                                let cache_read = cache.read().await;
                                if let Some(existing) = cache_read.get(mint_str) {
                                    (existing.request_count, existing.first_seen)
                                } else {
                                    (0, now)
                                }
                            };

                            let entry = HolderCacheEntry {
                                count,
                                timestamp: now,
                                mint,
                                request_count,
                                first_seen,
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check cache first
        {
            let mut cache_write = self.cache.write().await;
            if let Some(entry) = cache_write.get_mut(mint_str) {
                // Увеличиваем счетчик запросов
                entry.request_count += 1;
                info!("Cache hit for {} (request #{}), returning cached data", mint_str, entry.request_count);
                return Ok(entry.clone());
            }
        }

        // Not in cache, fetch it
        info!("Cache miss for {}, fetching from RPC...", mint_str);
        let fetch_start = std::time::Instant::now();
        let count = match Self::fetch_holder_count(&self.rpc_client, mint_str, self.api_timeout).await {
            Ok(count) => count,
            Err(e) => {
                let elapsed = fetch_start.elapsed();
                warn!("Failed to fetch holders for {} after {:.2}s: {}", mint_str, elapsed.as_secs_f64(), e);
                return Err(e);
            }
        };
        let fetch_elapsed = fetch_start.elapsed();
        info!("Fetched holders for {} in {:.2}s: {} holders", mint_str, fetch_elapsed.as_secs_f64(), count);
        let mint = Pubkey::from_str(mint_str)
            .context("Invalid mint address")?;

        let entry = HolderCacheEntry {
            count,
            timestamp: now,
            mint,
            request_count: 1,  // Первый запрос
            first_seen: now,   // Впервые запрошен сейчас
        };

        // Store in cache (with limit of 2 tokens)
        {
            let mut cache_write = self.cache.write().await;
            
            // Если кэш полон и добавляется новый токен, удаляем самый старый
            if cache_write.len() >= self.max_tokens && !cache_write.contains_key(mint_str) {
                // Находим токен с самым старым timestamp (первый добавленный)
                let oldest_mint = cache_write
                    .iter()
                    .min_by_key(|(_, entry)| entry.timestamp)
                    .map(|(mint, _)| mint.clone());
                
                if let Some(old_mint) = oldest_mint {
                    cache_write.remove(&old_mint);
                    info!("Removed oldest token {} from cache (limit: {} tokens)", old_mint, self.max_tokens);
                }
            }
            
            cache_write.insert(mint_str.to_string(), entry.clone());
            info!("Added {} to cache (total tracked tokens: {}/{})", mint_str, cache_write.len(), self.max_tokens);
        }

        Ok(entry)
    }

    /// Get list of all tracked tokens with statistics
    pub async fn get_tracked_tokens(&self) -> Vec<TokenStats> {
        let cache_read = self.cache.read().await;
        cache_read
            .iter()
            .map(|(mint, entry)| TokenStats {
                mint: mint.clone(),
                holders: entry.count,
                last_updated: entry.timestamp,
                request_count: entry.request_count,
                first_seen: entry.first_seen,
            })
            .collect()
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache_read = self.cache.read().await;
        let total_tokens = cache_read.len();
        let total_requests: u64 = cache_read.values().map(|e| e.request_count).sum();
        
        CacheStats {
            total_tracked_tokens: total_tokens,
            total_requests,
            cache_size_bytes: std::mem::size_of_val(&*cache_read) as u64,
        }
    }

    /// Fetch holder count from RPC with timeout
    async fn fetch_holder_count(
        rpc_client: &SolanaRpcClient,
        mint_str: &str,
    ) -> Result<usize> {
        let mint = Pubkey::from_str(mint_str)
            .context("Invalid mint address")?;

        // Apply API-level timeout (45 seconds max for API requests)
        // This is shorter than RPC timeout to fail fast for API users
        let api_timeout = Duration::from_secs(45);
        let fetch_result = tokio::time::timeout(
            api_timeout,
            rpc_client.get_token_accounts_by_mint(&mint)
        ).await;

        let accounts = match fetch_result {
            Ok(Ok(accounts)) => accounts,
            Ok(Err(e)) => {
                return Err(e).context("Failed to fetch token accounts");
            }
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "RPC request timed out after {} seconds. Please try again later or use a faster RPC endpoint.",
                    api_timeout.as_secs()
                ));
            }
        };

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
        Ok(entry) => {
            // Проверяем, был ли это кэш или новый запрос
            let was_cached = entry.request_count > 1;
            Ok(Json(HolderResponse {
                mint: mint_str,
                holders: entry.count,
                timestamp: entry.timestamp,
                cached: was_cached,
            }))
        },
        Err(e) => {
            error!("Error getting holder count for {}: {}", mint_str, e);
            // Return more specific error for timeout
            let error_msg = format!("{}", e);
            if error_msg.contains("timed out") {
                return Err(StatusCode::GATEWAY_TIMEOUT);
            }
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

/// Statistics for a tracked token
#[derive(Debug, Clone, Serialize)]
pub struct TokenStats {
    pub mint: String,
    pub holders: usize,
    pub last_updated: u64,
    pub request_count: u64,
    pub first_seen: u64,
}

/// Cache statistics
#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub total_tracked_tokens: usize,
    pub total_requests: u64,
    pub cache_size_bytes: u64,
}

/// Get list of all tracked tokens
async fn get_tracked_tokens(
    axum::extract::State(cache): axum::extract::State<Arc<HolderCache>>,
) -> Json<Vec<TokenStats>> {
    let tokens = cache.get_tracked_tokens().await;
    Json(tokens)
}

/// Get cache statistics
async fn get_cache_stats(
    axum::extract::State(cache): axum::extract::State<Arc<HolderCache>>,
) -> Json<CacheStats> {
    let stats = cache.get_cache_stats().await;
    Json(stats)
}

/// Create API router
pub fn create_api_router(cache: Arc<HolderCache>) -> Router {
    Router::new()
        .route("/holders/:mint", get(get_holders))
        .route("/health", get(health_check))
        .route("/tokens", get(get_tracked_tokens))
        .route("/stats", get(get_cache_stats))
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
    info!("  GET /tokens - Get list of all tracked tokens");
    info!("  GET /stats - Get cache statistics");

    axum::serve(listener, app)
        .await
        .context("API server error")?;

    Ok(())
}


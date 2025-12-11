use anyhow::{Context, Result};
use clap::Parser;
use solana_holder_bot::{
    api::HolderCache,
    check_alerts, calculate_stats, extract_holders, format_timestamp, Cli, Metrics,
    SolanaRpcClient,
};
use solana_sdk::pubkey::Pubkey;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::time::{interval, Duration};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Validate CLI arguments
    cli.validate().context("Invalid CLI arguments")?;

    // Initialize logging
    if cli.json_log {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    // Parse mint address
    let mint = cli.parse_mint().context("Failed to parse mint address")?;
    info!("Monitoring token: {}", mint);

    // Initialize RPC client
    let rpc_client = Arc::new(SolanaRpcClient::new(
        cli.rpc_url.clone(),
        cli.max_retries,
        cli.timeout,
    ));

    // Health check
    info!("Performing RPC health check...");
    rpc_client
        .health_check()
        .await
        .context("RPC health check failed. Please check your RPC URL")?;
    info!("RPC connection healthy");

    // Start API server if enabled
    if cli.api_server {
        let cache = Arc::new(HolderCache::new(rpc_client.clone(), cli.cache_ttl));
        cache.start_refresh_task();
        
        let api_port = cli.api_port;
        tokio::spawn(async move {
            if let Err(e) = solana_holder_bot::api::start_api_server(cache, api_port).await {
                error!("API server error: {}", e);
            }
        });
        info!("🚀 API server enabled on port {} (cache refresh: {}s)", api_port, cli.cache_ttl);
    }

    // Graceful shutdown handling
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Received shutdown signal (Ctrl+C)");
                shutdown_clone.store(true, Ordering::SeqCst);
            }
            Err(err) => {
                error!("Failed to listen for shutdown signal: {}", err);
            }
        }
    });

    // Monitoring loop
    let mut metrics = Metrics::new();
    let mut previous_count: Option<usize> = None;
    let poll_interval = Duration::from_secs(cli.interval);
    let mut interval_timer = interval(poll_interval);

    info!(
        "Starting monitoring loop (interval: {}s, RPC: {})",
        cli.interval, cli.rpc_url
    );
    info!("Press Ctrl+C to stop and view metrics");

    // Initial poll
    interval_timer.tick().await;

    loop {
        if shutdown.load(Ordering::SeqCst) {
            info!("Shutdown signal received, stopping monitoring...");
            break;
        }

        match monitor_holders(&rpc_client, &mint, previous_count, &mut metrics).await {
            Ok(count) => {
                previous_count = Some(count);
            }
            Err(e) => {
                error!("Error during monitoring cycle: {}", e);
                // Print error chain for debugging
                let mut source = e.source();
                while let Some(err) = source {
                    error!("  Caused by: {}", err);
                    source = err.source();
                }
                // Continue monitoring even on errors
            }
        }

        // Wait for next interval
        interval_timer.tick().await;
    }

    // Print final metrics
    print_final_metrics(&metrics, &mint);

    Ok(())
}

/// Monitor token holders for one cycle
async fn monitor_holders(
    rpc_client: &SolanaRpcClient,
    mint: &Pubkey,
    previous_count: Option<usize>,
    metrics: &mut Metrics,
) -> Result<usize> {
    let start_time = std::time::Instant::now();

    // Fetch token accounts
    let fetch_start = std::time::Instant::now();
    let accounts = rpc_client
        .get_token_accounts_by_mint(mint)
        .await
        .context("Failed to fetch token accounts")?;
    let fetch_elapsed = fetch_start.elapsed();

    // Extract unique holders
    let extract_start = std::time::Instant::now();
    let holders = extract_holders(&accounts)
        .context("Failed to extract holders from accounts")?;
    let extract_elapsed = extract_start.elapsed();

    let holder_count = holders.len();
    let elapsed = start_time.elapsed();
    
    // Log detailed timing if request took too long
    if elapsed.as_secs() > 10 {
        warn!(
            "Slow request detected: total={:.2}s, fetch={:.2}s, extract={:.2}s, accounts={}",
            elapsed.as_secs_f64(),
            fetch_elapsed.as_secs_f64(),
            extract_elapsed.as_secs_f64(),
            accounts.len()
        );
    }

    // Calculate statistics
    let stats = calculate_stats(holder_count, previous_count);

    // Update metrics
    metrics.update(holder_count);

    // Check for alerts
    check_alerts(&stats, previous_count, metrics);

    // Print status
    print_status(mint, &stats, elapsed);

    Ok(holder_count)
}

/// Print current status to console
fn print_status(mint: &Pubkey, stats: &solana_holder_bot::HolderStats, elapsed: std::time::Duration) {
    let change_str = if stats.change == 0 {
        "±0".to_string()
    } else if stats.change > 0 {
        format!("+{}", stats.change)
    } else {
        stats.change.to_string()
    };

    let change_percent_str = if stats.change_percent == 0.0 {
        "".to_string()
    } else {
        format!(" ({:+.1}%)", stats.change_percent)
    };

    let timestamp_str = format_timestamp(stats.timestamp);

    println!(
        "MINT: {} | Holders: {} | Δ: {}{} | Time: {} | Fetch: {:.2}s",
        mint,
        stats.count,
        change_str,
        change_percent_str,
        timestamp_str,
        elapsed.as_secs_f64()
    );
}

/// Print final metrics on shutdown
fn print_final_metrics(metrics: &Metrics, mint: &Pubkey) {
    let separator = "=".repeat(80);
    println!("\n{}", separator);
    println!("📊 FINAL METRICS for {}", mint);
    println!("{}", separator);
    println!("Total polls: {}", metrics.total_polls);
    
    if let Some(min) = metrics.min_holders {
        println!("Min holders: {}", min);
    }
    
    if let Some(max) = metrics.max_holders {
        println!("Max holders: {}", max);
    }
    
    println!("Average holders: {:.2}", metrics.average_holders());
    
    if !metrics.alerts.is_empty() {
        println!("\n🚨 ALERTS TRIGGERED:");
        for alert in &metrics.alerts {
            println!("  - {}", alert);
        }
    }
    
    println!("{}", separator);
}


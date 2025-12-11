use anyhow::Result;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use std::collections::HashSet;
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// Token holder monitoring statistics
#[derive(Debug, Clone)]
pub struct HolderStats {
    pub count: usize,
    pub timestamp: u64,
    pub change: i64,
    pub change_percent: f64,
}

/// Metrics tracker for holder monitoring
#[derive(Debug, Default)]
pub struct Metrics {
    pub min_holders: Option<usize>,
    pub max_holders: Option<usize>,
    pub total_polls: usize,
    pub total_holders_sum: usize,
    pub alerts: Vec<String>,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, holder_count: usize) {
        self.total_polls += 1;
        self.total_holders_sum += holder_count;

        if self.min_holders.is_none() || holder_count < self.min_holders.unwrap() {
            self.min_holders = Some(holder_count);
        }

        if self.max_holders.is_none() || holder_count > self.max_holders.unwrap() {
            self.max_holders = Some(holder_count);
        }
    }

    pub fn average_holders(&self) -> f64 {
        if self.total_polls == 0 {
            0.0
        } else {
            self.total_holders_sum as f64 / self.total_polls as f64
        }
    }

    pub fn add_alert(&mut self, message: String) {
        warn!("ALERT: {}", message);
        self.alerts.push(message);
    }
}

/// Extract unique token holders from token accounts
pub fn extract_holders(accounts: &[(Pubkey, Account)]) -> Result<HashSet<Pubkey>> {
    let mut holders = HashSet::new();
    let mut zero_balance_count = 0;

    for (token_account_pubkey, account) in accounts {
        // Parse token account data
        // TokenAccount structure: mint(32) + owner(32) + amount(8) + ...
        // Amount is at offset 64, 8 bytes (u64 little-endian)
        if account.data.len() < 72 {
            debug!(
                "Token account {} has invalid data length: {}",
                token_account_pubkey,
                account.data.len()
            );
            continue;
        }

        // Parse amount directly from bytes (faster than unpacking full struct)
        let amount_bytes: [u8; 8] = account.data[64..72]
            .try_into()
            .unwrap_or([0; 8]);
        let amount = u64::from_le_bytes(amount_bytes);

        if amount > 0 {
            // Parse owner from bytes (offset 32, 32 bytes)
            if account.data.len() >= 64 {
                let owner_bytes: [u8; 32] = account.data[32..64]
                    .try_into()
                    .unwrap_or([0; 32]);
                let owner = Pubkey::try_from(owner_bytes.as_ref())
                    .unwrap_or_else(|_| {
                        debug!("Invalid owner bytes in account {}", token_account_pubkey);
                        Pubkey::default()
                    });
                
                if owner != Pubkey::default() {
                    holders.insert(owner);
                    debug!(
                        "Found holder: {} with balance: {}",
                        owner, amount
                    );
                }
            }
        } else {
            zero_balance_count += 1;
        }
    }

    info!(
        "Extracted {} unique holders ({} zero-balance accounts filtered)",
        holders.len(),
        zero_balance_count
    );

    Ok(holders)
}

/// Calculate holder statistics
pub fn calculate_stats(
    current_count: usize,
    previous_count: Option<usize>,
) -> HolderStats {
    let timestamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (change, change_percent) = if let Some(prev) = previous_count {
        let diff = current_count as i64 - prev as i64;
        let percent = if prev > 0 {
            (diff as f64 / prev as f64) * 100.0
        } else {
            if current_count > 0 {
                100.0
            } else {
                0.0
            }
        };
        (diff, percent)
    } else {
        (0, 0.0)
    };

    HolderStats {
        count: current_count,
        timestamp,
        change,
        change_percent,
    }
}

/// Check for significant changes and generate alerts
pub fn check_alerts(
    stats: &HolderStats,
    previous_count: Option<usize>,
    metrics: &mut Metrics,
) {
    if let Some(prev) = previous_count {
        // +50% growth alert
        if stats.change_percent >= 50.0 {
            let message = format!(
                "🚀 SIGNIFICANT GROWTH: +{} holders (+{:.1}%) | {} -> {}",
                stats.change, stats.change_percent, prev, stats.count
            );
            metrics.add_alert(message);
        }

        // -20% drop alert
        if stats.change_percent <= -20.0 {
            let message = format!(
                "⚠️ SIGNIFICANT DROP: {} holders ({:.1}%) | {} -> {}",
                stats.change, stats.change_percent, prev, stats.count
            );
            metrics.add_alert(message);
        }
    }
}

/// Format timestamp for display
pub fn format_timestamp(secs: u64) -> String {
    let datetime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs);
    let datetime: chrono::DateTime<chrono::Utc> = datetime.into();
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_stats() {
        let stats = calculate_stats(100, Some(80));
        assert_eq!(stats.count, 100);
        assert_eq!(stats.change, 20);
        assert!((stats.change_percent - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_check_alerts_growth() {
        let mut metrics = Metrics::new();
        let stats = HolderStats {
            count: 150,
            timestamp: 0,
            change: 50,
            change_percent: 50.0,
        };
        check_alerts(&stats, Some(100), &mut metrics);
        assert_eq!(metrics.alerts.len(), 1);
        assert!(metrics.alerts[0].contains("GROWTH"));
    }

    #[test]
    fn test_check_alerts_drop() {
        let mut metrics = Metrics::new();
        let stats = HolderStats {
            count: 80,
            timestamp: 0,
            change: -20,
            change_percent: -20.0,
        };
        check_alerts(&stats, Some(100), &mut metrics);
        assert_eq!(metrics.alerts.len(), 1);
        assert!(metrics.alerts[0].contains("DROP"));
    }
}


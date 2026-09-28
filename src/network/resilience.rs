//! Reconnect bookkeeping for long-lived connections.
//!
//! The share loop calls this on every relay failure, and the view loop
//! counts attempts. It exists because retry policy was previously a
//! module that nothing in the product ever called, holding two
//! configuration flags (`jitter_buffer_size`, `error_correction_enabled`)
//! that no code read and that no FEC implementation backed. Those knobs
//! are gone: an unused flag that claims a feature is worse than no flag.

use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ResilienceConfig {
    /// Attempts to make before giving up. `0` means "keep trying", which
    /// is what a long-lived sharer wants from a relay that may restart.
    pub max_retries: u32,
    /// Base delay between attempts; doubled per attempt up to 4x.
    pub retry_delay: Duration,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            retry_delay: Duration::from_millis(100),
        }
    }
}

#[derive(Debug)]
pub struct NetworkResilience {
    config: ResilienceConfig,
    state: Mutex<State>,
}

#[derive(Debug, Default)]
struct State {
    consecutive_failures: u32,
    total_failures: u64,
    last_success: Option<Instant>,
    backoff: Duration,
}

impl NetworkResilience {
    pub fn new(config: ResilienceConfig) -> Self {
        let backoff = config.retry_delay;
        Self {
            config,
            state: Mutex::new(State {
                backoff,
                ..State::default()
            }),
        }
    }

    /// Record a failed attempt. Returns true when the budget is exhausted
    /// and the caller should stop trying.
    pub async fn note_failure(&self) -> bool {
        let mut state = self.state.lock().await;
        state.consecutive_failures += 1;
        state.total_failures += 1;
        state.last_success = None;
        state.backoff = (state.backoff * 2).min(self.config.retry_delay * 4);
        self.config.max_retries > 0 && state.consecutive_failures >= self.config.max_retries
    }

    /// Record a success and reset the backoff.
    pub async fn note_success(&self) {
        let mut state = self.state.lock().await;
        state.consecutive_failures = 0;
        state.last_success = Some(Instant::now());
        state.backoff = self.config.retry_delay;
    }

    /// How long to wait before the next attempt.
    pub async fn next_delay(&self) -> Duration {
        self.state.lock().await.backoff
    }

    /// True while the connection has not exhausted its retry budget.
    pub async fn is_healthy(&self) -> bool {
        let state = self.state.lock().await;
        self.config.max_retries == 0 || state.consecutive_failures < self.config.max_retries
    }

    pub async fn consecutive_failures(&self) -> u32 {
        self.state.lock().await.consecutive_failures
    }

    pub async fn total_failures(&self) -> u64 {
        self.state.lock().await.total_failures
    }

    /// Run an async operation, retrying it on failure with exponential
    /// backoff. An async closure, because retrying a blocking function on
    /// an async runtime is the bug the old signature invited.
    pub async fn retry_async<F, Fut, T>(&self, mut operation: F) -> anyhow::Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<T>>,
    {
        loop {
            match operation().await {
                Ok(value) => {
                    self.note_success().await;
                    return Ok(value);
                }
                Err(e) => {
                    if self.note_failure().await {
                        return Err(e.context(format!(
                            "failed after {} attempts",
                            self.state.lock().await.consecutive_failures
                        )));
                    }
                    tracing::warn!(
                        "Operation failed ({e}); retrying in {:?}",
                        self.next_delay().await
                    );
                    tokio::time::sleep(self.next_delay().await).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn a_failing_operation_is_retried_then_reported() {
        let resilience = NetworkResilience::new(ResilienceConfig {
            max_retries: 3,
            retry_delay: Duration::from_millis(1),
        });
        let calls = Arc::new(AtomicU32::new(0));
        let result: anyhow::Result<()> = resilience
            .retry_async(|| {
                let seen = calls.clone();
                async move {
                    let n = seen.fetch_add(1, Ordering::SeqCst);
                    if n < 2 {
                        Err(anyhow::anyhow!("transient"))
                    } else {
                        Ok(())
                    }
                }
            })
            .await;
        assert!(result.is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        assert!(resilience.is_healthy().await);
    }

    #[tokio::test]
    async fn the_budget_is_reported_when_it_is_exhausted() {
        let resilience = NetworkResilience::new(ResilienceConfig {
            max_retries: 2,
            retry_delay: Duration::from_millis(1),
        });
        let err = resilience
            .retry_async(|| async { Err::<(), _>(anyhow::anyhow!("permanent")) })
            .await
            .unwrap_err();
        let chain = format!("{err:#}");
        assert!(chain.contains("permanent"), "unhelpful: {chain}");
        assert!(chain.contains("2 attempts"), "unhelpful: {chain}");
        assert_eq!(resilience.consecutive_failures().await, 2);
        assert!(!resilience.is_healthy().await);
    }

    #[tokio::test]
    async fn an_unlimited_budget_never_gives_up() {
        let resilience = NetworkResilience::new(ResilienceConfig {
            max_retries: 0,
            retry_delay: Duration::from_millis(1),
        });
        for _ in 0..10 {
            assert!(!resilience.note_failure().await);
        }
        assert!(resilience.is_healthy().await);
    }
}

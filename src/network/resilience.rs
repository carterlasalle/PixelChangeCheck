use anyhow::Result;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};
use tracing::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

// Retry configuration
const MAX_RETRIES: u32 = 3;
const BASE_BACKOFF: Duration = Duration::from_millis(100);
const MAX_BACKOFF: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub jitter_buffer_size: usize,
    pub error_correction_enabled: bool,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            jitter_buffer_size: 5,
            error_correction_enabled: true,
        }
    }
}

#[derive(Debug)]
pub struct NetworkResilience {
    config: ResilienceConfig,
    retry_count: Arc<Mutex<u32>>,
    last_success: Arc<Mutex<Option<std::time::SystemTime>>>,
}

impl NetworkResilience {
    pub fn new(config: ResilienceConfig) -> Self {
        Self {
            config,
            retry_count: Arc::new(Mutex::new(0)),
            last_success: Arc::new(Mutex::new(None)),
        }
    }

    // Execute an operation with retry logic
    pub async fn with_retry<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Result<T> + Send + Sync,
    {
        let mut current_retry = 0;
        let mut backoff = self.config.retry_delay;

        loop {
            match operation() {
                Ok(result) => {
                    self.record_success().await;
                    return Ok(result);
                }
                Err(e) => {
                    current_retry += 1;
                    if current_retry >= self.config.max_retries {
                        error!("Operation failed after {} retries: {}", current_retry, e);
                        return Err(e);
                    }

                    warn!("Operation failed, retrying in {:?}: {}", backoff, e);
                    time::sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, self.config.retry_delay * 2);
                }
            }
        }
    }

    // Monitor connection health
    pub async fn monitor_connection<F>(&self, health_check: F) -> Result<()>
    where
        F: Fn() -> Result<bool> + Send + Sync + 'static,
    {
        let retry_count = self.retry_count.clone();
        let last_success = self.last_success.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;

                match health_check() {
                    Ok(true) => {
                        *retry_count.lock().await = 0;
                        *last_success.lock().await = Some(std::time::SystemTime::now());
                    }
                    Ok(false) => {
                        warn!("Health check failed");
                        *retry_count.lock().await += 1;
                    }
                    Err(e) => {
                        error!("Health check error: {}", e);
                        *retry_count.lock().await += 1;
                    }
                }

                // Check if connection is considered failed
                if *retry_count.lock().await >= config.max_retries {
                    error!("Connection considered failed after {} retries", config.max_retries);
                    break;
                }
            }
        });

        Ok(())
    }

    // Record successful operation
    async fn record_success(&self) {
        *self.retry_count.lock().await = 0;
        *self.last_success.lock().await = Some(std::time::SystemTime::now());
    }

    // Check if connection is healthy
    pub async fn is_healthy(&self) -> bool {
        let retry_count = *self.retry_count.lock().await;
        retry_count < self.config.max_retries
    }

    // Get connection statistics
    pub async fn get_stats(&self) -> ConnectionStats {
        let retry_count = *self.retry_count.lock().await;
        let last_success = *self.last_success.lock().await;

        ConnectionStats {
            retry_count,
            last_success,
            is_healthy: self.is_healthy().await,
        }
    }
}

#[derive(Debug)]
pub struct ConnectionStats {
    pub retry_count: u32,
    pub last_success: Option<std::time::SystemTime>,
    pub is_healthy: bool,
}

// Extension trait for resilient operations
#[async_trait::async_trait]
pub trait Resilient {
    // Retry an async operation with backoff
    async fn retry_async<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Result<T> + Send + Sync;

    // Execute with timeout
    async fn with_timeout<F, T>(&self, duration: Duration, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>> + Send;
}

#[async_trait::async_trait]
impl Resilient for NetworkResilience {
    async fn retry_async<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Result<T> + Send + Sync,
    {
        self.with_retry(operation).await
    }

    async fn with_timeout<F, T>(&self, duration: Duration, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>> + Send,
    {
        tokio::time::timeout(duration, operation)
            .await
            .map_err(|_| anyhow::anyhow!("Operation timed out"))?
    }
} 
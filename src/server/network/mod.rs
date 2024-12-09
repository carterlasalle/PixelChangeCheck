use crate::network::{NetworkConfig, ResilienceConfig};
use anyhow::Result;
use quinn::Endpoint;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info};

pub struct ServerNetwork {
    endpoint: Endpoint,
    config: NetworkConfig,
    resilience: ResilienceConfig,
}

impl ServerNetwork {
    pub fn new(config: NetworkConfig, resilience: ResilienceConfig) -> Result<Self> {
        let server_config = quinn::ServerConfig::with_crypto(Arc::new(config.crypto_config()));
        let endpoint = Endpoint::server(
            server_config,
            format!("0.0.0.0:{}", config.port.unwrap_or(5800)).parse()?,
        )?;

        Ok(Self {
            endpoint,
            config,
            resilience,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Server listening on port {}", self.config.port.unwrap_or(5800));
        
        while let Some(conn) = self.endpoint.accept().await {
            let connection = conn.await?;
            let remote = connection.remote_address();
            info!("Client connected from {}", remote);
            
            // Handle connection...
            tokio::spawn(async move {
                // Connection handling logic here
            });
        }
        
        Ok(())
    }
} 
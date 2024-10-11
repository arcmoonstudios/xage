// src/aproar/retrieval/redis_cache.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[RETRIEVAL]Xyn>=====S===t===u===d===i===o===s======[R|$>

use crate::omnixtracker::{OmniXError, OmniXMetry};
use super::RetrievalCache;
use redis::{AsyncCommands, Client};
use anyhow::{Context, Result};
use tokio::runtime::Runtime;
use parking_lot::RwLock;
use std::sync::Arc;


pub struct RedisCache {
    client: Client,
    runtime: Arc<Runtime>,
    metrics: OmniXMetry,
}

impl RedisCache {
    pub fn new(redis_url: &str, metrics: OmniXMetry) -> Result<Self, OmniXError> {
        let client = Client::open(redis_url)
            .with_context(|| "Failed to create Redis client")
            .map_err(|e| OmniXError::NetworkError(e.to_string()))?;

        let runtime = Runtime::new()
            .with_context(|| "Failed to create Tokio runtime")
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Creating Tokio runtime".to_string(),
                details: e.to_string(),
            })?;

        Ok(Self {
            client,
            runtime: Arc::new(runtime),
            metrics,
        })
    }
}

impl RetrievalCache for RedisCache {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, OmniXError> {
        let client = self.client.clone();
        let key = key.to_string();
        let metrics = self.metrics.clone();

        self.runtime.block_on(async move {
            let start_time = std::time::Instant::now();
            let mut con = client
                .get_async_connection()
                .await
                .with_context(|| "Failed to get Redis connection")
                .map_err(|e| OmniXError::NetworkError(e.to_string()))?;

            let result: Result<Option<Vec<u8>>, redis::RedisError> = con.get(key).await;
            let duration = start_time.elapsed();

            metrics.record_histogram("redis.get.duration".to_string(), duration.as_secs_f64());
            metrics.increment_counter("redis.get.total".to_string(), 1);

            match result {
                Ok(value) => {
                    metrics.increment_counter("redis.get.success".to_string(), 1);
                    Ok(value)
                }
                Err(e) => {
                    metrics.increment_counter("redis.get.failure".to_string(), 1);
                    Err(OmniXError::NetworkError(e.to_string()))
                }
            }
        })
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), OmniXError> {
        let client = self.client.clone();
        let key = key.to_string();
        let value = value.to_vec();
        let metrics = self.metrics.clone();

        self.runtime.block_on(async move {
            let start_time = std::time::Instant::now();
            let mut con = client
                .get_async_connection()
                .await
                .with_context(|| "Failed to get Redis connection")
                .map_err(|e| OmniXError::NetworkError(e.to_string()))?;

            let result: Result<(), redis::RedisError> = con.set(key, value).await;
            let duration = start_time.elapsed();

            metrics.record_histogram("redis.set.duration".to_string(), duration.as_secs_f64());
            metrics.increment_counter("redis.set.total".to_string(), 1);

            match result {
                Ok(_) => {
                    metrics.increment_counter("redis.set.success".to_string(), 1);
                    Ok(())
                }
                Err(e) => {
                    metrics.increment_counter("redis.set.failure".to_string(), 1);
                    Err(OmniXError::NetworkError(e.to_string()))
                }
            }
        })
    }
}
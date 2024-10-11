// src/aproar/retrieval/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[RETRIEVAL]Xyn>=====S===t===u===d===i===o===s======[R|$>

mod redis_cache;
mod rocksdb;

use crate::omnixtracker::OmniXError;
use anyhow::Result;
use async_trait::async_trait;

pub use redis_cache::RedisCache;
pub use rocksdb::{RocksDBStorage, RocksDBPersistence};

#[async_trait]
pub trait RetrievalCache: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, OmniXError>;
    fn set(&self, key: &str, value: &[u8]) -> Result<(), OmniXError>;
}
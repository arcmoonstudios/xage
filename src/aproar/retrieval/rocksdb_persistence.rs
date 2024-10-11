// src/aproar/retrieval/rocksdb_persistence.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[RETRIEVAL]Xyn>=====S===t===u===d===i===o===s======[R|$>

use super::RetrievalCache;
use crate::omnixtracker::{OmniXError, OmniXMetry};
use anyhow::{Context, Result};
use rocksdb::{Options, DB};
use std::path::PathBuf;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct RocksDBPersistence {
    db: Arc<RwLock<DB>>,
    metrics: OmniXMetry,
}

impl RocksDBPersistence {
    pub fn new(db_path: PathBuf, metrics: OmniXMetry) -> Result<Self, OmniXError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, &db_path)
            .with_context(|| format!("Failed to open RocksDB at {}", db_path.display()))
            .map_err(|e| OmniXError::DatabaseError(e.to_string()))?;

        Ok(Self {
            db: Arc::new(RwLock::new(db)),
            metrics,
        })
    }
}

impl RetrievalCache for RocksDBPersistence {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, OmniXError> {
        let start_time = std::time::Instant::now();
        let result = self.db.read().get(key.as_bytes());
        let duration = start_time.elapsed();

        self.metrics.record_histogram("rocksdb.get.duration".to_string(), duration.as_secs_f64());
        self.metrics.increment_counter("rocksdb.get.total".to_string(), 1);

        match result {
            Ok(value) => {
                self.metrics.increment_counter("rocksdb.get.success".to_string(), 1);
                Ok(value)
            }
            Err(e) => {
                self.metrics.increment_counter("rocksdb.get.failure".to_string(), 1);
                Err(OmniXError::DatabaseError(e.to_string()))
            }
        }
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), OmniXError> {
        let start_time = std::time::Instant::now();
        let result = self.db.write().put(key.as_bytes(), value);
        let duration = start_time.elapsed();

        self.metrics.record_histogram("rocksdb.set.duration".to_string(), duration.as_secs_f64());
        self.metrics.increment_counter("rocksdb.set.total".to_string(), 1);

        match result {
            Ok(_) => {
                self.metrics.increment_counter("rocksdb.set.success".to_string(), 1);
                Ok(())
            }
            Err(e) => {
                self.metrics.increment_counter("rocksdb.set.failure".to_string(), 1);
                Err(OmniXError::DatabaseError(e.to_string()))
            }
        }
    }
}
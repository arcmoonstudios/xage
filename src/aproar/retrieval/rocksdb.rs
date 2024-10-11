// src/aproar/retrieval/rocksdb.rs  ~=#######D]======A===r===c====M===o===o===n=====<Lord[RETRIEVAL]Xyn>=====S===t===u===d===i===o===s======[R|$>

use rocksdb::{DB, Options, ColumnFamilyDescriptor, WriteBatch, WriteOptions, ReadOptions, IteratorMode};
use crate::omnixtracker::{OmniXMetry, OmniXError};
use crate::constants::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::{Mutex, RwLock}; // Using Mutex for RocksDBStorage and RwLock for RocksDBPersistence
use serde::{Serialize, Deserialize};
use bincode;
use anyhow::{Context, Result};

pub struct RocksDBStorage {
    db: Arc<Mutex<DB>>,
    metrics: OmniXMetry,
}

impl RocksDBStorage {
    pub fn new(path: &Path, metrics: OmniXMetry) -> Result<Self, OmniXError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(ROCKSDB_MAX_OPEN_FILES);
        opts.set_use_fsync(false);
        opts.set_keep_log_file_num(ROCKSDB_KEEP_LOG_FILE_NUM);
        opts.set_max_total_wal_size(ROCKSDB_MAX_TOTAL_WAL_SIZE);
        opts.set_max_background_jobs(ROCKSDB_MAX_BACKGROUND_JOBS);
        opts.set_compaction_style(rocksdb::DBCompactionStyle::Level);

        let cf_opts = Options::default();
        let cf_descriptor = ColumnFamilyDescriptor::new("default", cf_opts);

        let db = DB::open_cf_descriptors(&opts, path, vec![cf_descriptor])
            .map_err(|e| OmniXError::DatabaseError(format!("Failed to open RocksDB: {}", e)))?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            metrics,
        })
    }

    pub async fn put<T: Serialize>(&self, key: &[u8], value: &T) -> Result<(), OmniXError> {
        let start = std::time::Instant::now();
        let serialized = bincode::serialize(value)
            .map_err(|e| OmniXError::SerializationError(e.to_string()))?;

        let db = self.db.lock().await;
        db.put(key, &serialized)
            .map_err(|e| OmniXError::DatabaseError(format!("Failed to put data: {}", e)))?;

        self.metrics.record_histogram("rocksdb.put.duration".to_string(), start.elapsed().as_secs_f64());
        self.metrics.increment_counter("rocksdb.put.count".to_string(), 1);
        Ok(())
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &[u8]) -> Result<Option<T>, OmniXError> {
        let start = std::time::Instant::now();
        let db = self.db.lock().await;
        let result = db.get(key)
            .map_err(|e| OmniXError::DatabaseError(format!("Failed to get data: {}", e)))?;

        let duration = start.elapsed().as_secs_f64();
        self.metrics.record_histogram("rocksdb.get.duration".to_string(), duration);
        self.metrics.increment_counter("rocksdb.get.count".to_string(), 1);

        match result {
            Some(data) => {
                let deserialized = bincode::deserialize(&data)
                    .map_err(|e| OmniXError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    pub async fn delete(&self, key: &[u8]) -> Result<(), OmniXError> {
        let start = std::time::Instant::now();
        let db = self.db.lock().await;
        db.delete(key)
            .map_err(|e| OmniXError::DatabaseError(format!("Failed to delete data: {}", e)))?;

        self.metrics.record_histogram("rocksdb.delete.duration".to_string(), start.elapsed().as_secs_f64());
        self.metrics.increment_counter("rocksdb.delete.count".to_string(), 1);
        Ok(())
    }

    pub async fn batch_write<T: Serialize>(&self, writes: Vec<(Vec<u8>, T)>) -> Result<(), OmniXError> {
        let start = std::time::Instant::now();
        let mut batch = WriteBatch::default();
        for (key, value) in writes {
            let serialized = bincode::serialize(&value)
                .map_err(|e| OmniXError::SerializationError(e.to_string()))?;
            batch.put(&key, &serialized);
        }

        let db = self.db.lock().await;
        let mut write_opts = WriteOptions::default();
        write_opts.set_sync(false);
        db.write_opt(batch, &write_opts)
            .map_err(|e| OmniXError::DatabaseError(format!("Failed to batch write: {}", e)))?;

        self.metrics.record_histogram("rocksdb.batch_write.duration".to_string(), start.elapsed().as_secs_f64());
        self.metrics.increment_counter("rocksdb.batch_write.count".to_string(), 1);
        Ok(())
    }

    pub async fn range_scan<T: for<'de> Deserialize<'de>>(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, T)>, OmniXError> {
        let start = std::time::Instant::now();
        let db = self.db.lock().await;
        let mut read_opts = ReadOptions::default();
        read_opts.set_iterate_lower_bound(start.to_vec());
        read_opts.set_iterate_upper_bound(end.to_vec());

        let iter = db.iterator_opt(IteratorMode::Start, read_opts);
        let mut result = Vec::new();

        for item in iter {
            let (key, value) = item.map_err(|e| OmniXError::DatabaseError(format!("Failed to iterate: {}", e)))?;
            let deserialized: T = bincode::deserialize(&value)
                .map_err(|e| OmniXError::DeserializationError(e.to_string()))?;
            result.push((key.to_vec(), deserialized));
        }

        self.metrics.record_histogram("rocksdb.range_scan.duration".to_string(), start.elapsed().as_secs_f64());
        self.metrics.increment_counter("rocksdb.range_scan.count".to_string(), 1);
        Ok(result)
    }

    pub async fn compact(&self) -> Result<(), OmniXError> {
        let start = std::time::Instant::now();
        let db = self.db.lock().await;
        db.compact_range::<&[u8], &[u8]>(None, None);

        self.metrics.record_histogram("rocksdb.compact.duration".to_string(), start.elapsed().as_secs_f64());
        self.metrics.increment_counter("rocksdb.compact.count".to_string(), 1);
        Ok(())
    }
}

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
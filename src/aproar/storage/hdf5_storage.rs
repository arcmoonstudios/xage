// src/aproar/storage/hdf5_storage.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[STORAGE]Xyn>=====S===t===u===d===i===o===s======[R|$>

use super::StorageBackend;
use crate::omnixtracker::{OmniXError, OmniXMetry};
use anyhow::{Context, Result};
use hdf5::File;
use std::path::PathBuf;
use std::sync::Arc;

pub struct HDF5Storage {
    file_path: PathBuf,
    metrics: OmniXMetry,
}

impl HDF5Storage {
    pub fn new(file_path: PathBuf, metrics: OmniXMetry) -> Self {
        Self { file_path, metrics }
    }
}

impl StorageBackend for HDF5Storage {
    fn store(&self, key: &str, data: &[u8]) -> Result<(), OmniXError> {
        let start_time = std::time::Instant::now();
        let file = File::open_rw(&self.file_path)
            .or_else(|_| File::create(&self.file_path))
            .with_context(|| "Failed to open or create HDF5 file")
            .map_err(|e| OmniXError::FileSystemError(e.to_string()))?;

        let dataset = file
            .new_dataset::<u8>()
            .shape(data.len())
            .create(key)
            .with_context(|| "Failed to create HDF5 dataset")
            .map_err(|e| OmniXError::OperationFailed {
                operation: "HDF5 dataset creation".to_string(),
                details: e.to_string(),
            })?;

        dataset.write(data).map_err(|e| OmniXError::OperationFailed {
            operation: "HDF5 write".to_string(),
            details: e.to_string(),
        })?;

        let duration = start_time.elapsed();
        self.metrics.record_histogram("hdf5.store.duration".to_string(), duration.as_secs_f64());
        self.metrics.increment_counter("hdf5.store.success".to_string(), 1);

        Ok(())
    }

    fn retrieve(&self, key: &str) -> Result<Vec<u8>, OmniXError> {
        let start_time = std::time::Instant::now();
        let file = File::open(&self.file_path)
            .with_context(|| "Failed to open HDF5 file")
            .map_err(|e| OmniXError::FileSystemError(e.to_string()))?;

        let dataset = file.dataset(key).map_err(|e| OmniXError::OperationFailed {
            operation: "HDF5 dataset access".to_string(),
            details: e.to_string(),
        })?;

        let data: Vec<u8> = dataset.read_raw().map_err(|e| OmniXError::OperationFailed {
            operation: "HDF5 read".to_string(),
            details: e.to_string(),
        })?;

        let duration = start_time.elapsed();
        self.metrics.record_histogram("hdf5.retrieve.duration".to_string(), duration.as_secs_f64());
        self.metrics.increment_counter("hdf5.retrieve.success".to_string(), 1);

        Ok(data)
    }
}
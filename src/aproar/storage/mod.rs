// src/aproar/storage/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[STORAGE]Xyn>=====S===t===u===d===i===o===s======[R|$>

mod hdf5_storage;
mod parquet_storage;
mod tiledb_storage;

use crate::omnixtracker::OmnixError;

pub trait StorageBackend {
    fn store(&self, key: &str, data: &[u8]) -> Result<(), OmnixError>;
    fn retrieve(&self, key: &str) -> Result<Vec<u8>, OmnixError>;
}

pub use hdf5_storage::HDF5Storage;
// TODO: Implement and export ParquetStorage and TileDBStorage
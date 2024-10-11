// src/aproar/compression/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[COMPRESSION]Xyn>=====S===t===u===d===i===o===s======[R|$>

use crate::omnixtracker::{OmniXMetry, OmniXError};
use crate::constants::*;

mod lz4_compression;
mod zstd_compression;

pub use lz4_compression::{compress_data_with_lz4, decompress_data_with_lz4};
pub use zstd_compression::{compress_data_with_zstd, decompress_data_with_zstd};

pub trait CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, OmniXError>;
    fn decompress(&self, compressed_data: &[u8]) -> Result<Vec<u8>, OmniXError>;
}

pub struct CompressionManager {
    metrics: OmniXMetry,
}

impl CompressionManager {
    pub fn new(metrics: OmniXMetry) -> Self {
        Self { metrics }
    }

    pub fn compress(&self, strategy: &dyn CompressionStrategy, data: &[u8]) -> Result<Vec<u8>, OmniXError> {
        let start_time = std::time::Instant::now();
        let result = strategy.compress(data);
        let duration = start_time.elapsed();

        self.metrics.record_histogram("compression.duration".to_string(), duration.as_secs_f64());
        self.metrics.increment_counter("compression.total".to_string(), 1);

        if result.is_ok() {
            self.metrics.increment_counter("compression.success".to_string(), 1);
        } else {
            self.metrics.increment_counter("compression.failure".to_string(), 1);
        }

        result
    }

    pub fn decompress(&self, strategy: &dyn CompressionStrategy, compressed_data: &[u8]) -> Result<Vec<u8>, OmniXError> {
        let start_time = std::time::Instant::now();
        let result = strategy.decompress(compressed_data);
        let duration = start_time.elapsed();

        self.metrics.record_histogram("decompression.duration".to_string(), duration.as_secs_f64());
        self.metrics.increment_counter("decompression.total".to_string(), 1);

        if result.is_ok() {
            self.metrics.increment_counter("decompression.success".to_string(), 1);
        } else {
            self.metrics.increment_counter("decompression.failure".to_string(), 1);
        }

        result
    }
}
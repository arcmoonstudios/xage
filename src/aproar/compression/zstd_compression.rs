// src/aproar/compression/zstd_compression.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[COMPRESSION]Xyn>=====S===t===u===d===i===o===s======[R|$>

use super::{CompressionStrategy, OmniXError};
use crate::constants::ZSTD_COMPRESSION_LEVEL;
use zstd::stream::{encode_all, decode_all};
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::fs::File;

pub struct ZstdCompression;

impl CompressionStrategy for ZstdCompression {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, OmniXError> {
        encode_all(data, ZSTD_COMPRESSION_LEVEL)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Zstd compression".to_string(),
                details: e.to_string(),
            })
    }

    fn decompress(&self, compressed_data: &[u8]) -> Result<Vec<u8>, OmniXError> {
        decode_all(compressed_data)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Zstd decompression".to_string(),
                details: e.to_string(),
            })
    }
}

/// Compresses data using Zstandard (Zstd) and writes to a file
pub fn compress_data_with_zstd(input_path: &str, output_path: &str) -> Result<()> {
    let mut input_file = File::open(input_path)
        .with_context(|| format!("Failed to open input file: {}", input_path))?;
    let mut buffer = Vec::new();
    input_file
        .read_to_end(&mut buffer)
        .with_context(|| format!("Failed to read input file: {}", input_path))?;

    let zstd_compressor = ZstdCompression;
    let compressed_data = zstd_compressor.compress(&buffer)
        .with_context(|| "Failed to compress data with Zstd")?;

    let mut output_file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path))?;
    output_file
        .write_all(&compressed_data)
        .with_context(|| "Failed to write compressed data to file")?;

    println!("Data compressed with Zstd and written to {}", output_path);
    Ok(())
}

/// Decompresses Zstd compressed data from a file
pub fn decompress_data_with_zstd(input_path: &str, output_path: &str) -> Result<()> {
    let mut input_file = File::open(input_path)
        .with_context(|| format!("Failed to open input file: {}", input_path))?;
    let mut compressed_data = Vec::new();
    input_file
        .read_to_end(&mut compressed_data)
        .with_context(|| format!("Failed to read compressed file: {}", input_path))?;

    let zstd_decompressor = ZstdCompression;
    let decompressed_data = zstd_decompressor.decompress(&compressed_data)
        .with_context(|| "Failed to decompress data with Zstd")?;

    let mut output_file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path))?;
    output_file
        .write_all(&decompressed_data)
        .with_context(|| "Failed to write decompressed data to file")?;

    println!("Data decompressed with Zstd and written to {}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_zstd_compression_decompression() -> Result<()> {
        let dir = tempdir()?;
        let input_path = dir.path().join("input.txt");
        let compressed_path = dir.path().join("compressed.zst");
        let decompressed_path = dir.path().join("decompressed.txt");

        let test_data = b"Hello, world! This is a test of Zstd compression.";
        std::fs::write(&input_path, test_data)?;

        compress_data_with_zstd(
            input_path.to_str().unwrap(),
            compressed_path.to_str().unwrap(),
        )?;
        decompress_data_with_zstd(
            compressed_path.to_str().unwrap(),
            decompressed_path.to_str().unwrap(),
        )?;

        let decompressed_content = std::fs::read(decompressed_path)?;
        assert_eq!(decompressed_content, test_data);

        Ok(())
    }
}
// src/aproar/compression/lz4_compression.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[COMPRESSION]Xyn>=====S===t===u===d===i===o===s======[R|$>

use lz4::{block::compress, block::decompress, block::CompressionMode};
use super::{CompressionStrategy, OmniXError};
use anyhow::{Context, Result};
use std::io::{Read, Write};

use std::fs::File;


pub struct LZ4Compression;

impl CompressionStrategy for LZ4Compression {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, OmniXError> {
        compress(data, Some(CompressionMode::HIGHCOMPRESSION(9)), false)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "LZ4 compression".to_string(),
                details: e.to_string(),
            })
    }

    fn decompress(&self, compressed_data: &[u8]) -> Result<Vec<u8>, OmniXError> {
        decompress(compressed_data, None)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "LZ4 decompression".to_string(),
                details: e.to_string(),
            })
    }
}

const BUFFER_SIZE: usize = 8192;

/// Compresses data using LZ4 and writes to a file
pub fn compress_data_with_lz4(input_path: &str, output_path: &str) -> Result<()> {
    let mut input_file = File::open(input_path)
        .with_context(|| format!("Failed to open input file: {}", input_path))?;
    let mut output_file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path))?;

    let mut buffer = Vec::with_capacity(BUFFER_SIZE);
    let lz4_compressor = LZ4Compression;

    loop {
        buffer.clear();
        let bytes_read = input_file
            .by_ref()
            .take(BUFFER_SIZE as u64)
            .read_to_end(&mut buffer)
            .with_context(|| "Failed to read input file")?;

        if bytes_read == 0 {
            break;
        }

        let compressed_chunk = lz4_compressor.compress(&buffer)
            .with_context(|| "Failed to compress data chunk with LZ4")?;

        output_file
            .write_all(&compressed_chunk)
            .with_context(|| "Failed to write compressed data to file")?;
    }

    println!("Data compressed with LZ4 and written to {}", output_path);
    Ok(())
}

/// Decompresses LZ4 compressed data from a file
pub fn decompress_data_with_lz4(input_path: &str, output_path: &str) -> Result<()> {
    let mut input_file = File::open(input_path)
        .with_context(|| format!("Failed to open input file: {}", input_path))?;
    let mut output_file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path))?;

    let mut compressed_data = Vec::new();
    input_file
        .read_to_end(&mut compressed_data)
        .with_context(|| "Failed to read compressed file")?;

    let lz4_decompressor = LZ4Compression;
    let decompressed_data = lz4_decompressor.decompress(&compressed_data)
        .with_context(|| "Failed to decompress data with LZ4")?;

    output_file
        .write_all(&decompressed_data)
        .with_context(|| "Failed to write decompressed data to file")?;

    println!("Data decompressed with LZ4 and written to {}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_lz4_compression_decompression() -> Result<()> {
        let dir = tempdir()?;
        let input_path = dir.path().join("input.txt");
        let compressed_path = dir.path().join("compressed.lz4");
        let decompressed_path = dir.path().join("decompressed.txt");

        let test_data = b"Hello, world! This is a test of LZ4 compression.";
        std::fs::write(&input_path, test_data)?;

        compress_data_with_lz4(
            input_path.to_str().unwrap(),
            compressed_path.to_str().unwrap(),
        )?;
        decompress_data_with_lz4(
            compressed_path.to_str().unwrap(),
            decompressed_path.to_str().unwrap(),
        )?;

        let decompressed_content = std::fs::read(decompressed_path)?;
        assert_eq!(decompressed_content, test_data);

        Ok(())
    }
}
// src/aproar/storage/parquet_storage.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[STORAGE]Xyn>=====S===t===u===d===i===o===s======[R|$>

use super::StorageBackend;
use crate::omnixtracker::OmniXError;
use anyhow::{Context, Result};
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::file::reader::SerializedFileReader;
use parquet::schema::parser::parse_message_type;
use parquet::basic::Compression;
use parquet::record::{Row, RowAccessor};
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ParquetStorage {
    file_path: PathBuf,
    schema: Arc<parquet::schema::types::Type>,
}

impl ParquetStorage {
    pub fn new(file_path: PathBuf) -> Self {
        let schema = Arc::new(Self::build_parquet_schema());
        Self { file_path, schema }
    }

    fn build_parquet_schema() -> parquet::schema::types::Type {
        parse_message_type(
            "
            message schema {
                REQUIRED BYTE_ARRAY key (UTF8);
                REQUIRED BINARY data;
            }
            ",
        )
        .expect("Failed to parse Parquet schema")
    }
}

impl StorageBackend for ParquetStorage {
    fn store(&self, key: &str, data: &[u8]) -> Result<(), OmniXError> {
        let file = File::create(&self.file_path)
            .with_context(|| "Failed to create Parquet file")
            .map_err(|e| OmniXError::FileSystemError(e.to_string()))?;

        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        let mut writer = SerializedFileWriter::new(file, self.schema.clone(), props)
            .with_context(|| "Failed to create Parquet writer")
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Parquet writer creation".to_string(),
                details: e.to_string(),
            })?;

        let mut row_group_writer = writer.next_row_group().unwrap();
        let mut key_column_writer = row_group_writer.next_column().unwrap().unwrap();
        if let parquet::column::writer::ColumnWriter::ByteArrayColumnWriter(ref mut typed_writer) = key_column_writer {
            let key_value = parquet::data_type::ByteArray::from(key.as_bytes());
            typed_writer.write_batch(&[key_value], None, None).unwrap();
        }
        row_group_writer.close_column(key_column_writer).unwrap();

        let mut data_column_writer = row_group_writer.next_column().unwrap().unwrap();
        if let parquet::column::writer::ColumnWriter::ByteArrayColumnWriter(ref mut typed_writer) = data_column_writer {
            let data_value = parquet::data_type::ByteArray::from(data);
            typed_writer.write_batch(&[data_value], None, None).unwrap();
        }
        row_group_writer.close_column(data_column_writer).unwrap();

        writer.close_row_group(row_group_writer).unwrap();
        writer.close().unwrap();

        Ok(())
    }

    fn retrieve(&self, key: &str) -> Result<Vec<u8>, OmniXError> {
        let file = File::open(&self.file_path)
            .with_context(|| "Failed to open Parquet file")
            .map_err(|e| OmniXError::FileSystemError(e.to_string()))?;

        let reader = SerializedFileReader::new(file)
            .with_context(|| "Failed to create Parquet reader")
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Parquet reader creation".to_string(),
                details: e.to_string(),
            })?;

        let iter = reader.get_row_iter(None).unwrap();

        for record in iter {
            let record_key = record.get_string(0).unwrap();
            if record_key == key {
                let data = record.get_bytes(1).unwrap();
                return Ok(data.to_vec());
            }
        }

        Err(OmniXError::OperationFailed {
            operation: "Parquet retrieval".to_string(),
            details: "Key not found".to_string(),
        })
    }
}
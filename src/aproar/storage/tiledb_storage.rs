// src/aproar/storage/tiledb_storage.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[STORAGE]Xyn>=====S===t===u===d===i===o===s======[R|$>

use super::StorageBackend;
use crate::omnixtracker::OmniXError;
use anyhow::{Context, Result};
use tiledb::Context;
use tiledb::Array;
use tiledb::Config;
use tiledb::Query;
use tiledb::Datatype;
use std::path::PathBuf;

pub struct TileDBStorage {
    array_uri: String,
    ctx: Context,
}

impl TileDBStorage {
    pub fn new(array_uri: &str) -> Self {
        let ctx = Context::new(&Config::default()).unwrap();
        Self {
            array_uri: array_uri.to_string(),
            ctx,
        }
    }

    fn create_array(&self) -> Result<(), OmniXError> {
        let array_schema = tiledb::ArraySchema::new(&self.ctx, tiledb::ArrayType::Sparse)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "TileDB ArraySchema creation".to_string(),
                details: e.to_string(),
            })?;

        let dim = tiledb::Dimension::new(&self.ctx, "key", Datatype::StringAscii)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "TileDB Dimension creation".to_string(),
                details: e.to_string(),
            })?;

        let domain = tiledb::Domain::new(&self.ctx)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "TileDB Domain creation".to_string(),
                details: e.to_string(),
            })?
            .add_dimension(&dim)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Adding dimension to Domain".to_string(),
                details: e.to_string(),
            })?;

        let attr = tiledb::Attribute::new(&self.ctx, "data", Datatype::Blob)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "TileDB Attribute creation".to_string(),
                details: e.to_string(),
            })?;

        let array_schema = array_schema
            .set_domain(&domain)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Setting Domain on ArraySchema".to_string(),
                details: e.to_string(),
            })?
            .add_attribute(&attr)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Adding Attribute to ArraySchema".to_string(),
                details: e.to_string(),
            })?;

        Array::create(&self.ctx, &self.array_uri, &array_schema)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Creating TileDB Array".to_string(),
                details: e.to_string(),
            })?;

        Ok(())
    }
}

impl StorageBackend for TileDBStorage {
    fn store(&self, key: &str, data: &[u8]) -> Result<(), OmniXError> {
        if !Array::exists(&self.ctx, &self.array_uri) {
            self.create_array()?;
        }

        let array = Array::open(&self.ctx, &self.array_uri, tiledb::QueryType::Write)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Opening TileDB Array for writing".to_string(),
                details: e.to_string(),
            })?;

        let mut query = Query::new(&self.ctx, &array, tiledb::QueryType::Write);

        query
            .set_layout(tiledb::Layout::Unordered)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Setting query layout".to_string(),
                details: e.to_string(),
            })?
            .set_buffer("key", vec![key.as_bytes()])
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Setting key buffer".to_string(),
                details: e.to_string(),
            })?
            .set_buffer("data", data.to_vec())
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Setting data buffer".to_string(),
                details: e.to_string(),
            })?;

        query.submit().map_err(|e| OmniXError::OperationFailed {
            operation: "Submitting TileDB query".to_string(),
            details: e.to_string(),
        })?;

        array.close().unwrap();

        Ok(())
    }

    fn retrieve(&self, key: &str) -> Result<Vec<u8>, OmniXError> {
        if !Array::exists(&self.ctx, &self.array_uri) {
            return Err(OmniXError::OperationFailed {
                operation: "TileDB retrieval".to_string(),
                details: "Array does not exist".to_string(),
            });
        }

        let array = Array::open(&self.ctx, &self.array_uri, tiledb::QueryType::Read)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Opening TileDB Array for reading".to_string(),
                details: e.to_string(),
            })?;

        let mut query = Query::new(&self.ctx, &array, tiledb::QueryType::Read);

        query
            .set_layout(tiledb::Layout::Unordered)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Setting query layout".to_string(),
                details: e.to_string(),
            })?
            .set_subarray(&[key])
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Setting subarray".to_string(),
                details: e.to_string(),
            })?;

        let data: Vec<u8> = Vec::new();
        query
            .set_buffer("data", data)
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Setting data buffer".to_string(),
                details: e.to_string(),
            })?;

        query.submit().map_err(|e| OmniXError::OperationFailed {
            operation: "Submitting TileDB query".to_string(),
            details: e.to_string(),
        })?;

        let result_data = query
            .result_buffer::<u8>("data")
            .map_err(|e| OmniXError::OperationFailed {
                operation: "Retrieving result buffer".to_string(),
                details: e.to_string(),
            })?
            .to_vec();

        array.close().unwrap();

        Ok(result_data)
    }
}
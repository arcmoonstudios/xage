// src/aproar/memory/context_window.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[MEMORY]Xyn>=====S===t===u===d===i===o===s======[R|$>
// src/aproar/memory/context_window.rs

use crate::omnixtracker::{OmniXMetry, OmniXError};
use crate::constants::*;
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct ContextChunk {
    pub id: Uuid,
    pub content: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub relevance_score: f64,
}

pub struct ContextWindowManager {
    chunks: RwLock<Vec<ContextChunk>>,
    max_window_size: usize,
    metrics: OmniXMetry,
}

impl ContextWindowManager {
    pub fn new(max_window_size: usize, metrics: OmniXMetry) -> Self {
        Self {
            chunks: RwLock::new(Vec::with_capacity(max_window_size)),
            max_window_size,
            metrics,
        }
    }

    pub async fn add_chunk(&self, content: Vec<u8>) -> Result<(), OmniXError> {
        let mut chunks = self.chunks.write().await;
        if chunks.len() >= self.max_window_size {
            chunks.remove(0);
        }
        chunks.push(ContextChunk {
            id: Uuid::new_v4(),
            content,
            timestamp: Utc::now(),
            relevance_score: 1.0,
        });
        self.metrics.increment_counter("context_window.chunks_added".to_string(), 1);
        Ok(())
    }

    pub async fn get_relevant_chunks(&self, query: &str, limit: usize) -> Result<Vec<ContextChunk>, OmniXError> {
        let chunks = self.chunks.read().await;
        let mut relevant_chunks: Vec<ContextChunk> = chunks.iter()
            .filter(|chunk| self.is_relevant(chunk, query))
            .take(limit)
            .cloned()
            .collect();
        relevant_chunks.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        self.metrics.increment_counter("context_window.chunks_retrieved".to_string(), relevant_chunks.len() as u64);
        Ok(relevant_chunks)
    }

    pub async fn get_all_chunks(&self) -> Result<Vec<ContextChunk>, OmniXError> {
        let chunks = self.chunks.read().await;
        Ok(chunks.clone())
    }

    fn is_relevant(&self, chunk: &ContextChunk, query: &str) -> bool {
        let query_bytes = query.as_bytes();
        chunk.content.windows(query_bytes.len()).any(|window| window == query_bytes)
    }

    pub async fn update_relevance(&self, chunk_id: Uuid, new_score: f64) -> Result<(), OmniXError> {
        let mut chunks = self.chunks.write().await;
        if let Some(chunk) = chunks.iter_mut().find(|c| c.id == chunk_id) {
            chunk.relevance_score = new_score;
            self.metrics.increment_counter("context_window.relevance_updates".to_string(), 1);
            Ok(())
        } else {
            Err(OmniXError::NotFound("Chunk not found".to_string()))
        }
    }
}
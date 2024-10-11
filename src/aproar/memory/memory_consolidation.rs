// src/aproar/memory/memory_consolidation.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[MEMORY]Xyn>=====S===t===u===d===i===o===s======[R|$>
// src/aproar/memory/memory_consolidation.rs

use crate::omnixtracker::{OmniXMetry, OmniXError};
use crate::aproar::memory::context_window::ContextChunk;
use crate::constants::*;
use std::sync::Arc;
use tokio::sync::Mutex;

pub trait ConsolidationStrategy: Send + Sync {
    fn consolidate(&self, chunks: &[ContextChunk]) -> Vec<ContextChunk>;
}

pub struct SimpleAveragingStrategy;

impl ConsolidationStrategy for SimpleAveragingStrategy {
    fn consolidate(&self, chunks: &[ContextChunk]) -> Vec<ContextChunk> {
        if chunks.is_empty() {
            return Vec::new();
        }

        let mut consolidated_content = Vec::new();
        let chunk_size = chunks[0].content.len();

        for i in 0..chunk_size {
            let sum: u32 = chunks.iter().map(|chunk| chunk.content[i] as u32).sum();
            let average = (sum / chunks.len() as u32) as u8;
            consolidated_content.push(average);
        }

        vec![ContextChunk {
            id: uuid::Uuid::new_v4(),
            content: consolidated_content,
            timestamp: chrono::Utc::now(),
            relevance_score: chunks.iter().map(|chunk| chunk.relevance_score).sum::<f64>() / chunks.len() as f64,
        }]
    }
}

pub struct MemoryConsolidator {
    strategy: Mutex<Box<dyn ConsolidationStrategy>>,
    metrics: OmniXMetry,
}

impl MemoryConsolidator {
    pub fn new(strategy: Box<dyn ConsolidationStrategy>, metrics: OmniXMetry) -> Self {
        Self {
            strategy: Mutex::new(strategy),
            metrics,
        }
    }

    pub async fn consolidate(&self, chunks: &[ContextChunk]) -> Result<Vec<ContextChunk>, OmniXError> {
        let start_time = std::time::Instant::now();
        let strategy = self.strategy.lock().await;
        let consolidated = strategy.consolidate(chunks);
        let duration = start_time.elapsed();
        self.metrics.record_histogram("memory_consolidation.duration".to_string(), duration.as_secs_f64());
        self.metrics.increment_counter("memory_consolidation.chunks_consolidated".to_string(), chunks.len() as u64);
        Ok(consolidated)
    }

    pub async fn set_strategy(&self, new_strategy: Box<dyn ConsolidationStrategy>) {
        let mut strategy = self.strategy.lock().await;
        *strategy = new_strategy;
    }
}
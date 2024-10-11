// src/aproar/memory/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[MEMORY]Xyn>=====S===t===u===d===i===o===s======[R|$>
// src/aproar/memory/mod.rs

mod context_window;
mod memory_consolidation;

pub use context_window::{ContextWindowManager, ContextChunk};
pub use memory_consolidation::{MemoryConsolidator, ConsolidationStrategy, SimpleAveragingStrategy};

use crate::omnixtracker::{OmniXMetry, OmniXError};
use crate::constants::*;
use std::sync::Arc;

pub struct MemoryManager {
    context_window: Arc<ContextWindowManager>,
    consolidator: Arc<MemoryConsolidator>,
    metrics: OmniXMetry,
}

impl MemoryManager {
    pub fn new(metrics: OmniXMetry) -> Self {
        let context_window = Arc::new(ContextWindowManager::new(CONTEXT_WINDOW_SIZE, metrics.clone()));
        let consolidator = Arc::new(MemoryConsolidator::new(
            Box::new(SimpleAveragingStrategy),
            metrics.clone(),
        ));

        Self {
            context_window,
            consolidator,
            metrics,
        }
    }

    pub async fn add_to_context(&self, content: Vec<u8>) -> Result<(), OmniXError> {
        self.context_window.add_chunk(content).await
    }

    pub async fn retrieve_context(&self, query: &str, limit: usize) -> Result<Vec<ContextChunk>, OmniXError> {
        self.context_window.get_relevant_chunks(query, limit).await
    }

    pub async fn consolidate_memory(&self) -> Result<(), OmniXError> {
        let chunks = self.context_window.get_all_chunks().await?;
        let consolidated = self.consolidator.consolidate(&chunks).await?;
        for chunk in consolidated {
            self.context_window.add_chunk(chunk.content).await?;
        }
        Ok(())
    }
}
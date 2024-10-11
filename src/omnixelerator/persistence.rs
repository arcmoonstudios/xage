// src/omnixelerator/persistence.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[OMNIXELERATOR]Xyn>=====S===t===u===d===i===o===s======[R|$>
// src/omnixelerator/persistence.rs

use crate::omnixtracker::omnixerror::{OmniXError, OmniXErrorManager};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sled::Db;

/// Manages task persistence.
pub struct PersistenceManager {
    pub task_id: Uuid,
    pub db: Db,
}

impl PersistenceManager {
    /// Initializes the PersistenceManager for a given task.
    pub fn new(task_id: Uuid) -> Result<Self, OmniXError> {
        let db_path = format!("task_{}_persistence", task_id);
        let db = sled::open(&db_path).map_err(|e| OmniXError::DatabaseError(e.to_string()))?;
        Ok(Self { task_id, db })
    }

    /// Persists the current state of the task.
    pub async fn persist_state(&self, state: &[u8]) -> Result<(), OmniXError> {
        self.db.insert("state", state).map_err(|e| OmniXError::DatabaseError(e.to_string()))?;
        self.db.flush_async().await.map_err(|e| OmniXError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Checks if the task should be cancelled.
    pub async fn should_cancel(&self) -> Result<bool, OmniXError> {
        Ok(self.db.contains_key("cancel").map_err(|e| OmniXError::DatabaseError(e.to_string()))?)
    }

    /// Marks the task as completed.
    pub async fn mark_as_completed(&self) -> Result<(), OmniXError> {
        self.db.insert("status", "completed").map_err(|e| OmniXError::DatabaseError(e.to_string()))?;
        self.db.flush_async().await.map_err(|e| OmniXError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

/// Represents the state of a complex task.
#[derive(Serialize, Deserialize)]
pub struct ComplexTaskState {
    pub current_iteration: usize,
    pub total_iterations: usize,
    pub partial_result: rust_decimal::Decimal,
    pub last_checkpoint: DateTime<Utc>,
    pub computation_hash: String,
}
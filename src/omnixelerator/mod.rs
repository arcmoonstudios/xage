// src/omnixelerator/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[OMNIXELERATOR]Xyn>=====S===t===u===d===i===o===s======[R|$>

pub mod execution;
pub mod parallexelerator;
pub mod persistence;
pub mod resource_monitor;
pub mod task_manager;

// Re-exports for convenient access
pub use crate::omnixelerator::execution::{ExecutionContext, CudaContext, OpenClContext, WgpuContext};
pub use crate::omnixelerator::parallexelerator::ParalleXelerator;
pub use crate::omnixelerator::persistence::{PersistenceManager, ComplexTaskState};
pub use crate::omnixelerator::resource_monitor::ResourceMonitor;
pub use crate::omnixelerator::task_manager::{TaskMaster, TaskMetadata, TaskStatus, TaskWrapper, Hyperparameters, TunerConfig, TaskProgress};
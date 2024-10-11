// src/aproar/ntm/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[NTM]Xyn>=====S===t===u===d===i===o===s======[R|$>
pub mod addressing;
pub mod controller;
pub mod memory;
pub mod read_head;
pub mod write_head;

pub use addressing::AddressingMechanism;
pub use controller::NTMController;
pub use memory::Memory;
pub use read_head::ReadHead;
pub use write_head::WriteHead;

use ndarray::{Array1, Array2};
use crate::omnixtracker::omnixerror::OmniXError;
use crate::omnixtracker::omnixmetry::{log_info, log_warning, log_error, collect_metrics};
use crate::omnixtracker::metrics::Metrics;
use crate::omnixtracker::constants::*;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OmniXurge: Send + Sync {
    async fn parallelize_task<T: TaskMaster + 'static>(&self, task: T) -> Result<Uuid, OmniXError>;
    async fn get_parallelized_task_status(&self, task_id: Uuid) -> Option<TaskMetadata>;
    async fn get_resource_utilization(&self) -> Result<ResourceMonitor, OmniXError>;
    async fn tune_hyperparameters<H: Hyperparameters>(&self, config: TunerConfig<H>) -> Result<H, OmniXError>;
    async fn accelerate(&self) -> Result<(), OmniXError>;
    async fn decelerate(&self) -> Result<(), OmniXError>;
    async fn collect_metrics(&self) -> Result<Metrics, OmniXError>;
    async fn shutdown(&self) -> Result<(), OmniXError>;
    async fn submit_task_with_progress<T: TaskMaster + 'static, F: Fn(TaskProgress) + Send + 'static>(
        &self,
        task: T,
        progress_callback: F,
    ) -> Result<Uuid, OmniXError>;
    async fn recover_and_resume_tasks(&self) -> Result<(), OmniXError>;
}

pub struct NTM {
    controller: NTMController,
    memory: Memory,
    read_head: ReadHead,
    write_head: WriteHead,
    memory_size: usize,
    memory_vector_size: usize,
    controller_output_size: usize,
}

impl NTM {
    pub fn new(
        input_size: usize,
        output_size: usize,
        memory_size: usize,
        memory_vector_size: usize,
        controller_size: usize,
        config: &Config,
    ) -> Result<Self, OmniXError> {
        log_info("Initializing NTM...");
        let controller_output_size = memory_vector_size * 2 + 6;
        let controller = NTMController::new(input_size + memory_vector_size, output_size + controller_output_size, memory_vector_size, memory_size, config)?;
        let memory = Memory::new(memory_size, memory_vector_size);
        let read_head = ReadHead::new(memory_size, memory_vector_size);
        let write_head = WriteHead::new(memory_size, memory_vector_size, memory_vector_size);

        Ok(Self {
            controller,
            memory,
            read_head,
            write_head,
            memory_size,
            memory_vector_size,
            controller_output_size,
        })
    }

    pub async fn forward(&mut self, input: &Array1<f32>) -> Result<Array1<f32>, OmniXError> {
        log_info("Forward pass initiated...");
        let prev_read_weights = Array1::ones(self.memory_size) / self.memory_size as f32;
        let prev_read = self.read_head.read(&self.memory, &prev_read_weights)?;

        let (output, controller_output) = self.controller.forward(input)?;

        let read_weights = self.read_head.get_weights(
            &controller_output,
            &prev_read_weights,
            &self.memory.read(&prev_read_weights)?,
        )?;
        
        let write_weights = self.write_head.get_weights(
            &controller_output,
            &prev_read_weights,
            &self.memory.read(&prev_read_weights)?,
        )?;
        
        let erase_vector = self.write_head.get_erase_vector(&controller_output)?;
        let add_vector = self.write_head.get_add_vector(&controller_output)?;

        self.memory.write(&write_weights, &erase_vector, &add_vector)?;

        let read_vector = self.read_head.read(&self.memory, &read_weights)?;

        collect_metrics("Forward pass completed successfully.".to_string());
        Ok(output)
    }

    pub async fn reset(&mut self) {
        log_info("Resetting NTM state...");
        self.memory.clear();
        collect_metrics("NTM state has been reset.");
    }
}
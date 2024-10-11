// src/aproar/ntm/memory.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[NTM]Xyn>=====S===t===u===d===i===o===s======[R|$>

use ndarray::{Array2, Array1, Axis};
use crate::omnixtracker::omnixerror::NTMError;
use std::sync::Arc;
use parking_lot::RwLock;
use rayon::prelude::*;

#[derive(Clone)]
pub struct Memory {
    memory: Arc<RwLock<Array2<f32>>>,
    usage: Arc<RwLock<Array1<f32>>>,
}

impl Memory {
    pub fn new(memory_size: usize, memory_vector_size: usize) -> Self {
        Memory {
            memory: Arc::new(RwLock::new(Array2::zeros((memory_size, memory_vector_size)))),
            usage: Arc::new(RwLock::new(Array1::zeros(memory_size))),
        }
    }

    pub fn read(&self, weights: &Array1<f32>) -> Result<Array1<f32>, NTMError> {
        let memory = self.memory.read();
        if weights.len() != memory.shape()[0] {
            return Err(NTMError::ShapeMismatch {
                expected: vec![memory.shape()[0]],
                actual: vec![weights.len()],
            });
        }
        Ok(memory.t().dot(weights))
    }

    pub fn write(&self, weights: &Array1<f32>, erase: &Array1<f32>, add: &Array1<f32>) -> Result<(), NTMError> {
        let mut memory = self.memory.write();
        let mut usage = self.usage.write();
        if weights.len() != memory.shape()[0] || erase.len() != memory.shape()[1] || add.len() != memory.shape()[1] {
            return Err(NTMError::ShapeMismatch {
                expected: vec![memory.shape()[0], memory.shape()[1], memory.shape()[1]],
                actual: vec![weights.len(), erase.len(), add.len()],
            });
        }
        let erase_term = weights.dot(&erase.t());
        let add_term = weights.dot(&add.t());
        *memory = &*memory * (1.0 - &erase_term) + &add_term;
        *usage = usage.mapv(|u| u.max(0.0).min(1.0)) + weights;
        Ok(())
    }

    pub fn get_least_used_locations(&self, n: usize) -> Result<Array1<usize>, NTMError> {
        let usage = self.usage.read();
        if n > usage.len() {
            return Err(NTMError::InvalidArgument("Requested more locations than available".to_string()));
        }
        let mut indices: Vec<usize> = (0..usage.len()).collect();
        indices.sort_unstable_by(|&i, &j| usage[i].partial_cmp(&usage[j]).unwrap_or(std::cmp::Ordering::Equal));
        Ok(Array1::from_vec(indices[..n].to_vec()))
    }

    pub fn clear(&self) {
        let mut memory = self.memory.write();
        let mut usage = self.usage.write();
        memory.fill(0.0);
        usage.fill(0.0);
    }
}
// src/aproar/ntm/addressing.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[NTM]Xyn>=====S===t===u===d===i===o===s======[R|$>

use ndarray::{Array1, Array2, Axis};
use ndarray_stats::QuantileExt;
use crate::omnixtracker::omnixerror::NTMError;
use rayon::prelude::*;

pub struct AddressingMechanism {
    memory_size: usize,
    key_size: usize,
}

impl AddressingMechanism {
    pub fn new(memory_size: usize, key_size: usize) -> Self {
        AddressingMechanism { memory_size, key_size }
    }

    pub fn content_addressing(&self, key: &Array1<f32>, beta: f32, memory: &Array2<f32>) -> Result<Array1<f32>, NTMError> {
        if key.len() != self.key_size {
            return Err(NTMError::ShapeMismatch {
                expected: vec![self.key_size],
                actual: vec![key.len()],
            });
        }
        let similarities = memory.axis_iter(Axis(0))
            .into_par_iter()
            .map(|row| {
                1.0 - cosine_similarity(key, &row.to_owned())
            })
            .collect::<Vec<f32>>();
        let similarities = Array1::from_vec(similarities);
        let scaled_similarities = similarities * beta;
        self.softmax(&scaled_similarities)
    }

    pub fn interpolate(&self, w_prev: &Array1<f32>, w_c: &Array1<f32>, g: f32) -> Result<Array1<f32>, NTMError> {
        if w_prev.len() != self.memory_size || w_c.len() != self.memory_size {
            return Err(NTMError::ShapeMismatch {
                expected: vec![self.memory_size, self.memory_size],
                actual: vec![w_prev.len(), w_c.len()],
            });
        }
        Ok(w_prev * (1.0 - g) + w_c * g)
    }

    pub fn shift(&self, w: &Array1<f32>, s: &Array1<f32>) -> Result<Array1<f32>, NTMError> {
        if w.len() != self.memory_size || s.len() != 3 {
            return Err(NTMError::ShapeMismatch {
                expected: vec![self.memory_size, 3],
                actual: vec![w.len(), s.len()],
            });
        }
        let mut w_shifted = Array1::zeros(self.memory_size);
        for i in 0..self.memory_size {
            for j in -1..=1 {
                let idx = (i as i32 + j).rem_euclid(self.memory_size as i32) as usize;
                w_shifted[i] += w[idx] * s[(j + 1) as usize];
            }
        }
        Ok(w_shifted)
    }

    pub fn sharpen(&self, w: &Array1<f32>, gamma: f32) -> Result<Array1<f32>, NTMError> {
        if w.len() != self.memory_size {
            return Err(NTMError::ShapeMismatch {
                expected: vec![self.memory_size],
                actual: vec![w.len()],
            });
        }
        let w_pow = w.mapv(|x| x.powf(gamma));
        let sum = w_pow.sum();
        Ok(w_pow / sum)
    }

    fn softmax(&self, x: &Array1<f32>) -> Result<Array1<f32>, NTMError> {
        if x.is_empty() {
            return Err(NTMError::InvalidArgument("Input array is empty in softmax function".to_string()));
        }

        if x.iter().any(|&a| a.is_nan()) {
            return Err(NTMError::InvalidArgument("Input array contains NaN values in softmax function".to_string()));
        }

        let max = x.fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let exp = x.mapv(|a| (a - max).exp());
        let sum = exp.sum();
        Ok(exp / sum)
    }}

fn cosine_similarity(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
    let dot_product = a.dot(b);
    let norm_a = a.dot(a).sqrt();
    let norm_b = b.dot(b).sqrt();
    dot_product / (norm_a * norm_b)
}
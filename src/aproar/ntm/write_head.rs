// src/aproar/ntm/write_head.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[NTM]Xyn>=====S===t===u===d===i===o===s======[R|$>
use super::*;

pub struct WriteHead {
    addressing: AddressingMechanism,
    key_size: usize,
    memory_vector_size: usize,
}

impl WriteHead {
    pub fn new(memory_size: usize, key_size: usize, memory_vector_size: usize) -> Self {
        WriteHead {
            addressing: AddressingMechanism::new(memory_size, key_size),
            key_size,
            memory_vector_size,
        }
    }

    pub fn get_weights(&self, controller_output: &Array1<f32>, prev_weights: &Array1<f32>, memory: &Array2<f32>) -> Result<Array1<f32>, NTMError> {
        let key = controller_output.slice(s![..self.key_size]).to_owned();
        let beta = controller_output[self.key_size].exp();
        let g = controller_output[self.key_size + 1].sigmoid();
        let s = controller_output.slice(s![self.key_size+2..self.key_size+5]).to_owned();
        let gamma = controller_output[self.key_size + 5].exp() + 1.0;

        let w_c = self.addressing.content_addressing(&key, beta, memory)?;
        let w_g = self.addressing.interpolate(prev_weights, &w_c, g)?;
        let w_s = self.addressing.shift(&w_g, &s)?;
        self.addressing.sharpen(&w_s, gamma)
    }

    pub fn get_erase_vector(&self, controller_output: &Array1<f32>) -> Result<Array1<f32>, NTMError> {
        let start = self.key_size + 6;
        let end = start + self.memory_vector_size;
        if end > controller_output.len() {
            return Err(NTMError::ShapeMismatch {
                expected: vec![end],
                actual: vec![controller_output.len()],
            });
        }
        Ok(controller_output.slice(s![start..end]).mapv(|x| x.sigmoid()))
    }

    pub fn get_add_vector(&self, controller_output: &Array1<f32>) -> Result<Array1<f32>, NTMError> {
        let start = self.key_size + 6 + self.memory_vector_size;
        let end = start + self.memory_vector_size;
        if end > controller_output.len() {
            return Err(NTMError::ShapeMismatch {
                expected: vec![end],
                actual: vec![controller_output.len()],
            });
        }
        Ok(controller_output.slice(s![start..end]).to_owned())
    }
}
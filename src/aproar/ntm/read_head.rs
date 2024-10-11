// src/aproar/ntm/read_head.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[NTM]Xyn>=====S===t===u===d===i===o===s======[R|$>

use super::*;

pub struct ReadHead {
    addressing: AddressingMechanism,
    key_size: usize,
}

impl ReadHead {
    pub fn new(memory_size: usize, key_size: usize) -> Self {
        ReadHead {
            addressing: AddressingMechanism::new(memory_size, key_size),
            key_size,
        }
    }

    pub fn read(&self, memory: &Memory, weights: &Array1<f32>) -> Result<Array1<f32>, NTMError> {
        memory.read(weights)
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
}
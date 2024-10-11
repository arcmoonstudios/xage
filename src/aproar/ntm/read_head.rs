// src/aproar/ntm/read_head.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[NTM]Xyn>=====S===t===u===d===i===o===s======[R|$>
// src/aproar/ntm/read_head.rs

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

    pub fn read(&self, memory: &Memory, weights: &Array1<f32>) -> Result<Array1<f32>> {
        memory.read(weights)
    }

    pub fn get_weights(&self, controller_output: &Array1<f32>, prev_weights: &Array1<f32>, memory: &Array2<f32>) -> Result<Array1<f32>> {
        if controller_output.len() != self.key_size + 6 {
            return Err(NTMError::ShapeMismatch {
                expected: vec![self.key_size + 6],
                actual: vec![controller_output.len()],
            });
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_read_head() -> Result<()> {
        let memory_size = 10;
        let key_size = 4;
        let read_head = ReadHead::new(memory_size, key_size);
        let memory = Memory::new(memory_size, key_size);
        
        // Initialize memory with some values
        let weights = Array1::from_vec(vec![0.1; memory_size]);
        let erase = Array1::zeros(key_size);
        let add = Array1::from_vec(vec![1.0; key_size]);
        memory.write(&weights, &erase, &add)?;

        // Test reading
        let read_weights = Array1::from_vec(vec![0.2; memory_size]);
        let read_result = read_head.read(&memory, &read_weights)?;
        assert_eq!(read_result.len(), key_size);

        // Test get_weights
        let controller_output = Array1::from_vec(vec![0.1; key_size + 6]);
        let prev_weights = Array1::from_vec(vec![0.1; memory_size]);
        let memory_content = memory.read_memory();
        let new_weights = read_head.get_weights(&controller_output, &prev_weights, &memory_content)?;
        
        assert_eq!(new_weights.len(), memory_size);
        assert_abs_diff_eq!(new_weights.sum(), 1.0, epsilon = 1e-6);

        Ok(())
    }

    #[test]
    fn test_read_head_errors() {
        let memory_size = 10;
        let key_size = 4;
        let read_head = ReadHead::new(memory_size, key_size);
        
        // Test error on invalid controller output size
        let invalid_controller_output = Array1::zeros(key_size);
        let prev_weights = Array1::from_vec(vec![0.1; memory_size]);
        let memory_content = Array2::zeros((memory_size, key_size));
        
        assert!(matches!(
            read_head.get_weights(&invalid_controller_output, &prev_weights, &memory_content),
            Err(NTMError::ShapeMismatch { .. })
        ));
    }
}
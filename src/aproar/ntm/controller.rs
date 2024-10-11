// src/aproar/ntm/controller.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[NTM]Xyn>=====S===t===u===d===i===o===s======[R|$>

use super::*;
use std::cell::RefCell;
use ndarray::{Array1, Array2, Axis};
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Uniform;
use crate::omnixtracker::omnixerror::NTMError;
use crate::omnixtracker::omnixmetry::OmniXMetry;
use crate::constants::*;
use rayon::prelude::*;

pub struct NTMController {
    memory: Memory,
    read_heads: Vec<ReadHead>,
    write_heads: Vec<WriteHead>,
    controller_size: usize,
    memory_vector_size: usize,
    num_read_heads: usize,
    num_write_heads: usize,
    lstm: LSTM,
    prev_read_weights: RefCell<Vec<Array1<f32>>>,
    prev_write_weights: RefCell<Vec<Array1<f32>>>,
    metrics: OmniXMetry,
}

impl NTMController {
    pub fn new(memory_size: usize, memory_vector_size: usize, controller_size: usize, num_read_heads: usize, num_write_heads: usize, metrics: OmniXMetry) -> Result<Self, NTMError> {
        let read_heads = (0..num_read_heads).map(|_| ReadHead::new(memory_size, memory_vector_size)).collect();
        let write_heads = (0..num_write_heads).map(|_| WriteHead::new(memory_size, memory_vector_size, memory_vector_size)).collect();
        let input_size = memory_vector_size * num_read_heads + memory_vector_size;
        let output_size = controller_size + num_read_heads * (memory_vector_size + 6) + num_write_heads * (memory_vector_size * 2 + 6);
        
        Ok(NTMController {
            memory: Memory::new(memory_size, memory_vector_size),
            read_heads,
            write_heads,
            controller_size,
            memory_vector_size,
            num_read_heads,
            num_write_heads,
            lstm: LSTM::new(input_size, controller_size, output_size),
            prev_read_weights: RefCell::new(vec![Array1::zeros(memory_size); num_read_heads]),
            prev_write_weights: RefCell::new(vec![Array1::zeros(memory_size); num_write_heads]),
            metrics,
        })
    }

    pub fn forward(&self, input: &Array1<f32>) -> Result<Array1<f32>, NTMError> {
        let start_time = std::time::Instant::now();
        let mut prev_read_weights = self.prev_read_weights.borrow_mut();
        let mut prev_write_weights = self.prev_write_weights.borrow_mut();
        let memory = self.memory.read_memory();

        let read_vectors: Vec<Array1<f32>> = prev_read_weights.par_iter()
            .zip(self.read_heads.par_iter())
            .map(|(weights, head)| head.read(&self.memory, weights))
            .collect::<Result<Vec<_>, _>>()?;

        let controller_input = Array1::from_iter(input.iter().cloned().chain(read_vectors.iter().flat_map(|v| v.iter().cloned())));
        let controller_output = self.lstm.forward(&controller_input)?;

        let mut output = controller_output.slice(s![..self.controller_size]).to_owned();
        let mut idx = self.controller_size;

        for (i, head) in self.read_heads.iter().enumerate() {
            let weights = head.get_weights(
                &controller_output.slice(s![idx..idx+self.memory_vector_size+6]),
                &prev_read_weights[i],
                &memory,
            )?;
            let read_vector = head.read(&self.memory, &weights)?;
            output.append(Axis(0), read_vector.view())?;
            prev_read_weights[i] = weights;
            idx += self.memory_vector_size + 6;
        }

        for (i, head) in self.write_heads.iter().enumerate() {
            let weights = head.get_weights(
                &controller_output.slice(s![idx..idx+self.memory_vector_size+6]),
                &prev_write_weights[i],
                &memory,
            )?;
            let erase_vector = head.get_erase_vector(&controller_output.slice(s![idx..]))?;
            let add_vector = head.get_add_vector(&controller_output.slice(s![idx..]))?;
            self.memory.write(&weights, &erase_vector, &add_vector)?;
            prev_write_weights[i] = weights;
            idx += self.memory_vector_size * 2 + 6;
        }

        let duration = start_time.elapsed();
        self.metrics.record_histogram("ntm_controller.forward_duration".to_string(), duration.as_secs_f64());

        Ok(output)
    }

    pub fn reset(&mut self) {
        self.memory.clear();
        self.prev_read_weights.borrow_mut().iter_mut().for_each(|w| *w = Array1::zeros(self.memory.size()));
        self.prev_write_weights.borrow_mut().iter_mut().for_each(|w| *w = Array1::zeros(self.memory.size()));
        self.lstm.reset();
    }

    pub fn optimize_memory_usage(&mut self) -> Result<(), NTMError> {
        let start_time = std::time::Instant::now();
        let current_usage = self.memory.usage();
        if current_usage > MEMORY_USAGE_THRESHOLD {
            self.memory.compact()?;
        }
        let duration = start_time.elapsed();
        self.metrics.record_histogram("ntm_controller.optimize_memory_duration".to_string(), duration.as_secs_f64());
        Ok(())
    }
}

struct LSTM {
    weight_ih: Array2<f32>,
    weight_hh: Array2<f32>,
    bias_ih: Array1<f32>,
    bias_hh: Array1<f32>,
    hidden_size: usize,
    cell_state: RefCell<Array1<f32>>,
    hidden_state: RefCell<Array1<f32>>,
}

impl LSTM {
    fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        LSTM {
            weight_ih: Array2::random((4 * hidden_size, input_size), Uniform::new(-0.1, 0.1)),
            weight_hh: Array2::random((4 * hidden_size, hidden_size), Uniform::new(-0.1, 0.1)),
            bias_ih: Array1::zeros(4 * hidden_size),
            bias_hh: Array1::zeros(4 * hidden_size),
            hidden_size,
            cell_state: RefCell::new(Array1::zeros(hidden_size)),
            hidden_state: RefCell::new(Array1::zeros(hidden_size)),
        }
    }

    fn forward(&self, input: &Array1<f32>) -> Result<Array1<f32>, NTMError> {
        let mut cell_state = self.cell_state.borrow_mut();
        let mut hidden_state = self.hidden_state.borrow_mut();

        let gates = self.weight_ih.dot(input) + &self.bias_ih + self.weight_hh.dot(&hidden_state) + &self.bias_hh;
        let chunked_gates: Vec<_> = gates.axis_chunks_iter(Axis(0), self.hidden_size).collect();

        let i = chunked_gates[0].mapv(|x| x.sigmoid());
        let f = chunked_gates[1].mapv(|x| x.sigmoid());
        let g = chunked_gates[2].mapv(|x| x.tanh());
        let o = chunked_gates[3].mapv(|x| x.sigmoid());

        *cell_state = &f * &*cell_state + &i * &g;
        *hidden_state = &o * &cell_state.mapv(|x| x.tanh());

        Ok(hidden_state.clone())
    }

    fn reset(&mut self) {
        *self.cell_state.borrow_mut() = Array1::zeros(self.hidden_size);
        *self.hidden_state.borrow_mut() = Array1::zeros(self.hidden_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use crate::omnixtracker::omnixmetry::OmniXMetry;

    #[test]
    fn test_ntm_controller() -> Result<(), NTMError> {
        let metrics = OmniXMetry::new("test".to_string());
        let controller = NTMController::new(10, 5, 20, 1, 1, metrics)?;
        let input = Array1::random(5, Uniform::new(0., 1.));
        
        let output = controller.forward(&input)?;
        
        assert_eq!(output.len(), 20 + 5);  // controller_size + read_vector_size
        
        Ok(())
    }

    #[test]
    fn test_lstm() -> Result<(), NTMError> {
        let lstm = LSTM::new(10, 20, 30);
        let input = Array1::random(10, Uniform::new(0., 1.));
        
        let output = lstm.forward(&input)?;
        
        assert_eq!(output.len(), 20);
        assert!(output.iter().all(|&x| x >= -1.0 && x <= 1.0));

        Ok(())
    }

    #[test]
    fn test_memory_optimization() -> Result<(), NTMError> {
        let metrics = OmniXMetry::new("test".to_string());
        let mut controller = NTMController::new(100, 10, 30, 2, 2, metrics)?;
        
        // Fill memory
        for _ in 0..120 {
            let input = Array1::random(10, Uniform::new(0., 1.));
            controller.forward(&input)?;
        }

        controller.optimize_memory_usage()?;
        
        assert!(controller.memory.usage() <= MEMORY_USAGE_THRESHOLD);

        Ok(())
    }
}
// src/aproar/ntm/controller.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[NTM]Xyn>=====S===t===u===d===i===o===s======[R|$>
// src/aproar/ntm/controller.rs 
use super::*;
use std::cell::RefCell;

pub struct NTMController {
    memory: Memory,
    read_head: ReadHead,
    write_head: WriteHead,
    controller_size: usize,
    memory_vector_size: usize,
    num_read_heads: usize,
    num_write_heads: usize,
    lstm: LSTM,
    prev_read_weights: RefCell<Vec<Array1<f32>>>,
    prev_write_weights: RefCell<Vec<Array1<f32>>>,
}

impl NTMController {
    pub fn new(memory_size: usize, memory_vector_size: usize, controller_size: usize, num_read_heads: usize, num_write_heads: usize) -> Result<Self> {
        let read_head = ReadHead::new(memory_size, memory_vector_size);
        let write_head = WriteHead::new(memory_size, memory_vector_size, memory_vector_size);
        let input_size = memory_vector_size * num_read_heads + memory_vector_size;
        let output_size = controller_size + num_read_heads * (memory_vector_size + 6) + num_write_heads * (memory_vector_size * 2 + 6);
        
        Ok(NTMController {
            memory: Memory::new(memory_size, memory_vector_size),
            read_head,
            write_head,
            controller_size,
            memory_vector_size,
            num_read_heads,
            num_write_heads,
            lstm: LSTM::new(input_size, controller_size, output_size),
            prev_read_weights: RefCell::new(vec![Array1::zeros(memory_size); num_read_heads]),
            prev_write_weights: RefCell::new(vec![Array1::zeros(memory_size); num_write_heads]),
        })
    }

    pub fn forward(&self, input: &Array1<f32>) -> Result<Array1<f32>> {
        let mut prev_read_weights = self.prev_read_weights.borrow_mut();
        let mut prev_write_weights = self.prev_write_weights.borrow_mut();
        let memory = self.memory.read_memory();

        let mut read_vectors = Vec::with_capacity(self.num_read_heads);
        for weights in prev_read_weights.iter() {
            read_vectors.push(self.read_head.read(&self.memory, weights)?);
        }

        let controller_input = Array1::from_iter(input.iter().cloned().chain(read_vectors.iter().flat_map(|v| v.iter().cloned())));
        let controller_output = self.lstm.forward(&controller_input)?;

        let mut output = controller_output.slice(s![..self.controller_size]).to_owned();
        let mut idx = self.controller_size;

        for i in 0..self.num_read_heads {
            let weights = self.read_head.get_weights(
                &controller_output.slice(s![idx..idx+self.memory_vector_size+6]),
                &prev_read_weights[i],
                &memory,
            )?;
            let read_vector = self.read_head.read(&self.memory, &weights)?;
            output = Array1::from_iter(output.iter().cloned().chain(read_vector.iter().cloned()));
            prev_read_weights[i] = weights;
            idx += self.memory_vector_size + 6;
        }

        for i in 0..self.num_write_heads {
            let weights = self.write_head.get_weights(
                &controller_output.slice(s![idx..idx+self.memory_vector_size+6]),
                &prev_write_weights[i],
                &memory,
            )?;
            let erase_vector = self.write_head.get_erase_vector(&controller_output.slice(s![idx..]))?;
            let add_vector = self.write_head.get_add_vector(&controller_output.slice(s![idx..]))?;
            self.memory.write(&weights, &erase_vector, &add_vector)?;
            prev_write_weights[i] = weights;
            idx += self.memory_vector_size * 2 + 6;
        }

        Ok(output)
    }
}

struct LSTM {
    weight_ih: Array2<f32>,
    weight_hh: Array2<f32>,
    bias_ih: Array1<f32>,
    bias_hh: Array1<f32>,
    hidden_size: usize,
}

impl LSTM {
    fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        LSTM {
            weight_ih: Array2::random((4 * hidden_size, input_size), Uniform::new(-0.1, 0.1)),
            weight_hh: Array2::random((4 * hidden_size, hidden_size), Uniform::new(-0.1, 0.1)),
            bias_ih: Array1::zeros(4 * hidden_size),
            bias_hh: Array1::zeros(4 * hidden_size),
            hidden_size,
        }
    }

    fn forward(&self, input: &Array1<f32>) -> Result<Array1<f32>> {
        let gates = self.weight_ih.dot(input) + &self.bias_ih + &self.weight_hh.dot(&Array1::zeros(self.hidden_size)) + &self.bias_hh;
        let chunked_gates: Vec<_> = gates.axis_chunks_iter(Axis(0), self.hidden_size).collect();

        let i = chunked_gates[0].mapv(|x| x.sigmoid());
        let f = chunked_gates[1].mapv(|x| x.sigmoid());
        let g = chunked_gates[2].mapv(|x| x.tanh());
        let o = chunked_gates[3].mapv(|x| x.sigmoid());

        let c = &f * &Array1::zeros(self.hidden_size) + &i * &g;
        let h = &o * &c.mapv(|x| x.tanh());

        Ok(h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_ntm_controller() -> Result<()> {
        let controller = NTMController::new(10, 5, 20, 1, 1)?;
        let input = Array1::random(5, Uniform::new(0., 1.));
        
        let output = controller.forward(&input)?;
        
        assert_eq!(output.len(), 20 + 5);  // controller_size + read_vector_size
        
        Ok(())
    }

    #[test]
    fn test_lstm() {
        let lstm = LSTM::new(10, 20, 30);
        let input = Array1::random(10, Uniform::new(0., 1.));
        
        let output = lstm.forward(&input).unwrap();
        
        assert_eq!(output.len(), 20);
        assert!(output.iter().all(|&x| x >= -1.0 && x <= 1.0));
    }
}
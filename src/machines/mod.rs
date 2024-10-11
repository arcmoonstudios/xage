// src/machines/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[MACHINES]Xyn>=====S===t===u===d===i===o===s======[R|$>

pub mod liquid_state_machine;
pub mod neural_turing_machine;

use crate::lsm::Reservoir;
use crate::ntm::NTM;

pub struct HybridMachine {
    lsm: Reservoir,
    ntm: NTM,
}

impl HybridMachine {
    pub fn new(input_size: usize, reservoir_size: usize, memory_size: usize) -> Self {
        Self {
            lsm: Reservoir::new(input_size, reservoir_size),
            ntm: NTM::new(reservoir_size, memory_size),
        }
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let lsm_output = self.lsm.process(input);
        self.ntm.process(&lsm_output)
    }
}
// src/expert/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[EXPERT]Xyn>=====S===t===u===d===i===o===s======[R|$>
// src/expert/mod.rs

pub mod bevy_expert;
pub mod research_expert;
pub mod rust_expert;
pub mod solana_expert;

use crate::omnixtracker::OmniXError;
use bevy_expert::BevyExpert;
use research_expert::ResearchExpert;
use rust_expert::RustExpert;
use solana_expert::SolanaExpert;

pub trait Expert {
    fn process_task(&self, task: &str) -> Result<String, OmniXError>;
}

pub struct ExpertSystem {
    bevy_expert: BevyExpert,
    research_expert: ResearchExpert,
    rust_expert: RustExpert,
    solana_expert: SolanaExpert,
}

impl ExpertSystem {
    pub fn new() -> Self {
        Self {
            bevy_expert: BevyExpert::new(),
            research_expert: ResearchExpert::new(),
            rust_expert: RustExpert::new(),
            solana_expert: SolanaExpert::new(
                "https://api.devnet.solana.com",
                "https://devnet.solana.com",
            ),
        }
    }

    pub fn process_task(&self, task: &str) -> Result<String, OmniXError> {
        let (expert, task) = self.route(task)?;
        expert.process_task(task)
    }

    fn route<'a>(&'a self, task: &'a str) -> Result<(&'a dyn Expert, &'a str), OmniXError> {
        let parts: Vec<&str> = task.splitn(2, ':').collect();
        match parts.as_slice() {
            ["bevy", rest] => Ok((&self.bevy_expert as &dyn Expert, rest)),
            ["research", rest] => Ok((&self.research_expert as &dyn Expert, rest)),
            ["rust", rest] => Ok((&self.rust_expert as &dyn Expert, rest)),
            ["solana", rest] => Ok((&self.solana_expert as &dyn Expert, rest)),
            _ => Err(OmniXError::InvalidInput(format!("Unknown expert for task: {}", task))),
        }
    }
}
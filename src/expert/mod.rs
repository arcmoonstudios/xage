// src/expert/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[EXPERT]Xyn>=====S===t===u===d===i===o===s======[R|$>

pub mod bevy_expert;
pub mod research_expert;
pub mod rust_expert;
pub mod router;
pub mod solana_expert;

use crate::omnixtracker::OmniXError;

pub trait Expert {
    fn process_task(&self, task: &str) -> Result<String, OmniXError>;
}

pub struct ExpertSystem {
    experts: Vec<Box<dyn Expert>>,
    router: router::Router,
}

impl ExpertSystem {
    pub fn new() -> Self {
        let experts: Vec<Box<dyn Expert>> = vec![
            Box::new(bevy_expert::BevyExpert::new()),
            Box::new(research_expert::ResearchExpert::new()),
            Box::new(solana_expert::SolanaExpert::new(
                "https://api.devnet.solana.com",
                "https://devnet.solana.com",
            )),
            Box::new(rust_expert::RustExpert::new()),
        ];
        Self {
            experts,
            router: router::Router::new(),
        }
    }

    pub fn process_task(&self, task: &str) -> Result<String, OmniXError> {
        let expert_index = self.router.route(task)?;
        self.experts[expert_index].process_task(task)
    }
}
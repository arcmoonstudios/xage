// src/multi_modal/text_processor.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[MULTI-MODAL]Xyn>=====S===t===u===d===i===o===s======[R|$>

use tokenizers::Tokenizer;
use rust_bert::bert::{BertModel, BertConfig};
use tch::{nn, Tensor};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextProcessingError {
    #[error("Tokenization error: {0}")]
    TokenizationError(String),
    #[error("Model error: {0}")]
    ModelError(String),
}

pub struct TextProcessor {
    tokenizer: Tokenizer,
    model: BertModel,
}

impl TextProcessor {
    pub fn new() -> Result<Self, TextProcessingError> {
        let tokenizer = Tokenizer::from_pretrained("bert-base-uncased", None)
            .map_err(|e| TextProcessingError::TokenizationError(e.to_string()))?;
        
        let config = BertConfig::from_file("path/to/bert_config.json")
            .map_err(|e| TextProcessingError::ModelError(e.to_string()))?;
        let model = BertModel::new(&nn::VarStore::new(tch::Device::Cpu), &config)
            .map_err(|e| TextProcessingError::ModelError(e.to_string()))?;
        
        Ok(TextProcessor { tokenizer, model })
    }

    pub fn process(&self, text: &str) -> Result<Tensor, TextProcessingError> {
        let encoding = self.tokenizer.encode(text, true)
            .map_err(|e| TextProcessingError::TokenizationError(e.to_string()))?;
        
        let input_ids = Tensor::of_slice(&encoding.get_ids());
        let attention_mask = Tensor::of_slice(&encoding.get_attention_mask());
        
        let (_, pooled_output) = self.model.forward_t(
            Some(&input_ids),
            Some(&attention_mask),
            None,
            None,
            None,
            false,
        ).map_err(|e| TextProcessingError::ModelError(e.to_string()))?;
        
        Ok(pooled_output)
    }
}
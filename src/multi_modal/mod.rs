// src/multi_modal/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[MULTI-MODAL]Xyn>=====S===t===u===d===i===o===s======[R|$>

pub mod text_processor;
pub mod image_processor;
pub mod audio_processor;
pub mod fusion;

pub use text_processor::TextProcessor;
pub use image_processor::ImageProcessor;
pub use audio_processor::AudioProcessor;
pub use fusion::ModalityFusion;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MultiModalError {
    #[error("Text processing error: {0}")]
    TextError(#[from] text_processor::TextProcessingError),
    #[error("Image processing error: {0}")]
    ImageError(#[from] image_processor::ImageProcessingError),
    #[error("Audio processing error: {0}")]
    AudioError(#[from] audio_processor::AudioProcessingError),
    #[error("Fusion error: {0}")]
    FusionError(#[from] fusion::FusionError),
}
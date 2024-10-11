// src/multi_modal/audio_processor.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[MULTI-MODAL]Xyn>=====S===t===u===d===i===o===s======[R|$>

use tch::{nn, Tensor};
use rustfft::{FftPlanner, num_complex::Complex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioProcessingError {
    #[error("Audio loading error: {0}")]
    AudioLoadError(String),
    #[error("FFT error: {0}")]
    FftError(String),
    #[error("Model error: {0}")]
    ModelError(String),
}

pub struct AudioProcessor {
    model: nn::Sequential,
}

impl AudioProcessor {
    pub fn new() -> Result<Self, AudioProcessingError> {
        let vs = nn::VarStore::new(tch::Device::Cpu);
        let model = nn::seq()
            .add(nn::conv1d(&vs.root(), 1, 32, 3, Default::default()))
            .add_fn(|x| x.relu())
            .add(nn::conv1d(&vs.root(), 32, 64, 3, Default::default()))
            .add_fn(|x| x.relu())
            .add(nn::conv1d(&vs.root(), 64, 128, 3, Default::default()))
            .add_fn(|x| x.relu())
            .add_fn(|x| x.adaptive_avg_pool1d(&[1]))
            .add_fn(|x| x.flat_view());
        
        Ok(AudioProcessor { model })
    }

    pub fn process(&self, audio_path: &str) -> Result<Tensor, AudioProcessingError> {
        let samples = self.load_audio(audio_path)?;
        let spectrogram = self.compute_spectrogram(&samples)?;
        let tensor = Tensor::of_slice(&spectrogram).view([1, 1, -1]);
        let output = self.model.forward_t(&tensor, false)
            .map_err(|e| AudioProcessingError::ModelError(e.to_string()))?;
        Ok(output)
    }

    fn load_audio(&self, audio_path: &str) -> Result<Vec<f32>, AudioProcessingError> {
        // Implement audio loading logic here
        // This is a placeholder implementation
        Ok(vec![0.0; 44100])
    }

    fn compute_spectrogram(&self, samples: &[f32]) -> Result<Vec<f32>, AudioProcessingError> {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(samples.len());
        
        let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&x| Complex::new(x, 0.0)).collect();
        fft.process(&mut buffer);
        
        let spectrogram: Vec<f32> = buffer.into_iter().map(|c| c.norm()).collect();
        Ok(spectrogram)
    }
}
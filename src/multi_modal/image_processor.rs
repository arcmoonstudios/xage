// src/multi_modal/image_processor.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[MULTI-MODAL]Xyn>=====S===t===u===d===i===o===s======[R|$>

use image::{GenericImageView, DynamicImage};
use tch::{nn, vision, Tensor};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageProcessingError {
    #[error("Image loading error: {0}")]
    ImageLoadError(#[from] image::ImageError),
    #[error("Model error: {0}")]
    ModelError(String),
}

pub struct ImageProcessor {
    model: vision::resnet::ResNet18,
}

impl ImageProcessor {
    pub fn new() -> Result<Self, ImageProcessingError> {
        let vs = nn::VarStore::new(tch::Device::Cpu);
        let model = vision::resnet::resnet18(&vs.root(), 1000)
            .map_err(|e| ImageProcessingError::ModelError(e.to_string()))?;
        Ok(ImageProcessor { model })
    }

    pub fn process(&self, image_path: &str) -> Result<Tensor, ImageProcessingError> {
        let image = image::open(image_path)?;
        let tensor = self.image_to_tensor(&image)?;
        let output = self.model.forward_t(&tensor, false)
            .map_err(|e| ImageProcessingError::ModelError(e.to_string()))?;
        Ok(output)
    }

    fn image_to_tensor(&self, image: &DynamicImage) -> Result<Tensor, ImageProcessingError> {
        let resized = image.resize_exact(224, 224, image::imageops::FilterType::Lanczos3);
        let mut tensor = Tensor::of_shape([1, 3, 224, 224], tch::Kind::Float);
        for (i, pixel) in resized.pixels().enumerate() {
            let x = (i % 224) as i64;
            let y = (i / 224) as i64;
            tensor.narrow(2, y, 1).narrow(3, x, 1).copy_(&Tensor::of_slice(&[
                pixel.2 as f32 / 255.0,
                pixel.1 as f32 / 255.0,
                pixel.0 as f32 / 255.0,
            ]));
        }
        Ok(tensor)
    }
}
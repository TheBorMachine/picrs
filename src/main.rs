mod errors;
mod processor;

use processor::{ProcessorConfig, process_image};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::image::image_processor_server::{ImageProcessor, ImageProcessorServer};
use crate::image::{ProcessRequest, ProcessResponse};

pub mod image {
    tonic::include_proto!("image");
}

#[derive(Default)]
pub struct ImageService {
    #[allow(dead_code)]
    config: ProcessorConfig,
}

impl ImageService {
    pub fn new(config: ProcessorConfig) -> Self {
        Self { config }
    }

    fn validate_request(&self, req: &ProcessRequest) -> Result<(), Status> {
        if req.image_data.is_empty() {
            return Err(Status::invalid_argument("Image data is empty"));
        }

        if req.widths.len() != req.heights.len() {
            return Err(Status::invalid_argument(
                "Widths and heights arrays must have same length",
            ));
        }

        for (i, (w, h)) in req.widths.iter().zip(req.heights.iter()).enumerate() {
            if *w == 0 || *h == 0 {
                return Err(Status::invalid_argument(format!(
                    "Dimension at index {} is zero: {}x{}",
                    i, w, h
                )));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl ImageProcessor for ImageService {
    async fn process_image(
        &self,
        request: Request<ProcessRequest>,
    ) -> Result<Response<ProcessResponse>, Status> {
        let req = request.into_inner();

        self.validate_request(&req)?;

        let result = process_image(req.image_data, req.format, req.widths, req.heights)
            .await
            .map_err(|e| match e {
                errors::ServiceError::InvalidFormat { format } => {
                    Status::invalid_argument(format!("Invalid format: {}", format))
                }
                errors::ServiceError::InvalidParameters { message } => {
                    Status::invalid_argument(message)
                }
                errors::ServiceError::ImageLoadError { message } => {
                    Status::invalid_argument(format!("Failed to load image: {}", message))
                }
                errors::ServiceError::ImageSaveError { message } => {
                    Status::internal(format!("Failed to save image: {}", message))
                }
                errors::ServiceError::InternalError { message } => Status::internal(message),
            })?;

        Ok(Response::new(ProcessResponse {
            original_path: result.original_path,
            processed_paths: result.processed_paths,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let config = ProcessorConfig::default();
    let service = ImageService::new(config);

    println!("ImageProcessor listening on {}", addr);

    Server::builder()
        .add_service(ImageProcessorServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_service_new() {
        let config = ProcessorConfig::default();
        let service = ImageService::new(config);
        assert_eq!(
            service.config.base_dir,
            std::path::PathBuf::from("download")
        );
    }

    #[test]
    fn test_validate_request_empty_image() {
        let service = ImageService::default();
        let req = ProcessRequest {
            image_data: Vec::new(),
            format: "png".to_string(),
            quality: 85,
            widths: vec![100],
            heights: vec![100],
        };
        assert!(service.validate_request(&req).is_err());
    }

    #[test]
    fn test_validate_request_mismatched_dimensions() {
        let service = ImageService::default();
        let req = ProcessRequest {
            image_data: vec![1, 2, 3],
            format: "png".to_string(),
            quality: 85,
            widths: vec![100, 200],
            heights: vec![100],
        };
        assert!(service.validate_request(&req).is_err());
    }

    #[test]
    fn test_validate_request_zero_dimensions() {
        let service = ImageService::default();
        let req = ProcessRequest {
            image_data: vec![1, 2, 3],
            format: "png".to_string(),
            quality: 85,
            widths: vec![0],
            heights: vec![100],
        };
        assert!(service.validate_request(&req).is_err());
    }

    #[test]
    fn test_validate_request_valid() {
        let service = ImageService::default();
        let req = ProcessRequest {
            image_data: vec![1, 2, 3],
            format: "png".to_string(),
            quality: 85,
            widths: vec![100, 200],
            heights: vec![100, 150],
        };
        assert!(service.validate_request(&req).is_ok());
    }
}

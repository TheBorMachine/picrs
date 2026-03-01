use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Internal service error: {message}")]
    InternalError { message: String },

    #[error("Invalid image format: {format}")]
    InvalidFormat { format: String },

    #[error("Failed to load image: {message}")]
    ImageLoadError { message: String },

    #[error("Failed to save image: {message}")]
    ImageSaveError { message: String },

    #[error("Invalid processing parameters: {message}")]
    InvalidParameters { message: String },
}

pub type ServiceResult<T> = Result<T, ServiceError>;

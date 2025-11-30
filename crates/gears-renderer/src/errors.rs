use thiserror::Error;

/// Errors that can occur in the renderer.
#[derive(Debug, Error)]
pub enum RendererError {
    /// Failed to convert a texture.
    #[error("Failed to convert texture: {0}")]
    TextureConversion(String),
    /// Resource loading operation failed.
    #[error("Resource loading failed: {0}")]
    ResourceLoadingFailed(String),
    /// Requested resource was not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    /// IO operation failed.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Image processing operation failed.
    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),
    /// GLTF model loading failed.
    #[error("GLTF loading error: {0}")]
    Gltf(#[from] gltf::Error),
    /// OBJ model loading failed.
    #[error("OBJ model loading error: {0}")]
    ObjLoad(#[from] tobj::LoadError),
    /// Required data is missing.
    #[error("Missing required data: {0}")]
    MissingData(String),
    /// File path is invalid.
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
    /// Model conversion failed.
    #[error("Model conversion error: {0}")]
    ModelConversion(String),
    /// Animation loading or processing failed.
    #[error("Animation loading error: {0}")]
    AnimationError(String),
    /// Other unspecified error.
    #[error("{0}")]
    Other(String),
}

impl From<&str> for RendererError {
    fn from(s: &str) -> Self {
        RendererError::Other(s.to_string())
    }
}

impl From<String> for RendererError {
    fn from(s: String) -> Self {
        RendererError::Other(s)
    }
}

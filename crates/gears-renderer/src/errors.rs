use thiserror::Error;

/// Errors that can occur in the renderer
#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Failed to convert texture: {0}")]
    TextureConversion(String),

    #[error("Resource loading failed: {0}")]
    ResourceLoadingFailed(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("GLTF loading error: {0}")]
    Gltf(#[from] gltf::Error),

    #[error("OBJ model loading error: {0}")]
    ObjLoad(#[from] tobj::LoadError),

    #[error("Missing required data: {0}")]
    MissingData(String),

    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    #[error("Model conversion error: {0}")]
    ModelConversion(String),

    #[error("Animation loading error: {0}")]
    AnimationError(String),

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

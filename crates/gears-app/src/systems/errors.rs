use thiserror::Error;

/// Errors that can occur in systems
#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Missing component: {0}")]
    MissingComponent(String),

    #[error("Failed to access component: {0}")]
    ComponentAccess(String),

    #[error("Animation error: {0}")]
    Animation(String),

    #[error("Buffer operation failed: {0}")]
    BufferOperation(String),

    #[error("System execution error: {0}")]
    Execution(String),

    #[error("{0}")]
    Other(String),
}

pub type SystemResult<T> = Result<T, SystemError>;

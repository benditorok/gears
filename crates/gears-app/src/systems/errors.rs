use thiserror::Error;

/// Errors that can occur in systems.
#[derive(Debug, Error)]
pub enum SystemError {
    /// Missing required component in the ECS.
    #[error("Missing component: {0}")]
    MissingComponent(String),
    /// Failed to access a component in the ECS.
    #[error("Failed to access component: {0}")]
    ComponentAccess(String),
    /// Animation related error.
    #[error("Animation error: {0}")]
    Animation(String),
    /// Buffer operation error.
    #[error("Buffer operation failed: {0}")]
    BufferOperation(String),
    /// System execution error.
    #[error("System execution error: {0}")]
    Execution(String),
    /// Other unspecified error.
    #[error("{0}")]
    Other(String),
}

/// Type alias for a result type that can contain an [`SystemError`].
pub type SystemResult<T> = Result<T, SystemError>;

use gears_renderer::errors::RendererError;
use thiserror::Error;

/// Errors that can occur during the execution of the gears engine.
#[derive(Debug, Error)]
pub enum EngineError {
    /// Missing required component in the ECS.
    #[error("Missing component: {0}")]
    MissingComponent(String),
    /// Winit initialization error.
    #[error("Winit initialization error: {0}")]
    WinitInitialization(#[from] winit::error::OsError),
    /// Event loop error from winit.
    #[error("Event loop error: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
    /// Renderer-related errors.
    #[error("Renderer error: {0}")]
    Renderer(#[from] RendererError),
    /// Component initialization failure.
    #[error("Component initialization failed: {0}")]
    ComponentInitialization(String),
    /// System execution failure.
    #[error("System execution error: {0}")]
    SystemExecution(String),
    /// State update failure.
    #[error("State update error: {0}")]
    StateUpdate(String),
    /// Resource management error.
    #[error("Resource management error: {0}")]
    ResourceManagement(String),
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),
    /// ECS-related errors.
    #[error("ECS error: {0}")]
    Ecs(String),
    /// Threading-related errors.
    #[error("Threading error: {0}")]
    Threading(String),
    /// Other unspecified error.
    #[error("{0}")]
    Other(String),
}

/// Type alias for a result type that can contain an [`EngineError`].
pub type EngineResult<T> = Result<T, EngineError>;

impl From<&str> for EngineError {
    fn from(s: &str) -> Self {
        EngineError::Other(s.to_string())
    }
}

impl From<String> for EngineError {
    fn from(s: String) -> Self {
        EngineError::Other(s)
    }
}

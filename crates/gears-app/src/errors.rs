use gears_renderer::errors::RendererError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Missing component: {0}")]
    MissingComponent(String),

    #[error("Winit initialization error: {0}")]
    WinitInitialization(#[from] winit::error::OsError),

    #[error("Event loop error: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),

    #[error("Renderer error: {0}")]
    Renderer(#[from] RendererError),

    #[error("Component initialization failed: {0}")]
    ComponentInitialization(String),

    #[error("System execution error: {0}")]
    SystemExecution(String),

    #[error("State update error: {0}")]
    StateUpdate(String),

    #[error("Resource management error: {0}")]
    ResourceManagement(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("ECS error: {0}")]
    Ecs(String),

    #[error("Threading error: {0}")]
    Threading(String),

    #[error("{0}")]
    Other(String),
}

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

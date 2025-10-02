use thiserror::Error;

/// Result type for BAML operations
pub type BamlResult<T> = std::result::Result<T, BamlError>;

/// Main error type for BAML operations
#[derive(Debug, Error)]
pub enum BamlError {
    #[error("Runtime error: {0}")]
    Runtime(#[from] anyhow::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Context error: {0}")]
    Context(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Error type categories for programmatic handling
#[derive(Debug, Clone, PartialEq)]
pub enum BamlErrorType {
    Runtime,
    Serialization,
    Deserialization,
    FunctionNotFound,
    InvalidArgument,
    Timeout,
    Stream,
    Context,
    Configuration,
}

impl BamlError {
    /// Get the error type for programmatic handling
    pub fn error_type(&self) -> BamlErrorType {
        match self {
            BamlError::Runtime(_) => BamlErrorType::Runtime,
            BamlError::Serialization(_) => BamlErrorType::Serialization,
            BamlError::Deserialization(_) => BamlErrorType::Deserialization,
            BamlError::FunctionNotFound(_) => BamlErrorType::FunctionNotFound,
            BamlError::InvalidArgument(_) => BamlErrorType::InvalidArgument,
            BamlError::Timeout { .. } => BamlErrorType::Timeout,
            BamlError::Stream(_) => BamlErrorType::Stream,
            BamlError::Context(_) => BamlErrorType::Context,
            BamlError::Configuration(_) => BamlErrorType::Configuration,
        }
    }

    /// Create a serialization error
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        BamlError::Serialization(msg.into())
    }

    /// Create a deserialization error
    pub fn deserialization<S: Into<String>>(msg: S) -> Self {
        BamlError::Deserialization(msg.into())
    }

    /// Create a function not found error
    pub fn function_not_found<S: Into<String>>(name: S) -> Self {
        BamlError::FunctionNotFound(name.into())
    }

    /// Create an invalid argument error
    pub fn invalid_argument<S: Into<String>>(msg: S) -> Self {
        BamlError::InvalidArgument(msg.into())
    }
}

impl From<serde_json::Error> for BamlError {
    fn from(err: serde_json::Error) -> Self {
        BamlError::Serialization(err.to_string())
    }
}

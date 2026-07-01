use thiserror::Error;

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    FileNotFound(String),

    #[error("Failed to read config file: {0}")]
    FileReadError(String),

    #[error("Failed to write config file: {0}")]
    FileWriteError(String),

    #[error("Invalid JSON in config file: {0}")]
    JsonParseError(String),

    #[error("Environment variable not found: {0}")]
    EnvVariableNotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

use serde_json::Value;
use thiserror::Error;

/// Runtime error types
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Initialization error: {0}")]
    InitializationError(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    #[error("LLM error: {0}")]
    LLMError(String),
    
    #[error("Tool error: {0}")]
    ToolError(String),
    
    #[error("Session error: {0}")]
    SessionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Skill error: {0}")]
    SkillError(String),
    
    #[error("MCP error: {0}")]
    MCPError(String),
    
    #[error("Max tool call iterations reached: {0}")]
    MaxIterationsError(usize),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl From<crate::llm::types::LLMError> for RuntimeError {
    fn from(err: crate::llm::types::LLMError) -> Self {
        RuntimeError::LLMError(err.to_string())
    }
}

impl From<crate::tools::types::ToolError> for RuntimeError {
    fn from(err: crate::tools::types::ToolError) -> Self {
        RuntimeError::ToolError(err.to_string())
    }
}

/// Chat response from the agent
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Vec<Value>,
    pub finish_reason: Option<String>,
    pub usage: Option<Value>,
}

impl ChatResponse {
    pub fn new(content: String) -> Self {
        Self {
            content,
            tool_calls: Vec::new(),
            finish_reason: None,
            usage: None,
        }
    }
    
    pub fn with_tool_calls(mut self, tool_calls: Vec<Value>) -> Self {
        self.tool_calls = tool_calls;
        self
    }
    
    pub fn with_finish_reason(mut self, finish_reason: String) -> Self {
        self.finish_reason = Some(finish_reason);
        self
    }
    
    pub fn with_usage(mut self, usage: Value) -> Self {
        self.usage = Some(usage);
        self
    }
}

/// Logger trait for runtime logging
pub trait Logger {
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
    fn debug(&self, message: &str);
}

/// Simple console logger implementation
#[derive(Default)]
pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn info(&self, message: &str) {
        println!("[INFO] {}", message);
    }
    
    fn warn(&self, message: &str) {
        println!("[WARN] {}", message);
    }
    
    fn error(&self, message: &str) {
        eprintln!("[ERROR] {}", message);
    }
    
    fn debug(&self, message: &str) {
        println!("[DEBUG] {}", message);
    }
}

/// No-op logger for testing
#[derive(Default)]
pub struct NoOpLogger;

impl Logger for NoOpLogger {
    fn info(&self, _message: &str) {}
    fn warn(&self, _message: &str) {}
    fn error(&self, _message: &str) {}
    fn debug(&self, _message: &str) {}
}

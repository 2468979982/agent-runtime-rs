// Agent Runtime RS - Library Entry Point
// This library provides AI Agent runtime capabilities including:
// - LLM integration (OpenAI-compatible APIs)
// - Tool management and execution
// - Session management
// - Skill system
// - MCP (Model Context Protocol) integration

// Declare modules (matching existing directory structure)
pub mod error;
pub mod runtime;
pub mod config;
pub mod llm;
pub mod tools;
pub mod session;
pub mod skill;
pub mod mcp;
pub mod utils;
pub mod api;

// Re-export key types for convenience
pub use runtime::agent::AgentRuntime;
pub use config::loader::ConfigLoader;
pub use llm::connector::LLMConnector;
pub use tools::manager::ToolManager;
pub use session::manager::SessionManager;

// Note: SkillManager might not exist, depending on skill module structure
// pub use skill::manager::SkillManager;

// Re-export common types (using correct paths)
pub use runtime::types::{RuntimeError, Logger, ConsoleLogger};
pub use config::types::{AgentConfig, LLMConfig, SessionConfig, ToolsConfig};
pub use llm::types::{ChatMessage, MessageRole, ChatCompletionResponse, ToolDefinition, LLMError};
pub use tools::types::{ToolParameter, ToolResult, ToolError};
pub use session::types::{Session, Message};
pub use skill::types::Skill;

// Re-export API types
pub use api::types::{RunRequest, RunResponse};
pub use api::handlers::{run_handler, tool_call_handler};

// Public API functions
use std::sync::Arc;

/// Create a new AgentRuntime instance with configuration
///
/// # Arguments
/// * `agent_config_path` - Path to agent configuration file (e.g., "config/agent-config.json")
/// * `tools_config_path` - Path to tools configuration file (e.g., "config/tools-config.json")
/// * `prompt_config_path` - Path to prompt configuration file (e.g., "config/prompt-config.json")
///
/// # Returns
/// A shared, thread-safe AgentRuntime instance wrapped in Arc<Mutex<>>
///
/// # Example
/// ```no_run
/// use agent_runtime_rs::create_agent_runtime;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let runtime = create_agent_runtime(
///         "config/agent-config.json",
///         "config/tools-config.json",
///         "config/prompt-config.json",
///     ).await?;
///     
///     // Use runtime...
///     
///     Ok(())
/// }
/// ```
pub async fn create_agent_runtime(
    agent_config_path: &str,
    tools_config_path: &str,
    prompt_config_path: &str,
) -> anyhow::Result<Arc<runtime::agent::AgentRuntime>> {
    // Create a simple console logger
    let logger: Option<Box<dyn Logger + Send + Sync>> = Some(Box::new(ConsoleLogger::default()));
    
    let mut runtime = runtime::agent::AgentRuntime::new(logger);
    
    runtime.initialize(
        agent_config_path,
        tools_config_path,
        prompt_config_path,
    ).await?;
    
    Ok(Arc::new(runtime))
}

/// Create a new AgentRuntime instance with custom logger
///
/// # Arguments
/// * `logger` - Custom logger implementation
/// * `agent_config_path` - Path to agent configuration file
/// * `tools_config_path` - Path to tools configuration file
/// * `prompt_config_path` - Path to prompt configuration file
///
/// # Returns
/// A shared, thread-safe AgentRuntime instance wrapped in Arc<Mutex<>>
pub async fn create_agent_runtime_with_logger(
    logger: Box<dyn Logger + Send + Sync>,
    agent_config_path: &str,
    tools_config_path: &str,
    prompt_config_path: &str,
) -> anyhow::Result<Arc<runtime::agent::AgentRuntime>> {
    let mut runtime = runtime::agent::AgentRuntime::new(Some(logger));
    
    runtime.initialize(
        agent_config_path,
        tools_config_path,
        prompt_config_path,
    ).await?;
    
    Ok(Arc::new(runtime))
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Crate description
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
    }
}

pub mod config;
pub mod utils;
pub mod error;
pub mod tools;
pub mod llm;
pub mod session;
pub mod mcp;
pub mod skill;
pub mod runtime;
pub mod api;

pub use error::*;
pub use runtime::types::RuntimeError;
pub use runtime::agent::AgentRuntime;
pub use runtime::types::Logger;
pub use api::*;

/// Create a new AgentRuntime instance with the specified configurations
///
/// # Arguments
///
/// * `agent_config_path` - Path to the agent configuration file
/// * `tools_config_path` - Path to the tools configuration file
/// * `prompt_config_path` - Path to the prompt configuration file
/// * `logger` - Optional logger for runtime events
///
/// # Returns
///
/// Returns a Result containing the initialized AgentRuntime or an error
pub async fn create_agent_runtime(
    agent_config_path: &str,
    tools_config_path: &str,
    prompt_config_path: &str,
    logger: Option<Box<dyn Logger + Send + Sync>>,
) -> std::result::Result<AgentRuntime, RuntimeError> {
    let mut runtime = AgentRuntime::new(logger);
    
    runtime.initialize(
        agent_config_path,
        tools_config_path,
        prompt_config_path,
    ).await?;
    
    Ok(runtime)
}

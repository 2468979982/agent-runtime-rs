use tracing::info;

use crate::config::loader::ConfigLoader;
use crate::llm::connector::LLMConnector;
use crate::tools::manager::ToolManager;
use crate::session::manager::SessionManager;
use crate::runtime::types::*;

/// AgentRuntime - Core runtime that integrates all components
#[allow(dead_code)]
pub struct AgentRuntime {
    llm_connector: Option<LLMConnector>,
    tool_manager: ToolManager,
    session_manager: SessionManager,
    logger: Box<dyn Logger + Send + Sync>,
    initialized: bool,
}

impl AgentRuntime {
    /// Create a new AgentRuntime instance
    pub fn new(logger: Option<Box<dyn Logger + Send + Sync>>) -> Self {
        let logger = logger.unwrap_or_else(|| Box::new(ConsoleLogger::default()));
        
        Self {
            llm_connector: None,
            tool_manager: ToolManager::new(),
            session_manager: SessionManager::new(crate::session::types::SessionConfig {
                max_history_length: 100,
                session_ttl: None,
            }),
            logger,
            initialized: false,
        }
    }
    
    /// Initialize the agent runtime with all components
    pub async fn initialize(
        &mut self,
        agent_config_path: &str,
        _tools_config_path: &str,
        _prompt_config_path: &str,
    ) -> Result<(), RuntimeError> {
        info!("Initializing AgentRuntime...");
        
        // Load agent config
        let agent_config = ConfigLoader::load_agent_config(agent_config_path)
            .map_err(|e| RuntimeError::ConfigError(e.to_string()))?;
        
        // Initialize LLM connector
        let llm_config = crate::llm::types::LLMConfig {
            provider: "openai".to_string(),
            api_key: agent_config.llm.api_key.clone(),
            base_url: agent_config.llm.base_url.clone().unwrap_or_default(),
            model: agent_config.llm.model,
            temperature: agent_config.llm.temperature,
            max_tokens: agent_config.llm.max_tokens,
        };
        
        self.llm_connector = Some(LLMConnector::new(&llm_config)
            .map_err(|e| RuntimeError::LLMError(e.to_string()))?);
        
        self.initialized = true;
        info!("AgentRuntime initialized successfully");
        
        Ok(())
    }
    
    /// Check if runtime is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Get a reference to the LLM connector
    pub fn get_llm_connector(&self) -> Option<&LLMConnector> {
        self.llm_connector.as_ref()
    }
    
    /// Get a reference to the tool manager
    pub fn get_tool_manager(&self) -> &ToolManager {
        &self.tool_manager
    }
    
    /// Get a reference to the session manager
    pub fn get_session_manager(&self) -> &SessionManager {
        &self.session_manager
    }
    
    /// Get a mutable reference to the session manager
    pub fn get_session_manager_mut(&mut self) -> &mut SessionManager {
        &mut self.session_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_runtime_creation() {
        let runtime = AgentRuntime::new(None);
        assert!(!runtime.is_initialized());
    }
}

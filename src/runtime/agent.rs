use tracing::{info, warn};

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::mcp::client::MCPClient;

use crate::config::loader::ConfigLoader;
use crate::llm::connector::LLMConnector;
use crate::tools::manager::ToolManager;
use crate::session::manager::SessionManager;
use crate::skill::types::SkillManager;
use crate::runtime::types::*;

/// AgentRuntime - Core runtime that integrates all components
#[allow(dead_code)]
pub struct AgentRuntime {
    llm_connector: Option<LLMConnector>,
    tool_manager: ToolManager,
    session_manager: SessionManager,
    skill_manager: Option<SkillManager>,
    logger: Box<dyn Logger + Send + Sync>,
    initialized: bool,
    
    // Agent configuration content (loaded from agent-config/ directory)
    agent_config_content: Option<String>,
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
            skill_manager: None,
            logger,
            initialized: false,
            agent_config_content: None,  // Initialize as None
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
        
        // Debug: print LLM config
        tracing::debug!("LLM config from agent-config.json: api_key prefix={}..., base_url={:?}, model={}",
            &agent_config.llm.api_key[..20.min(agent_config.llm.api_key.len())],
            agent_config.llm.base_url,
            agent_config.llm.model
        );
        
        // Initialize LLM connector
        let llm_config = crate::llm::types::LLMConfig {
            provider: "openai".to_string(),
            api_key: agent_config.llm.api_key.clone(),
            base_url: agent_config.llm.base_url.clone().ok_or_else(|| {
                RuntimeError::ConfigError("base_url is required in LLM config".to_string())
            })?,
            model: agent_config.llm.model,
            temperature: agent_config.llm.temperature,
            max_tokens: agent_config.llm.max_tokens,
        };
        
        self.llm_connector = Some(LLMConnector::new(&llm_config)
            .map_err(|e| RuntimeError::LLMError(e.to_string()))?);
        
        // Register builtin tools
        info!("Registering builtin tools...");
        self.register_builtin_tools()?;
        
        // Load MCP servers (if configured)
        if !_tools_config_path.is_empty() {
            info!("Loading MCP servers from: {}", _tools_config_path);
            self.load_mcp_servers(_tools_config_path).await?;
        }
        
        // Load skills (if configured)
        if let Some(skills_config) = &agent_config.skills {
            let skills_folder = skills_config.skills_folder.clone().unwrap_or_else(|| "./skills".to_string());
            let auto_load = skills_config.auto_load_skills.unwrap_or(true);
            
            info!("Loading skills from: {}", skills_folder);
            
            let mut skill_manager = SkillManager::new(&skills_folder, auto_load);
            
            match skill_manager.load_all_skills() {
                Ok(skills) => {
                    info!("Loaded {} skills", skills.len());
                    for skill in &skills {
                        info!("  - {}", skill.metadata.name);
                    }
                    self.skill_manager = Some(skill_manager);
                }
                Err(e) => {
                    warn!("Failed to load skills: {}", e);
                }
            }
        }
        
        // Load agent-config/ files (SOUL.md, IDENTITY.md, AGENTS.md, etc.)
        info!("Loading agent-config files...");
        self.load_agent_config_files();
        
        self.initialized = true;
        info!("AgentRuntime initialized successfully");
        info!("Registered tools: {:?}", self.tool_manager.get_tool_names());
        
        Ok(())
    }
    
    /// Check if runtime is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Register builtin tools
    fn register_builtin_tools(&mut self) -> Result<(), RuntimeError> {
        // Use the built-in function to register all tools
        crate::tools::builtin::register_builtin_tools(&mut self.tool_manager);
        Ok(())
    }
    
    /// Load MCP servers from tools config
    async fn load_mcp_servers(&mut self, tools_config_path: &str) -> Result<(), RuntimeError> {
        // Load tools config
        let tools_config = crate::config::loader::ConfigLoader::load_tools_config(tools_config_path)
            .map_err(|e| RuntimeError::ConfigError(format!("Failed to load tools config: {}", e)))?;
        
        // Check if MCP servers are configured
        match &tools_config.mcp_servers {
            crate::config::types::MCPServers::New(servers) => {
                for (name, config) in servers {
                    info!("Starting MCP server: {}", name);
                    
                    // Wrap in Arc<Mutex<...>> for sharing
                    let client: Arc<Mutex<Box<dyn MCPClient + Send>>> = Arc::new(Mutex::new(Box::new(
                        crate::mcp::stdio_client::MCPStdioClient::new(
                            name,
                            &config.command,
                            config.args.clone(),
                            config.env.clone().unwrap_or_default(),
                        )
                    )));
                    
                    // Initialize connection
                    let init_result = client.lock().await.initialize().await
                        .map_err(|e| RuntimeError::MCPError(format!("Failed to initialize MCP server '{}': {}", name, e)))?;
                    
                    info!("MCP server '{}' initialized: {:?}", name, init_result.server_info);
                    
                    // List available tools
                    let tools = client.lock().await.list_tools().await
                        .map_err(|e| RuntimeError::MCPError(format!("Failed to list tools from MCP server '{}': {}", name, e)))?;
                    
                    info!("MCP server '{}' provides {} tools", name, tools.len());
                    
                    // Register each tool
                    for mcp_tool in tools {
                        info!("Registering MCP tool: {}", mcp_tool.name);
                        let executor = crate::tools::builtin::MCPToolExecutor::new(client.clone(), mcp_tool);
                        self.tool_manager.register_tool(Box::new(executor));
                    }
                }
            }
            crate::config::types::MCPServers::Old(_) => {
                warn!("Old MCP config format detected. Please update to new format (object with command/args).");
            }
        }
        
        Ok(())
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
    
    /// Get a reference to the skill manager
    pub fn get_skill_manager(&self) -> Option<&SkillManager> {
        self.skill_manager.as_ref()
    }
    
    /// Load agent-config/ files and combine them into agent_config_content
    fn load_agent_config_files(&mut self) {
        let config_files = vec![
            "agent-config/SOUL.md",
            "agent-config/IDENTITY.md",
            "agent-config/AGENTS.md",
            "agent-config/MEMORY.md",
            "agent-config/USER.md",
            "agent-config/TOOLS.md",
            "agent-config/HEARTBEAT.md",
        ];
        
        let mut combined_content = String::from("# Agent Configuration\n\n");
        
        for file_path in config_files {
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    info!("Loaded agent-config file: {}", file_path);
                    combined_content.push_str(&format!("\n---\n\n## {}\n\n{}", file_path, content));
                }
                Err(e) => {
                    warn!("Failed to load agent-config file '{}': {}", file_path, e);
                }
            }
        }
        
        let content_len = combined_content.len();
        self.agent_config_content = Some(combined_content);
        info!("Agent config content loaded ({} bytes)", content_len);
    }
    
    /// Get the agent config content (for injecting into LLM system prompt)
    pub fn get_agent_config_content(&self) -> Option<&str> {
        self.agent_config_content.as_deref()
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

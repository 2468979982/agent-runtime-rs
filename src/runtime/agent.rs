use tracing::{info, warn};

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::mcp::client::MCPClient;

use crate::config::loader::ConfigLoader;
use crate::llm::connector::LLMConnector;
use crate::tools::manager::ToolManager;
use crate::session::manager::SessionManager;
use crate::session::persistence::SessionPersistenceManager;
use crate::skill::types::SkillManager;
use crate::runtime::types::*;
use crate::api::types;  // Add this line

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
    
    // Config summary threshold (from config/agent-config.json)
    config_summary_threshold: usize,
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
            agent_config_content: None,
            config_summary_threshold: 5000,
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
        
        // Read configSummaryThreshold from config file (optional field)
        self.load_config_summary_threshold(agent_config_path).await?;
        
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
        
        // Initialize session persistence manager
        if let Some(ref persistence_config) = agent_config.session.persistence {
            if persistence_config.enabled {
                info!("Initializing session persistence manager...");
                match SessionPersistenceManager::new(persistence_config) {
                    Ok(manager) => {
                        info!("Session persistence enabled: storage_path={}", 
                            manager.get_storage_path().display());
                        let manager_arc = std::sync::Arc::new(manager);
                        
                        // 设置到 SessionManager
                        self.session_manager.set_persistence_manager(manager_arc);
                        
                        // 加载持久化会话
                        if let Err(e) = self.session_manager.load_persisted_sessions().await {
                            warn!("Failed to load persisted sessions: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to initialize session persistence: {}. Persistence disabled.", e);
                    }
                }
            } else {
                info!("Session persistence is disabled in config");
            }
        } else {
            info!("No persistence config found, using default (disabled)");
        }
        
        // Generate summary of agent-config using LLM (optimization: reduce token usage)
        if let Some(ref config_content) = self.agent_config_content {
            let content_len = config_content.len();
            
            if content_len > self.config_summary_threshold && self.config_summary_threshold > 0 {
                info!("Generating summary of agent-config (original: {} bytes, threshold: {} bytes)", 
                    content_len, self.config_summary_threshold);
                
                if let Some(llm_connector) = &self.llm_connector {
                    let summary_prompt = format!(
                        "Please summarize the following agent configuration in 500-1000 words. \
                         Focus on the key aspects: personality, identity, workspace, and behavior guidelines. \
                         Keep the summary concise but informative.\n\nConfig content:\n\n{}",
                        config_content
                    );
                    
                    let summary_messages = vec![crate::llm::types::ChatMessage {
                        role: crate::llm::types::MessageRole::User,
                        content: summary_prompt,
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                    }];
                    
                    // Don't pass tools or tool_choice (summary generation doesn't need tools)
                    match llm_connector.chat_completion(summary_messages, None).await {
                        Ok(response) => {
                            if let Some(choice) = response.choices.first() {
                                let summary = choice.message.content.clone();
                                info!("Agent config summary generated ({} bytes, saved {}%)", 
                                    summary.len(), 
                                    (content_len - summary.len()) * 100 / content_len
                                );
                                self.agent_config_content = Some(summary);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to generate agent-config summary: {}. Using full config.", e);
                            // Keep full config (self.agent_config_content already set)
                        }
                    }
                }
            } else {
                if self.config_summary_threshold == 0 {
                    info!("Config summary generation skipped (threshold is 0, always use full config)");
                } else {
                    info!("Config content ({} bytes) is below threshold ({} bytes), using full config", 
                        content_len, self.config_summary_threshold);
                }
            }
        }
        
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
    
    /// Load config_summary_threshold from config file (optional field)
    async fn load_config_summary_threshold(&mut self, config_path: &str) -> Result<(), RuntimeError> {
        match std::fs::read_to_string(config_path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => {
                        if let Some(threshold) = json.get("configSummaryThreshold").and_then(|v| v.as_u64()) {
                            self.config_summary_threshold = threshold as usize;
                            info!("Loaded configSummaryThreshold from {}: {} bytes", config_path, self.config_summary_threshold);
                        } else {
                            warn!("configSummaryThreshold not found in {}, using default: 5000 bytes", config_path);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse {}: {}, using default threshold", config_path, e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to load {}: {}, using default threshold", config_path, e);
            }
        }
        
        Ok(())
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
    
    /// Run a chat conversation with the agent (library API, no HTTP needed)
    ///
    /// # Arguments
    /// * `session_id` - Optional session ID (if None, a new session will be created)
    /// * `message` - User message
    ///
    /// # Returns
    /// * `types::RunResponse` - Agent response (content, tool_calls, session_id, finish_reason)
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
    ///     let response = runtime.chat(
    ///         Some("my-session"),
    ///         "Hello, agent!".to_string(),
    ///     ).await?;
    ///     
    ///     println!("Agent: {}", response.content);
    ///     Ok(())
    /// }
    /// ```
    pub async fn chat(
        &self,
        session_id: Option<&str>,
        message: String,
    ) -> anyhow::Result<types::RunResponse> {
        // Create request
        let request = types::RunRequest {
            session_id: session_id.map(|s| s.to_string()),
            message,
        };
        
        // Call internal handler logic (extracted from run_handler)
        self.handle_run_request(request).await
    }
    
    /// Internal method: handle run request (core logic from run_handler)
    async fn handle_run_request(
        &self,
        request: types::RunRequest,
    ) -> anyhow::Result<types::RunResponse> {
        use crate::session::types::{Message, MessageRole};
        use crate::llm::types::ChatMessage;
        
        // Check initialization
        if !self.initialized {
            return Err(anyhow::anyhow!("AgentRuntime not initialized. Call initialize() first."));
        }
        
        // Get or create session
        let session_id = match request.session_id {
            Some(ref id) => id.clone(),
            None => {
                let session = self.session_manager.create_session()?;
                session.id
            }
        };
        
        // Ensure session exists
        if self.session_manager.get_session(&session_id).is_err() {
            let _ = self.session_manager.create_session_with_id(session_id.clone())?;
        }
        
        // Add user message to session
        let user_message = Message {
            role: MessageRole::User,
            content: request.message.clone(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };
        self.session_manager.add_message(&session_id, user_message)?;
        
        // Get conversation history
        let history = self.session_manager.get_history(&session_id)?;
        
        // Convert history to LLM messages
        let mut messages: Vec<ChatMessage> = history.iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => crate::llm::types::MessageRole::System,
                    MessageRole::User => crate::llm::types::MessageRole::User,
                    MessageRole::Assistant => crate::llm::types::MessageRole::Assistant,
                    MessageRole::Tool => crate::llm::types::MessageRole::Tool,
                };
                ChatMessage {
                    role,
                    content: msg.content.clone(),
                    name: msg.name.clone(),
                    tool_calls: None,  // TODO: convert session::types::ToolCall to llm::types::ToolCall
                    tool_call_id: msg.tool_call_id.clone(),
                }
            })
            .collect();
        
        // Inject agent-config content into system message
        if let Some(config_content) = self.get_agent_config_content() {
            tracing::info!("Injecting agent-config content into system message");
            let config_message = ChatMessage {
                role: crate::llm::types::MessageRole::System,
                content: format!("# Agent Configuration\n\nYou are an AI assistant with the following configuration:\n\n{}", config_content),
                name: Some("agent-config".to_string()),
                tool_calls: None,
                tool_call_id: None,
            };
            messages.insert(0, config_message);
        }
        
        // Check if any skill should be triggered
        let skill_used = if let Some(ref skill_manager) = self.skill_manager {
            skill_manager.find_skill_by_trigger(&request.message)
        } else {
            None
        };
        
        // If a skill is triggered, add its content to the messages
        if let Some(ref skill_name) = skill_used {
            tracing::info!("Skill triggered: {}", skill_name);
            if let Some(skill) = self.skill_manager.as_ref().and_then(|sm| sm.get_skill(skill_name)) {
                let skill_message = ChatMessage {
                    role: crate::llm::types::MessageRole::System,
                    content: format!("You have access to the following skill:\n\n{}", skill.content),
                    name: Some(format!("skill-{}", skill_name)),
                    tool_calls: None,
                    tool_call_id: None,
                };
                messages.insert(0, skill_message);
            }
        }
        
        // Get LLM connector
        let llm = self.llm_connector.as_ref().ok_or_else(|| {
            anyhow::anyhow!("LLM connector not initialized")
        })?;
        
        // Get available tools (for function calling)
        let tool_names = self.tool_manager.get_tool_names();
        let tools: Vec<crate::tools::types::ToolMetadata> = tool_names.iter()
            .filter_map(|name| self.tool_manager.get_tool_metadata(name).cloned())
            .collect();
        
        // Tool call loop (max 5 iterations to prevent infinite loops)
        let mut final_response_text = String::new();
        let mut tool_calls_results = Vec::new();
        let max_iterations = 5;
        
        for iteration in 0..max_iterations {
            tracing::info!("LLM call iteration {}/{}", iteration + 1, max_iterations);
            
            // Call LLM
            tracing::info!("Calling LLM with {} messages and {} tools", messages.len(), tools.len());
            let llm_response = llm.chat_completion(messages.clone(), Some(tools.clone())).await
                .map_err(|e| anyhow::anyhow!("LLM error: {}", e))?;
            
            // Check if LLM wants to call tools
            if let Some(first_choice) = llm_response.choices.first() {
                if let Some(ref tool_calls_list) = first_choice.message.tool_calls {
                    tracing::info!("LLM requested {} tool calls", tool_calls_list.len());
                    
                    // Execute all tool calls
                    let mut tool_results = Vec::new();
                    for tool_call in tool_calls_list {
                        let tool_name = tool_call.function.name.clone();
                        let parameters = serde_json::from_str::<serde_json::Value>(&tool_call.function.arguments)
                            .unwrap_or(serde_json::json!({}));
                        println!("parameters: {}",parameters);
                        tracing::info!("Executing tool: {} with params: {}", tool_name, parameters);
                        
                        // Execute tool
                        match self.tool_manager.execute_tool_call(&tool_name, parameters).await {
                            Ok(result) => {
                                tracing::info!("Tool execution succeeded: {}", result.output);
                                tool_results.push(serde_json::json!({
                                    "tool_call_id": tool_call.id,
                                    "role": "tool",
                                    "content": result.output
                                }));
                                tool_calls_results.push(serde_json::json!({
                                    "tool_call_id": tool_call.id,
                                    "tool_name": tool_name,
                                    "result": result.output
                                }));
                            }
                            Err(e) => {
                                tracing::error!("Tool execution failed: {}", e);
                                let error_msg = format!("Tool execution error: {}", e);
                                tool_results.push(serde_json::json!({
                                    "tool_call_id": tool_call.id,
                                    "role": "tool",
                                    "content": error_msg
                                }));
                                tool_calls_results.push(serde_json::json!({
                                    "tool_call_id": tool_call.id,
                                    "tool_name": tool_name,
                                    "error": e.to_string()
                                }));
                            }
                        }
                    }
                    
                    // Add assistant message (with tool_calls) to messages
                    messages.push(ChatMessage {
                        role: crate::llm::types::MessageRole::Assistant,
                        content: first_choice.message.content.clone(),
                        name: first_choice.message.name.clone(),
                        tool_calls: Some(tool_calls_list.clone()),
                        tool_call_id: None,
                    });
                    
                    // Add tool results as tool messages
                    for tool_result in tool_results {
                        messages.push(ChatMessage {
                            role: crate::llm::types::MessageRole::Tool,
                            content: tool_result["content"].as_str().unwrap_or("").to_string(),
                            name: None,
                            tool_calls: None,
                            tool_call_id: tool_result["tool_call_id"].as_str().map(|s| s.to_string()),
                        });
                    }
                    
                    // Continue loop (send updated messages back to LLM)
                    continue;
                } else {
                    // No tool calls - this is the final text response
                    final_response_text = first_choice.message.content.clone();
                    break;
                }
            } else {
                final_response_text = "No response from LLM".to_string();
                break;
            }
        }
        
        // If we exhausted iterations, use last response
        if final_response_text.is_empty() {
            tracing::warn!("Exhausted max iterations ({}) for tool calls", max_iterations);
            final_response_text = "Tool call loop exceeded maximum iterations".to_string();
        }
        
        // Add assistant message to session
        let assistant_message = Message {
            role: MessageRole::Assistant,
            content: final_response_text.clone(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };
        self.session_manager.add_message(&session_id, assistant_message)?;
        
        // Build response
        let response = types::RunResponse {
            response: final_response_text,
            tool_calls: tool_calls_results,
            session_id,
            skill_used,  // Option<String>
        };
        
        Ok(response)
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

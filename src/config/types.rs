use serde::{Deserialize, Serialize};

/// Main agent configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    pub llm: LLMConfig,
    pub session: SessionConfig,
    pub logging: LoggingConfig,
    pub skills: Option<SkillsConfig>,
    pub tools: Option<ToolsConfigAgent>,
}

/// Skills configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillsConfig {
    pub skills_folder: Option<String>,
    pub auto_load_skills: Option<bool>,
    pub lazy_load_references: Option<bool>,
    pub lazy_load_scripts: Option<bool>,
    pub builtin_skills: Option<Vec<String>>,
}

/// Tools configuration for agent
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolsConfigAgent {
    pub sandbox_dir: Option<String>,
    pub auto_execute_tools: Option<bool>,
    pub builtin_tools: Option<Vec<String>>,
}

/// LLM configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub api_key: String,
    #[serde(rename = "baseURL")]
    pub base_url: Option<String>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub mock: Option<bool>,
}

/// LLM provider enumeration
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LLMProvider {
    OpenAI,
    OpenAICompatible,
}

/// Session configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    pub max_history_length: usize,
    pub session_ttl: Option<u64>,
    pub persistence: Option<SessionPersistenceConfig>,
}

/// Session persistence configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionPersistenceConfig {
    pub enabled: bool,
    pub storage_path: String,
    pub auto_save_interval: u64,  // seconds
    pub format: String,  // "jsonl" or "json"
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub file: Option<String>,
}

/// Log level enumeration
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

/// Tools configuration (MCP servers + builtin tools)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsConfig {
    pub mcp_servers: MCPServers,
    pub builtin_tools: Vec<BuiltinToolConfig>,
    pub auto_execute_tools: Option<bool>,
    pub mcp_tool_definitions: Option<std::collections::HashMap<String, Vec<serde_json::Value>>>,
}

/// MCP servers can be either new format (object) or old format (array)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MCPServers {
    New(std::collections::HashMap<String, MCPServerConfigNew>),
    Old(Vec<MCPServerConfigOld>),
}

/// New MCP server configuration format (standard MCP format)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerConfigNew {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub disabled: Option<bool>,
    pub description: Option<String>,
}

/// Old MCP server configuration format (deprecated)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MCPServerConfigOld {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
}

/// Builtin tool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinToolConfig {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

/// Tool parameter definition
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: ParameterType,
    pub required: bool,
    pub description: Option<String>,
}

/// Parameter type enumeration
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

/// Prompt configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptConfig {
    pub system_prompt: String,
    pub templates: Option<Vec<PromptTemplate>>,
}

/// Prompt template
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplate {
    pub name: String,
    pub template: String,
    pub variables: Option<Vec<String>>,
}

/// Message in a conversation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

/// Message role enumeration
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Tool call definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String, // Always "function"
    pub function: FunctionCall,
}

/// Function call definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON string
}

/// Session information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub id: String,
    pub messages: Vec<Message>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Chat response from LLM
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: Option<TokenUsage>,
}

/// Token usage information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// MCP tool request (JSON-RPC 2.0)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MCPToolRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: MCPToolRequestParams,
    pub id: MCPRequestId,
}

/// MCP tool request parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MCPToolRequestParams {
    pub name: String,
    pub arguments: std::collections::HashMap<String, serde_json::Value>,
}

/// MCP request ID (can be string or number)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MCPRequestId {
    String(String),
    Number(u64),
}

/// MCP tool response (JSON-RPC 2.0)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MCPToolResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<MCPError>,
    pub id: MCPRequestId,
}

/// MCP error definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Tool execution result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolExecutionResult {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

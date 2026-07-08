use async_trait::async_trait;
use serde_json::Value;
use crate::mcp::types::*;

/// Trait defining the interface for MCP clients
#[async_trait]
pub trait MCPClient: Send + Sync {
    /// Get the server name identifier
    fn server_name(&self) -> &str;
    
    /// Initialize connection with the MCP server
    /// This sends the `initialize` request and waits for server capabilities
    async fn initialize(&mut self) -> Result<MCPInitializeResult, MCPError>;
    
    /// List all available tools from the server
    async fn list_tools(&self) -> Result<Vec<MCPTool>, MCPError>;
    
    /// Call a specific tool with the given parameters
    async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<MCPToolResult, MCPError>;
    
    /// Get server capabilities (available after initialization)
    fn capabilities(&self) -> Option<&MCPServerCapabilities>;
    
    /// Get server info (available after initialization)
    fn server_info(&self) -> Option<&MCPServerInfo>;
    
    /// Check if the client is initialized
    fn is_initialized(&self) -> bool;
    
    /// Shutdown the connection gracefully
    async fn shutdown(&mut self) -> Result<(), MCPError>;
}

/// Common helper functions for MCP clients
pub fn validate_tool_call(_tool: &MCPTool, arguments: &Value) -> Result<(), MCPError> {
    // Basic validation that arguments match the schema
    // In a full implementation, this would validate against input_schema
    if !arguments.is_object() {
        return Err(MCPError::InvalidMessage(
            "Tool arguments must be a JSON object".to_string()
        ));
    }
    
    Ok(())
}

/// Standard MCP method names
pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const SHUTDOWN: &str = "shutdown";
    pub const NOTIFICATIONS_INITIALIZED: &str = "notifications/initialized";
    pub const TOOLS_LIST: &str = "tools/list";
    pub const TOOLS_CALL: &str = "tools/call";
    pub const RESOURCES_LIST: &str = "resources/list";
    pub const RESOURCES_READ: &str = "resources/read";
    pub const PROMPTS_LIST: &str = "prompts/list";
    pub const PROMPTS_GET: &str = "prompts/get";
}

/// Default client info for initialization
pub fn default_client_info() -> MCPClientInfo {
    MCPClientInfo {
        name: "agent-runtime-rs".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Default client capabilities
pub fn default_client_capabilities() -> Value {
    serde_json::json!({
        "tools": {},
        "resources": {},
        "prompts": {}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_validate_tool_call_valid() {
        let tool = MCPTool {
            name: "test_tool".to_string(),
            description: None,
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "param1": {"type": "string"}
                }
            })),
        };
        
        let args = json!({"param1": "value1"});
        let result = validate_tool_call(&tool, &args);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_tool_call_invalid_not_object() {
        let tool = MCPTool {
            name: "test_tool".to_string(),
            description: None,
            input_schema: Some(json!({
                "type": "object",
                "properties": {}
            })),
        };
        
        let args = json!("not an object");
        let result = validate_tool_call(&tool, &args);
        assert!(result.is_err());
        match result {
            Err(MCPError::InvalidMessage(msg)) => {
                assert!(msg.contains("must be a JSON object"));
            }
            _ => panic!("Expected InvalidMessage error"),
        }
    }
    
    #[test]
    fn test_validate_tool_call_invalid_array() {
        let tool = MCPTool {
            name: "test_tool".to_string(),
            description: None,
            input_schema: Some(json!({
                "type": "object",
                "properties": {}
            })),
        };
        
        let args = json!([1, 2, 3]);
        let result = validate_tool_call(&tool, &args);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_default_client_info() {
        let info = default_client_info();
        assert_eq!(info.name, "agent-runtime-rs");
        assert!(!info.version.is_empty());
    }
    
    #[test]
    fn test_default_client_capabilities() {
        let caps = default_client_capabilities();
        assert!(caps.is_object());
        assert!(caps.get("tools").is_some());
        assert!(caps.get("resources").is_some());
        assert!(caps.get("prompts").is_some());
    }
    
    #[test]
    fn test_methods_constants() {
        assert_eq!(methods::INITIALIZE, "initialize");
        assert_eq!(methods::TOOLS_LIST, "tools/list");
        assert_eq!(methods::TOOLS_CALL, "tools/call");
        assert_eq!(methods::SHUTDOWN, "shutdown");
    }
}

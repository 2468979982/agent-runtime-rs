//! MCP tool executor for integrating MCP tools into ToolManager
//! 
//! This module provides a `ToolExecutor` implementation that forwards
//! tool calls to an MCP server via an `MCPClient` instance.

use async_trait::async_trait;
use serde_json::{Value, json};
use crate::tools::types::*;
use crate::mcp::client::MCPClient;
use crate::mcp::types::MCPTool;

use std::sync::Arc;
use tokio::sync::Mutex;

/// Tool executor that forwards tool calls to an MCP server
pub struct MCPToolExecutor {
    /// The MCP client for communicating with the server (shared)
    client: Arc<Mutex<Box<dyn MCPClient + Send>>>,
    /// Tool metadata (converted from MCPTool)
    metadata: ToolMetadata,
}

impl MCPToolExecutor {
    /// Create a new MCPToolExecutor for a specific tool
    pub fn new(client: Arc<Mutex<Box<dyn MCPClient + Send>>>, mcp_tool: MCPTool) -> Self {
        // Convert MCPTool to ToolMetadata
        let metadata = ToolMetadata {
            name: mcp_tool.name.clone(),
            description: mcp_tool.description.unwrap_or_default(),
            parameters: vec![
                ToolParameter {
                    name: "arguments".to_string(),
                    description: "Tool arguments (JSON object)".to_string(),
                    required: true,
                    parameter_type: ParameterType::Object,
                },
            ],
        };
        
        Self {
            client,
            metadata,
        }
    }
}

#[async_trait]
impl ToolExecutor for MCPToolExecutor {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
    
    async fn execute(&self, parameters: Value) -> Result<ToolResult, ToolError> {
        // Extract tool arguments from parameters
        let arguments = parameters.get("arguments").unwrap_or(&json!({})).clone();
        
        // Get client lock
        let client = self.client.lock().await;
        
        // Call the tool via MCP client
        match client.call_tool(&self.metadata.name, arguments).await {
            Ok(mcp_result) => {
                // Convert MCPToolResult to ToolResult
                let output = if let Some(content) = mcp_result.content.first() {
                    if content.content_type == "text" {
                        content.text.clone().unwrap_or_default()
                    } else {
                        serde_json::to_string(&mcp_result.content).unwrap_or_default()
                    }
                } else {
                    "".to_string()
                };
                
                let is_error = mcp_result.is_error.unwrap_or(false);
                
                Ok(ToolResult {
                    success: !is_error,
                    output: output.clone(),
                    error: if is_error {
                        Some(output)
                    } else {
                        None
                    },
                })
            }
            Err(e) => {
                Err(ToolError::ExecutionError(format!("MCP tool execution failed: {}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::mcp::types::*;
    
    /// Mock MCP client for testing
    struct MockMCPClient {
        server_name: String,
        tools: Vec<MCPTool>,
    }
    
    impl MockMCPClient {
        fn new(server_name: &str) -> Self {
            Self {
                server_name: server_name.to_string(),
                tools: vec![
                    MCPTool {
                        name: "mock_tool".to_string(),
                        description: Some("A mock tool".to_string()),
                        input_schema: Some(json!({"type": "object"})),  // 注意：Option<Value>
                    },
                ],
            }
        }
    }
    
    #[async_trait]
    impl MCPClient for MockMCPClient {
        fn server_name(&self) -> &str {
            &self.server_name
        }
        
        async fn initialize(&mut self) -> Result<MCPInitializeResult, MCPError> {
            Ok(MCPInitializeResult {
                protocol_version: Some("2024-11-05".to_string()),
                capabilities: Some(MCPServerCapabilities {
                    tools: Some(json!({})),
                    resources: None,
                    prompts: None,
                }),
                server_info: Some(MCPServerInfo {
                    name: self.server_name.clone(),
                    version: "1.0.0".to_string(),
                }),
            })
        }
        
        async fn list_tools(&self) -> Result<Vec<MCPTool>, MCPError> {
            Ok(self.tools.clone())
        }
        
        async fn call_tool(&self, tool_name: &str, _arguments: Value) -> Result<MCPToolResult, MCPError> {
            Ok(MCPToolResult {
                content: vec![
                    MCPContent::text(&format!("Mock result from {}", tool_name)),
                ],
                is_error: Some(false),
            })
        }
        
        fn capabilities(&self) -> Option<&MCPServerCapabilities> {
            None
        }
        
        fn server_info(&self) -> Option<&MCPServerInfo> {
            None
        }
        
        fn is_initialized(&self) -> bool {
            true
        }
        
        async fn shutdown(&mut self) -> Result<(), MCPError> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_mcp_tool_executor() {
        let client = Arc::new(Mutex::new(Box::new(MockMCPClient::new("mock-server")) as Box<dyn MCPClient + Send>));
        let mcp_tool = MCPTool {
            name: "mock_tool".to_string(),
            description: Some("A mock tool".to_string()),
            input_schema: Some(json!({"type": "object"})),  // Option<Value>
        };
        
        let executor = MCPToolExecutor::new(client, mcp_tool);
        
        // Check metadata
        assert_eq!(executor.metadata().name, "mock_tool");
        assert_eq!(executor.metadata().description, "A mock tool");
        
        // Execute tool
        let result = executor.execute(json!({"arguments": {"param": "value"}})).await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.success);
        assert!(tool_result.output.contains("Mock result from mock_tool"));
    }
}

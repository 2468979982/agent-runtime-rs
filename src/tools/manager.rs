use std::collections::HashMap;
use serde_json::Value;
use crate::tools::types::*;

/// Tool manager for registering and executing tools
pub struct ToolManager {
    tools: HashMap<String, Box<dyn ToolExecutor>>,
}

impl ToolManager {
    /// Create a new ToolManager
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
    
    /// Register a tool executor
    pub fn register_tool(&mut self, tool: Box<dyn ToolExecutor>) {
        let name = tool.metadata().name.clone();
        self.tools.insert(name, tool);
    }
    
    /// Execute a tool call by name
    pub async fn execute_tool_call(&self, tool_name: &str, parameters: Value) -> Result<ToolResult, ToolError> {
        match self.tools.get(tool_name) {
            Some(tool) => tool.execute(parameters).await,
            None => Err(ToolError::NotFound(tool_name.to_string())),
        }
    }
    
    /// Get all tool definitions in OpenAI format
    pub fn get_tool_definitions(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|tool| tool.metadata().into())
            .collect()
    }
    
    /// Check if a tool is registered
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.tools.contains_key(tool_name)
    }
    
    /// Get list of registered tool names
    pub fn get_tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
    
    /// Get tool metadata by name
    pub fn get_tool_metadata(&self, tool_name: &str) -> Option<&ToolMetadata> {
        self.tools.get(tool_name).map(|tool| tool.metadata())
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    
    /// Mock tool for testing
    struct MockTool {
        metadata: ToolMetadata,
    }
    
    impl MockTool {
        fn new() -> Self {
            Self {
                metadata: ToolMetadata {
                    name: "mock_tool".to_string(),
                    description: "A mock tool for testing".to_string(),
                    parameters: vec![
                        ToolParameter {
                            name: "input".to_string(),
                            description: "Test input".to_string(),
                            required: true,
                            parameter_type: ParameterType::String,
                        },
                    ],
                },
            }
        }
    }
    
    #[async_trait]
    impl ToolExecutor for MockTool {
        fn metadata(&self) -> &ToolMetadata {
            &self.metadata
        }
        
        async fn execute(&self, parameters: Value) -> Result<ToolResult, ToolError> {
            Ok(ToolResult {
                success: true,
                output: parameters["input"].as_str().unwrap_or("").to_string(),
                error: None,
            })
        }
    }
    
    #[tokio::test]
    async fn test_register_and_execute_tool() {
        let mut manager = ToolManager::new();
        let tool = Box::new(MockTool::new());
        
        manager.register_tool(tool);
        
        assert!(manager.has_tool("mock_tool"));
        assert_eq!(manager.get_tool_names(), vec!["mock_tool"]);
        
        let result = manager.execute_tool_call(
            "mock_tool",
            serde_json::json!({"input": "test_value"})
        ).await;
        
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.success);
        assert_eq!(tool_result.output, "test_value");
    }
    
    #[tokio::test]
    async fn test_execute_nonexistent_tool() {
        let manager = ToolManager::new();
        
        let result = manager.execute_tool_call(
            "nonexistent",
            serde_json::json!({})
        ).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ToolError::NotFound(name) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected NotFound error"),
        }
    }
    
    #[test]
    fn test_get_tool_definitions() {
        let mut manager = ToolManager::new();
        manager.register_tool(Box::new(MockTool::new()));
        
        let definitions = manager.get_tool_definitions();
        
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0]["function"]["name"], "mock_tool");
    }
    
    #[test]
    fn test_get_tool_metadata() {
        let mut manager = ToolManager::new();
        manager.register_tool(Box::new(MockTool::new()));
        
        let metadata = manager.get_tool_metadata("mock_tool");
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().name, "mock_tool");
        
        let nonexistent = manager.get_tool_metadata("nonexistent");
        assert!(nonexistent.is_none());
    }
}

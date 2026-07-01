use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Tool metadata describing a tool's name, description, and parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

/// Tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub parameter_type: ParameterType,
}

/// Parameter type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub parameters: Value,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Tool execution error
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Tool executor trait
#[async_trait::async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Get tool metadata
    fn metadata(&self) -> &ToolMetadata;
    
    /// Execute the tool with given parameters
    async fn execute(&self, parameters: Value) -> Result<ToolResult, ToolError>;
}

/// Convert ToolMetadata to OpenAI function definition format
impl From<&ToolMetadata> for serde_json::Value {
    fn from(metadata: &ToolMetadata) -> Self {
        let properties = metadata
            .parameters
            .iter()
            .map(|p| {
                let mut param_obj = serde_json::json!({
                    "type": match p.parameter_type {
                        ParameterType::String => "string",
                        ParameterType::Number => "number",
                        ParameterType::Boolean => "boolean",
                        ParameterType::Array => "array",
                        ParameterType::Object => "object",
                    },
                    "description": p.description,
                });
                
                if let Some(obj) = param_obj.as_object_mut() {
                    if p.parameter_type == ParameterType::Array {
                        obj.insert("items".to_string(), serde_json::json!({}));
                    }
                }
                
                (p.name.clone(), param_obj)
            })
            .collect::<serde_json::Map<String, Value>>();
        
        let required = metadata
            .parameters
            .iter()
            .filter(|p| p.required)
            .map(|p| p.name.clone())
            .collect::<Vec<String>>();
        
        serde_json::json!({
            "type": "function",
            "function": {
                "name": metadata.name,
                "description": metadata.description,
                "parameters": {
                    "type": "object",
                    "properties": properties,
                    "required": required
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tool_metadata_creation() {
        let metadata = ToolMetadata {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "param1".to_string(),
                    description: "First parameter".to_string(),
                    required: true,
                    parameter_type: ParameterType::String,
                },
                ToolParameter {
                    name: "param2".to_string(),
                    description: "Second parameter".to_string(),
                    required: false,
                    parameter_type: ParameterType::Number,
                },
            ],
        };
        
        assert_eq!(metadata.name, "test_tool");
        assert_eq!(metadata.parameters.len(), 2);
        assert!(metadata.parameters[0].required);
        assert!(!metadata.parameters[1].required);
    }
    
    #[test]
    fn test_tool_metadata_to_openai_format() {
        let metadata = ToolMetadata {
            name: "calculator".to_string(),
            description: "Perform mathematical calculations".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "expression".to_string(),
                    description: "Mathematical expression to evaluate".to_string(),
                    required: true,
                    parameter_type: ParameterType::String,
                },
            ],
        };
        
        let openai_format: Value = (&metadata).into();
        
        assert_eq!(openai_format["type"], "function");
        assert_eq!(openai_format["function"]["name"], "calculator");
        assert_eq!(openai_format["function"]["parameters"]["type"], "object");
        assert!(openai_format["function"]["parameters"]["properties"].is_object());
        assert!(openai_format["function"]["parameters"]["required"].is_array());
    }
    
    #[test]
    fn test_tool_result_success() {
        let result = ToolResult {
            success: true,
            output: "42".to_string(),
            error: None,
        };
        
        assert!(result.success);
        assert_eq!(result.output, "42");
        assert!(result.error.is_none());
    }
    
    #[test]
    fn test_tool_result_error() {
        let result = ToolResult {
            success: false,
            output: String::new(),
            error: Some("Division by zero".to_string()),
        };
        
        assert!(!result.success);
        assert_eq!(result.error.unwrap(), "Division by zero");
    }
}

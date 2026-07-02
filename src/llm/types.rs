use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// LLM configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LLMConfig {
    pub provider: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "baseURL")]
    pub base_url: String,
    pub model: String,
    pub temperature: Option<f32>,
    #[serde(rename = "maxTokens")]
    pub max_tokens: Option<u32>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: "qwen".to_string(),
            api_key: std::env::var("QWEN_API_KEY").unwrap_or_default(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
        }
    }
}

/// Chat message for LLM
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

/// Message role
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

/// Tool definition for LLM
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionDefinition,
}

/// Function definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// Tool choice strategy
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoice {
    Auto,
    None,
    Required,
}

/// Chat completion request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Chat completion response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Option<TokenUsage>,
    pub created: u64,
    pub model: String,
}

/// Choice in response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// LLM error types
#[derive(Debug, Error)]
pub enum LLMError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Rate limit error: {0}")]
    RateLimitError(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

/// Convert ToolMetadata from tools module to ToolDefinition
impl From<&crate::tools::types::ToolMetadata> for ToolDefinition {
    fn from(metadata: &crate::tools::types::ToolMetadata) -> Self {
        let properties = metadata
            .parameters
            .iter()
            .map(|p| {
                let param_obj = serde_json::json!({
                    "type": match p.parameter_type {
                        crate::tools::types::ParameterType::String => "string",
                        crate::tools::types::ParameterType::Number => "number",
                        crate::tools::types::ParameterType::Boolean => "boolean",
                        crate::tools::types::ParameterType::Array => "array",
                        crate::tools::types::ParameterType::Object => "object",
                    },
                    "description": p.description,
                });
                
                (p.name.clone(), param_obj)
            })
            .collect::<serde_json::Map<String, Value>>();
        
        let required = metadata
            .parameters
            .iter()
            .filter(|p| p.required)
            .map(|p| p.name.clone())
            .collect::<Vec<String>>();
        
        ToolDefinition {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": properties,
                    "required": required
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::types::{ToolMetadata, ToolParameter, ParameterType};
    
    #[test]
    fn test_llm_config_default() {
        let config = LLMConfig::default();
        assert_eq!(config.provider, "qwen");
        assert_eq!(config.base_url, "https://dashscope.aliyuncs.com/compatible-mode/v1");
        assert_eq!(config.model, "qwen-turbo");
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.max_tokens, Some(2000));
    }
    
    #[test]
    fn test_chat_message_creation() {
        let message = ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };
        
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, "Hello");
    }
    
    #[test]
    fn test_tool_definition_from_tool_metadata() {
        let metadata = ToolMetadata {
            name: "calculator".to_string(),
            description: "Perform calculations".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "expression".to_string(),
                    description: "Math expression".to_string(),
                    required: true,
                    parameter_type: ParameterType::String,
                },
            ],
        };
        
        let tool_def: ToolDefinition = (&metadata).into();
        
        assert_eq!(tool_def.type_, "function");
        assert_eq!(tool_def.function.name, "calculator");
        assert_eq!(tool_def.function.description, "Perform calculations");
        assert!(tool_def.function.parameters.is_object());
        
        let params = tool_def.function.parameters.as_object().unwrap();
        assert!(params.contains_key("properties"));
        assert!(params.contains_key("required"));
    }
    
    #[test]
    fn test_chat_completion_request_serialization() {
        let request = ChatCompletionRequest {
            model: "qwen-turbo".to_string(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(2000),
            tools: None,
            tool_choice: None,
            stream: Some(false),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("qwen-turbo"));
        assert!(json.contains("Hello"));
        
        let deserialized: ChatCompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.model, "qwen-turbo");
        assert_eq!(deserialized.messages[0].content, "Hello");
    }
    
    #[test]
    fn test_chat_completion_response_deserialization() {
        let json = r#"{
            "id": "chatcmpl-123",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I help you?"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "total_tokens": 30
            },
            "created": 1234567890,
            "model": "qwen-turbo"
        }"#;
        
        let response: ChatCompletionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "chatcmpl-123");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.content, "Hello! How can I help you?");
        assert_eq!(response.usage.unwrap().total_tokens, 30);
    }
    
    #[test]
    fn test_tool_choice_serialization() {
        let auto = ToolChoice::Auto;
        let json = serde_json::to_string(&auto).unwrap();
        assert_eq!(json, "\"auto\"");
        
        let none = ToolChoice::None;
        let json = serde_json::to_string(&none).unwrap();
        assert_eq!(json, "\"none\"");
        
        let required = ToolChoice::Required;
        let json = serde_json::to_string(&required).unwrap();
        assert_eq!(json, "\"required\"");
    }
    
    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::System;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"system\"");
        
        let role = MessageRole::User;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"user\"");
        
        let role = MessageRole::Assistant;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"assistant\"");
        
        let role = MessageRole::Tool;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"tool\"");
    }
}

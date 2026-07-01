use tracing::{info, warn, error};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};

use crate::llm::types::*;
use crate::tools::types::ToolMetadata;

/// LLM Connector for interacting with LLM APIs
pub struct LLMConnector {
    config: LLMConfig,
    client: reqwest::Client,
}

impl LLMConnector {
    /// Create a new LLMConnector
    pub fn new(config: &LLMConfig) -> Result<Self, LLMError> {
        if config.api_key.is_empty() {
            return Err(LLMError::ConfigError("API key is required".to_string()));
        }
        
        let client = reqwest::Client::new();
        
        Ok(Self {
            config: config.clone(),
            client,
        })
    }
    
    /// Send chat completion request
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolMetadata>>,
    ) -> Result<ChatCompletionResponse, LLMError> {
        info!("Sending chat completion request with {} messages", messages.len());
        
        // Convert tools to OpenAI format if provided
        let tool_definitions = tools.map(|tool_list| {
            tool_list.iter().map(|t| ToolDefinition::from(t)).collect()
        });
        
        // Build request
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            tools: tool_definitions,
            tool_choice: if self.config.temperature.is_some() {
                Some(ToolChoice::Auto)
            } else {
                None
            },
            stream: Some(false),
        };
        
        // Log request (without sensitive data)
        info!("Chat completion request built");
        
        // Send request
        let response = self.send_request(request).await?;
        
        info!("Chat completion successful, response ID: {}", response.id);
        
        Ok(response)
    }
    
    /// Send request to LLM API
    async fn send_request(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, LLMError> {
        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));
        
        info!("Sending request to: {}", url);
        
        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.api_key))
                .map_err(|e| LLMError::ConfigError(format!("Invalid API key: {}", e)))?,
        );
        
        // Send request
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(LLMError::NetworkError)?;
        
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            error!("LLM API error: {} - {}", status, error_text);
            
            return match status.as_u16() {
                401 => Err(LLMError::AuthError(format!("Authentication failed: {}", error_text))),
                429 => Err(LLMError::RateLimitError(format!("Rate limit exceeded: {}", error_text))),
                _ => Err(LLMError::ApiError(format!("API error ({}): {}", status, error_text))),
            };
        }
        
        // Parse response
        let response_json: serde_json::Value = response.json().await
            .map_err(LLMError::NetworkError)?;
        
        // Deserialize to ChatCompletionResponse
        let chat_response: ChatCompletionResponse = serde_json::from_value(response_json)
            .map_err(LLMError::SerializationError)?;
        
        Ok(chat_response)
    }
    
    /// Chat completion with streaming (placeholder for future implementation)
    pub async fn chat_completion_stream(
        &self,
        _messages: Vec<ChatMessage>,
        _tools: Option<Vec<ToolMetadata>>,
    ) -> Result<(), LLMError> {
        warn!("Streaming is not yet implemented");
        Err(LLMError::ConfigError("Streaming not implemented".to_string()))
    }
    
    /// Test connection to LLM API
    pub async fn test_connection(&self) -> Result<String, LLMError> {
        info!("Testing LLM connection");
        
        let test_messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Hello, please respond with 'Connection successful'".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        
        match self.chat_completion(test_messages, None).await {
            Ok(response) => {
                let result = format!(
                    "Connection successful! Model: {}, Response: {}",
                    response.model,
                    response.choices[0].message.content
                );
                info!("{}", result);
                Ok(result)
            }
            Err(e) => {
                error!("Connection test failed: {}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_llm_connector_creation() {
        let config = LLMConfig {
            provider: "qwen".to_string(),
            api_key: "test-api-key".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
        };
        
        let connector = LLMConnector::new(&config);
        
        assert!(connector.is_ok());
    }
    
    #[tokio::test]
    async fn test_llm_connector_creation_with_empty_api_key() {
        let config = LLMConfig {
            provider: "qwen".to_string(),
            api_key: String::new(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
        };
        
        let connector = LLMConnector::new(&config);
        
        assert!(connector.is_err());
        match connector.err().unwrap() {
            LLMError::ConfigError(msg) => assert_eq!(msg, "API key is required"),
            _ => panic!("Expected ConfigError"),
        }
    }
    
    #[test]
    fn test_chat_completion_request_builder() {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: "You are a helpful assistant.".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: "Hello!".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        
        let request = ChatCompletionRequest {
            model: "qwen-turbo".to_string(),
            messages,
            temperature: Some(0.7),
            max_tokens: Some(2000),
            tools: None,
            tool_choice: Some(ToolChoice::Auto),
            stream: Some(false),
        };
        
        assert_eq!(request.model, "qwen-turbo");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(2000));
        assert!(request.stream.unwrap() == false);
    }
    
    #[test]
    fn test_request_serialization() {
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
        
        // Verify it can be deserialized back
        let deserialized: ChatCompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.model, "qwen-turbo");
    }
    
    #[test]
    fn test_response_deserialization() {
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
        assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
        assert_eq!(response.choices[0].message.content, "Hello! How can I help you?");
        assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));
        assert!(response.usage.is_some());
        assert_eq!(response.usage.unwrap().total_tokens, 30);
    }
    
    #[test]
    fn test_tool_definition_conversion() {
        use crate::tools::types::{ToolMetadata, ToolParameter, ParameterType};
        
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
        
        let tool_def: ToolDefinition = (&metadata).into();
        
        assert_eq!(tool_def.type_, "function");
        assert_eq!(tool_def.function.name, "calculator");
        assert_eq!(tool_def.function.description, "Perform mathematical calculations");
        
        let params = tool_def.function.parameters.as_object().unwrap();
        let properties = params["properties"].as_object().unwrap();
        assert!(properties.contains_key("expression"));
        assert_eq!(properties["expression"]["type"], "string");
        assert_eq!(properties["expression"]["description"], "Mathematical expression to evaluate");
        
        let required = params["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "expression");
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
        assert_eq!(serde_json::to_string(&MessageRole::System).unwrap(), "\"system\"");
        assert_eq!(serde_json::to_string(&MessageRole::User).unwrap(), "\"user\"");
        assert_eq!(serde_json::to_string(&MessageRole::Assistant).unwrap(), "\"assistant\"");
        assert_eq!(serde_json::to_string(&MessageRole::Tool).unwrap(), "\"tool\"");
    }
    
    #[test]
    fn test_chat_message_with_tool_calls() {
        let message = ChatMessage {
            role: MessageRole::Assistant,
            content: "I'll help you with that.".to_string(),
            name: None,
            tool_calls: Some(vec![
                ToolCall {
                    id: "call_123".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "calculator".to_string(),
                        arguments: r#"{"expression": "2+2"}"#.to_string(),
                    },
                },
            ]),
            tool_call_id: None,
        };
        
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("tool_calls"));
        assert!(json.contains("call_123"));
        assert!(json.contains("calculator"));
        
        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
        assert!(deserialized.tool_calls.is_some());
        assert_eq!(deserialized.tool_calls.unwrap().len(), 1);
    }
}

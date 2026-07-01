//! Integration tests for LLM connector
//! 
//! These tests verify the LLM connector works correctly with mock responses.
//! In production, these would be replaced with actual API tests or use a mock server.

use agent_runtime_rs::llm::types::*;
use agent_runtime_rs::tools::types::{ToolMetadata, ToolParameter, ParameterType};

/// Mock LLM connector for testing
struct MockLLMConnector {
    config: LLMConfig,
}

impl MockLLMConnector {
    fn new(config: &LLMConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
    
    /// Simulate a chat completion response
    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolMetadata>>,
    ) -> Result<ChatCompletionResponse, LLMError> {
        // Simulate API delay
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        // Check if tools are provided and simulate tool calls
        if let Some(tool_list) = tools {
            if !tool_list.is_empty() {
                // Simulate a tool call response
                return Ok(ChatCompletionResponse {
                    id: "mock-response-123".to_string(),
                    choices: vec![
                        Choice {
                            index: 0,
                            message: ChatMessage {
                                role: MessageRole::Assistant,
                                content: "I'll help you with that.".to_string(),
                                name: None,
                                tool_calls: Some(vec![
                                    ToolCall {
                                        id: "call_123".to_string(),
                                        call_type: "function".to_string(),
                                        function: FunctionCall {
                                            name: tool_list[0].name.clone(),
                                            arguments: r#"{"expression": "2+2"}"#.to_string(),
                                        },
                                    },
                                ]),
                                tool_call_id: None,
                            },
                            finish_reason: Some("tool_calls".to_string()),
                        },
                    ],
                    usage: Some(TokenUsage {
                        prompt_tokens: 50,
                        completion_tokens: 30,
                        total_tokens: 80,
                    }),
                    created: 1234567890,
                    model: self.config.model.clone(),
                });
            }
        }
        
        // Simulate a normal text response
        let last_message = messages.last().unwrap();
        let response_content = format!("Mock response to: {}", last_message.content);
        
        Ok(ChatCompletionResponse {
            id: "mock-response-456".to_string(),
            choices: vec![
                Choice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: response_content,
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some("stop".to_string()),
                },
            ],
            usage: Some(TokenUsage {
                prompt_tokens: 20,
                completion_tokens: 15,
                total_tokens: 35,
            }),
            created: 1234567890,
            model: self.config.model.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_connector_text_response() {
        let config = LLMConfig {
            provider: "qwen".to_string(),
            api_key: "test-key".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
        };
        
        let connector = MockLLMConnector::new(&config);
        
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Hello!".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        
        let response = connector.chat_completion(messages, None).await.unwrap();
        
        assert_eq!(response.id, "mock-response-456");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
        assert!(response.choices[0].message.content.contains("Mock response to: Hello!"));
        assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));
        assert!(response.usage.is_some());
        assert_eq!(response.usage.unwrap().total_tokens, 35);
    }
    
    #[tokio::test]
    async fn test_mock_connector_tool_call_response() {
        let config = LLMConfig {
            provider: "qwen".to_string(),
            api_key: "test-key".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
        };
        
        let connector = MockLLMConnector::new(&config);
        
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Calculate 2+2".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        
        let tools = vec![
            ToolMetadata {
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
            },
        ];
        
        let response = connector.chat_completion(messages, Some(tools)).await.unwrap();
        
        assert_eq!(response.id, "mock-response-123");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].finish_reason, Some("tool_calls".to_string()));
        assert!(response.choices[0].message.tool_calls.is_some());
        
        let tool_calls = response.choices[0].message.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.name, "calculator");
        assert_eq!(tool_calls[0].function.arguments, r#"{"expression": "2+2"}"#);
    }
    
    #[test]
    fn test_llm_config_validation() {
        // Valid config
        let config = LLMConfig {
            provider: "qwen".to_string(),
            api_key: "valid-key".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
        };
        
        assert!(!config.api_key.is_empty());
        assert_eq!(config.base_url, "https://dashscope.aliyuncs.com/compatible-mode/v1");
        assert_eq!(config.model, "qwen-turbo");
    }
    
    #[test]
    fn test_message_conversion_to_openai_format() {
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
            ChatMessage {
                role: MessageRole::Assistant,
                content: "Hi there!".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        
        // Convert to JSON (simulating OpenAI format)
        let json = serde_json::to_value(&messages).unwrap();
        
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 3);
        assert_eq!(json[0]["role"], "system");
        assert_eq!(json[1]["role"], "user");
        assert_eq!(json[2]["role"], "assistant");
    }
    
    #[test]
    fn test_tool_definition_serialization() {
        let tool = ToolDefinition {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "calculator".to_string(),
                description: "Perform calculations".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "Math expression"
                        }
                    },
                    "required": ["expression"]
                }),
            },
        };
        
        let json = serde_json::to_value(&tool).unwrap();
        
        assert_eq!(json["type"], "function");
        assert_eq!(json["function"]["name"], "calculator");
        assert_eq!(json["function"]["description"], "Perform calculations");
        assert!(json["function"]["parameters"].is_object());
    }
    
    #[test]
    fn test_chat_completion_request_with_tools() {
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Calculate 2+2".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        
        let tools = vec![
            ToolDefinition {
                type_: "function".to_string(),
                function: FunctionDefinition {
                    name: "calculator".to_string(),
                    description: "Perform calculations".to_string(),
                    parameters: serde_json::json!({}),
                },
            },
        ];
        
        let request = ChatCompletionRequest {
            model: "qwen-turbo".to_string(),
            messages,
            temperature: Some(0.7),
            max_tokens: Some(2000),
            tools: Some(tools),
            tool_choice: Some(ToolChoice::Auto),
            stream: Some(false),
        };
        
        let json = serde_json::to_value(&request).unwrap();
        
        assert!(json["tools"].is_array());
        assert_eq!(json["tools"].as_array().unwrap().len(), 1);
        assert_eq!(json["tool_choice"], "auto");
    }
    
    #[test]
    fn test_response_with_tool_calls() {
        let json = r#"{
            "id": "chatcmpl-123",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "I'll calculate that for you.",
                    "tool_calls": [{
                        "id": "call_123",
                        "type": "function",
                        "function": {
                            "name": "calculator",
                            "arguments": "{\"expression\": \"2+2\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "prompt_tokens": 50,
                "completion_tokens": 30,
                "total_tokens": 80
            },
            "created": 1234567890,
            "model": "qwen-turbo"
        }"#;
        
        let response: ChatCompletionResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.id, "chatcmpl-123");
        assert_eq!(response.choices[0].finish_reason, Some("tool_calls".to_string()));
        assert!(response.choices[0].message.tool_calls.is_some());
        
        let tool_calls = response.choices[0].message.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_123");
        assert_eq!(tool_calls[0].function.name, "calculator");
    }
    
    #[test]
    fn test_token_usage_calculation() {
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        
        assert_eq!(usage.total_tokens, usage.prompt_tokens + usage.completion_tokens);
    }
    
    #[tokio::test]
    async fn test_multiple_messages_context() {
        let config = LLMConfig {
            provider: "qwen".to_string(),
            api_key: "test-key".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
        };
        
        let connector = MockLLMConnector::new(&config);
        
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: "You are a math tutor.".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: "What is 2+2?".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: "2+2 equals 4.".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: "What about 3+3?".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        
        let response = connector.chat_completion(messages, None).await.unwrap();
        
        assert!(response.choices[0].message.content.contains("Mock response to: What about 3+3?"));
    }
    
    #[test]
    fn test_tool_choice_enum() {
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
    fn test_message_role_enum() {
        assert_eq!(serde_json::to_string(&MessageRole::System).unwrap(), "\"system\"");
        assert_eq!(serde_json::to_string(&MessageRole::User).unwrap(), "\"user\"");
        assert_eq!(serde_json::to_string(&MessageRole::Assistant).unwrap(), "\"assistant\"");
        assert_eq!(serde_json::to_string(&MessageRole::Tool).unwrap(), "\"tool\"");
    }
}

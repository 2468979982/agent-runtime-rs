//! Manual HTTP client implementation for LLM API
//! 
//! This module provides an alternative to async-openai crate for cases
//! where more control over the HTTP request/response is needed.

use tracing::{info, error};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};

use crate::llm::types::*;

/// Manual HTTP client for LLM API
pub struct LLMHttpClient {
    config: LLMConfig,
    client: reqwest::Client,
}

impl LLMHttpClient {
    /// Create a new HTTP client
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
    
    /// Send chat completion request using manual HTTP
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatCompletionResponse, LLMError> {
        info!("Sending chat completion request via HTTP client");
        
        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));
        
        // Build request body
        let mut request_body = serde_json::json!({
            "model": self.config.model,
            "messages": self.messages_to_json(messages),
            "stream": false
        });
        
        if let Some(temp) = self.config.temperature {
            request_body["temperature"] = serde_json::json!(temp);
        }
        
        if let Some(tokens) = self.config.max_tokens {
            request_body["max_tokens"] = serde_json::json!(tokens);
        }
        
        if let Some(tools) = tools {
            request_body["tools"] = serde_json::to_value(tools)
                .map_err(LLMError::SerializationError)?;
            request_body["tool_choice"] = serde_json::json!("auto");
        }
        
        info!("Request URL: {}", url);
        info!("Request body: {}", serde_json::to_string_pretty(&request_body).unwrap_or_default());
        
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
            .json(&request_body)
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
        
        info!("Response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());
        
        // Deserialize to ChatCompletionResponse
        let chat_response: ChatCompletionResponse = serde_json::from_value(response_json)
            .map_err(LLMError::SerializationError)?;
        
        Ok(chat_response)
    }
    
    /// Convert messages to JSON format
    fn messages_to_json(&self, messages: Vec<ChatMessage>) -> Vec<serde_json::Value> {
        messages
            .into_iter()
            .map(|msg| {
                let mut json_msg = serde_json::json!({
                    "role": msg.role,
                    "content": msg.content
                });
                
                if let Some(name) = msg.name {
                    json_msg["name"] = serde_json::json!(name);
                }
                
                if let Some(tool_calls) = msg.tool_calls {
                    json_msg["tool_calls"] = serde_json::to_value(tool_calls).unwrap_or_default();
                }
                
                if let Some(tool_call_id) = msg.tool_call_id {
                    json_msg["tool_call_id"] = serde_json::json!(tool_call_id);
                }
                
                json_msg
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_http_client_creation() {
        let config = LLMConfig {
            provider: "qwen".to_string(),
            api_key: "test-key".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
        };
        
        let client = LLMHttpClient::new(&config);
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_http_client_creation_empty_key() {
        let config = LLMConfig {
            provider: "qwen".to_string(),
            api_key: String::new(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
        };
        
        let client = LLMHttpClient::new(&config);
        assert!(client.is_err());
    }
}

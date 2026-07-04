use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

/// MCP protocol errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MCPError {
    #[error("JSON-RPC error: {code} - {message}")]
    JsonRpc { code: i64, message: String, data: Option<Value> },
    
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Tool execution error: {0}")]
    ToolExecution(String),
    
    #[error("Server error: {0}")]
    Server(String),
}

impl From<std::io::Error> for MCPError {
    fn from(err: std::io::Error) -> Self {
        MCPError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for MCPError {
    fn from(err: serde_json::Error) -> Self {
        MCPError::Serialization(err.to_string())
    }
}

/// JSON-RPC 2.0 request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
    Null,
}

impl From<Uuid> for RequestId {
    fn from(uuid: Uuid) -> Self {
        RequestId::String(uuid.to_string())
    }
}

/// JSON-RPC 2.0 response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(flatten)]
    pub payload: ResponsePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponsePayload {
    Result { result: Value },
    Error { error: JsonRpcError },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

/// MCP notification message (no id, no response expected)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub input_schema: Option<Value>,
}

/// MCP tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolCall {
    pub name: String,
    pub arguments: Value,
}

/// MCP tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolResult {
    pub content: Vec<MCPContent>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    pub data: Option<String>,
    pub mime_type: Option<String>,
}

impl MCPContent {
    pub fn text(text: &str) -> Self {
        Self {
            content_type: "text".to_string(),
            text: Some(text.to_string()),
            data: None,
            mime_type: None,
        }
    }
    
    pub fn image(data: &str, mime_type: &str) -> Self {
        Self {
            content_type: "image".to_string(),
            text: None,
            data: Some(data.to_string()),
            mime_type: Some(mime_type.to_string()),
        }
    }
}

/// MCP initialization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPInitializeParams {
    pub protocol_version: String,
    pub capabilities: Value,
    pub client_info: MCPClientInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerCapabilities {
    pub tools: Option<Value>,
    pub resources: Option<Value>,
    pub prompts: Option<Value>,
}

/// MCP server info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerInfo {
    pub name: String,
    pub version: String,
}

/// MCP initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPInitializeResult {
    #[serde(default)]
    pub protocol_version: Option<String>,
    #[serde(default)]
    pub capabilities: Option<MCPServerCapabilities>,
    #[serde(default)]
    pub server_info: Option<MCPServerInfo>,
}

/// Helper to create a JSON-RPC request
pub fn create_request(id: RequestId, method: &str, params: Option<Value>) -> MCPRequest {
    MCPRequest {
        jsonrpc: "2.0".to_string(),
        id,
        method: method.to_string(),
        params,
    }
}

/// Helper to create a JSON-RPC notification
pub fn create_notification(method: &str, params: Option<Value>) -> MCPNotification {
    MCPNotification {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
    }
}

/// Helper to create a JSON-RPC response with result
pub fn create_success_response(id: RequestId, result: Value) -> MCPResponse {
    MCPResponse {
        jsonrpc: "2.0".to_string(),
        id,
        payload: ResponsePayload::Result { result },
    }
}

/// Helper to create a JSON-RPC response with error
pub fn create_error_response(id: RequestId, code: i64, message: &str, data: Option<Value>) -> MCPResponse {
    MCPResponse {
        jsonrpc: "2.0".to_string(),
        id,
        payload: ResponsePayload::Error {
            error: JsonRpcError {
                code,
                message: message.to_string(),
                data,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_request_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id: RequestId = uuid.into();
        match id {
            RequestId::String(s) => assert_eq!(s, uuid.to_string()),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_create_request() {
        let id = RequestId::String("test-id".to_string());
        let params = serde_json::json!({"key": "value"});
        let request = create_request(id.clone(), "test_method", Some(params.clone()));
        
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test_method");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_create_notification() {
        let params = serde_json::json!({"key": "value"});
        let notification = create_notification("test_notification", Some(params));
        
        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method, "test_notification");
    }

    #[test]
    fn test_create_success_response() {
        let id = RequestId::Number(1);
        let result = serde_json::json!({"success": true});
        let response = create_success_response(id, result);
        
        assert_eq!(response.jsonrpc, "2.0");
        match response.payload {
            ResponsePayload::Result { result } => {
                assert_eq!(result["success"], true);
            }
            _ => panic!("Expected Result payload"),
        }
    }

    #[test]
    fn test_create_error_response() {
        let id = RequestId::String("error-id".to_string());
        let response = create_error_response(id, -32600, "Invalid Request", None);
        
        assert_eq!(response.jsonrpc, "2.0");
        match response.payload {
            ResponsePayload::Error { error } => {
                assert_eq!(error.code, -32600);
                assert_eq!(error.message, "Invalid Request");
            }
            _ => panic!("Expected Error payload"),
        }
    }

    #[test]
    fn test_mcp_tool_creation() {
        let tool = MCPTool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: Some(serde_json::json!({  // 注意：Option<Value>
                "type": "object",
                "properties": {
                    "param1": {"type": "string"}
                }
            })),
        };
        
        assert_eq!(tool.name, "test_tool");
        assert!(tool.description.is_some());
    }

    #[test]
    fn test_mcp_content_text() {
        let content = MCPContent::text("Hello, world!");
        assert_eq!(content.content_type, "text");
        assert_eq!(content.text, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_mcp_content_image() {
        let content = MCPContent::image("base64data", "image/png");
        assert_eq!(content.content_type, "image");
        assert_eq!(content.data, Some("base64data".to_string()));
        assert_eq!(content.mime_type, Some("image/png".to_string()));
    }

    #[test]
    fn test_serialize_request() {
        let request = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::String("test".to_string()),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({})),
        };
        
        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"method\":\"initialize\""));
    }

    #[test]
    fn test_deserialize_response() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": "test-id",
            "result": {"success": true}
        }"#;
        
        let response: MCPResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        match response.payload {
            ResponsePayload::Result { result } => {
                assert_eq!(result["success"], true);
            }
            _ => panic!("Expected Result"),
        }
    }
}

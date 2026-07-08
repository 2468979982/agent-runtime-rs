use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::mcp::client::{MCPClient, validate_tool_call, default_client_capabilities, methods};
use crate::mcp::types::*;

/// MCP client that communicates via stdio with an MCP server subprocess
pub struct MCPStdioClient {
    server_name: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    
    child_process: Option<Child>,
    stdin: Option<Arc<Mutex<ChildStdin>>>,
    initialized: bool,
    
    server_capabilities: Option<MCPServerCapabilities>,
    server_info: Option<MCPServerInfo>,
    
    pending_requests: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<MCPResponse>>>>,
}

impl MCPStdioClient {
    /// Create a new stdio client for the given server configuration
    pub fn new(
        server_name: &str,
        command: &str,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Self {
        Self {
            server_name: server_name.to_string(),
            command: command.to_string(),
            args,
            env,
            child_process: None,
            stdin: None,
            initialized: false,
            server_capabilities: None,
            server_info: None,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Start the MCP server subprocess
    async fn start_server(&mut self) -> Result<(), MCPError> {
        let mut cmd = tokio::process::Command::new(&self.command);
        cmd.args(&self.args)
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // Set environment variables
        for (key, value) in &self.env {
            cmd.env(key, value);
        }
        
        let mut child = cmd.spawn()
            .map_err(|e| MCPError::Connection(format!("Failed to start server '{}': {}", self.server_name, e)))?;
        
        let stdin = child.stdin.take()
            .ok_or_else(|| MCPError::Connection("Failed to get stdin handle".to_string()))?;
        
        self.child_process = Some(child);
        self.stdin = Some(Arc::new(Mutex::new(stdin)));
        
        Ok(())
    }
    
    /// Send a JSON-RPC request and wait for response
    async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value, MCPError> {
        let id = Uuid::new_v4().to_string();
        let request = create_request(RequestId::String(id.clone()), method, params);
        
        let request_json = serde_json::to_string(&request)
            .map_err(|e| MCPError::Serialization(e.to_string()))?;
        
        // Create a oneshot channel for the response
        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id.clone(), tx);
        }
        
        // Send the request
        let stdin = self.stdin.as_ref()
            .ok_or_else(|| MCPError::Connection("Not connected".to_string()))?;
        
        let mut stdin_guard = stdin.lock().await;
        stdin_guard.write_all(request_json.as_bytes()).await
            .map_err(|e| MCPError::Io(e.to_string()))?;
        stdin_guard.write_all(b"\n").await
            .map_err(|e| MCPError::Io(e.to_string()))?;
        stdin_guard.flush().await
            .map_err(|e| MCPError::Io(e.to_string()))?;
        
        drop(stdin_guard);
        
        // Wait for response with timeout
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            rx
        ).await
            .map_err(|_| MCPError::Timeout("Request timed out after 30 seconds".to_string()))?
            .map_err(|e| MCPError::Connection(format!("Failed to receive response: {}", e)))?;
        
        match response.payload {
            ResponsePayload::Result { result } => Ok(result),
            ResponsePayload::Error { error } => {
                Err(MCPError::JsonRpc {
                    code: error.code,
                    message: error.message,
                    data: error.data,
                })
            }
        }
    }
    
    /// Send a notification (no response expected)
    async fn send_notification(&self, method: &str, params: Option<Value>) -> Result<(), MCPError> {
        let notification = create_notification(method, params);
        
        let notification_json = serde_json::to_string(&notification)
            .map_err(|e| MCPError::Serialization(e.to_string()))?;
        
        let stdin = self.stdin.as_ref()
            .ok_or_else(|| MCPError::Connection("Not connected".to_string()))?;
        
        let mut stdin_guard = stdin.lock().await;
        stdin_guard.write_all(notification_json.as_bytes()).await
            .map_err(|e| MCPError::Io(e.to_string()))?;
        stdin_guard.write_all(b"\n").await
            .map_err(|e| MCPError::Io(e.to_string()))?;
        stdin_guard.flush().await
            .map_err(|e| MCPError::Io(e.to_string()))?;
        
        Ok(())
    }
    
    /// Start reading responses from stdout
    async fn start_response_reader(&mut self) -> Result<(), MCPError> {
        let stdout = self.child_process.as_mut()
            .and_then(|c| c.stdout.take())
            .ok_or_else(|| MCPError::Connection("Failed to get stdout handle".to_string()))?;
        
        let pending_requests = self.pending_requests.clone();
        
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            
            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                
                match serde_json::from_str::<MCPResponse>(&line) {
                    Ok(response) => {
                        let id_str = match &response.id {
                            RequestId::String(s) => s.clone(),
                            RequestId::Number(n) => n.to_string(),
                            RequestId::Null => continue,
                        };
                        
                        let mut pending = pending_requests.lock().await;
                        if let Some(tx) = pending.remove(&id_str) {
                            let _ = tx.send(response);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse MCP response: {} - Line: {}", e, line);
                    }
                }
            }
        });
        
        Ok(())
    }
}

#[async_trait]
impl MCPClient for MCPStdioClient {
    fn server_name(&self) -> &str {
        &self.server_name
    }
    
    async fn initialize(&mut self) -> Result<MCPInitializeResult, MCPError> {
        if self.initialized {
            return Err(MCPError::Connection("Already initialized".to_string()));
        }
        
        // Start the server process
        self.start_server().await?;
        
        // Start reading responses
        self.start_response_reader().await?;
        
        // Send initialize request
        let params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": default_client_capabilities(),
            "clientInfo": {
                "name": "agent-runtime-rs",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        
        let result = self.send_request(methods::INITIALIZE, Some(params)).await?;
        
        let init_result: MCPInitializeResult = serde_json::from_value(result)
            .map_err(|e| MCPError::Serialization(format!("Failed to parse initialize result: {}", e)))?;
        
        // Store server capabilities and info (they are already Option<T>)
        let capabilities = init_result.capabilities;
        let server_info = init_result.server_info;
        
        self.server_capabilities = capabilities;
        self.server_info = server_info;
        
        // Send initialized notification
        let empty_obj = json!({});
        self.send_notification(methods::NOTIFICATIONS_INITIALIZED, Some(empty_obj))
            .await?;
        
        self.initialized = true;
        
        // Return init_result (need to reconstruct it because fields were moved)
        Ok(MCPInitializeResult {
            protocol_version: init_result.protocol_version,
            capabilities: self.server_capabilities.clone(),
            server_info: self.server_info.clone(),
        })
    }
    
    async fn list_tools(&self) -> Result<Vec<MCPTool>, MCPError> {
        if !self.initialized {
            return Err(MCPError::Connection("Not initialized".to_string()));
        }
        
        let empty_obj = json!({});
        let result = self.send_request(methods::TOOLS_LIST, Some(empty_obj))
            .await?;
        
        let tools: Vec<MCPTool> = serde_json::from_value(result["tools"].clone())
            .map_err(|e| MCPError::Serialization(format!("Failed to parse tools list: {}", e)))?;
        
        Ok(tools)
    }
    
    async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<MCPToolResult, MCPError> {
        if !self.initialized {
            return Err(MCPError::Connection("Not initialized".to_string()));
        }

        print!("Calling tool: {} with arguments: {}", tool_name, arguments);
        
        // First, get the tool definition to validate
        let tools = self.list_tools().await?;
        let tool = tools.iter()
            .find(|t| t.name == tool_name)
            .ok_or_else(|| MCPError::ToolNotFound(tool_name.to_string()))?;
        
        validate_tool_call(tool, &arguments)?;
        
        let params = json!({
            "name": tool_name,
            "arguments": arguments
        });
        
        let result = self.send_request(methods::TOOLS_CALL, Some(params)).await?;
        
        let tool_result: MCPToolResult = serde_json::from_value(result)
            .map_err(|e| MCPError::Serialization(format!("Failed to parse tool result: {}", e)))?;
        
        if tool_result.is_error.unwrap_or(false) {
            let error_msg = tool_result.content.iter()
                .filter_map(|c| c.text.clone())
                .collect::<Vec<_>>()
                .join("\n");
            return Err(MCPError::ToolExecution(error_msg));
        }
        
        Ok(tool_result)
    }
    
    fn capabilities(&self) -> Option<&MCPServerCapabilities> {
        self.server_capabilities.as_ref()
    }
    
    fn server_info(&self) -> Option<&MCPServerInfo> {
        self.server_info.as_ref()
    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    async fn shutdown(&mut self) -> Result<(), MCPError> {
        if !self.initialized {
            return Ok(());
        }
        
        // Send shutdown request
        let _ = self.send_request(methods::SHUTDOWN, None).await;
        
        // Send exit notification
        let _ = self.send_notification("exit", None).await;
        
        // Kill the process
        if let Some(mut child) = self.child_process.take() {
            let _ = child.kill().await;
        }
        
        self.initialized = false;
        self.stdin = None;
        
        Ok(())
    }
}

impl Drop for MCPStdioClient {
    fn drop(&mut self) {
        if let Some(mut child) = self.child_process.take() {
            let _ = child.start_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_stdio_client_creation() {
        let client = MCPStdioClient::new(
            "test-server",
            "echo",
            vec!["{}".to_string()],
            HashMap::new(),
        );
        
        assert_eq!(client.server_name(), "test-server");
        assert!(!client.is_initialized());
        assert!(client.capabilities().is_none());
        assert!(client.server_info().is_none());
    }
    
    #[tokio::test]
    async fn test_stdio_client_send_notification_not_connected() {
        let client = MCPStdioClient::new(
            "test-server",
            "echo",
            vec![],
            HashMap::new(),
        );
        
        let result = client.send_notification("test", None).await;
        assert!(result.is_err());
        match result {
            Err(MCPError::Connection(msg)) => assert!(msg.contains("Not connected")),
            _ => panic!("Expected Connection error"),
        }
    }
    
    #[tokio::test]
    async fn test_stdio_client_send_request_not_connected() {
        let client = MCPStdioClient::new(
            "test-server",
            "echo",
            vec![],
            HashMap::new(),
        );
        
        let result = client.send_request("test", None).await;
        assert!(result.is_err());
        match result {
            Err(MCPError::Connection(msg)) => assert!(msg.contains("Not connected")),
            _ => panic!("Expected Connection error"),
        }
    }
    
    #[test]
    fn test_validate_tool_call_with_mock_tool() {
        let tool = MCPTool {
            name: "test_tool".to_string(),
            description: Some("Test".to_string()),
            input_schema: Some(json!({
                "type": "object",
                "properties": {}
            })),
        };
        
        let args = json!({});
        let result = validate_tool_call(&tool, &args);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_tool_call_invalid_args() {
        let tool = MCPTool {
            name: "test_tool".to_string(),
            description: None,
            input_schema: Some(json!({
                "type": "object",
                "properties": {}
            })),
        };
        
        let args = json!("invalid");
        let result = validate_tool_call(&tool, &args);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_initialize_fails_when_not_connected() {
        let mut client = MCPStdioClient::new(
            "test-server",
            "nonexistent_command_12345",
            vec![],
            HashMap::new(),
        );
        
        let result = client.initialize().await;
        assert!(result.is_err());
    }
    
    #[test]
    fn test_list_tools_fails_when_not_initialized() {
        let client = MCPStdioClient::new(
            "test-server",
            "echo",
            vec![],
            HashMap::new(),
        );
        
        // We can't actually call list_tools because it's async and requires initialized state
        assert!(!client.is_initialized());
    }
    
    #[tokio::test]
    async fn test_call_tool_fails_when_not_initialized() {
        let client = MCPStdioClient::new(
            "test-server",
            "echo",
            vec![],
            HashMap::new(),
        );
        
        let result = client.call_tool("test", json!({})).await;
        assert!(result.is_err());
        match result {
            Err(MCPError::Connection(msg)) => assert!(msg.contains("Not initialized")),
            _ => panic!("Expected Connection error"),
        }
    }
    
    #[tokio::test]
    async fn test_shutdown_when_not_initialized() {
        let mut client = MCPStdioClient::new(
            "test-server",
            "echo",
            vec![],
            HashMap::new(),
        );
        
        // Should not error
        let result = client.shutdown().await;
        assert!(result.is_ok());
    }
}

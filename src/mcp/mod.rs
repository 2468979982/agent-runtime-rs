//! MCP (Model Context Protocol) module
//! 
//! This module implements the MCP client for communicating with MCP servers
//! via different transport layers (stdio, HTTP, WebSocket).
//! 
//! # Architecture
//! 
//! - `types`: Core type definitions for MCP protocol (JSON-RPC 2.0 messages, tools, etc.)
//! - `client`: `MCPClient` trait defining the interface for MCP clients
//! - `stdio_client`: `MCPStdioClient` implementing stdio transport
//! - `config`: Configuration loading for MCP servers
//! 
//! # Example
//! 
//! ```rust,no_run
//! use agent_runtime_rs::mcp::stdio_client::MCPStdioClient;
//! use agent_runtime_rs::mcp::client::MCPClient;
//! use std::collections::HashMap;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = MCPStdioClient::new(
//!         "my-server",
//!         "node",
//!         vec!["server.js".to_string()],
//!         HashMap::new(),
//!     );
//!     
//!     // Initialize the connection
//!     let init_result = client.initialize().await?;
//!     println!("Server: {}", init_result.server_info.name);
//!     
//!     // List available tools
//!     let tools = client.list_tools().await?;
//!     for tool in &tools {
//!         println!("Tool: {}", tool.name);
//!     }
//!     
//!     // Call a tool
//!     let result = client.call_tool("echo", serde_json::json!({"message": "Hello"})).await?;
//!     println!("Result: {:?}", result);
//!     
//!     // Shutdown
//!     client.shutdown().await?;
//!     Ok(())
//! }
//! ```

pub mod types;
pub mod client;
pub mod stdio_client;
pub mod config;

pub use types::*;
pub use client::MCPClient;
pub use stdio_client::MCPStdioClient;
pub use config::*;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::collections::HashMap;
    
    /// Mock MCP server for testing
    /// This creates a simple echo server that responds to MCP requests
    #[allow(dead_code)]
    async fn start_mock_mcp_server() -> std::process::Child {
        // In a real test, we would start a Python/Node script that implements a mock MCP server
        // For now, this is a placeholder
        todo!("Implement mock MCP server for integration tests")
    }
    
    #[tokio::test]
    #[ignore = "Requires mock MCP server implementation"]
    async fn test_full_mcp_workflow() {
        // This test would:
        // 1. Start a mock MCP server
        // 2. Initialize the client
        // 3. List tools
        // 4. Call a tool
        // 5. Shutdown
        
        let mut client = MCPStdioClient::new(
            "mock-server",
            "python",
            vec!["mock_mcp_server.py".to_string()],
            HashMap::new(),
        );
        
        let init_result = client.initialize().await.unwrap();
        assert!(!init_result.server_info.name.is_empty());
        
        let tools = client.list_tools().await.unwrap();
        assert!(!tools.is_empty());
        
        let result = client.call_tool("echo", serde_json::json!({"message": "test"})).await.unwrap();
        assert!(!result.content.is_empty());
        
        client.shutdown().await.unwrap();
    }
}

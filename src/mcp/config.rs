use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::config::types::ToolsConfig;
use crate::error::Result;
use crate::mcp::stdio_client::MCPStdioClient;
use crate::mcp::client::MCPClient;
use crate::config::types::MCPServers;

/// Load MCP servers from ToolsConfig
/// Supports both legacy format (array) and new format (object)
pub fn load_mcp_servers_from_config(config: &ToolsConfig) -> HashMap<String, MCPServerConfig> {
    let mut servers = HashMap::new();
    
    // Handle the MCPServers enum
    match &config.mcp_servers {
        MCPServers::New(new_servers) => {
            // New format: object with server names as keys
            for (name, server_config) in new_servers {
                servers.insert(
                    name.clone(),
                    MCPServerConfig {
                        name: name.clone(),
                        transport: MCPServerTransport::Stdio {
                            command: server_config.command.clone(),
                            args: server_config.args.clone(),
                            env: server_config.env.clone().unwrap_or_default(),
                        },
                        enabled: !server_config.disabled.unwrap_or(false),
                    }
                );
            }
        }
        MCPServers::Old(old_servers) => {
            // Old format: array of server configs
            for server_config in old_servers {
                servers.insert(
                    server_config.name.clone(),
                    MCPServerConfig {
                        name: server_config.name.clone(),
                        transport: MCPServerTransport::Http {
                            url: server_config.url.clone(),
                            headers: None,
                        },
                        enabled: true,
                    }
                );
            }
        }
    }
    
    servers
}

/// MCP server configuration for the client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub name: String,
    pub transport: MCPServerTransport,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MCPServerTransport {
    #[serde(rename = "stdio")]
    Stdio {
        command: String,
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },
    #[serde(rename = "http")]
    Http {
        url: String,
        headers: Option<HashMap<String, String>>,
    },
    #[serde(rename = "websocket")]
    WebSocket {
        url: String,
    },
}

/// Create MCPStdioClient instances from configuration
pub fn create_stdio_clients(
    config: &ToolsConfig
) -> Vec<(String, Box<dyn MCPClient>)> {
    let servers = load_mcp_servers_from_config(config);
    let mut clients: Vec<(String, Box<dyn MCPClient>)> = Vec::new();
    
    for (name, server_config) in servers {
        if !server_config.enabled {
            continue;
        }
        
        match server_config.transport {
            MCPServerTransport::Stdio { command, args, env } => {
                let client = MCPStdioClient::new(&name, &command, args, env);
                clients.push((name.clone(), Box::new(client)));
            }
            MCPServerTransport::Http { .. } => {
                eprintln!("HTTP transport not yet implemented for server: {}", name);
            }
            MCPServerTransport::WebSocket { .. } => {
                eprintln!("WebSocket transport not yet implemented for server: {}", name);
            }
        }
    }
    
    clients
}

/// Validate MCP server configuration
pub fn validate_mcp_config(config: &ToolsConfig) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    
    match &config.mcp_servers {
        MCPServers::New(servers) => {
            for (name, server) in servers {
                if server.command.is_empty() {
                    errors.push(format!("MCP server '{}' has empty command", name));
                }
            }
        }
        MCPServers::Old(servers) => {
            for server in servers {
                if server.url.is_empty() {
                    errors.push(format!("MCP server '{}' has empty URL", server.name));
                }
            }
        }
    }
    
    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::config::types::{MCPServers, MCPServerConfigNew};
    
    fn create_test_config_new_format() -> ToolsConfig {
        let mut servers = HashMap::new();
        servers.insert(
            "test-server".to_string(),
            MCPServerConfigNew {
                command: "node".to_string(),
                args: vec!["server.js".to_string()],
                env: None,
                disabled: None,
                description: None,
            }
        );
        
        ToolsConfig {
            mcp_servers: MCPServers::New(servers),
            builtin_tools: vec![],
            auto_execute_tools: None,
            mcp_tool_definitions: None,
        }
    }
    
    #[test]
    fn test_load_mcp_servers_from_config_new_format() {
        let config = create_test_config_new_format();
        let servers = load_mcp_servers_from_config(&config);
        
        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("test-server"));
        
        let server = &servers["test-server"];
        assert_eq!(server.name, "test-server");
        assert!(server.enabled);
        
        match &server.transport {
            MCPServerTransport::Stdio { command, args, .. } => {
                assert_eq!(command, "node");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], "server.js");
            }
            _ => panic!("Expected Stdio transport"),
        }
    }
    
    #[test]
    fn test_load_mcp_servers_empty_config() {
        let config = ToolsConfig {
            mcp_servers: MCPServers::New(HashMap::new()),
            builtin_tools: vec![],
            auto_execute_tools: None,
            mcp_tool_definitions: None,
        };
        
        let servers = load_mcp_servers_from_config(&config);
        assert_eq!(servers.len(), 0);
    }
    
    #[test]
    fn test_create_stdio_clients() {
        let config = create_test_config_new_format();
        let clients = create_stdio_clients(&config);
        
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].0, "test-server");
    }
    
    #[test]
    fn test_validate_mcp_config_valid() {
        let config = create_test_config_new_format();
        let result = validate_mcp_config(&config);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert_eq!(errors.len(), 0);
    }
    
    #[test]
    fn test_validate_mcp_config_empty_command() {
        let mut servers = HashMap::new();
        servers.insert(
            "bad-server".to_string(),
            MCPServerConfigNew {
                command: "".to_string(),
                args: vec![],
                env: None,
                disabled: None,
                description: None,
            }
        );
        
        let config = ToolsConfig {
            mcp_servers: MCPServers::New(servers),
            builtin_tools: vec![],
            auto_execute_tools: None,
            mcp_tool_definitions: None,
        };
        
        let result = validate_mcp_config(&config);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("empty command"));
    }
    
    #[test]
    fn test_mcp_server_config_serialization() {
        let config = MCPServerConfig {
            name: "test".to_string(),
            transport: MCPServerTransport::Stdio {
                command: "python".to_string(),
                args: vec!["script.py".to_string()],
                env: HashMap::new(),
            },
            enabled: true,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("stdio"));
        assert!(json.contains("python"));
    }
}

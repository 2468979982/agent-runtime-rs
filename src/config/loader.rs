use std::fs;
use std::path::Path;

use crate::config::types::*;
use crate::error::ConfigError;

/// Configuration loader for agent runtime
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load and parse agent configuration file
    pub fn load_agent_config<P: AsRef<Path>>(path: P) -> Result<AgentConfig, ConfigError> {
        // Load as raw JSON value (to support ${ENV:...} substitution)
        let mut config_value: serde_json::Value = Self::load_and_parse_json(path)?;
        
        tracing::debug!("Config before env substitution: {}", config_value);
        
        // Substitute environment variables
        Self::substitute_env_variables(&mut config_value)?;
        
        tracing::debug!("Config after env substitution: {}", config_value);
        
        // Convert to AgentConfig
        let config: AgentConfig = serde_json::from_value(config_value)
            .map_err(|e| ConfigError::JsonParseError(e.to_string()))?;
        
        Self::validate_agent_config(&config)?;
        Ok(config)
    }

    /// Load raw agent configuration file (without environment variable substitution)
    pub fn load_raw_agent_config<P: AsRef<Path>>(path: P) -> Result<serde_json::Value, ConfigError> {
        let config = Self::load_and_parse_json(path)?;
        Self::validate_agent_config_raw(&config)?;
        Ok(config)
    }

    /// Load and parse tools configuration file
    pub fn load_tools_config<P: AsRef<Path>>(path: P) -> Result<ToolsConfig, ConfigError> {
        let config: ToolsConfig = Self::load_and_parse(path)?;
        Self::validate_tools_config(&config)?;
        Ok(config)
    }

    /// Load and parse prompt configuration file
    pub fn load_prompt_config<P: AsRef<Path>>(path: P) -> Result<PromptConfig, ConfigError> {
        let config: PromptConfig = Self::load_and_parse(path)?;
        Self::validate_prompt_config(&config)?;
        Ok(config)
    }

    /// Save agent configuration to file
    pub fn save_agent_config<P: AsRef<Path>>(path: P, config: &AgentConfig) -> Result<(), ConfigError> {
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;
        
        fs::write(path, json)
            .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
        
        Ok(())
    }

    /// Load config file and parse JSON
    fn load_and_parse<T: serde::de::DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, ConfigError> {
        let path_ref = path.as_ref();
        
        // Check if file exists
        if !path_ref.exists() {
            return Err(ConfigError::FileNotFound(path_ref.to_string_lossy().to_string()));
        }

        // Read file content
        let content = fs::read_to_string(path_ref)
            .map_err(|e| ConfigError::FileReadError(e.to_string()))?;

        // Parse JSON
        let config: T = serde_json::from_str(&content)
            .map_err(|e| ConfigError::JsonParseError(e.to_string()))?;

        Ok(config)
    }

    /// Load config file and parse as raw JSON value
    fn load_and_parse_json<P: AsRef<Path>>(path: P) -> Result<serde_json::Value, ConfigError> {
        let path_ref = path.as_ref();
        
        if !path_ref.exists() {
            return Err(ConfigError::FileNotFound(path_ref.to_string_lossy().to_string()));
        }

        let content = fs::read_to_string(path_ref)
            .map_err(|e| ConfigError::FileReadError(e.to_string()))?;

        let config: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ConfigError::JsonParseError(e.to_string()))?;

        Ok(config)
    }

    /// Substitute environment variables in config
    /// Syntax: ${ENV:VAR_NAME}
    pub fn substitute_env_variables(value: &mut serde_json::Value) -> Result<(), ConfigError> {
        match value {
            serde_json::Value::String(s) => {
                *value = serde_json::Value::String(Self::substitute_string_env_vars(s)?);
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    Self::substitute_env_variables(item)?;
                }
            }
            serde_json::Value::Object(obj) => {
                let keys: Vec<String> = obj.keys().cloned().collect();
                for key in keys {
                    if let Some(val) = obj.get_mut(&key) {
                        Self::substitute_env_variables(val)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Substitute environment variables in a string
    fn substitute_string_env_vars(s: &str) -> Result<String, ConfigError> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                
                // Check if it's ${ENV:VAR_NAME}
                let mut prefix = String::new();
                for _ in 0..4 {
                    if let Some(ch) = chars.next() {
                        prefix.push(ch);
                    }
                }
                
                if prefix == "ENV:" {
                    // Read variable name until '}'
                    let mut var_name = String::new();
                    while let Some(ch) = chars.next() {
                        if ch == '}' {
                            break;
                        }
                        var_name.push(ch);
                    }
                    
                    let value = std::env::var(&var_name)
                        .map_err(|_| ConfigError::EnvVariableNotFound(var_name.clone()))?;
                    
                    result.push_str(&value);
                } else {
                    // Not ${ENV: pattern, keep original
                    result.push('$');
                    result.push('{');
                    result.push_str(&prefix);
                }
            } else {
                result.push(c);
            }
        }
        
        Ok(result)
    }

    /// Validate agent configuration
    fn validate_agent_config(config: &AgentConfig) -> Result<(), ConfigError> {
        // Validate LLM config
        match config.llm.provider {
            LLMProvider::OpenAI | LLMProvider::OpenAICompatible => {}
        }

        if let Some(temp) = config.llm.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(ConfigError::ValidationError(
                    "LLM temperature must be between 0 and 2".to_string()
                ));
            }
        }

        if let Some(max_tokens) = config.llm.max_tokens {
            if max_tokens < 1 {
                return Err(ConfigError::ValidationError(
                    "LLM max_tokens must be at least 1".to_string()
                ));
            }
        }

        if config.session.max_history_length < 1 {
            return Err(ConfigError::ValidationError(
                "session.max_history_length must be at least 1".to_string()
            ));
        }

        Ok(())
    }

    /// Validate raw agent configuration (without typing)
    fn validate_agent_config_raw(config: &serde_json::Value) -> Result<(), ConfigError> {
        if let Some(llm) = config.get("llm") {
            if let Some(temp) = llm.get("temperature").and_then(|v| v.as_f64()) {
                if temp < 0.0 || temp > 2.0 {
                    return Err(ConfigError::ValidationError(
                        "LLM temperature must be between 0 and 2".to_string()
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate tools configuration
    fn validate_tools_config(config: &ToolsConfig) -> Result<(), ConfigError> {
        match &config.mcp_servers {
            MCPServers::New(servers) => {
                for (name, server) in servers {
                    if server.command.is_empty() {
                        return Err(ConfigError::ValidationError(
                            format!("MCP server '{}' must have a 'command' field", name)
                        ));
                    }
                }
            }
            MCPServers::Old(servers) => {
                for server in servers {
                    if server.name.is_empty() || server.url.is_empty() {
                        return Err(ConfigError::ValidationError(
                            "Old MCP config format requires 'name' and 'url' fields".to_string()
                        ));
                    }
                }
                tracing::warn!("Old MCP config format detected (array). Please update to new format (object with command/args).");
            }
        }

        // Validate builtin tools
        for tool in &config.builtin_tools {
            if tool.name.is_empty() || tool.description.is_empty() {
                return Err(ConfigError::ValidationError(
                    "Each builtin tool must have name and description".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Validate prompt configuration
    fn validate_prompt_config(config: &PromptConfig) -> Result<(), ConfigError> {
        if config.system_prompt.is_empty() {
            return Err(ConfigError::ValidationError(
                "system_prompt is required".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_agent_config() {
        let config_json = r#"{
            "llm": {
                "provider": "openai",
                "apiKey": "test-key",
                "model": "gpt-4"
            },
            "session": {
                "maxHistoryLength": 10
            },
            "logging": {
                "level": "info"
            }
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", config_json).unwrap();

        let result = ConfigLoader::load_agent_config(file.path());
        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.llm.provider, LLMProvider::OpenAI);
        assert_eq!(config.llm.api_key, "test-key");
        assert_eq!(config.session.max_history_length, 10);
    }

    #[test]
    fn test_load_invalid_agent_config() {
        let config_json = r#"{
            "llm": {
                "provider": "invalid",
                "apiKey": "test-key"
            },
            "session": {
                "maxHistoryLength": 10
            }
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", config_json).unwrap();

        let result = ConfigLoader::load_agent_config(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_tools_config() {
        let config_json = r#"{
            "mcpServers": {
                "test-server": {
                    "command": "node",
                    "args": ["server.js"]
                }
            },
            "builtinTools": []
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", config_json).unwrap();

        let result = ConfigLoader::load_tools_config(file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_prompt_config() {
        let config_json = r#"{
            "systemPrompt": "You are a helpful assistant."
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", config_json).unwrap();

        let result = ConfigLoader::load_prompt_config(file.path());
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.system_prompt, "You are a helpful assistant.");
    }

    #[test]
    fn test_file_not_found() {
        let result = ConfigLoader::load_agent_config("nonexistent.json");
        assert!(matches!(result, Err(ConfigError::FileNotFound(_))));
    }

    #[test]
    fn test_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "invalid json").unwrap();

        let result = ConfigLoader::load_agent_config(file.path());
        assert!(matches!(result, Err(ConfigError::JsonParseError(_))));
    }

    #[test]
    fn test_env_variable_substitution() {
        std::env::set_var("TEST_VAR", "test_value");
        
        let mut value = serde_json::Value::String("${ENV:TEST_VAR}".to_string());
        let result = ConfigLoader::substitute_env_variables(&mut value);
        
        assert!(result.is_ok());
        assert_eq!(value, serde_json::Value::String("test_value".to_string()));
        
        std::env::remove_var("TEST_VAR");
    }
}

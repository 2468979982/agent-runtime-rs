use crate::tools::types::*;
use async_trait::async_trait;
use std::fs;

/// Tool for creating directories
pub struct DirectoryCreatorTool {
    metadata: ToolMetadata,
}

impl DirectoryCreatorTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "directory_creator".to_string(),
                description: "Create a new directory. Can create nested directories if they don't exist.".to_string(),
                parameters: vec![
                    ToolParameter {
                        name: "path".to_string(),
                        description: "Path of the directory to create".to_string(),
                        required: true,
                        parameter_type: ParameterType::String,
                    },
                    ToolParameter {
                        name: "recursive".to_string(),
                        description: "Whether to create parent directories if they don't exist (default: true)".to_string(),
                        required: false,
                        parameter_type: ParameterType::Boolean,
                    },
                ],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for DirectoryCreatorTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path = parameters["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'path' parameter".to_string()))?;
        
        let recursive = parameters["recursive"].as_bool().unwrap_or(true);
        
        let result = if recursive {
            fs::create_dir_all(path)
        } else {
            fs::create_dir(path)
        };
        
        match result {
            Ok(_) => Ok(ToolResult {
                success: true,
                output: format!("Successfully created directory: {}", path),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to create directory: {}", e)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    
    #[tokio::test]
    async fn test_directory_creator_basic() {
        let tool = DirectoryCreatorTool::new();
        let dir_path = "test_dir_create_basic";
        
        // Ensure directory doesn't exist
        fs::remove_dir_all(dir_path).ok();
        
        let result = tool.execute(json!({
            "path": dir_path
        })).await.unwrap();
        
        assert!(result.success);
        assert!(fs::metadata(dir_path).is_ok());
        assert!(fs::metadata(dir_path).unwrap().is_dir());
        
        // Cleanup
        fs::remove_dir_all(dir_path).ok();
    }
    
    #[tokio::test]
    async fn test_directory_creator_nested() {
        let tool = DirectoryCreatorTool::new();
        let nested_path = "test_dir_nested/level1/level2";
        
        // Ensure directory doesn't exist
        fs::remove_dir_all("test_dir_nested").ok();
        
        let result = tool.execute(json!({
            "path": nested_path,
            "recursive": true
        })).await.unwrap();
        
        assert!(result.success);
        assert!(fs::metadata(nested_path).is_ok());
        assert!(fs::metadata(nested_path).unwrap().is_dir());
        
        // Cleanup
        fs::remove_dir_all("test_dir_nested").ok();
    }
    
    #[tokio::test]
    async fn test_directory_creator_already_exists() {
        let tool = DirectoryCreatorTool::new();
        let dir_path = "test_dir_exists";
        
        // Create directory first
        fs::create_dir_all(dir_path).unwrap();
        
        let result = tool.execute(json!({
            "path": dir_path
        })).await.unwrap();
        
        // Should succeed (idempotent operation)
        assert!(result.success);
        
        // Cleanup
        fs::remove_dir_all(dir_path).ok();
    }
    
    #[tokio::test]
    async fn test_directory_creator_missing_parameter() {
        let tool = DirectoryCreatorTool::new();
        
        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
    }
}

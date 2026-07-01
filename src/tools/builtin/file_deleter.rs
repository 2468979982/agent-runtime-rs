use crate::tools::types::*;
use async_trait::async_trait;
use std::fs;

/// Tool for deleting files
pub struct FileDeleterTool {
    metadata: ToolMetadata,
}

impl FileDeleterTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "file_deleter".to_string(),
                description: "Delete a file or directory. Use with caution!".to_string(),
                parameters: vec![
                    ToolParameter {
                        name: "path".to_string(),
                        description: "Path to the file or directory to delete".to_string(),
                        required: true,
                        parameter_type: ParameterType::String,
                    },
                    ToolParameter {
                        name: "recursive".to_string(),
                        description: "Whether to recursively delete directories (default: false)".to_string(),
                        required: false,
                        parameter_type: ParameterType::Boolean,
                    },
                ],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for FileDeleterTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path = parameters["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'path' parameter".to_string()))?;
        
        let recursive = parameters["recursive"].as_bool().unwrap_or(false);
        
        let path_obj = std::path::Path::new(path);
        
        if !path_obj.exists() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Path does not exist: {}", path)),
            });
        }
        
        let result = if path_obj.is_dir() {
            if recursive {
                fs::remove_dir_all(path)
            } else {
                fs::remove_dir(path)
            }
        } else {
            fs::remove_file(path)
        };
        
        match result {
            Ok(_) => Ok(ToolResult {
                success: true,
                output: format!("Successfully deleted: {}", path),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to delete: {}", e)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_file_deleter_delete_file() {
        let tool = FileDeleterTool::new();
        let temp_file = "test_delete_file.txt";
        
        // Create test file
        fs::write(temp_file, "test").unwrap();
        assert!(fs::metadata(temp_file).is_ok());
        
        let result = tool.execute(json!({
            "path": temp_file
        })).await.unwrap();
        
        assert!(result.success);
        assert!(fs::metadata(temp_file).is_err()); // File should be deleted
    }
    
    #[tokio::test]
    async fn test_file_deleter_delete_empty_dir() {
        let tool = FileDeleterTool::new();
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("empty_dir");
        
        fs::create_dir(&dir_path).unwrap();
        assert!(dir_path.exists());
        
        let result = tool.execute(json!({
            "path": dir_path.to_str().unwrap()
        })).await.unwrap();
        
        assert!(result.success);
        assert!(!dir_path.exists());
    }
    
    #[tokio::test]
    async fn test_file_deleter_delete_nonempty_dir_without_recursive() {
        let tool = FileDeleterTool::new();
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("nonempty_dir");
        
        fs::create_dir(&dir_path).unwrap();
        let file_path = dir_path.join("test.txt");
        fs::write(file_path, "test").unwrap();
        
        let result = tool.execute(json!({
            "path": dir_path.to_str().unwrap(),
            "recursive": false
        })).await.unwrap();
        
        // Should fail because directory is not empty
        assert!(!result.success);
        assert!(result.error.is_some());
    }
    
    #[tokio::test]
    async fn test_file_deleter_delete_nonempty_dir_with_recursive() {
        let tool = FileDeleterTool::new();
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("nonempty_dir");
        
        fs::create_dir(&dir_path).unwrap();
        let file_path = dir_path.join("test.txt");
        fs::write(file_path, "test").unwrap();
        
        let result = tool.execute(json!({
            "path": dir_path.to_str().unwrap(),
            "recursive": true
        })).await.unwrap();
        
        assert!(result.success);
        assert!(!dir_path.exists());
    }
    
    #[tokio::test]
    async fn test_file_deleter_nonexistent_path() {
        let tool = FileDeleterTool::new();
        
        let result = tool.execute(json!({
            "path": "nonexistent_file_or_dir"
        })).await.unwrap();
        
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}

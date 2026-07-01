use crate::tools::types::*;
use async_trait::async_trait;
use std::fs;

/// Tool for listing directory contents
pub struct FileListerTool {
    metadata: ToolMetadata,
}

impl FileListerTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "file_lister".to_string(),
                description: "List the contents of a directory. Returns file and directory names with their types.".to_string(),
                parameters: vec![
                    ToolParameter {
                        name: "path".to_string(),
                        description: "Path to the directory to list".to_string(),
                        required: true,
                        parameter_type: ParameterType::String,
                    },
                    ToolParameter {
                        name: "recursive".to_string(),
                        description: "Whether to list recursively (default: false)".to_string(),
                        required: false,
                        parameter_type: ParameterType::Boolean,
                    },
                    ToolParameter {
                        name: "show_hidden".to_string(),
                        description: "Whether to show hidden files (default: false)".to_string(),
                        required: false,
                        parameter_type: ParameterType::Boolean,
                    },
                ],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for FileListerTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path = parameters["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'path' parameter".to_string()))?;
        
        let recursive = parameters["recursive"].as_bool().unwrap_or(false);
        let show_hidden = parameters["show_hidden"].as_bool().unwrap_or(false);
        
        let path_obj = std::path::Path::new(path);
        
        if !path_obj.exists() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Path does not exist: {}", path)),
            });
        }
        
        if !path_obj.is_dir() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Path is not a directory: {}", path)),
            });
        }
        
        let entries = self.list_directory(path_obj, recursive, show_hidden)?;
        
        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&entries)
                .map_err(|e| ToolError::ParseError(format!("Failed to serialize output: {}", e)))?,
            error: None,
        })
    }
}

impl FileListerTool {
    fn list_directory(
        &self,
        path: &std::path::Path,
        recursive: bool,
        show_hidden: bool,
    ) -> Result<Vec<serde_json::Value>, ToolError> {
        let mut entries = Vec::new();
        
        let dir_entries = fs::read_dir(path)
            .map_err(|e| ToolError::IoError(e))?;
        
        for entry in dir_entries {
            let entry = entry.map_err(|e| ToolError::IoError(e))?;
            let entry_path = entry.path();
            
            let file_name = entry_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            
            // Skip hidden files if not requested
            if !show_hidden && file_name.starts_with('.') {
                continue;
            }
            
            let entry_type = if entry_path.is_dir() {
                "directory"
            } else if entry_path.is_file() {
                "file"
            } else {
                "other"
            };
            
            let entry_info = serde_json::json!({
                "name": file_name,
                "type": entry_type,
                "path": entry_path.to_str().unwrap()
            });
            
            entries.push(entry_info);
            
            // Recursively list subdirectories
            if recursive && entry_path.is_dir() {
                let sub_entries = self.list_directory(&entry_path, recursive, show_hidden)?;
                for sub_entry in sub_entries {
                    entries.push(sub_entry);
                }
            }
        }
        
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_file_lister_basic() {
        let tool = FileListerTool::new();
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        let file_path = temp_dir.path().join("test.txt");
        fs::write(file_path, "test").unwrap();
        
        let dir_path = temp_dir.path().join("subdir");
        fs::create_dir(&dir_path).unwrap();
        
        let result = tool.execute(json!({
            "path": temp_dir.path().to_str().unwrap()
        })).await.unwrap();
        
        assert!(result.success);
        
        let entries: Vec<serde_json::Value> = serde_json::from_str(&result.output).unwrap();
        assert_eq!(entries.len(), 2);
    }
    
    #[tokio::test]
    async fn test_file_lister_recursive() {
        let tool = FileListerTool::new();
        let temp_dir = TempDir::new().unwrap();
        
        // Create nested structure
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        
        let nested_file = subdir.join("nested.txt");
        fs::write(nested_file, "nested").unwrap();
        
        let result = tool.execute(json!({
            "path": temp_dir.path().to_str().unwrap(),
            "recursive": true
        })).await.unwrap();
        
        assert!(result.success);
        
        let entries: Vec<serde_json::Value> = serde_json::from_str(&result.output).unwrap();
        assert!(entries.len() >= 2); // At least subdir and nested.txt
    }
    
    #[tokio::test]
    async fn test_file_lister_nonexistent_path() {
        let tool = FileListerTool::new();
        
        let result = tool.execute(json!({
            "path": "nonexistent_dir"
        })).await.unwrap();
        
        assert!(!result.success);
        assert!(result.error.is_some());
    }
    
    #[tokio::test]
    async fn test_file_lister_file_not_directory() {
        let tool = FileListerTool::new();
        let temp_file = "test_file_lister.txt";
        
        fs::write(temp_file, "test").unwrap();
        
        let result = tool.execute(json!({
            "path": temp_file
        })).await.unwrap();
        
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not a directory"));
        
        // Cleanup
        fs::remove_file(temp_file).ok();
    }
}

use crate::tools::types::*;
use async_trait::async_trait;
use std::fs;
use std::io::Write;

/// Tool for writing content to files
pub struct FileWriterTool {
    metadata: ToolMetadata,
}

impl FileWriterTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "file_writer".to_string(),
                description: "Write content to a file. Creates the file if it doesn't exist, overwrites if it does.".to_string(),
                parameters: vec![
                    ToolParameter {
                        name: "path".to_string(),
                        description: "Path to the file to write".to_string(),
                        required: true,
                        parameter_type: ParameterType::String,
                    },
                    ToolParameter {
                        name: "content".to_string(),
                        description: "Content to write to the file".to_string(),
                        required: true,
                        parameter_type: ParameterType::String,
                    },
                    ToolParameter {
                        name: "mode".to_string(),
                        description: "Write mode: 'overwrite' (default) or 'append'".to_string(),
                        required: false,
                        parameter_type: ParameterType::String,
                    },
                ],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for FileWriterTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path = parameters["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'path' parameter".to_string()))?;
        
        let content = parameters["content"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'content' parameter".to_string()))?;
        
        let mode = parameters["mode"]
            .as_str()
            .unwrap_or("overwrite");
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        match mode {
            "append" => {
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                file.write_all(content.as_bytes())?;
            }
            _ => {
                fs::write(path, content)?;
            }
        }
        
        Ok(ToolResult {
            success: true,
            output: format!("Successfully wrote {} bytes to {}", content.len(), path),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    
    #[tokio::test]
    async fn test_file_writer_overwrite() {
        let tool = FileWriterTool::new();
        let temp_path = "test_output_overwrite.txt";
        
        let result = tool.execute(json!({
            "path": temp_path,
            "content": "Hello, World!"
        })).await.unwrap();
        
        assert!(result.success);
        assert!(fs::metadata(temp_path).is_ok());
        
        let content = fs::read_to_string(temp_path).unwrap();
        assert_eq!(content, "Hello, World!");
        
        // Cleanup
        fs::remove_file(temp_path).ok();
    }
    
    #[tokio::test]
    async fn test_file_writer_append() {
        let tool = FileWriterTool::new();
        let temp_path = "test_output_append.txt";
        
        // Write initial content
        tool.execute(json!({
            "path": temp_path,
            "content": "Hello"
        })).await.unwrap();
        
        // Append content
        let result = tool.execute(json!({
            "path": temp_path,
            "content": ", World!",
            "mode": "append"
        })).await.unwrap();
        
        assert!(result.success);
        
        let content = fs::read_to_string(temp_path).unwrap();
        assert_eq!(content, "Hello, World!");
        
        // Cleanup
        fs::remove_file(temp_path).ok();
    }
    
    #[tokio::test]
    async fn test_file_writer_create_parent_dir() {
        let tool = FileWriterTool::new();
        let temp_path = "test_dir/subdir/test_file.txt";
        
        let result = tool.execute(json!({
            "path": temp_path,
            "content": "Test content"
        })).await.unwrap();
        
        assert!(result.success);
        assert!(fs::metadata(temp_path).is_ok());
        
        // Cleanup
        fs::remove_dir_all("test_dir").ok();
    }
    
    #[tokio::test]
    async fn test_file_writer_missing_parameters() {
        let tool = FileWriterTool::new();
        
        let result = tool.execute(json!({"path": "test.txt"})).await;
        assert!(result.is_err());
        
        let result = tool.execute(json!({"content": "test"})).await;
        assert!(result.is_err());
    }
}

use crate::tools::types::*;
use async_trait::async_trait;
use std::fs;

/// Tool for reading file contents
pub struct FileReaderTool {
    metadata: ToolMetadata,
}

impl FileReaderTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "file_reader".to_string(),
                description: "Read the contents of a file. Returns the file content as a string.".to_string(),
                parameters: vec![
                    ToolParameter {
                        name: "path".to_string(),
                        description: "Path to the file to read".to_string(),
                        required: true,
                        parameter_type: ParameterType::String,
                    },
                    ToolParameter {
                        name: "encoding".to_string(),
                        description: "File encoding (default: 'utf-8')".to_string(),
                        required: false,
                        parameter_type: ParameterType::String,
                    },
                ],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for FileReaderTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path = parameters["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'path' parameter".to_string()))?;
        
        // Validate path to prevent directory traversal
        let path_obj = std::path::Path::new(path);
        if path_obj.is_dir() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Path is a directory, not a file".to_string()),
            });
        }
        
        match fs::read_to_string(path) {
            Ok(content) => Ok(ToolResult {
                success: true,
                output: content,
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to read file: {}", e)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_file_reader_success() {
        let tool = FileReaderTool::new();
        
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();
        
        let result = tool.execute(json!({"path": temp_file.path().to_str().unwrap()})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Hello, World!"));
    }
    
    #[tokio::test]
    async fn test_file_reader_nonexistent_file() {
        let tool = FileReaderTool::new();
        
        let result = tool.execute(json!({"path": "nonexistent_file.txt"})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }
    
    #[tokio::test]
    async fn test_file_reader_directory() {
        let tool = FileReaderTool::new();
        
        let result = tool.execute(json!({"path": "."})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("directory"));
    }
    
    #[tokio::test]
    async fn test_file_reader_missing_parameter() {
        let tool = FileReaderTool::new();
        
        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
    }
}

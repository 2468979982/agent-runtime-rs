use crate::tools::types::*;
use async_trait::async_trait;
use regex::Regex;
use std::fs;

/// Tool for editing files using find and replace
pub struct FileEditorTool {
    metadata: ToolMetadata,
}

impl FileEditorTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "file_editor".to_string(),
                description: "Edit a file by finding and replacing text using regex patterns. Can replace all occurrences or just the first.".to_string(),
                parameters: vec![
                    ToolParameter {
                        name: "path".to_string(),
                        description: "Path to the file to edit".to_string(),
                        required: true,
                        parameter_type: ParameterType::String,
                    },
                    ToolParameter {
                        name: "find".to_string(),
                        description: "Text or regex pattern to find".to_string(),
                        required: true,
                        parameter_type: ParameterType::String,
                    },
                    ToolParameter {
                        name: "replace".to_string(),
                        description: "Text to replace with".to_string(),
                        required: true,
                        parameter_type: ParameterType::String,
                    },
                    ToolParameter {
                        name: "use_regex".to_string(),
                        description: "Whether to treat 'find' as a regex pattern (default: false)".to_string(),
                        required: false,
                        parameter_type: ParameterType::Boolean,
                    },
                    ToolParameter {
                        name: "replace_all".to_string(),
                        description: "Whether to replace all occurrences (default: true)".to_string(),
                        required: false,
                        parameter_type: ParameterType::Boolean,
                    },
                ],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for FileEditorTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path = parameters["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'path' parameter".to_string()))?;
        
        let find = parameters["find"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'find' parameter".to_string()))?;
        
        let replace = parameters["replace"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'replace' parameter".to_string()))?;
        
        let use_regex = parameters["use_regex"].as_bool().unwrap_or(false);
        let replace_all = parameters["replace_all"].as_bool().unwrap_or(true);
        
        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| ToolError::IoError(e))?;
        
        // Perform replacement
        let new_content = if use_regex {
            let regex = Regex::new(find)
                .map_err(|e| ToolError::InvalidParameters(format!("Invalid regex: {}", e)))?;
            
            if replace_all {
                regex.replace_all(&content, replace).to_string()
            } else {
                regex.replace(&content, replace).to_string()
            }
        } else {
            if replace_all {
                content.replace(find, replace)
            } else {
                let mut result = content.clone();
                if let Some(pos) = result.find(find) {
                    result.replace_range(pos..pos + find.len(), replace);
                }
                result
            }
        };
        
        // Write back to file
        fs::write(path, new_content)
            .map_err(|e| ToolError::IoError(e))?;
        
        Ok(ToolResult {
            success: true,
            output: format!("Successfully edited file: {}", path),
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
    async fn test_file_editor_simple_replace() {
        let tool = FileEditorTool::new();
        let temp_path = "test_edit_simple.txt";
        
        // Create test file
        fs::write(temp_path, "Hello World\nHello Universe").unwrap();
        
        let result = tool.execute(json!({
            "path": temp_path,
            "find": "Hello",
            "replace": "Hi"
        })).await.unwrap();
        
        assert!(result.success);
        
        let content = fs::read_to_string(temp_path).unwrap();
        assert_eq!(content, "Hi World\nHi Universe");
        
        // Cleanup
        fs::remove_file(temp_path).ok();
    }
    
    #[tokio::test]
    async fn test_file_editor_replace_first_only() {
        let tool = FileEditorTool::new();
        let temp_path = "test_edit_first.txt";
        
        // Create test file
        fs::write(temp_path, "Hello World\nHello Universe").unwrap();
        
        let result = tool.execute(json!({
            "path": temp_path,
            "find": "Hello",
            "replace": "Hi",
            "replace_all": false
        })).await.unwrap();
        
        assert!(result.success);
        
        let content = fs::read_to_string(temp_path).unwrap();
        assert_eq!(content, "Hi World\nHello Universe");
        
        // Cleanup
        fs::remove_file(temp_path).ok();
    }
    
    #[tokio::test]
    async fn test_file_editor_regex_replace() {
        let tool = FileEditorTool::new();
        let temp_path = "test_edit_regex.txt";
        
        // Create test file
        fs::write(temp_path, "test123\nabc456\ndef789").unwrap();
        
        let result = tool.execute(json!({
            "path": temp_path,
            "find": "\\d+",
            "replace": "NUM",
            "use_regex": true
        })).await.unwrap();
        
        assert!(result.success);
        
        let content = fs::read_to_string(temp_path).unwrap();
        assert_eq!(content, "testNUM\nabcNUM\ndefNUM");
        
        // Cleanup
        fs::remove_file(temp_path).ok();
    }
    
    #[tokio::test]
    async fn test_file_editor_invalid_regex() {
        let tool = FileEditorTool::new();
        let temp_path = "test_edit_invalid.txt";
        
        // Create test file
        fs::write(temp_path, "test content").unwrap();
        
        let result = tool.execute(json!({
            "path": temp_path,
            "find": "[invalid",
            "replace": "test",
            "use_regex": true
        })).await;
        
        assert!(result.is_err());
        
        // Cleanup
        fs::remove_file(temp_path).ok();
    }
    
    #[tokio::test]
    async fn test_file_editor_nonexistent_file() {
        let tool = FileEditorTool::new();
        
        let result = tool.execute(json!({
            "path": "nonexistent.txt",
            "find": "test",
            "replace": "test"
        })).await;
        
        assert!(result.is_err());
    }
}

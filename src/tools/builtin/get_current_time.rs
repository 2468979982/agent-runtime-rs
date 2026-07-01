use crate::tools::types::*;
use async_trait::async_trait;
use chrono::Utc;

/// Datetime tool for getting current date and time
pub struct DatetimeTool {
    metadata: ToolMetadata,
}

impl DatetimeTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "get_current_time".to_string(),
                description: "Get the current date and time in UTC".to_string(),
                parameters: vec![],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for DatetimeTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
    
    async fn execute(&self, _parameters: serde_json::Value) -> Result<ToolResult, ToolError> {
        let now = Utc::now();
        
        Ok(ToolResult {
            success: true,
            output: now.to_rfc3339(),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_datetime_tool() {
        let tool = DatetimeTool::new();
        
        let result = tool.execute(json!({})).await.unwrap();
        assert!(result.success);
        assert!(!result.output.is_empty());
        // Check that it looks like an RFC3339 date
        assert!(result.output.contains("T"));
        assert!(result.output.contains("Z") || result.output.contains("+"));
    }
}

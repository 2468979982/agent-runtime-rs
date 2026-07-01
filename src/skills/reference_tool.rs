//! Skill reference tool implementation
//!
//! This module provides the tool executor for skill references.
//! Each skill is exposed as a tool that returns the skill content.

use async_trait::async_trait;
use serde_json::Value;
use tracing::info;

use crate::tools::types::{ToolExecutor, ToolResult, ToolError};
use crate::tools::types::ToolMetadata;
use crate::skills::types::SkillReferenceTool;

/// Implementation of ToolExecutor for SkillReferenceTool
#[async_trait]
impl ToolExecutor for SkillReferenceTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.tool_metadata
    }
    
    async fn execute(&self, _parameters: Value) -> Result<ToolResult, ToolError> {
        info!("Executing skill reference tool: {}", self.skill_id);
        
        // Skill reference tools don't typically use parameters
        // They just return the skill content
        let result = ToolResult {
            success: true,
            output: self.skill_content.clone(),
            error: None,
        };
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::SkillMetadata;
    
    #[test]
    fn test_skill_reference_tool_creation() {
        let skill = crate::skills::types::Skill {
            slug: "test-skill".to_string(),
            metadata: serde_json::json!({"name": "test"}),
            skill_metadata: SkillMetadata {
                name: "Test Skill".to_string(),
                description: "A test skill".to_string(),
                metadata: None,
            },
            content: "# Test Skill Content\n\nThis is test content.".to_string(),
            full_content: "---\nname: Test Skill\n---\n\n# Test Skill Content".to_string(),
        };
        
        let tool = SkillReferenceTool::new(&skill);
        
        assert_eq!(tool.skill_id, "test-skill");
        assert_eq!(tool.skill_name, "Test Skill");
        assert_eq!(tool.tool_metadata.description, "A test skill");
        assert!(tool.skill_content.contains("Test Skill Content"));
    }
    
    #[tokio::test]
    async fn test_skill_reference_tool_execute() {
        let skill = crate::skills::types::Skill {
            slug: "test-skill".to_string(),
            metadata: serde_json::json!({"name": "test"}),
            skill_metadata: SkillMetadata {
                name: "Test Skill".to_string(),
                description: "A test skill".to_string(),
                metadata: None,
            },
            content: "# Test Content".to_string(),
            full_content: "---\nname: Test Skill\n---\n\n# Test Content".to_string(),
        };
        
        let tool = SkillReferenceTool::new(&skill);
        let result = tool.execute(serde_json::json!({})).await;
        
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.success);
        assert_eq!(tool_result.output, "# Test Content");
        assert!(tool_result.error.is_none());
    }
    
    #[test]
    fn test_skill_reference_tool_metadata() {
        let skill = crate::skills::types::Skill {
            slug: "my-skill".to_string(),
            metadata: serde_json::json!({}),
            skill_metadata: SkillMetadata {
                name: "My Skill".to_string(),
                description: "My skill description".to_string(),
                metadata: None,
            },
            content: "content".to_string(),
            full_content: "full content".to_string(),
        };
        
        let tool = SkillReferenceTool::new(&skill);
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "skill/my-skill");
        assert_eq!(metadata.description, "My skill description");
        assert_eq!(metadata.parameters.len(), 0);
    }
}

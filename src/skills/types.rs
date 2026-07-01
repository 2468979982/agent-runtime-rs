//! Skill types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::tools::types::ToolMetadata;

/// Skill metadata extracted from SKILL.md YAML front matter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Skill name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Additional metadata (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_yaml::Value>>,
}

impl Default for SkillMetadata {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            description: "No description".to_string(),
            metadata: None,
        }
    }
}

/// Skill object representing a loaded skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Skill slug (folder name)
    pub slug: String,
    /// Metadata from _meta.json
    pub metadata: serde_json::Value,
    /// Skill metadata from SKILL.md YAML front matter
    pub skill_metadata: SkillMetadata,
    /// SKILL.md content (without YAML front matter)
    pub content: String,
    /// SKILL.md full content (with YAML front matter)
    pub full_content: String,
}

/// Skill reference tool for tool execution
#[derive(Debug, Clone)]
pub struct SkillReferenceTool {
    /// Tool metadata (stored as field to implement ToolExecutor)
    pub tool_metadata: ToolMetadata,
    /// Skill ID (slug)
    pub skill_id: String,
    /// Skill name
    pub skill_name: String,
    /// Skill content
    pub skill_content: String,
}

impl SkillReferenceTool {
    /// Create a new skill reference tool
    pub fn new(skill: &Skill) -> Self {
        let tool_metadata = ToolMetadata {
            name: format!("skill/{}", skill.slug),
            description: skill.skill_metadata.description.clone(),
            parameters: vec![], // No parameters required for skill reference
        };
        
        Self {
            tool_metadata,
            skill_id: skill.slug.clone(),
            skill_name: skill.skill_metadata.name.clone(),
            skill_content: skill.content.clone(),
        }
    }
}

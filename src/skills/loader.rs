//! Skill loader implementation
//!
//! Loads skills from the filesystem and parses Markdown files with YAML front matter.

use std::path::{PathBuf};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{info, debug, warn, error};

use crate::skills::types::*;

/// Errors that can occur during skill loading
#[derive(Error, Debug)]
pub enum SkillLoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Skill not found: {0}")]
    NotFound(String),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}

/// Result type for skill loader operations
pub type SkillLoaderResult<T> = Result<T, SkillLoaderError>;

/// SkillLoader - Loads document-based skills from the skills/ folder
///
/// Supports loading skills from Markdown files with YAML front matter.
/// Each skill is a directory containing _meta.json and SKILL.md files.
pub struct SkillLoader {
    /// Path to the skills directory
    skills_dir: PathBuf,
    /// Cache of loaded skills
    loaded_skills: HashMap<String, Skill>,
}

impl SkillLoader {
    /// Create a new SkillLoader
    pub fn new(skills_dir: PathBuf) -> Self {
        Self {
            skills_dir,
            loaded_skills: HashMap::new(),
        }
    }
    
    /// Load all skills from the skills directory
    pub fn load_skills(&mut self) -> SkillLoaderResult<Vec<Skill>> {
        info!("Loading skills from: {}", self.skills_dir.display());
        
        // Check if skills directory exists
        if !self.skills_dir.exists() {
            warn!("Skills directory does not exist: {}", self.skills_dir.display());
            return Ok(Vec::new());
        }
        
        // Read directory entries
        let entries = std::fs::read_dir(&self.skills_dir)?;
        let mut skills = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Only process directories
            if path.is_dir() {
                let folder_name = path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                
                match self.load_skill(&folder_name) {
                    Ok(Some(skill)) => {
                        skills.push(skill);
                    }
                    Ok(None) => {
                        // Skill missing required files, skip
                        debug!("Skill {} missing required files, skipping", folder_name);
                    }
                    Err(e) => {
                        error!("Failed to load skill {}: {}", folder_name, e);
                    }
                }
            }
        }
        
        info!("Successfully loaded {} skills", skills.len());
        Ok(skills)
    }
    
    /// Load a single skill by folder name
    fn load_skill(&mut self, folder_name: &str) -> SkillLoaderResult<Option<Skill>> {
        let skill_path = self.skills_dir.join(folder_name);
        
        // Check for _meta.json
        let meta_path = skill_path.join("_meta.json");
        if !meta_path.exists() {
            return Ok(None);
        }
        
        // Check for SKILL.md
        let skill_md_path = skill_path.join("SKILL.md");
        if !skill_md_path.exists() {
            return Ok(None);
        }
        
        // Read and parse _meta.json
        let meta_content = std::fs::read_to_string(&meta_path)?;
        let meta: serde_json::Value = serde_json::from_str(&meta_content)?;
        
        // Read and parse SKILL.md
        let skill_md_content = std::fs::read_to_string(&skill_md_path)?;
        let (skill_metadata, content) = Self::parse_skill_markdown(&skill_md_content)?;
        
        let skill = Skill {
            slug: folder_name.to_string(),
            metadata: meta,
            skill_metadata,
            content,
            full_content: skill_md_content,
        };
        
        // Cache the skill
        self.loaded_skills.insert(skill.slug.clone(), skill.clone());
        
        info!("Loaded skill: {}", folder_name);
        Ok(Some(skill))
    }
    
    /// Parse SKILL.md to extract YAML front matter and content
    fn parse_skill_markdown(content: &str) -> SkillLoaderResult<(SkillMetadata, String)> {
        // Look for YAML front matter between --- markers
        let front_matter_regex = regex::Regex::new(r"^---\s*\n([\s\S]*?)\n---")?;
        
        if let Some(captures) = front_matter_regex.captures(content) {
            let yaml_content = captures.get(1).unwrap().as_str();
            let remaining_content = &content[captures.get(0).unwrap().end()..].trim();
            
            // Parse YAML
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content)?;
            
            // Extract metadata fields
            let name = yaml_value.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            
            let description = yaml_value.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("No description")
                .to_string();
            
            let mut metadata_map = HashMap::new();
            if let serde_yaml::Value::Mapping(mapping) = &yaml_value {
                for (key, value) in mapping {
                    if let Some(key_str) = key.as_str() {
                        metadata_map.insert(key_str.to_string(), value.clone());
                    }
                }
            }
            
            let skill_metadata = SkillMetadata {
                name,
                description,
                metadata: Some(metadata_map),
            };
            
            Ok((skill_metadata, remaining_content.to_string()))
        } else {
            // No YAML front matter
            Ok((
                SkillMetadata::default(),
                content.to_string(),
            ))
        }
    }
    
    /// Get a skill by ID (slug)
    pub fn get_skill(&self, id: &str) -> Option<&Skill> {
        self.loaded_skills.get(id)
    }
    
    /// List all loaded skills
    pub fn list_skills(&self) -> Vec<&Skill> {
        self.loaded_skills.values().collect()
    }
    
    /// Search skills by keyword
    pub fn search_skills(&self, keyword: &str) -> Vec<&Skill> {
        let lower_keyword = keyword.to_lowercase();
        let mut results = Vec::new();
        
        for skill in self.loaded_skills.values() {
            let name = skill.skill_metadata.name.to_lowercase();
            let description = skill.skill_metadata.description.to_lowercase();
            let content = skill.content.to_lowercase();
            
            if name.contains(&lower_keyword) || 
               description.contains(&lower_keyword) || 
               content.contains(&lower_keyword) {
                results.push(skill);
            }
        }
        
        results
    }
    
    /// Reload all skills
    pub fn reload_skills(&mut self) -> SkillLoaderResult<Vec<Skill>> {
        self.loaded_skills.clear();
        self.load_skills()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    /// Helper function to create a test skill directory
    fn create_test_skill_dir(temp_dir: &TempDir, slug: &str, name: &str, description: &str, content: &str) -> PathBuf {
        let skill_dir = temp_dir.path().join(slug);
        fs::create_dir(&skill_dir).unwrap();
        
        // Create _meta.json
        let meta = serde_json::json!({"slug": slug});
        fs::write(skill_dir.join("_meta.json"), serde_json::to_string_pretty(&meta).unwrap()).unwrap();
        
        // Create SKILL.md with YAML front matter
        let skill_md = format!("---\nname: {}\ndescription: {}\n---\n\n{}", name, description, content);
        fs::write(skill_dir.join("SKILL.md"), skill_md).unwrap();
        
        skill_dir
    }
    
    #[test]
    fn test_skill_loader_creation() {
        let temp_dir = TempDir::new().unwrap();
        let loader = SkillLoader::new(temp_dir.path().to_path_buf());
        
        assert_eq!(loader.skills_dir, temp_dir.path());
        assert_eq!(loader.list_skills().len(), 0);
    }
    
    #[test]
    fn test_load_skills_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let mut loader = SkillLoader::new(temp_dir.path().to_path_buf());
        
        let skills = loader.load_skills().unwrap();
        assert_eq!(skills.len(), 0);
    }
    
    #[test]
    fn test_load_single_skill() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill_dir(&temp_dir, "test-skill", "Test Skill", "A test skill", "# Test Content");
        
        let mut loader = SkillLoader::new(temp_dir.path().to_path_buf());
        let skills = loader.load_skills().unwrap();
        
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].slug, "test-skill");
        assert_eq!(skills[0].skill_metadata.name, "Test Skill");
        assert_eq!(skills[0].content, "# Test Content");
    }
    
    #[test]
    fn test_load_multiple_skills() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill_dir(&temp_dir, "skill1", "Skill 1", "First skill", "# Skill 1");
        create_test_skill_dir(&temp_dir, "skill2", "Skill 2", "Second skill", "# Skill 2");
        create_test_skill_dir(&temp_dir, "skill3", "Skill 3", "Third skill", "# Skill 3");
        
        let mut loader = SkillLoader::new(temp_dir.path().to_path_buf());
        let skills = loader.load_skills().unwrap();
        
        assert_eq!(skills.len(), 3);
    }
    
    #[test]
    fn test_load_skill_missing_meta() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir(&skill_dir).unwrap();
        
        // Only create SKILL.md, no _meta.json
        let skill_md = "---\nname: Test\n---\n\nContent";
        fs::write(skill_dir.join("SKILL.md"), skill_md).unwrap();
        
        let mut loader = SkillLoader::new(temp_dir.path().to_path_buf());
        let skills = loader.load_skills().unwrap();
        
        // Should skip this skill
        assert_eq!(skills.len(), 0);
    }
    
    #[test]
    fn test_load_skill_missing_skill_md() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir(&skill_dir).unwrap();
        
        // Only create _meta.json, no SKILL.md
        let meta = serde_json::json!({"slug": "test-skill"});
        fs::write(skill_dir.join("_meta.json"), serde_json::to_string_pretty(&meta).unwrap()).unwrap();
        
        let mut loader = SkillLoader::new(temp_dir.path().to_path_buf());
        let skills = loader.load_skills().unwrap();
        
        // Should skip this skill
        assert_eq!(skills.len(), 0);
    }
    
    #[test]
    fn test_get_skill() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill_dir(&temp_dir, "test-skill", "Test Skill", "A test skill", "# Test");
        
        let mut loader = SkillLoader::new(temp_dir.path().to_path_buf());
        loader.load_skills().unwrap();
        
        let skill = loader.get_skill("test-skill");
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().slug, "test-skill");
        
        let missing_skill = loader.get_skill("non-existent");
        assert!(missing_skill.is_none());
    }
    
    #[test]
    fn test_list_skills() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill_dir(&temp_dir, "skill1", "Skill 1", "First", "# 1");
        create_test_skill_dir(&temp_dir, "skill2", "Skill 2", "Second", "# 2");
        
        let mut loader = SkillLoader::new(temp_dir.path().to_path_buf());
        loader.load_skills().unwrap();
        
        let skills = loader.list_skills();
        assert_eq!(skills.len(), 2);
    }
    
    #[test]
    fn test_search_skills() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill_dir(&temp_dir, "python-skill", "Python Helper", "Python programming help", "# Python\n\nUse Python for scripting");
        create_test_skill_dir(&temp_dir, "rust-skill", "Rust Guide", "Rust programming guide", "# Rust\n\nUse Rust for performance");
        create_test_skill_dir(&temp_dir, "general-skill", "General Assistant", "General purpose assistant", "# General\n\nHelper for various tasks");
        
        let mut loader = SkillLoader::new(temp_dir.path().to_path_buf());
        loader.load_skills().unwrap();
        
        // Search by name
        let results = loader.search_skills("Python");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "python-skill");
        
        // Search by description
        let results = loader.search_skills("programming");
        assert_eq!(results.len(), 2);
        
        // Search by content
        let results = loader.search_skills("performance");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "rust-skill");
        
        // Search with no results
        let results = loader.search_skills("JavaScript");
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_reload_skills() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill_dir(&temp_dir, "skill1", "Skill 1", "First", "# 1");
        
        let mut loader = SkillLoader::new(temp_dir.path().to_path_buf());
        loader.load_skills().unwrap();
        assert_eq!(loader.list_skills().len(), 1);
        
        // Add another skill
        create_test_skill_dir(&temp_dir, "skill2", "Skill 2", "Second", "# 2");
        
        // Reload
        loader.reload_skills().unwrap();
        assert_eq!(loader.list_skills().len(), 2);
    }
    
    #[test]
    fn test_parse_skill_markdown_with_frontmatter() {
        let content = "---\nname: Test Skill\ndescription: A test skill\n---\n\n# Content\n\nThis is the content.";
        
        let result = SkillLoader::parse_skill_markdown(content);
        assert!(result.is_ok());
        
        let (metadata, _remaining) = result.unwrap();
        assert_eq!(metadata.name, "Test Skill");
        assert_eq!(metadata.description, "A test skill");
    }
    
    #[test]
    fn test_parse_skill_markdown_without_frontmatter() {
        let content = "# Content\n\nThis is the content without front matter.";
        
        let result = SkillLoader::parse_skill_markdown(content);
        assert!(result.is_ok());
        
        let (metadata, remaining) = result.unwrap();
        assert_eq!(metadata.name, "Unknown");
        assert_eq!(metadata.description, "No description");
        assert_eq!(remaining, "# Content\n\nThis is the content without front matter.");
    }
    
    #[test]
    fn test_parse_skill_markdown_with_complex_yaml() {
        let content = "---\nname: Complex Skill\ndescription: A complex skill\nversion: 1.0\nauthor: Test Author\ntags: [rust, programming, tutorial]\n---\n\n# Complex Content";
        
        let result = SkillLoader::parse_skill_markdown(content);
        assert!(result.is_ok());
        
        let (metadata, _remaining) = result.unwrap();
        assert_eq!(metadata.name, "Complex Skill");
        assert_eq!(metadata.description, "A complex skill");
        assert!(metadata.metadata.is_some());
        
        let meta_map = metadata.metadata.unwrap();
        assert!(meta_map.contains_key("version"));
        assert!(meta_map.contains_key("author"));
        assert!(meta_map.contains_key("tags"));
    }
}

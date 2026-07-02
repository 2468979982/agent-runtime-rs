//! Skill types for the agent runtime
//! 
//! This module defines the core types for the skill system,
//! including skill metadata, references, scripts, and execution results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill metadata (frontmatter in Markdown skill files)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Skill name (unique identifier)
    pub name: String,
    
    /// Skill description
    pub description: String,
    
    /// Skill version
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Author information
    #[serde(default)]
    pub author: Option<String>,
    
    /// Trigger words/phrases that activate this skill
    #[serde(default)]
    pub triggers: Vec<String>,
    
    /// Required tools for this skill
    #[serde(default)]
    pub required_tools: Vec<String>,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Default for SkillMetadata {
    fn default() -> Self {
        Self {
            name: "unnamed-skill".to_string(),
            description: "No description".to_string(),
            version: default_version(),
            author: None,
            triggers: Vec::new(),
            required_tools: Vec::new(),
            tags: Vec::new(),
        }
    }
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Skill reference (lazy loading)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillReference {
    /// Reference name
    pub name: String,
    
    /// Reference description
    pub description: String,
    
    /// Reference content (lazily loaded)
    #[serde(skip)]
    pub content: Option<String>,
}

/// Skill script (executable code block)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillScript {
    /// Script name
    pub name: String,
    
    /// Script description
    pub description: String,
    
    /// Script language (e.g., "bash", "python", "javascript")
    pub language: String,
    
    /// Script code
    pub code: String,
    
    /// Whether this script should be executed during skill loading
    #[serde(default)]
    pub auto_execute: bool,
}

/// Skill definition (loaded from Markdown file)
#[derive(Debug, Clone)]
pub struct Skill {
    /// Skill metadata
    pub metadata: SkillMetadata,
    
    /// Skill content (Markdown body)
    pub content: String,
    
    /// References (lazy loaded)
    pub references: Vec<SkillReference>,
    
    /// Scripts (code blocks)
    pub scripts: Vec<SkillScript>,
    
    /// File path (for lazy loading)
    pub file_path: Option<String>,
}

impl Skill {
    /// Create a new skill
    pub fn new(metadata: SkillMetadata, content: String) -> Self {
        Self {
            metadata,
            content,
            references: Vec::new(),
            scripts: Vec::new(),
            file_path: None,
        }
    }
    
    /// Add a reference
    pub fn add_reference(&mut self, reference: SkillReference) {
        self.references.push(reference);
    }
    
    /// Add a script
    pub fn add_script(&mut self, script: SkillScript) {
        self.scripts.push(script);
    }
    
    /// Get skill as Markdown string
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        
        // Add frontmatter
        md.push_str("---\n");
        md.push_str(&serde_yaml::to_string(&self.metadata).unwrap_or_default());
        md.push_str("---\n\n");
        
        // Add content
        md.push_str(&self.content);
        
        // Add references
        if !self.references.is_empty() {
            md.push_str("\n\n## References\n\n");
            for reference in &self.references {
                md.push_str(&format!("### {}\n\n{}\n\n", reference.name, reference.description));
            }
        }
        
        // Add scripts
        if !self.scripts.is_empty() {
            md.push_str("\n\n## Scripts\n\n");
            for script in &self.scripts {
                md.push_str(&format!("### {} ({})\n\n", script.name, script.language));
                md.push_str(&format!("```{} \n{}\n```\n\n", script.language, script.code));
            }
        }
        
        md
    }
}

/// Skill execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    
    /// Execution output
    pub output: String,
    
    /// Execution error (if any)
    pub error: Option<String>,
    
    /// Execution duration (ms)
    pub duration_ms: u64,
}

/// Skill manager for loading and executing skills
#[derive(Debug, Clone)]
pub struct SkillManager {
    /// Loaded skills (name -> Skill)
    skills: HashMap<String, Skill>,
    
    /// Skill folder path
    skills_folder: String,
    
    /// Whether to auto-load skills
    auto_load: bool,
}

impl SkillManager {
    /// Create a new skill manager
    pub fn new(skills_folder: &str, auto_load: bool) -> Self {
        Self {
            skills: HashMap::new(),
            skills_folder: skills_folder.to_string(),
            auto_load,
        }
    }
    
    /// Load a skill from a Markdown file
    pub fn load_skill(&mut self, file_path: &str) -> Result<Skill, String> {
        // Read file content
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read skill file '{}': {}", file_path, e))?;
        
        // Parse Markdown skill
        let skill = Self::parse_markdown_skill(&content, file_path)?;
        
        // Store skill
        let name = skill.metadata.name.clone();
        self.skills.insert(name.clone(), skill);
        
        Ok(self.skills.get(&name).unwrap().clone())
    }
    
    /// Parse a Markdown skill file
    fn parse_markdown_skill(content: &str, file_path: &str) -> Result<Skill, String> {
        // Split frontmatter and body
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        
        if parts.len() < 3 {
            return Err("Invalid skill format: missing frontmatter".to_string());
        }
        
        let frontmatter = parts[1];
        let body = parts[2];
        
        // Parse frontmatter as YAML
        let metadata: SkillMetadata = serde_yaml::from_str(frontmatter)
            .map_err(|e| format!("Failed to parse skill frontmatter: {}", e))?;
        
        // Create skill
        let mut skill = Skill::new(metadata, body.to_string());
        skill.file_path = Some(file_path.to_string());
        
        // Parse code blocks (scripts)
        let code_blocks = Self::extract_code_blocks(body);
        for (language, code) in code_blocks {
            let script = SkillScript {
                name: format!("script_{}", skill.scripts.len() + 1),
                description: "Auto-extracted script".to_string(),
                language,
                code,
                auto_execute: false,
            };
            skill.add_script(script);
        }
        
        Ok(skill)
    }
    
    /// Extract code blocks from Markdown
    fn extract_code_blocks(markdown: &str) -> Vec<(String, String)> {
        let mut blocks = Vec::new();
        let mut in_code_block = false;
        let mut current_language = String::new();
        let mut current_code = String::new();
        
        for line in markdown.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block
                    blocks.push((current_language.clone(), current_code.clone()));
                    current_code.clear();
                    in_code_block = false;
                } else {
                    // Start of code block
                    current_language = line.trim_start_matches('`').to_string();
                    in_code_block = true;
                }
            } else if in_code_block {
                current_code.push_str(line);
                current_code.push('\n');
            }
        }
        
        blocks
    }
    
    /// Load all skills from the skills folder
    pub fn load_all_skills(&mut self) -> Result<Vec<Skill>, String> {
        let skills_folder = std::path::Path::new(&self.skills_folder);
        
        if !skills_folder.exists() {
            return Err(format!("Skills folder '{}' does not exist", self.skills_folder));
        }
        
        let mut loaded_skills = Vec::new();
        
        // Walk through all Markdown files
        for entry in walkdir::WalkDir::new(skills_folder)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.extension().map(|ext| ext == "md").unwrap_or(false) {
                let file_path = path.to_string_lossy().to_string();
                
                match self.load_skill(&file_path) {
                    Ok(skill) => {
                        loaded_skills.push(skill);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load skill from '{}': {}", file_path, e);
                    }
                }
            }
        }
        
        Ok(loaded_skills)
    }
    
    /// Get a skill by name
    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }
    
    /// Get all skill names
    pub fn get_skill_names(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }
    
    /// Execute a skill script
    pub async fn execute_skill(&self, skill_name: &str, script_name: &str, arguments: Option<HashMap<String, String>>) -> Result<SkillExecutionResult, String> {
        let skill = self.skills.get(skill_name)
            .ok_or_else(|| format!("Skill '{}' not found", skill_name))?;
        
        let script = skill.scripts.iter()
            .find(|s| s.name == script_name)
            .ok_or_else(|| format!("Script '{}' not found in skill '{}'", script_name, skill_name))?;
        
        // Execute script based on language
        let start_time = std::time::Instant::now();
        
        let output = match script.language.as_str() {
            "bash" | "sh" => {
                self.execute_bash_script(&script.code, &arguments).await?
            }
            "python" | "py" => {
                self.execute_python_script(&script.code, &arguments).await?
            }
            "javascript" | "js" => {
                self.execute_javascript_script(&script.code, &arguments).await?
            }
            _ => {
                return Err(format!("Unsupported script language: {}", script.language));
            }
        };
        
        let duration = start_time.elapsed();
        
        Ok(SkillExecutionResult {
            success: true,
            output,
            error: None,
            duration_ms: duration.as_millis() as u64,
        })
    }
    
    /// Execute a bash script
    async fn execute_bash_script(&self, code: &str, _arguments: &Option<HashMap<String, String>>) -> Result<String, String> {
        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(code)
            .output()
            .await
            .map_err(|e| format!("Failed to execute bash script: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
    
    /// Execute a Python script
    async fn execute_python_script(&self, code: &str, _arguments: &Option<HashMap<String, String>>) -> Result<String, String> {
        let output = tokio::process::Command::new("python3")
            .arg("-c")
            .arg(code)
            .output()
            .await
            .map_err(|e| format!("Failed to execute Python script: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
    
    /// Execute a JavaScript script
    async fn execute_javascript_script(&self, code: &str, _arguments: &Option<HashMap<String, String>>) -> Result<String, String> {
        let output = tokio::process::Command::new("node")
            .arg("-e")
            .arg(code)
            .output()
            .await
            .map_err(|e| format!("Failed to execute JavaScript script: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_markdown_skill() {
        let markdown = r#"---
name: test-skill
description: A test skill
version: 1.0.0
triggers:
  - test
  - example
tags:
  - test
  - example
---

# Test Skill

This is a test skill.

```bash
echo "Hello, World!"
```

```python
print("Hello from Python!")
```
"#;
        
        let skill = SkillManager::parse_markdown_skill(markdown, "test.md").unwrap();
        
        assert_eq!(skill.metadata.name, "test-skill");
        assert_eq!(skill.metadata.description, "A test skill");
        assert_eq!(skill.metadata.version, "1.0.0");
        assert_eq!(skill.metadata.triggers.len(), 2);
        assert_eq!(skill.scripts.len(), 2);
        assert_eq!(skill.scripts[0].language, "bash");
        assert_eq!(skill.scripts[1].language, "python");
    }
    
    #[test]
    fn test_extract_code_blocks() {
        let markdown = r#"
# Test

```bash
echo "Hello"
```

```python
print("World")
```
"#;
        
        let blocks = SkillManager::extract_code_blocks(markdown);
        
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].0, "bash");
        assert_eq!(blocks[0].1.trim(), "echo \"Hello\"");
        assert_eq!(blocks[1].0, "python");
        assert_eq!(blocks[1].1.trim(), "print(\"World\")");
    }
}

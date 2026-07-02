//! Skill system for the agent runtime
//! 
//! This module implements the skill system for loading, managing,
//! and executing skills (reusable agent capabilities defined in Markdown).
//! 
//! # Architecture
//! 
//! - `types`: Core type definitions and SkillManager implementation
//! 
//! # Example
//! 
//! ```rust,no_run
//! use agent_runtime_rs::skill::types::SkillManager;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create skill manager
//!     let mut manager = SkillManager::new("./skills", true);
//!     
//!     // Load all skills
//!     let skills = manager.load_all_skills()?;
//!     println!("Loaded {} skills", skills.len());
//!     
//!     Ok(())
//! }
//! ```

pub mod types;

pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skill_metadata_default() {
        let metadata = SkillMetadata::default();
        assert_eq!(metadata.name, "unnamed-skill");
        assert_eq!(metadata.version, "1.0.0");
        assert!(metadata.triggers.is_empty());
    }
}

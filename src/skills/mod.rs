//! Skills module for agent runtime
//!
//! This module provides skill loading and management capabilities.
//! Skills are Markdown-based documents that can be loaded and referenced as tools.

pub mod types;
pub mod loader;
pub mod reference_tool;

// Re-export main types
pub use types::{Skill, SkillMetadata, SkillReferenceTool};
pub use loader::SkillLoader;

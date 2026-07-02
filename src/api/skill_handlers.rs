//! Skill management API handlers
//! 
//! This module implements read-only API endpoints for skill management:
//! - GET /api/skills - List all skills
//! - GET /api/skills/:name - Get skill details
//! 
//! Note: Skill execution is not provided via API due to mutable state requirements.
//! Users should read skill files and execute scripts manually.

use axum::{
    extract::{State, Path, Json},
    http::StatusCode,
};
use std::sync::Arc;
use std::collections::HashMap;

use crate::api::types::*;
use crate::runtime::agent::AgentRuntime;

/// Handler for GET /api/skills
///
/// Lists all available skills.
pub async fn list_skills_handler(
    State(runtime): State<Arc<AgentRuntime>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    tracing::info!("Received list skills request");
    
    // Check if runtime is initialized
    if !runtime.is_initialized() {
        return Err(ApiError::InternalServerError(
            "AgentRuntime not initialized".to_string()
        ));
    }
    
    // Get skill manager
    let skill_manager = runtime.get_skill_manager()
        .ok_or_else(|| ApiError::InternalServerError(
            "Skill manager not available".to_string()
        ))?;
    
    // Get all skill names
    let skill_names = skill_manager.get_skill_names();
    
    // Build response with skill metadata
    let mut skills = Vec::new();
    for name in skill_names {
        if let Some(skill) = skill_manager.get_skill(&name) {
            skills.push(serde_json::json!({
                "name": skill.metadata.name,
                "description": skill.metadata.description,
                "version": skill.metadata.version,
                "author": skill.metadata.author,
                "triggers": skill.metadata.triggers,
                "tags": skill.metadata.tags,
                "script_count": skill.scripts.len(),
            }));
        }
    }
    
    let response = serde_json::json!({
        "skills": skills,
        "count": skills.len(),
    });
    
    Ok(Json(response))
}

/// Handler for GET /api/skills/:skill_name
///
/// Gets details of a specific skill.
pub async fn get_skill_handler(
    State(runtime): State<Arc<AgentRuntime>>,
    Path(skill_name): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    tracing::info!("Received get skill request for: {}", skill_name);
    
    // Validate skill_name
    if skill_name.is_empty() {
        return Err(ApiError::BadRequest("Skill name cannot be empty".to_string()));
    }
    
    // Check if runtime is initialized
    if !runtime.is_initialized() {
        return Err(ApiError::InternalServerError(
            "AgentRuntime not initialized".to_string()
        ));
    }
    
    // Get skill manager
    let skill_manager = runtime.get_skill_manager()
        .ok_or_else(|| ApiError::InternalServerError(
            "Skill manager not available".to_string()
        ))?;
    
    // Get skill
    let skill = skill_manager.get_skill(&skill_name)
        .ok_or_else(|| ApiError::NotFound(format!("Skill '{}' not found", skill_name)))?;
    
    // Build response
    let scripts: Vec<serde_json::Value> = skill.scripts.iter()
        .map(|s| serde_json::json!({
            "name": s.name,
            "description": s.description,
            "language": s.language,
            "auto_execute": s.auto_execute,
        }))
        .collect();
    
    let response = serde_json::json!({
        "name": skill.metadata.name,
        "description": skill.metadata.description,
        "version": skill.metadata.version,
        "author": skill.metadata.author,
        "triggers": skill.metadata.triggers,
        "required_tools": skill.metadata.required_tools,
        "tags": skill.metadata.tags,
        "scripts": scripts,
        "content": skill.content,
    });
    
    Ok(Json(response))
}

/// Handler for POST /api/skills/:skill_name/execute
///
/// Note: This endpoint is disabled because skill execution requires mutable access
/// to the SkillManager, which conflicts with the shared Arc<AgentRuntime> pattern.
/// 
/// Instead, users should:
/// 1. Read the skill details via GET /api/skills/:name
/// 2. Read the skill Markdown file
/// 3. Execute scripts manually using the appropriate tools
pub async fn execute_skill_handler(
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Err(ApiError::InternalServerError(
        "Skill execution via API is not supported. \
         Please read the skill Markdown file and execute scripts manually.".to_string()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skill_handlers_compile() {
        // This test ensures the module compiles correctly
        assert!(true);
    }
}

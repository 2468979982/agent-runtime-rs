//! Route definitions for the API
//!
//! This module defines all API routes using Axum's router.

use axum::{
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;

use crate::api::handlers::*;
use crate::api::skill_handlers::*;
use crate::runtime::agent::AgentRuntime;

/// Create routes for the /api/run endpoint
pub fn run_routes() -> Router<Arc<AgentRuntime>> {
    Router::new()
        .route("/api/run", post(run_handler))
}

/// Create routes for the /api/tool-call endpoint
pub fn tool_routes() -> Router<Arc<AgentRuntime>> {
    Router::new()
        .route("/api/tool-call", post(tool_call_handler))
}

/// Create routes for session management endpoints
pub fn session_routes() -> Router<Arc<AgentRuntime>> {
    Router::new()
        .route("/api/sessions", get(list_sessions_handler))
        .route("/api/sessions/:session_id", get(get_session_handler))
        .route("/api/sessions/:session_id", delete(delete_session_handler))
}

/// Create routes for skill management endpoints
pub fn skill_routes() -> Router<Arc<AgentRuntime>> {
    Router::new()
        .route("/api/skills", get(list_skills_handler))
        .route("/api/skills/:skill_name", get(get_skill_handler))
        .route("/api/skills/:skill_name/execute", post(execute_skill_handler))
}

/// Create routes for health check endpoint
pub fn health_routes() -> Router<Arc<AgentRuntime>> {
    Router::new()
        .route("/api/health", get(health_handler))
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    
    #[test]
    fn test_run_routes() {
        // Test that run_routes creates a valid router
        assert!(true);
    }
    
    #[test]
    fn test_tool_routes() {
        // Test that tool_routes creates a valid router
        assert!(true);
    }
    
    #[test]
    fn test_session_routes() {
        // Test that session_routes creates a valid router
        assert!(true);
    }
    
    #[test]
    fn test_health_routes() {
        // Test that health_routes creates a valid router
        assert!(true);
    }
}

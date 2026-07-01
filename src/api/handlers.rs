//! Request handlers for the API
//!
//! This module implements all the request handlers for the API endpoints.

use axum::{
    extract::{State, Path, Json},
    http::StatusCode,
};
use std::sync::Arc;

use crate::api::types::*;
use crate::runtime::agent::AgentRuntime;

/// Handler for POST /api/run
///
/// Runs the agent with the provided message and returns the response.
pub async fn run_handler(
    State(_runtime): State<Arc<AgentRuntime>>,
    Json(request): Json<RunRequest>,
) -> Result<Json<RunResponse>, ApiError> {
    tracing::info!("Received run request: session_id={:?}, message={}", 
        request.session_id, request.message);
    
    // Validate request
    if request.message.is_empty() {
        return Err(ApiError::BadRequest("Message cannot be empty".to_string()));
    }
    
    // Check if runtime is initialized
    if !_runtime.is_initialized() {
        return Err(ApiError::InternalServerError(
            "AgentRuntime not initialized".to_string()
        ));
    }
    
    // TODO: Implement actual agent execution
    // For now, return a placeholder response
    let response = RunResponse {
        response: format!("Agent response to: {}", request.message),
        tool_calls: vec![],
        session_id: request.session_id.unwrap_or_else(|| "default".to_string()),
    };
    
    Ok(Json(response))
}

/// Handler for POST /api/tool-call
///
/// Executes a tool call with the provided parameters.
pub async fn tool_call_handler(
    State(_runtime): State<Arc<AgentRuntime>>,
    Json(request): Json<ToolCallRequest>,
) -> Result<Json<ToolCallResponse>, ApiError> {
    tracing::info!("Received tool call request: tool_name={}", request.tool_name);
    
    // Validate request
    if request.tool_name.is_empty() {
        return Err(ApiError::BadRequest("Tool name cannot be empty".to_string()));
    }
    
    // TODO: Implement actual tool execution
    // For now, return a placeholder response
    let response = ToolCallResponse {
        result: serde_json::json!({"status": "success"}),
        success: true,
        error: None,
    };
    
    Ok(Json(response))
}

/// Handler for GET /api/sessions
///
/// Returns a list of all active sessions.
pub async fn list_sessions_handler(
    State(_runtime): State<Arc<AgentRuntime>>,
) -> Result<Json<SessionListResponse>, ApiError> {
    tracing::info!("Received list sessions request");
    
    // TODO: Implement actual session listing
    // For now, return a placeholder response
    let response = SessionListResponse {
        sessions: vec!["session-1".to_string(), "session-2".to_string()],
        total: 2,
    };
    
    Ok(Json(response))
}

/// Handler for GET /api/sessions/:session_id
///
/// Returns the details and history of a specific session.
pub async fn get_session_handler(
    State(_runtime): State<Arc<AgentRuntime>>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponse>, ApiError> {
    tracing::info!("Received get session request for: {}", session_id);
    
    // Validate session_id
    if session_id.is_empty() {
        return Err(ApiError::BadRequest("Session ID cannot be empty".to_string()));
    }
    
    // TODO: Implement actual session retrieval
    // For now, return a placeholder response
    let response = SessionResponse {
        session_id,
        history: vec![],
        metadata: serde_json::json!({}),
    };
    
    Ok(Json(response))
}

/// Handler for DELETE /api/sessions/:session_id
///
/// Deletes a specific session.
pub async fn delete_session_handler(
    State(_runtime): State<Arc<AgentRuntime>>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    tracing::info!("Received delete session request for: {}", session_id);
    
    // Validate session_id
    if session_id.is_empty() {
        return Err(ApiError::BadRequest("Session ID cannot be empty".to_string()));
    }
    
    // TODO: Implement actual session deletion
    // For now, just return NO_CONTENT
    Ok(StatusCode::NO_CONTENT)
}

/// Handler for GET /api/health
///
/// Returns the health status of the API server.
pub async fn health_handler() -> Json<serde_json::Value> {
    tracing::info!("Received health check request");
    
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_handler() {
        let response = health_handler().await;
        let body = response.0;
        assert!(body.get("status").is_some());
        assert_eq!(body.get("status").unwrap(), "healthy");
    }
}

//! API types for request and response payloads
//!
//! This module defines the types used for API requests and responses,
//! including JSON serialization/deserialization.

use serde::{Deserialize, Serialize};
use crate::runtime::types::RuntimeError;

/// Request payload for running the agent
#[derive(Debug, Deserialize, Serialize)]
pub struct RunRequest {
    /// Session ID for the conversation
    pub session_id: Option<String>,
    
    /// Message to send to the agent
    pub message: String,
}

/// Response payload for running the agent
#[derive(Debug, Serialize)]
pub struct RunResponse {
    /// Response message from the agent
    pub response: String,
    
    /// Tool calls made during execution
    pub tool_calls: Vec<serde_json::Value>,
    
    /// Session ID for the conversation
    pub session_id: String,
    
    /// Name of the skill used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_used: Option<String>,
}

/// Request payload for tool calls
#[derive(Debug, Deserialize)]
pub struct ToolCallRequest {
    /// Name of the tool to call
    pub tool_name: String,
    
    /// Parameters for the tool
    pub parameters: serde_json::Value,
}

/// Response payload for tool calls
#[derive(Debug, Serialize)]
pub struct ToolCallResponse {
    /// Result of the tool call
    pub result: serde_json::Value,
    
    /// Whether the call was successful
    pub success: bool,
    
    /// Error message if the call failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Request payload for session management
#[derive(Debug, Deserialize)]
pub struct SessionRequest {
    /// Session ID
    pub session_id: String,
}

/// Response payload for session listing
#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    /// List of session IDs
    pub sessions: Vec<String>,
    
    /// Total number of sessions
    pub total: usize,
}

/// Response payload for session details
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    /// Session ID
    pub session_id: String,
    
    /// Session history
    pub history: Vec<serde_json::Value>,
    
    /// Session metadata
    pub metadata: serde_json::Value,
}

/// Error response payload
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    
    /// Error code
    pub code: u16,
    
    /// Detailed error information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// API error type for handling HTTP errors
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Bad request (400)
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    /// Unauthorized (401)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    /// Not found (404)
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Internal server error (500)
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    
    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
}

impl ApiError {
    /// Convert ApiError to HTTP status code
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            ApiError::BadRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => axum::http::StatusCode::UNAUTHORIZED,
            ApiError::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
            ApiError::InternalServerError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Runtime(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let body = axum::Json(ErrorResponse {
            error: self.to_string(),
            code: status.as_u16(),
            details: None,
        });
        
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_request_deserialization() {
        let request = RunRequest {
            session_id: Some("test-session".to_string()),
            message: "Hello".to_string(),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("session_id"));
        assert!(json.contains("message"));
    }

    #[test]
    fn test_run_response_serialization() {
        let response = RunResponse {
            response: "Hello there!".to_string(),
            tool_calls: vec![],
            session_id: "test-session".to_string(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("response"));
        assert!(json.contains("tool_calls"));
    }

    #[test]
    fn test_api_error_status_codes() {
        let bad_request = ApiError::BadRequest("test".to_string());
        assert_eq!(bad_request.status_code(), axum::http::StatusCode::BAD_REQUEST);
        
        let not_found = ApiError::NotFound("test".to_string());
        assert_eq!(not_found.status_code(), axum::http::StatusCode::NOT_FOUND);
        
        let internal = ApiError::InternalServerError("test".to_string());
        assert_eq!(internal.status_code(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }
}

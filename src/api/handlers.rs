//! Request handlers for the API
//!
//! This module implements all the request handlers for the API endpoints.

use axum::{
    extract::{State, Path, Json},
    http::StatusCode,
};
use std::sync::Arc;
use serde_json;
use uuid::Uuid;

use crate::api::types::*;
use crate::runtime::agent::AgentRuntime;
use crate::session::types::{Message, MessageRole};

/// Handler for POST /api/run
///
/// Runs the agent with the provided message and returns the response.
pub async fn run_handler(
    State(runtime): State<Arc<AgentRuntime>>,
    Json(request): Json<RunRequest>,
) -> Result<Json<RunResponse>, ApiError> {
    tracing::info!("Received run request: session_id={:?}, message={}", 
        request.session_id, request.message);
    
    // Validate request
    if request.message.is_empty() {
        return Err(ApiError::BadRequest("Message cannot be empty".to_string()));
    }
    
    // Check if runtime is initialized
    if !runtime.is_initialized() {
        return Err(ApiError::InternalServerError(
            "AgentRuntime not initialized".to_string()
        ));
    }
    
    // Get session ID (use provided or generate new one)
    let session_id = request.session_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Get components from runtime
    let llm_connector = runtime.get_llm_connector()
        .ok_or_else(|| ApiError::InternalServerError(
            "LLM connector not available".to_string()
        ))?;
    
    let tool_manager = runtime.get_tool_manager();
    let session_manager = runtime.get_session_manager();
    
    // Get or create session
    // Try to get existing session, or create new one with the specified ID
    if session_manager.get_session(&session_id).is_err() {
        // Session not found, create new one with the specified ID
        session_manager.create_session_with_id(session_id.clone())
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create session: {}", e)))?;
    }
    
    // Add user message to session
    let user_message = Message {
        role: MessageRole::User,
        content: request.message.clone(),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    };
    session_manager.add_message(&session_id, user_message)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to add message: {}", e)))?;
    
    // Get conversation history
    let history = session_manager.get_history(&session_id)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get history: {}", e)))?;
    
    // Convert history to LLM format (ChatMessage)
    let messages: Vec<crate::llm::types::ChatMessage> = history.iter()
        .map(|msg| {
            let role = match msg.role {
                MessageRole::System => crate::llm::types::MessageRole::System,
                MessageRole::User => crate::llm::types::MessageRole::User,
                MessageRole::Assistant => crate::llm::types::MessageRole::Assistant,
                MessageRole::Tool => crate::llm::types::MessageRole::Tool,
            };
            crate::llm::types::ChatMessage {
                role,
                content: msg.content.clone(),
                name: msg.name.clone(),
                tool_calls: None, // TODO: Convert session::types::ToolCall to llm::types::ToolCall
                tool_call_id: msg.tool_call_id.clone(),
            }
        })
        .collect();
    
    // Get available tools (for function calling)
    let tool_names = tool_manager.get_tool_names();
    let tools: Vec<crate::tools::types::ToolMetadata> = tool_names.iter()
        .filter_map(|name| {
            tool_manager.get_tool_metadata(name).cloned()
        })
        .collect();
    
    // Call LLM
    tracing::info!("Calling LLM with {} messages and {} tools", messages.len(), tools.len());
    let llm_response = llm_connector.chat_completion(messages, Some(tools)).await
        .map_err(|e| {
            tracing::error!("LLM call failed: {}", e);
            ApiError::InternalServerError(format!("LLM call failed: {}", e))
        })?;
    
    // Process LLM response
    let mut tool_calls_results = vec![];
    let response_text = if let Some(first_choice) = llm_response.choices.first() {
        if let Some(tool_calls_list) = &first_choice.message.tool_calls {
            // LLM wants to call tools
            tracing::info!("LLM requested {} tool calls", tool_calls_list.len());
            
            for tool_call in tool_calls_list {
                let tool_name = tool_call.function.name.clone();
                let parameters = serde_json::from_str::<serde_json::Value>(&tool_call.function.arguments)
                    .unwrap_or(serde_json::json!({}));
                
                tracing::info!("Executing tool: {} with params: {}", tool_name, parameters);
                
                // Execute tool
                match tool_manager.execute_tool_call(&tool_name, parameters).await {
                    Ok(result) => {
                        tool_calls_results.push(serde_json::json!({
                            "tool_call_id": tool_call.id,
                            "tool_name": tool_name,
                            "result": result.output
                        }));
                    }
                    Err(e) => {
                        tracing::error!("Tool execution failed: {}", e);
                        tool_calls_results.push(serde_json::json!({
                            "tool_call_id": tool_call.id,
                            "tool_name": tool_name,
                            "error": e.to_string()
                        }));
                    }
                }
            }
            
            // TODO: Send tool results back to LLM for final response
            // For now, return tool call results
            format!("Tool calls executed: {}", serde_json::to_string(&tool_calls_results).unwrap_or_default())
        } else {
            // Direct text response
            first_choice.message.content.clone()
        }
    } else {
        "No response from LLM".to_string()
    };
    
    // Add assistant response to session
    let assistant_message = Message {
        role: MessageRole::Assistant,
        content: response_text.clone(),
        name: None,
        tool_calls: None, // TODO: Convert tool_calls_results to ToolCall objects
        tool_call_id: None,
    };
    session_manager.add_message(&session_id, assistant_message)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to add assistant message: {}", e)))?;
    
    // Return response
    let response = RunResponse {
        response: response_text,
        tool_calls: tool_calls_results,
        session_id,
    };
    
    Ok(Json(response))
}

/// Handler for POST /api/tool-call
///
/// Executes a tool call with the provided parameters.
pub async fn tool_call_handler(
    State(runtime): State<Arc<AgentRuntime>>,
    Json(request): Json<ToolCallRequest>,
) -> Result<Json<ToolCallResponse>, ApiError> {
    tracing::info!("Received tool call request: tool_name={}", request.tool_name);
    
    // Validate request
    if request.tool_name.is_empty() {
        return Err(ApiError::BadRequest("Tool name cannot be empty".to_string()));
    }
    
    // Get tool manager
    let tool_manager = runtime.get_tool_manager();
    
    // Execute tool
    match tool_manager.execute_tool_call(&request.tool_name, request.parameters).await {
        Ok(result) => {
            let response = ToolCallResponse {
                result: serde_json::json!({"output": result.output}),
                success: true,
                error: None,
            };
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Tool execution failed: {}", e);
            let response = ToolCallResponse {
                result: serde_json::json!({"error": e.to_string()}),
                success: false,
                error: Some(e.to_string()),
            };
            Ok(Json(response))
        }
    }
}

/// Handler for GET /api/sessions
///
/// Returns a list of all active sessions.
pub async fn list_sessions_handler(
    State(runtime): State<Arc<AgentRuntime>>,
) -> Result<Json<SessionListResponse>, ApiError> {
    tracing::info!("Received list sessions request");
    
    // Get session manager
    let session_manager = runtime.get_session_manager();
    
    // Get all session IDs
    let sessions = session_manager.list_sessions();
    let total = sessions.len();
    
    let response = SessionListResponse {
        sessions,
        total,
    };
    
    Ok(Json(response))
}

/// Handler for GET /api/sessions/:session_id
///
/// Returns the details and history of a specific session.
pub async fn get_session_handler(
    State(runtime): State<Arc<AgentRuntime>>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponse>, ApiError> {
    tracing::info!("Received get session request for: {}", session_id);
    
    // Validate session_id
    if session_id.is_empty() {
        return Err(ApiError::BadRequest("Session ID cannot be empty".to_string()));
    }
    
    // Get session manager
    let session_manager = runtime.get_session_manager();
    
    // Get session
    let session = session_manager.get_session(&session_id)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get session: {}", e)))?;
    
    // Convert history to JSON format
    let history: Vec<serde_json::Value> = session.messages.iter()
        .map(|msg| {
            let role_str = match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
            };
            serde_json::json!({
                "role": role_str,
                "content": msg.content,
                "name": msg.name,
                "tool_calls": msg.tool_calls,
                "tool_call_id": msg.tool_call_id
            })
        })
        .collect();
    
    let response = SessionResponse {
        session_id,
        history,
        metadata: serde_json::json!({}),
    };
    
    Ok(Json(response))
}

/// Handler for DELETE /api/sessions/:session_id
///
/// Deletes a specific session.
pub async fn delete_session_handler(
    State(runtime): State<Arc<AgentRuntime>>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    tracing::info!("Received delete session request for: {}", session_id);
    
    // Validate session_id
    if session_id.is_empty() {
        return Err(ApiError::BadRequest("Session ID cannot be empty".to_string()));
    }
    
    // Get session manager
    let session_manager = runtime.get_session_manager();
    
    // Delete session
    match session_manager.delete_session(&session_id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to delete session: {}", e);
            Err(ApiError::InternalServerError(format!("Failed to delete session: {}", e)))
        }
    }
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

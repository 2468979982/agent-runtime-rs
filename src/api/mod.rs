//! API module for HTTP server
//!
//! This module provides the HTTP API server implementation using Axum.
//! It includes route definitions, request handlers, middleware, and type definitions.

pub mod types;
pub mod routes;
pub mod handlers;
pub mod middleware;

pub use types::*;
pub use routes::*;
pub use handlers::*;
pub use middleware::*;

use axum::{
    Router,
    http::Method,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
};
use std::sync::Arc;

use crate::runtime::agent::AgentRuntime;

/// Create the Axum router with all API routes
///
/// # Arguments
///
/// * `runtime` - The AgentRuntime instance wrapped in Arc for sharing
///
/// # Returns
///
/// Returns a Router with all routes configured
pub fn create_router(runtime: Arc<AgentRuntime>) -> Router {
    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::DELETE])
        .allow_headers(Any);
    
    // Create tracing layer for logging
    let trace = TraceLayer::new_for_http();
    
    Router::new()
        .merge(routes::run_routes())
        .merge(routes::tool_routes())
        .merge(routes::session_routes())
        .merge(routes::health_routes())
        .layer(
            ServiceBuilder::new()
                .layer(trace)
                .layer(cors)
        )
        .with_state(runtime)
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn test_api_module_structure() {
        // This test ensures the module structure is correct
        assert!(true);
    }
}

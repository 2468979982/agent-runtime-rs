// Agent Runtime RS - Main Entry Point
// This binary uses the agent_runtime_rs library (defined in src/lib.rs)

use agent_runtime_rs::{
    utils::logger,
    api::create_router,
    create_agent_runtime,
};
use std::sync::Arc;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();
    
    // Initialize logger (set to debug for troubleshooting)
    logger::init_logger("debug")?;
    
    tracing::info!("Agent Runtime RS starting...");
    
    // Load configuration paths
    let agent_config_path = "config/agent-config.json";
    let tools_config_path = "config/tools-config.json";
    let prompt_config_path = "config/prompt-config.json";
    
    // Create AgentRuntime
    tracing::info!("Initializing AgentRuntime...");
    let runtime = match create_agent_runtime(
        agent_config_path,
        tools_config_path,
        prompt_config_path,
    ).await {
        Ok(rt) => {
            tracing::info!("AgentRuntime initialized successfully");
            rt  // Already Arc<AgentRuntime>, no need to wrap again
        }
        Err(e) => {
            tracing::error!("Failed to initialize AgentRuntime: {}", e);
            return Err(anyhow::anyhow!("Failed to initialize runtime: {}", e));
        }
    };
    
    // Create Axum router with API routes
    let app = create_router(runtime);
    
    // Start HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Starting HTTP server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on http://{}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

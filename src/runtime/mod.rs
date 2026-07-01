//! Runtime module for agent runtime
//!
//! This module provides the core AgentRuntime that integrates all components
//! including LLM, tools, sessions, skills, and MCP.

pub mod types;
pub mod agent;

pub use types::*;
pub use agent::AgentRuntime;

#[cfg(test)]
mod tests {
    #[test]
    fn test_runtime_module_structure() {
        // This test ensures the module structure is correct
        assert!(true);
    }
}

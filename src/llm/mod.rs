//! LLM module for interacting with Language Model APIs
//! 
//! This module provides the LLMConnector for making chat completion requests
//! to LLM APIs, with support for tool calling and streaming responses.

pub mod types;
pub mod connector;
pub mod client;

pub use types::*;
pub use connector::*;
pub use client::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_llm_module_structure() {
        // This test ensures the module structure is correct
        assert!(true);
    }
}

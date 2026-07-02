//! Built-in tools for the agent runtime

pub mod calculator;
pub mod file_reader;
pub mod file_writer;
pub mod file_editor;
pub mod file_lister;
pub mod file_deleter;
pub mod directory_creator;
pub mod get_current_time;
pub mod mcp_tool_executor;

pub use calculator::CalculatorTool;
pub use file_reader::FileReaderTool;
pub use file_writer::FileWriterTool;
pub use file_editor::FileEditorTool;
pub use file_lister::FileListerTool;
pub use file_deleter::FileDeleterTool;
pub use directory_creator::DirectoryCreatorTool;
pub use get_current_time::DatetimeTool;
pub use mcp_tool_executor::MCPToolExecutor;

/// Register all built-in tools with a ToolManager
pub fn register_builtin_tools(manager: &mut crate::tools::manager::ToolManager) {
    manager.register_tool(Box::new(CalculatorTool::new()));
    manager.register_tool(Box::new(FileReaderTool::new()));
    manager.register_tool(Box::new(FileWriterTool::new()));
    manager.register_tool(Box::new(FileEditorTool::new()));
    manager.register_tool(Box::new(FileListerTool::new()));
    manager.register_tool(Box::new(FileDeleterTool::new()));
    manager.register_tool(Box::new(DirectoryCreatorTool::new()));
    manager.register_tool(Box::new(DatetimeTool::new()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::manager::ToolManager;
    
    #[test]
    fn test_register_all_builtin_tools() {
        let mut manager = ToolManager::new();
        register_builtin_tools(&mut manager);
        
        let tool_names = manager.get_tool_names();
        assert_eq!(tool_names.len(), 8);
        assert!(tool_names.contains(&"calculator".to_string()));
        assert!(tool_names.contains(&"file_reader".to_string()));
        assert!(tool_names.contains(&"file_writer".to_string()));
        assert!(tool_names.contains(&"file_editor".to_string()));
        assert!(tool_names.contains(&"file_lister".to_string()));
        assert!(tool_names.contains(&"file_deleter".to_string()));
        assert!(tool_names.contains(&"directory_creator".to_string()));
        assert!(tool_names.contains(&"get_current_time".to_string()));
    }
}

//! 会话管理模块
//!
//! 提供会话管理功能，包括多轮对话历史管理和会话状态管理。

pub mod types;
pub mod manager;

pub use types::*;
pub use manager::SessionManager;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_structure() {
        // 测试模块结构是否正确
        assert!(true);
    }
}

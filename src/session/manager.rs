use crate::session::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use uuid::Uuid;

/// 会话管理器
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    config: SessionConfig,
    store: Option<Arc<dyn SessionStore>>,
}

unsafe impl Send for SessionManager {}
unsafe impl Sync for SessionManager {}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            config,
            store: None,
        }
    }

    /// 创建新的会话管理器（带持久化存储）
    pub fn with_store(config: SessionConfig, store: Arc<dyn SessionStore>) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            config,
            store: Some(store),
        }
    }

    /// 创建新会话
    pub fn create_session(&self) -> Result<Session> {
        let session_id = Uuid::new_v4().to_string();
        self.create_session_with_id(session_id)
    }
    
    /// 使用指定的 ID 创建新会话
    pub fn create_session_with_id(&self, session_id: String) -> Result<Session> {
        let session = Session::new(session_id.clone());

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), session.clone());

        info!("Session created: {}", session_id);

        // 如果配置了存储，异步保存到磁盘
        if let Some(store) = &self.store {
            let store_clone = Arc::clone(store);
            let session_clone = session.clone();
            tokio::spawn(async move {
                if let Err(e) = store_clone.save(&session_clone).await {
                    eprintln!("Failed to save session: {}", e);
                }
            });
        }

        Ok(session)
    }

    /// 获取会话
    pub fn get_session(&self, session_id: &str) -> Result<Session> {
        self.check_session_exists(session_id)?;
        self.check_session_ttl(session_id)?;

        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id).cloned();

        session.ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))
    }

    /// 添加消息到会话
    pub fn add_message(&self, session_id: &str, message: Message) -> Result<()> {
        self.check_session_exists(session_id)?;
        self.check_session_ttl(session_id)?;

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.add_message(message);

            // 限制历史消息数量
            if session.messages.len() > self.config.max_history_length {
                let excess = session.messages.len() - self.config.max_history_length;
                session.messages.drain(0..excess);
                debug!(
                    "Trimmed {} messages from session {}",
                    excess, session_id
                );
            }

            debug!(
                "Message added to session {} (total: {})",
                session_id,
                session.messages.len()
            );

            // 克隆会话用于异步保存
            let session_clone = session.clone();
            drop(sessions); // 释放锁

            // 如果配置了存储，异步保存到磁盘
            if let Some(store) = &self.store {
                let store_clone = Arc::clone(store);
                tokio::spawn(async move {
                    if let Err(e) = store_clone.save(&session_clone).await {
                        eprintln!("Failed to save session: {}", e);
                    }
                });
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }

    /// 获取会话历史
    pub fn get_history(&self, session_id: &str) -> Result<Vec<Message>> {
        self.check_session_exists(session_id)?;
        self.check_session_ttl(session_id)?;

        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            Ok(session.messages.clone())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }

    /// 清空会话消息
    pub fn clear_session(&self, session_id: &str) -> Result<()> {
        self.check_session_exists(session_id)?;
        self.check_session_ttl(session_id)?;

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.clear_messages();

            info!("Session cleared: {}", session_id);

            // 克隆会话用于异步保存
            let session_clone = session.clone();
            drop(sessions); // 释放锁

            // 如果配置了存储，异步保存到磁盘
            if let Some(store) = &self.store {
                let store_clone = Arc::clone(store);
                tokio::spawn(async move {
                    if let Err(e) = store_clone.save(&session_clone).await {
                        eprintln!("Failed to save session: {}", e);
                    }
                });
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }

    /// 删除会话
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        self.check_session_exists(session_id)?;

        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(session_id);

        info!("Session deleted: {}", session_id);

        // 如果配置了存储，异步删除
        if let Some(store) = &self.store {
            let store_clone = Arc::clone(store);
            let session_id = session_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = store_clone.delete(&session_id).await {
                    eprintln!("Failed to delete session: {}", e);
                }
            });
        }

        Ok(())
    }

    /// 列出所有会话 ID
    pub fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.lock().unwrap();
        sessions.keys().cloned().collect()
    }

    /// 获取会话数量
    pub fn session_count(&self) -> usize {
        let sessions = self.sessions.lock().unwrap();
        sessions.len()
    }

    /// 清理过期会话
    pub fn cleanup_expired_sessions(&self) -> usize {
        let ttl = match self.config.session_ttl {
            Some(ttl) => ttl,
            None => return 0,
        };

        let mut sessions = self.sessions.lock().unwrap();
        let mut expired_ids = Vec::new();

        for (session_id, session) in sessions.iter() {
            if session.is_expired(ttl) {
                expired_ids.push(session_id.clone());
            }
        }

        let expired_count = expired_ids.len();

        // 删除过期会话
        for session_id in &expired_ids {
            sessions.remove(session_id);

            // 如果配置了存储，异步删除
            if let Some(store) = &self.store {
                let store_clone = Arc::clone(store);
                let session_id = session_id.clone();
                tokio::spawn(async move {
                    if let Err(e) = store_clone.delete(&session_id).await {
                        eprintln!("Failed to delete expired session: {}", e);
                    }
                });
            }
        }

        if expired_count > 0 {
            info!(
                "Cleaned up {} expired sessions",
                expired_count
            );
        }

        expired_count
    }

    /// 检查会话是否存在
    fn check_session_exists(&self, session_id: &str) -> Result<()> {
        let sessions = self.sessions.lock().unwrap();
        if !sessions.contains_key(session_id) {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        } else {
            Ok(())
        }
    }

    /// 检查会话是否过期
    fn check_session_ttl(&self, session_id: &str) -> Result<()> {
        let ttl = match self.config.session_ttl {
            Some(ttl) => ttl,
            None => return Ok(()),
        };

        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            if session.is_expired(ttl) {
                // 会话已过期，删除它
                drop(sessions); // 释放锁
                let mut sessions = self.sessions.lock().unwrap();
                sessions.remove(session_id);

                info!(
                    "Session expired and removed: {}",
                    session_id
                );

                return Err(anyhow::anyhow!("Session expired: {}", session_id));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::types::MessageRole;

    fn create_test_config() -> SessionConfig {
        SessionConfig {
            max_history_length: 100,
            session_ttl: None,
        }
    }

    #[test]
    fn test_create_session() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let session = manager.create_session().unwrap();
        assert!(!session.id.is_empty());
        assert_eq!(session.messages.len(), 0);
    }

    #[test]
    fn test_get_session() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let session = manager.create_session().unwrap();
        let retrieved = manager.get_session(&session.id).unwrap();

        assert_eq!(retrieved.id, session.id);
    }

    #[test]
    fn test_add_message() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let session = manager.create_session().unwrap();
        let message = Message::new(MessageRole::User, "Hello".to_string());

        manager.add_message(&session.id, message).unwrap();

        let history = manager.get_history(&session.id).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "Hello");
    }

    #[test]
    fn test_get_history() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let session = manager.create_session().unwrap();

        let msg1 = Message::new(MessageRole::User, "Message 1".to_string());
        let msg2 = Message::new(MessageRole::Assistant, "Message 2".to_string());

        manager.add_message(&session.id, msg1).unwrap();
        manager.add_message(&session.id, msg2).unwrap();

        let history = manager.get_history(&session.id).unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_clear_session() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let session = manager.create_session().unwrap();
        let message = Message::new(MessageRole::User, "Hello".to_string());

        manager.add_message(&session.id, message).unwrap();
        manager.clear_session(&session.id).unwrap();

        let history = manager.get_history(&session.id).unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_delete_session() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let session = manager.create_session().unwrap();
        let session_id = session.id.clone();

        manager.delete_session(&session_id).unwrap();

        assert!(manager.get_session(&session_id).is_err());
    }

    #[test]
    fn test_list_sessions() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let _session1 = manager.create_session().unwrap();
        let _session2 = manager.create_session().unwrap();

        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_session_count() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let _session1 = manager.create_session().unwrap();
        let _session2 = manager.create_session().unwrap();

        assert_eq!(manager.session_count(), 2);
    }

    #[test]
    fn test_session_not_found() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let result = manager.get_session("non-existent");
        assert!(result.is_err());
    }

    #[test]
    fn test_message_with_tool_calls() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let session = manager.create_session().unwrap();

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            call_type: "function".to_string(),
            function: ToolCallFunction {
                name: "test_tool".to_string(),
                arguments: "{}".to_string(),
            },
        };

        let message = Message::new(MessageRole::Assistant, "Thinking".to_string())
            .with_tool_calls(vec![tool_call]);

        manager.add_message(&session.id, message).unwrap();

        let history = manager.get_history(&session.id).unwrap();
        assert!(history[0].tool_calls.is_some());
    }

    #[test]
    fn test_message_with_tool_call_id() {
        let config = create_test_config();
        let manager = SessionManager::new(config);

        let session = manager.create_session().unwrap();

        let message = Message::new(MessageRole::Tool, "Result".to_string())
            .with_tool_call_id("call_123".to_string());

        manager.add_message(&session.id, message).unwrap();

        let history = manager.get_history(&session.id).unwrap();
        assert_eq!(
            history[0].tool_call_id,
            Some("call_123".to_string())
        );
    }

    #[tokio::test]
    async fn test_with_store() {
        let config = create_test_config();
        let store = Arc::new(InMemorySessionStore::new());
        let manager = SessionManager::with_store(config, store);

        let session = manager.create_session().unwrap();
        assert!(!session.id.is_empty());
    }

    #[test]
    fn test_history_limit() {
        let config = SessionConfig {
            max_history_length: 5,
            session_ttl: None,
        };
        let manager = SessionManager::new(config);

        let session = manager.create_session().unwrap();

        // 添加 10 条消息
        for i in 0..10 {
            let message = Message::new(MessageRole::User, format!("Message {}", i));
            manager.add_message(&session.id, message).unwrap();
        }

        let history = manager.get_history(&session.id).unwrap();
        assert_eq!(history.len(), 5); // 应该只有最后 5 条
    }
}

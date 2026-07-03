use crate::session::types::*;
use crate::session::persistence::SessionPersistenceManager;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// 会话管理器
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    config: SessionConfig,
    /// 持久化管理器（运行时可设置，用 Mutex 包装以支持 &self 修改）
    persistence_manager: Mutex<Option<Arc<SessionPersistenceManager>>>,
}

unsafe impl Send for SessionManager {}
unsafe impl Sync for SessionManager {}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            config,
            persistence_manager: Mutex::new(None),
        }
    }
    
    /// 创建新的会话管理器（带持久化管理器）
    pub fn with_persistence(config: SessionConfig, pm: Arc<SessionPersistenceManager>) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            config,
            persistence_manager: Mutex::new(Some(pm)),
        }
    }
    
    /// 设置持久化管理器（初始化后调用，支持 &self）
    pub fn set_persistence_manager(&self, pm: Arc<SessionPersistenceManager>) {
        let mut guard = self.persistence_manager.lock().unwrap();
        *guard = Some(pm);
    }
    
    /// 加载持久化会话（初始化时调用）
    pub async fn load_persisted_sessions(&self) -> Result<()> {
        let pm = {
            let guard = self.persistence_manager.lock().unwrap();
            match &*guard {
                Some(pm) => pm.clone(),
                None => return Ok(()),
            }
        };
        
        if !pm.is_enabled() {
            return Ok(());
        }
        
        info!("Loading persisted sessions...");
        
        let session_ids = pm.list_sessions().await?;
        info!("Found {} persisted sessions", session_ids.len());
        
        for session_id in session_ids {
            match pm.load_session(&session_id).await {
                Ok(Some(session)) => {
                    let mut sessions = self.sessions.lock().unwrap();
                    sessions.insert(session.id.clone(), session);
                    info!("Loaded session: {}", session_id);
                }
                Ok(None) => {
                    warn!("Session not found on disk: {}", session_id);
                }
                Err(e) => {
                    warn!("Failed to load session '{}': {}", session_id, e);
                }
            }
        }
        
        info!("Persisted sessions loaded successfully");
        Ok(())
    }
    
    /// 获取持久化管理器引用（内部使用）
    fn get_persistence_manager(&self) -> Option<Arc<SessionPersistenceManager>> {
        let guard = self.persistence_manager.lock().unwrap();
        guard.clone()
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
        
        // 异步保存到磁盘
        self.async_save_session(&session);
        
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
    
    /// 添加会话（用于加载持久化会话）
    pub fn add_session(&self, session: Session) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session.id.clone(), session);
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
            
            // 异步保存到磁盘
            self.async_save_session(&session_clone);
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }
    
    /// 异步保存会话到磁盘（内部方法）
    fn async_save_session(&self, session: &Session) {
        let pm = match self.get_persistence_manager() {
            Some(pm) => pm,
            None => return,
        };
        let session_clone = session.clone();
        tokio::spawn(async move {
            if let Err(e) = pm.save_session(&session_clone).await {
                eprintln!("Failed to save session: {}", e);
            }
        });
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
            
            // 异步保存到磁盘
            self.async_save_session(&session_clone);
            
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
        
        // 异步删除磁盘文件
        let pm = match self.get_persistence_manager() {
            Some(pm) => pm,
            None => return Ok(()),
        };
        let session_id = session_id.to_string();
        tokio::spawn(async move {
            if let Err(e) = pm.delete_session(&session_id).await {
                eprintln!("Failed to delete session: {}", e);
            }
        });
        
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
            
            // 异步删除磁盘文件
            let pm = match self.get_persistence_manager() {
                Some(pm) => pm,
                None => continue,
            };
            let session_id = session_id.clone();
            tokio::spawn(async move {
                if let Err(e) = pm.delete_session(&session_id).await {
                    eprintln!("Failed to delete expired session: {}", e);
                }
            });
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
        let manager = SessionManager::new(config);
        
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

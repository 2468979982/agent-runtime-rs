use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 聊天消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String, // 通常是 "function"
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String, // JSON 字符串
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }

    pub fn with_tool_call_id(mut self, tool_call_id: String) -> Self {
        self.tool_call_id = Some(tool_call_id);
        self
    }
}

/// 会话配置
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub max_history_length: usize,
    pub session_ttl: Option<i64>, // 秒
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_history_length: 100,
            session_ttl: None,
        }
    }
}

unsafe impl Send for SessionConfig {}
unsafe impl Sync for SessionConfig {}

/// 会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.updated_at = Utc::now();
    }

    pub fn is_expired(&self, ttl_seconds: i64) -> bool {
        let now = Utc::now();
        let age = (now - self.updated_at).num_seconds();
        age > ttl_seconds
    }
}

/// 会话存储 trait
#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    async fn save(&self, session: &Session) -> anyhow::Result<()>;
    async fn load(&self, session_id: &str) -> anyhow::Result<Option<Session>>;
    async fn delete(&self, session_id: &str) -> anyhow::Result<()>;
    async fn list_ids(&self) -> anyhow::Result<Vec<String>>;
}

/// 内存会话存储
#[derive(Debug, Default)]
pub struct InMemorySessionStore {
    sessions: std::sync::Arc<std::sync::Mutex<HashMap<String, Session>>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl SessionStore for InMemorySessionStore {
    async fn save(&self, session: &Session) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn load(&self, session_id: &str) -> anyhow::Result<Option<Session>> {
        let sessions = self.sessions.lock().unwrap();
        Ok(sessions.get(session_id).cloned())
    }

    async fn delete(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(session_id);
        Ok(())
    }

    async fn list_ids(&self) -> anyhow::Result<Vec<String>> {
        let sessions = self.sessions.lock().unwrap();
        Ok(sessions.keys().cloned().collect())
    }
}

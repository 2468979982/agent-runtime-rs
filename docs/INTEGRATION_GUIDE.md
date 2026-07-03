# Agent Runtime RS - 集成指南

## 概述

本文档说明如何将 Agent Runtime RS 集成到其他 Rust 项目。

Agent Runtime RS 提供：
- LLM 集成（OpenAI 兼容 API）
- 工具管理和执行
- 会话管理
- 技能系统
- MCP（Model Context Protocol）集成

---

## 方案 A：作为本地依赖（推荐，开发阶段）

### 1. 修改目标项目的 `Cargo.toml`

```toml
[dependencies]
agent-runtime-rs = { path = "../path/to/agent-runtime-rs" }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

### 2. 在代码中使用

```rust
use agent_runtime_rs::create_agent_runtime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建 AgentRuntime 实例
    let runtime = create_agent_runtime(
        "config/agent-config.json",
        "config/tools-config.json",
        "config/prompt-config.json",
    ).await?;
    
    // 使用 runtime...
    
    Ok(())
}
```

---

## 方案 B：发布到 crates.io（生产环境）

### 1. 发布准备

```bash
# 登录 crates.io
cargo login

# 检查包内容
cargo package --list

# 打包并发布
cargo publish
```

### 2. 在其他项目中使用

```toml
[dependencies]
agent-runtime-rs = "0.1.0"
```

---

## 完整集成示例

### 示例 1：简单的 Agent 调用

```rust
use agent_runtime_rs::{create_agent_runtime, AgentRuntime};
use agent_runtime_rs::llm::types::{ChatMessage, MessageRole};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建 runtime
    let runtime = create_agent_runtime(
        "config/agent-config.json",
        "config/tools-config.json",
        "config/prompt-config.json",
    ).await?;
    
    // 调用聊天接口
    let runtime_lock = runtime.lock().await;
    
    let messages = vec![
        ChatMessage {
            role: MessageRole::User,
            content: "Hello!".to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    ];
    
    let response = runtime_lock
        .get_llm_connector()
        .unwrap()
        .chat_completion(messages, None)
        .await?;
    
    println!("Response: {:?}", response);
    
    Ok(())
}
```

---

### 示例 2：集成到 Web 服务器（Axum）

```rust
use agent_runtime_rs::create_agent_runtime;
use agent_runtime_rs::AgentRuntime;
use axum::{Router, extract::State, routing::post};
use std::sync::Arc;
use tokio::sync::Mutex;

// 应用状态
struct AppState {
    runtime: Arc<Mutex<AgentRuntime>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建 AgentRuntime
    let runtime = create_agent_runtime(
        "config/agent-config.json",
        "config/tools-config.json",
        "config/prompt-config.json",
    ).await?;
    
    // 创建 Axum 应用
    let state = Arc::new(AppState { runtime });
    
    let app = Router::new()
        .route("/chat", post(chat_handler))
        .with_state(state);
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

// 聊天处理器
async fn chat_handler(
    State(state): State<Arc<AppState>>,
    body: String,
) -> String {
    let runtime = state.runtime.lock().await;
    
    let messages = vec![
        ChatMessage {
            role: MessageRole::User,
            content: body,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    ];
    
    match runtime.get_llm_connector().unwrap().chat_completion(messages, None).await {
        Ok(response) => response.choices[0].message.content.clone(),
        Err(e) => format!("Error: {}", e),
    }
}
```

---

## 关键 API 说明

### 1. `create_agent_runtime()`

创建并初始化 AgentRuntime 实例。

**签名**：
```rust
pub async fn create_agent_runtime(
    agent_config_path: &str,
    tools_config_path: &str,
    prompt_config_path: &str,
) -> anyhow::Result<Arc<Mutex<AgentRuntime>>>
```

**参数**：
- `agent_config_path`: Agent 配置文件路径（包含 LLM 配置、技能配置等）
- `tools_config_path`: 工具配置文件路径（包含 MCP 服务器配置）
- `prompt_config_path`: Prompt 配置文件路径

**返回**：`Arc<Mutex<AgentRuntime>>`（线程安全的共享引用）

---

### 2. `AgentRuntime` 主要方法

```rust
impl AgentRuntime {
    /// 获取 LLM 连接器
    pub fn get_llm_connector(&self) -> Option<&LLMConnector>;
    
    /// 获取工具管理器
    pub fn get_tool_manager(&self) -> &ToolManager;
    
    /// 获取会话管理器
    pub fn get_session_manager(&self) -> &SessionManager;
    
    /// 获取技能管理器
    pub fn get_skill_manager(&self) -> Option<&SkillManager>;
    
    /// 聊天完成接口
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatCompletionResponse, RuntimeError>;
    
    /// 运行工具
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        parameters: serde_json::Value,
    ) -> Result<ToolResult, RuntimeError>;
}
```

---

## ⚠️ 注意事项

### 1. 配置文件路径
- 确保配置文件路径相对于**当前工作目录**
- 建议使用绝对路径，或显式设置工作目录

```rust
// 建议使用绝对路径
let config_path = std::path::PathBuf::from(
    std::env::var("CARGO_MANIFEST_DIR").unwrap()
).join("config/agent-config.json");
```

### 2. 异步运行时
- Agent Runtime 使用 Tokio 异步运行时
- 调用方项目也需要使用 Tokio

```toml
# 在调用方项目的 Cargo.toml 中
[dependencies]
tokio = { version = "1.0", features = ["full"] }
```

### 3. 线程安全
- `AgentRuntime` 使用 `Arc<Mutex<...>>` 包装
- 在多线程环境中安全共享

```rust
// 正确：使用 Arc<Mutex<>>
let runtime = create_agent_runtime(...).await?;
let runtime_clone = runtime.clone(); // Arc 引用计数 +1

tokio::spawn(async move {
    let lock = runtime_clone.lock().await;
    // 使用 lock...
});
```

### 4. 环境变量
- 确保环境变量已设置（`OPENAI_API_KEY`, `OPENAI_BASE_URL`）
- 或在配置文件中使用 `${ENV:...}` 语法

```json
{
  "llm": {
    "apiKey": "${ENV:OPENAI_API_KEY}",
    "baseURL": "${ENV:OPENAI_BASE_URL}"
  }
}
```

---

## 🧪 测试集成

### 1. 编译库

```bash
cd C:\Users\24689\.qclaw\workspace\tdd-developer\agent-runtime-rs
cargo build --lib
```

### 2. 运行测试

```bash
cargo test --lib
```

### 3. 在另一个项目中测试

```bash
# 创建测试项目
cargo new test-agent-integration
cd test-agent-integration

# 添加依赖（修改 Cargo.toml）
# [dependencies]
# agent-runtime-rs = { path = "../tdd-developer/agent-runtime-rs" }

# 编写测试代码（src/main.rs）
# 运行测试
cargo run
```

---

## 📁 项目结构示例

```
my-ai-project/
├── Cargo.toml
├── src/
│   └── main.rs
├── config/
│   ├── agent-config.json
│   ├── tools-config.json
│   └── prompt-config.json
├── agent-config/
│   ├── SOUL.md
│   ├── IDENTITY.md
│   └── ...
└── skills/
    ├── find-skills/
    ├── frontend-design/
    └── ...
```

---

## 🚀 高级用法

### 1. 自定义 Logger

```rust
use agent_runtime_rs::{create_agent_runtime_with_logger, utils::logger::Logger};

struct MyLogger;

impl Logger for MyLogger {
    fn log(&self, level: LogLevel, message: &str) {
        // 自定义日志逻辑
        println!("[{}] {}", level, message);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let logger = Box::new(MyLogger);
    
    let runtime = create_agent_runtime_with_logger(
        logger,
        "config/agent-config.json",
        "config/tools-config.json",
        "config/prompt-config.json",
    ).await?;
    
    // 使用 runtime...
    
    Ok(())
}
```

### 2. 动态加载技能

```rust
let runtime = create_agent_runtime(...).await?;
let runtime_lock = runtime.lock().await;

// 获取技能管理器
if let Some(skill_manager) = runtime_lock.get_skill_manager() {
    // 动态加载新技能
    let new_skill = skill_manager.load_skill_from_file("path/to/new-skill.md")?;
    println!("Loaded skill: {}", new_skill.metadata.name);
}
```

---

## 📚 更多信息

- **GitHub 仓库**: https://github.com/yourusername/agent-runtime-rs
- **文档**: https://docs.rs/agent-runtime-rs
- **示例项目**: https://github.com/yourusername/agent-runtime-rs/tree/main/examples

---

*创建时间: 2026-07-03 07:20:00*
*创建者: PM Assistant (project-manager)*
*文档版本: 1.0*

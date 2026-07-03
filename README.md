# Agent Runtime RS

[![Rust](https://img.shields.io/badge/Rust-1.75+-dea584?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.7-00bcd4)](https://github.com/tokio-rs/axum)
[![Tests](https://img.shields.io/badge/Tests-153%20passing-brightgreen)](.)

**Agent Runtime RS** 是一个用 Rust 编写的高性能 Agent 运行时，提供 LLM 集成、工具管理、技能系统和 MCP 集成。

这是 [agent-runtime-integration-example](https://github.com/your-repo/agent-runtime-integration-example) (TypeScript) 的 Rust 移植版本，提供更好的性能、类型安全和内存安全。

---

## ✨ 特性

- 🚀 **高性能**: Rust + Axum 提供高并发和低延迟
- 🔒 **类型安全**: 利用 Rust 的类型系统在编译时捕获错误
- 🛡️ **内存安全**: 没有段错误、缓冲区溢出或悬空指针
- 🧩 **模块化设计**: 清晰的模块分离（LLM、工具、会话、技能、MCP）
- 🔌 **MCP 集成**: 支持 Model Context Protocol (MCP) 通过 stdio 通信
- 📚 **技能系统**: 从 Markdown 文件加载技能（支持 YAML frontmatter）
- 🤖 **Agent 配置**: 通过 `agent-config/` 目录配置人格、身份、工作区（自动加载 + 按需加载） 🆕
- 💾 **会话持久化**: 自动保存/恢复会话到磁盘（JSONL 格式，异步保存，避免双重写入） 🆕
- 🌐 **HTTP API**: RESTful API 使用 Axum (类似 Express)
- 📊 **全面测试**: 153 个测试全部通过（11 个集成测试）

---

## 📂 项目结构

```
agent-runtime-rs/
├── src/
│   ├── config/          # 配置加载 (JSON + 环境变量)
│   │   ├── loader.rs     # ConfigLoader (支持 ${ENV:VAR} 替换)
│   │   ├── types.rs     # 配置类型定义
│   │   └── mod.rs
│   ├── llm/             # LLM 集成
│   │   ├── connector.rs  # LLMConnector (通义千问 qwen-plus)
│   │   ├── types.rs     # LLM 类型定义 (ChatMessage, ToolDefinition)
│   │   └── mod.rs
│   ├── tools/           # 工具管理
│   │   ├── manager.rs    # ToolManager
│   │   ├── types.rs      # 工具类型定义 (ToolExecutor, ToolResult)
│   │   ├── builtin/      # 内置工具
│   │   │   ├── calculator.rs
│   │   │   ├── file_reader.rs
│   │   │   ├── file_writer.rs
│   │   │   ├── file_editor.rs
│   │   │   ├── file_lister.rs
│   │   │   ├── file_deleter.rs
│   │   │   ├── directory_creator.rs
│   │   │   ├── get_current_time.rs
│   │   │   ├── mcp_tool_executor.rs  # MCP 工具适配器
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── session/         # 会话管理
│   │   ├── manager.rs    # SessionManager (UUID, TTL, 历史限制)
│   │   ├── types.rs      # 会话类型定义 (Message, MessageRole)
│   │   ├── store.rs      # 可选持久化存储
│   │   └── mod.rs
│   ├── mcp/             # MCP 集成
│   │   ├── client.rs      # MCPClient trait
│   │   ├── stdio_client.rs # MCPStdioClient (JSON-RPC 2.0)
│   │   ├── config.rs     # MCP 配置加载
│   │   ├── types.rs      # MCP 类型定义 (InitializeResult, Tool)
│   │   └── mod.rs
│   ├── skill/           # 技能系统
│   │   ├── types.rs      # Skill, SkillMetadata, SkillManager
│   │   └── mod.rs
│   ├── runtime/         # 核心运行时
│   │   ├── agent.rs      # AgentRuntime (协调所有组件)
│   │   └── mod.rs
│   ├── api/             # HTTP API (Axum)
│   │   ├── handlers.rs   # 请求处理器 (run, tool-call, sessions)
│   │   ├── skill_handlers.rs  # 技能管理 API
│   │   ├── middleware.rs # 中间件 (CORS, logging)
│   │   ├── routes.rs     # 路由定义
│   │   ├── types.rs      # API 请求/响应类型
│   │   └── mod.rs
│   ├── error.rs         # 全局错误类型
│   ├── lib.rs           # 库入口
│   └── main.rs          # 可执行文件入口
├── config/              # 配置文件
│   ├── agent-config.json   # LLM 配置 (支持 ${ENV:...})
│   ├── tools-config.json   # 工具 + MCP 服务器配置
│   └── prompt-config.json  # 提示词配置
├── agent-config/        # 🆕 Agent 配置文件 (启动时自动加载)
│   ├── SOUL.md          # 人格定义（活泼、严肃、幽默等）
│   ├── IDENTITY.md      # 身份定义（名称、Emoji、Vibe）
│   ├── AGENTS.md        # 工作区定义和启动指令
│   ├── MEMORY.md        # 长期记忆（用户信息、项目上下文）
│   ├── USER.md          # 用户信息（偏好、工作风格）
│   ├── TOOLS.md         # 工具使用说明和本地配置
│   └── HEARTBEAT.md     # 心跳检查任务列表
├── skills/              # 技能文件夹 (Markdown + YAML frontmatter)
│   ├── find-skills.md
│   ├── frontend-design.md
│   └── frontend-design/
│       └── SKILL.md
├── examples/            # 🆕 示例和模板
│   └── agent-config/    # Agent 配置模板（供用户参考）
│       ├── SOUL.md      # 人格定义模板（含注释）
│       ├── IDENTITY.md  # 身份定义模板（含注释）
│       ├── AGENTS.md    # 工作区模板
│       ├── MEMORY.md    # 长期记忆模板
│       ├── USER.md      # 用户信息模板
│       ├── TOOLS.md     # 工具配置模板
│       ├── HEARTBEAT.md # 心跳任务模板
│       └── README.md    # 使用指南
├── .env.example        # 环境变量示例
├── Cargo.toml          # Rust 项目配置
├── Cargo.lock          # 依赖锁定文件
└── README.md           # 本文档
```

---

## 📦 作为库使用（集成到其他 Rust 项目）

Agent Runtime RS 不仅可以作为独立服务器运行，还可以作为 **Rust 库（crate）** 集成到您的 Rust 项目中。

### 安装（作为依赖）

在您的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
agent-runtime-rs = { path = "../agent-runtime-rs" }  # 本地路径
# 或指定 git 仓库
# agent-runtime-rs = { git = "https://github.com/your-repo/agent-runtime-rs.git", branch = "main" }
```

### 基本用法

```rust
use agent_runtime_rs::{
    create_agent_runtime,
    create_agent_runtime_with_logger,
    AgentRuntime,          // 核心运行时
    ConfigLoader,         // 配置加载器
    LLMConnector,         // LLM 连接器
    ToolManager,          // 工具管理器
    SessionManager,        // 会话管理器
    RuntimeError,          // 错误类型
    Logger, ConsoleLogger, // 日志接口
    // 常用类型
    ChatMessage, MessageRole,
    RunRequest, RunResponse,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 方式 1：使用默认 logger 创建运行时
    let runtime = create_agent_runtime(
        "config/agent-config.json",
        "config/tools-config.json",
        "config/prompt-config.json",
    ).await?;
    
    // 方式 2：使用自定义 logger
    let custom_logger = Box::new(MyCustomLogger {});
    let runtime = create_agent_runtime_with_logger(
        custom_logger,
        "config/agent-config.json",
        "config/tools-config.json",
        "config/prompt-config.json",
    ).await?;
    
    // 使用运行时（Arc<AgentRuntime>）
    let response = runtime.chat(
        Some("session-123"),
        "Hello, agent!".to_string(),
    ).await?;
    
    println!("Response: {}", response.content);
    
    Ok(())
}
```

### 常用集成场景

#### 场景 1：在现有 Tokio 应用中使用

```rust
use agent_runtime_rs::create_agent_runtime;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建 Agent 运行时（异步初始化）
    let agent_runtime = create_agent_runtime(
        "config/agent-config.json",
        "config/tools-config.json",
        "config/prompt-config.json",
    ).await?;
    
    // 共享给多个服务/路由（Arc 线程安全）
    let runtime_clone = Arc::clone(&agent_runtime);
    tokio::spawn(async move {
        // 在后台任务中使用
        let _ = runtime_clone.chat(None, "Background task".to_string()).await;
    });
    
    // 主线程继续使用
    let response = agent_runtime.chat(
        Some("my-session"),
        "What can you do?".to_string(),
    ).await?;
    
    println!("Agent: {}", response.content);
    Ok(())
}
```

#### 场景 2：只使用 LLM 连接器（不带工具/会话）

```rust
use agent_runtime_rs::{
    config::loader::ConfigLoader,
    llm::connector::LLMConnector,
    llm::types::*, 
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 手动加载配置
    let agent_config = ConfigLoader::load_agent_config("config/agent-config.json")?;
    
    // 创建 LLM 连接器（不初始化完整运行时）
    let llm = LLMConnector::new(
        &agent_config.llm.model,
        &agent_config.llm.api_key,
        agent_config.llm.base_url.as_deref(),
        Some(agent_config.llm.temperature),
        Some(agent_config.llm.max_tokens),
    )?;
    
    // 直接调用 LLM
    let messages = vec![
        ChatMessage {
            role: MessageRole::System,
            content: "You are a helpful assistant.".to_string(),
            tool_calls: None,
            name: None,
        },
        ChatMessage {
            role: MessageRole::User,
            content: "Hello!".to_string(),
            tool_calls: None,
            name: None,
        },
    ];
    
    let response = llm.chat_completion(messages, None).await?;
    println!("LLM: {}", response.content);
    Ok(())
}
```

#### 场景 3：自定义工具 + Agent 运行时

```rust
use agent_runtime_rs::{
    create_agent_runtime,
    tools::types::{ToolExecutor, ToolParameter, ToolResult},
    session::types::MessageRole,
};
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::json;

// 定义自定义工具
struct MyCustomTool;

#[async_trait]
impl ToolExecutor for MyCustomTool {
    async fn execute(&self, parameters: serde_json::Value) -> ToolResult {
        let input = parameters.get("input").and_then(|v| v.as_str()).unwrap_or("");
        ToolResult::success(format!("Processed: {}", input))
    }
    
    fn get_metadata(&self) -> agent_runtime_rs::tools::types::ToolMetadata {
        agent_runtime_rs::tools::types::ToolMetadata {
            name: "my_custom_tool".to_string(),
            description: "My custom tool".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "input".to_string(),
                    description: "Input string".to_string(),
                    required: true,
                    parameter_type: "string".to_string(),
                }
            ],
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime = create_agent_runtime(
        "config/agent-config.json",
        "config/tools-config.json",
        "config/prompt-config.json",
    ).await?;
    
    // 注册自定义工具
    runtime.register_tool("my_custom_tool", Arc::new(MyCustomTool)).await?;
    
    // 使用 Agent（工具会自动被调用）
    let response = runtime.chat(
        Some("session-1"),
        "Please use my_custom_tool to process 'hello'".to_string(),
    ).await?;
    
    println!("Response: {}", response.content);
    Ok(())
}
```

#### 场景 4：集成到 Axum HTTP 服务器

```rust
use agent_runtime_rs::create_agent_runtime;
use axum::{extract::State, routing::post, Json, Router};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
    session_id: Option<String>,
}

#[derive(Serialize)]
struct ChatResponse {
    content: String,
    session_id: String,
}

// Axum handler
async fn chat_handler(
    State(runtime): State<Arc<agent_runtime_rs::AgentRuntime>>,
    Json(req): Json<ChatRequest>,
) -> Json<ChatResponse> {
    let session_id = req.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    match runtime.chat(Some(&session_id), req.message).await {
        Ok(resp) => Json(ChatResponse {
            content: resp.content,
            session_id,
        }),
        Err(e) => Json(ChatResponse {
            content: format!("Error: {}", e),
            session_id,
        }),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 Agent 运行时
    let runtime = create_agent_runtime(
        "config/agent-config.json",
        "config/tools-config.json",
        "config/prompt-config.json",
    ).await?;
    
    // 创建 Axum 路由
    let app = Router::new()
        .route("/chat", post(chat_handler))
        .with_state(runtime);
    
    // 启动 HTTP 服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

#### 场景 5：会话持久化（自动保存到磁盘）

```rust
use agent_runtime_rs::create_agent_runtime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime = create_agent_runtime(
        "config/agent-config.json",  // 确保其中 session.persistence.enabled = true
        "config/tools-config.json",
        "config/prompt-config.json",
    ).await?;
    
    // 会话会自动保存到 ./data/sessions/*.jsonl
    // 重启后自动恢复
    
    // 第一次运行：创建会话
    let response = runtime.chat(
        Some("persistent-session"),
        "Remember: my name is Alice".to_string(),
    ).await?;
    println!("{}", response.content);
    
    // 重启程序后...
    // 第二次运行：会话自动恢复，Agent 记得之前的对话
    let response2 = runtime.chat(
        Some("persistent-session"),
        "What's my name?".to_string(),
    ).await?;
    println!("{}", response2.content);  // 应该输出 "Alice"
    
    Ok(())
}
```

### 配置说明

#### Agent 配置（config/agent-config.json）

```json
{
  "llm": {
    "provider": "openaicompatible",
    "apiKey": "${ENV:OPENAI_API_KEY}",
    "baseURL": "${ENV:OPENAI_BASE_URL}",
    "model": "qwen-plus",
    "temperature": 0.7,
    "maxTokens": 10000,
    "mock": false
  },
  "session": {
    "maxHistoryLength": 100,
    "sessionTTL": 3600,
    "persistence": {
      "enabled": true,
      "storagePath": "./data/sessions",
      "autoSaveInterval": 300,
      "format": "jsonl"
    }
  },
  "tools": {
    "builtinTools": ["calculator", "get_current_time", "read_file", "write_file"],
    "autoExecuteTools": true,
    "sandboxDir": "./data"
  },
  "skills": {
    "autoLoadSkills": true,
    "skillsFolder": "./skills"
  }
}
```

### 更多示例

完整示例请查看 `examples/` 目录：
- `examples/basic_usage.rs` - 基本用法
- `examples/custom_tool.rs` - 自定义工具
- `examples/axum_integration.rs` - 集成到 Axum
- `examples/session_persistence.rs` - 会话持久化

---

## 🚀 快速开始

### 前置条件

- Rust 1.75+ (安装: https://rustup.rs/)
- Cargo (Rust 包管理器)
- 通义千问 API Key (或使用 OpenAI 兼容 API)

### 安装

```bash
# 克隆仓库
git clone https://github.com/your-repo/agent-runtime-rs.git
cd agent-runtime-rs

# 构建项目 (debug 模式)
cargo build

# 构建项目 (release 模式，优化)
cargo build --release
```

### 配置

#### 1. 环境变量配置 (推荐)

创建 `.env` 文件：

```bash
# .env
OPENAI_API_KEY=sk-gw-xxx
OPENAI_BASE_URL=https://openai.u2o6.com/v1
OPENAI_MODEL=qwen-plus
PORT=3000
```

#### 2. 配置文件 (可选)

编辑 `config/agent-config.json`：

```json
{
  "llm": {
    "provider": "openaicompatible",
    "apiKey": "${ENV:OPENAI_API_KEY}",
    "baseURL": "${ENV:OPENAI_BASE_URL}",
    "model": "qwen-plus",
    "temperature": 0.7,
    "maxTokens": 10000,
    "mock": false
  },
  "session": {
    "maxHistoryLength": 100,
    "sessionTTL": 3600
  },
  "tools": {
    "builtinTools": ["calculator", "get_current_time", "read_file", "write_file", "edit_file", "list_files", "create_directory", "delete_file"],
    "autoExecuteTools": true,
    "sandboxDir": "./data"
  },
  "skills": {
    "autoLoadSkills": true,
    "skillsFolder": "./skills"
  }
}
```

#### 3. MCP 服务器配置 (可选)

编辑 `config/tools-config.json`：

```json
{
  "mcpServers": {
    "apphunter": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-apphunter"],
      "env": {}
    }
  }
}
```

### 运行

```bash
# 加载 .env 并运行服务器 (默认端口 3000)
cargo run

# 或运行 release 版本
./target/release/agent-runtime-rs
```n
服务器启动后，可以访问 `http://localhost:3000/api/health` 检查健康状态。

### 🤖 Agent 配置 (可选)

自定义 Agent 人格、身份、工作区等：

```bash
# 复制配置模板
cp -r examples/agent-config ./agent-config

# 修改配置文件
vim agent-config/SOUL.md  # 修改人格定义
vim agent-config/IDENTITY.md  # 修改身份定义

# 重启服务器
cargo run
```

详细文档请查看 [Agent 配置](#-agent-配置-agent-config) 部分。

---

## 📡 API 文档

### 基础 URL

```
http://localhost:3000/api
```

---

### 1. **运行 Agent** `POST /api/run`

运行 Agent 并处理用户消息。

**请求**:
```http
POST /api/run
Content-Type: application/json

{
  "session_id": "optional-session-id",
  "message": "Hello, agent!"
}
```

**响应**:
```json
{
  "response": "Hello there! How can I help you today?",
  "tool_calls": [],
  "session_id": "session-123",
  "finish_reason": "stop"
}
```

---

### 2. **执行工具调用** `POST /api/tool-call`

直接执行工具（内置工具或 MCP 工具）。

**请求**:
```http
POST /api/tool-call
Content-Type: application/json

{
  "tool_name": "calculator",
  "parameters": {
    "expression": "2 + 3"
  }
}
```

**响应**:
```json
{
  "success": true,
  "output": "5",
  "error": null
}
```

---

### 3. **列出所有会话** `GET /api/sessions`

**请求**:
```http
GET /api/sessions
```

**响应**:
```json
{
  "sessions": ["session-1", "session-2"],
  "count": 2
}
```

---

### 4. **获取会话详情** `GET /api/sessions/:session_id`

**请求**:
```http
GET /api/sessions/session-123
```

**响应**:
```json
{
  "session_id": "session-123",
  "history": [
    {
      "role": "user",
      "content": "Hello!",
      "name": null,
      "tool_calls": null,
      "tool_call_id": null
    },
    {
      "role": "assistant",
      "content": "Hi there!",
      "name": null,
      "tool_calls": null,
      "tool_call_id": null
    }
  ],
  "metadata": {}
}
```

---

### 5. **删除会话** `DELETE /api/sessions/:session_id`

**请求**:
```http
DELETE /api/sessions/session-123
```

**响应**: `204 No Content`

---

### 6. **列出所有技能** `GET /api/skills` 🆕

列出所有已加载的技能。

**请求**:
```http
GET /api/skills
```

**响应**:
```json
{
  "skills": [
    {
      "name": "frontend-design",
      "description": "Generate frontend UI designs with HTML/CSS/JavaScript",
      "version": "1.0.0",
      "author": "Agent Runtime RS",
      "triggers": ["design UI", "create frontend", "build interface"],
      "tags": ["frontend", "design", "UI", "HTML", "CSS"],
      "script_count": 2
    }
  ],
  "count": 1
}
```

---

### 7. **获取技能详情** `GET /api/skills/:skill_name` 🆕

获取特定技能的详细信息。

**请求**:
```http
GET /api/skills/frontend-design
```

**响应**:
```json
{
  "name": "frontend-design",
  "description": "Generate frontend UI designs with HTML/CSS/JavaScript",
  "version": "1.0.0",
  "author": "Agent Runtime RS",
  "triggers": ["design UI", "create frontend", "build interface", "frontend design"],
  "required_tools": ["file_writer", "file_reader"],
  "tags": ["frontend", "design", "UI", "HTML", "CSS"],
  "scripts": [
    {
      "name": "script_1",
      "description": "Auto-extracted script",
      "language": "bash",
      "auto_execute": false
    }
  ],
  "content": "# Frontend Design Skill\n\nThis skill helps generate frontend UI designs..."
}
```

---

### 8. **健康检查** `GET /api/health`

**请求**:
```http
GET /api/health
```

**响应**:
```json
{
  "status": "healthy",
  "timestamp": "2026-07-03T04:00:00Z",
  "version": "0.1.0"
}
```

---

## 🧰 内置工具

Agent Runtime RS 提供以下 **8 个内置工具**：

| 工具 | 描述 | 示例 |
|------|------|------|
| **calculator** | 数学计算 | `2 + 3 * 4` |
| **get_current_time** | 获取当前时间 | - |
| **file_reader** | 读取文件内容 | `path/to/file.txt` |
| **file_writer** | 写入文件 | `content="Hello", path="output.txt"` |
| **file_editor** | 编辑文件 (正则表达式替换) | `s/foo/bar/g` |
| **file_lister** | 列出目录内容 | `path/to/dir` |
| **file_deleter** | 删除文件/目录 | `path/to/file.txt` |
| **directory_creator** | 创建目录 | `path/to/dir` |

---

## 📚 技能系统

技能是从 Markdown 文件加载的可复用知识模块。

### 技能文件格式

技能使用 **Markdown + YAML frontmatter** 格式：

```markdown
---
name: frontend-design
description: Generate frontend UI designs with HTML/CSS/JavaScript
version: 1.0.0
author: Agent Runtime RS
triggers:
  - design UI
  - create frontend
  - build interface
  - frontend design
tags:
  - frontend
  - design
  - UI
  - HTML
  - CSS
required_tools:
  - file_writer
  - file_reader
---

# Frontend Design Skill

This skill helps you generate frontend UI designs...

## Usage

To use this skill, describe the UI you want to create...

## Example

Here's an example login page:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Login</title>
    <style>
        body { font-family: Arial, sans-serif; }
    </style>
</head>
<body>
    <form>
        <input type="text" placeholder="Username">
        <input type="password" placeholder="Password">
        <button>Login</button>
    </form>
</body>
</html>
```

## Scripts

You can also include executable scripts:

```bash
# Create a new HTML file
echo "<!DOCTYPE html>" > index.html
```

```python
# Generate CSS from design tokens
import json
# ... Python code ...
```
```

### 技能目录结构

```
skills/
├── find-skills.md          # 技能文件 (直接放在 skills/ 下)
├── frontend-design.md      # 技能文件
└── frontend-design/        # 或使用子目录
    └── SKILL.md           # 技能文件 (子目录中的 SKILL.md)
```

### 加载技能

1. 将技能文件放入 `skills/` 目录
2. 重启服务器（或实现热加载）
3. 技能将自动加载并可通过 `GET /api/skills` 查看

### 使用技能

由于架构限制，技能执行暂不支持通过 API 调用。建议的使用方式：

1. 通过 `GET /api/skills/:name` 获取技能详情
2. 读取技能 Markdown 文件
3. 根据技能文档手动执行脚本或操作

---

## 🤖 Agent 配置 (agent-config/)

Agent Runtime RS 支持通过 `agent-config/` 目录配置 Agent 的人格、身份、工作区等。

### 📋 配置文件

在 `agent-config/` 目录中放置以下配置文件：

| 文件 | 用途 | 加载时机 |
|------|------|----------|
| `SOUL.md` | 人格定义（活泼、严肃、幽默等） | **启动时自动加载** ✅ |
| `IDENTITY.md` | 身份定义（名称、Emoji、Vibe） | **启动时自动加载** ✅ |
| `AGENTS.md` | 工作区定义和启动指令 | **启动时自动加载** ✅ |
| `MEMORY.md` | 长期记忆（用户信息、项目上下文） | **按需加载** (Skill) |
| `USER.md` | 用户信息（偏好、工作风格） | **按需加载** (Skill) |
| `TOOLS.md` | 工具使用说明和本地配置 | **按需加载** (Skill) |
| `HEARTBEAT.md` | 心跳检查任务列表 | **按需加载** (Skill) |

---

### 🚀 自动加载（方式 2）

**默认启用**：启动时自动读取 `agent-config/` 目录中的文件，并注入到 LLM 系统提示中。

**工作原理**：
1. 服务器启动时，`AgentRuntime::initialize()` 调用 `load_agent_config_files()`
2. 读取 `agent-config/SOUL.md`、`IDENTITY.md`、`AGENTS.md` 等文件
3. 组合所有配置内容，存储到 `agent_config_content` 字段
4. 每次 LLM 调用时，配置内容作为 **system 消息** 添加到对话开头
5. LLM 可以参考配置内容（人格、身份、工作区等）生成回答

**日志示例**：
```
INFO Loading agent-config files...
INFO Loaded agent-config file: agent-config/SOUL.md
INFO Loaded agent-config file: agent-config/IDENTITY.md
INFO Loaded agent-config file: agent-config/AGENTS.md
INFO Agent config content loaded (25352 bytes)
INFO AgentRuntime initialized successfully
```

**优点**：
- ✅ 每次对话都包含配置（一致性）
- ✅ 不需要手动触发
- ✅ 适合核心配置（人格、身份）

**缺点**：
- ⚠️ 消耗更多 token（配置内容较长）
- ⚠️ 每次都加载（即使不需要）

---

### 🎯 按需加载（方式 1 - Skill）

**可选**：将配置文件作为 Skill 加载，通过触发词调用。

**工作原理**：
1. 将 `agent-config/*.md` 复制到 `skills/agent-*.md`
2. 在 Skill 文件中定义 `triggers`（触发词）
3. 用户发送消息时，Agent 检测触发词
4. 如果匹配，加载 Skill 内容到对话上下文

**示例**：`skills/agent-soul.md`
```markdown
---
name: agent-soul
description: Agent 人格定义
triggers:
  - 你的人格是什么
  - 你是谁
  - 你是什么性格
---

# SOUL.md - Agent 人格定义

你是**活泼好动的全能小助手** 🦀...
```

**触发示例**：
```
用户: "你的人格是什么？"
Agent: "我是**活泼好动的全能小助手** 🦀！（加载了 agent-soul skill）"
```

**优点**：
- ✅ 按需加载，节省 token
- ✅ 灵活控制（只在需要时加载）
- ✅ 适合大型配置（MEMORY.md、TOOLS.md）

**缺点**：
- ⚠️ 需要手动触发（或依赖 LLM 检测）
- ⚠️ 可能不是每次都需要

---

### 📁 配置目录结构

```
agent-config/
├── SOUL.md          # 人格定义（活泼、严肃、幽默等）
├── IDENTITY.md      # 身份定义（名称、Emoji、Vibe）
├── AGENTS.md        # 工作区定义和启动指令
├── MEMORY.md        # 长期记忆（用户信息、项目上下文、经验教训）
├── USER.md          # 用户信息（偏好、工作风格、技术背景）
├── TOOLS.md         # 工具使用说明和本地配置
└── HEARTBEAT.md     # 心跳检查任务列表
```

---

### 🛠️ 自定义配置

#### 修改人格（SOUL.md）

编辑 `agent-config/SOUL.md`：

```markdown
# SOUL.md - Agent 人格定义

## 角色定位

# TODO: 修改这里定义您的 Agent 角色
# 示例：
# - 你是**专业的项目管理助手**，注重细节和效率。
# - 你是**幽默的编程伙伴**，喜欢用笑话解释复杂概念。

你是**活泼好动的全能小助手** 🦀，性格非常讨喜！

## 行为准则

### ✅ 应该做的
1. **使用表情符号**：适当使用 emoji 🎉🚀✅
2. **保持积极语气**：用"好的！"、"没问题！"
...
```

#### 修改身份（IDENTITY.md）

编辑 `agent-config/IDENTITY.md`：

```markdown
# IDENTITY.md - Agent 身份定义

## 基本信息

- **Name**: 全能小助手 (Full-Stack Helper)
- **Emoji**: 🦀 (代表 Rust 蟹)
- **Vibe**: 活泼、好动、积极、讨喜
```

#### 修改工作区（AGENTS.md）

编辑 `agent-config/AGENTS.md`：

```markdown
# AGENTS.md - Agent 工作区与启动指令

## 工作区定义

### 📂 主工作区

- **路径**: `{workspace_root_dir}`
- **说明**: 这是 Agent 的主工作目录
```

---

### 📋 配置模板

项目提供配置模板，供用户参考和复制：

```
examples/
└── agent-config/      # 配置模板（供用户参考）
    ├── SOUL.md        # 人格定义模板（含注释）
    ├── IDENTITY.md    # 身份定义模板（含注释）
    ├── AGENTS.md      # 工作区模板
    ├── MEMORY.md      # 长期记忆模板
    ├── USER.md        # 用户信息模板
    ├── TOOLS.md       # 工具配置模板
    ├── HEARTBEAT.md   # 心跳任务模板
    └── README.md      # 使用指南
```

**使用模板**：

```bash
# 复制模板到您的项目
cp -r examples/agent-config /your/project/agent-config

# 修改模板文件（查找 TODO 注释）
vim agent-config/SOUL.md
```

---

### 🧪 验证配置加载

#### 检查启动日志

```bash
# 启动服务器
cargo run

# 观察日志，确认配置已加载
INFO Loading agent-config files...
INFO Loaded agent-config file: agent-config/SOUL.md
INFO Loaded agent-config file: agent-config/IDENTITY.md
INFO Loaded agent-config file: agent-config/AGENTS.md
INFO Agent config content loaded (25352 bytes)
```

#### 测试配置生效

发送消息：
```json
{
  "message": "你的人格是什么？"
}
```

**预期回答**（参考 SOUL.md）：
```
"我是**活泼好动的全能小助手** 🦀！
我充满能量，喜欢快速行动，不拖泥带水！🎉"
```

---

### 💡 最佳实践

1. **核心配置自动加载** - `SOUL.md`、`IDENTITY.md`、`AGENTS.md` 应该自动加载
2. **大型配置按需加载** - `MEMORY.md`、`TOOLS.md` 适合作为 Skill 按需加载
3. **定期更新记忆** - `MEMORY.md` 应该动态更新（Agent 写入重要信息）
4. **使用模板** - 初次配置时，从 `examples/agent-config/` 复制模板
5. **测试配置** - 修改后，重启服务器并测试 LLM 回答是否符合预期

---

## 🔌 MCP 集成

Agent Runtime RS 支持 Model Context Protocol (MCP)，允许通过 stdio 与 MCP 服务器通信。

### 配置 MCP 服务器

在 `config/tools-config.json` 中添加 MCP 服务器配置：

```json
{
  "mcpServers": {
    "apphunter": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-apphunter"],
      "env": {}
    }
  }
}
```

### 工作原理

Agent Runtime 将自动：

1. 启动 MCP 服务器子进程 (通过 `command` + `args`)
2. 通过 stdin/stdout 进行 JSON-RPC 2.0 通信
3. 调用 `initialize()` 方法初始化 MCP 连接
4. 调用 `tools/list` 获取 MCP 服务器提供的工具
5. 将 MCP 工具注册到 ToolManager (作为 `MCPToolExecutor`)
6. 当工具被调用时，通过 `tools/call` 转发到 MCP 服务器

### 示例：apphunter MCP 服务器

当前配置已集成 **apphunter** MCP 服务器，提供 24 个工具：

- `opportunity_list` - 列出商机
- `opportunity_create` - 创建商机
- `opportunity_update` - 更新商机
- `project_list` - 列出项目
- `task_list` - 列出任务
- ... (共 24 个工具)

这些工具会作为普通工具注册，可通过 `POST /api/tool-call` 调用。

---

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test --lib config::
cargo test --lib llm::
cargo test --lib tools::
cargo test --lib session::
cargo test --lib mcp::
cargo test --lib skill::

# 运行集成测试
cargo test --test llm_integration_test
cargo test --test api_integration_test

# 检查测试覆盖率 (需要 nightly Rust)
cargo +nightly test --flags="--coverage"
```

**测试结果**:
- ✅ **153 个测试**全部通过
  - 141 个单元测试
  - 11 个集成测试
  - 1 个文档测试
- ✅ **0 失败**
- ✅ **代码覆盖率** > 80% (估计)

---

## 📦 依赖

主要依赖：

| Crate | 版本 | 用途 |
|-------|------|------|
| `axum` | 0.7 | HTTP 服务器 (Web 框架) |
| `tokio` | 1.0 | 异步运行时 |
| `serde` / `serde_json` | 1.0 | 序列化/反序列化 |
| `async-openai` | 0.18 | OpenAI API (通义千问兼容) |
| `tower-http` | 0.5 | HTTP 中间件 (CORS, Trace) |
| `tracing` | 0.1 | 日志 |
| `thiserror` | 1.0 | 错误处理 |
| `uuid` | 1.0 | UUID 生成 |
| `chrono` | 0.4 | 时间处理 |
| `dotenvy` | 0.15 | 加载 .env 文件 |
| `walkdir` | 2.5 | 递归遍历目录 (技能加载) |
| `tokio-process` | - | 子进程管理 (MCP) |

完整依赖列表请查看 `Cargo.toml`。

---

## 🛠️ 开发

### 构建

```bash
# Debug 模式 (快速编译，较低性能)
cargo build

# Release 模式 (优化编译，较高性能)
cargo build --release
```

### 运行

```bash
# 运行服务器
cargo run

# 传递命令行参数
cargo run -- --port 8080 --config-dir ./config
```

### Linting 和格式化

```bash
# 检查代码风格 (clippy)
cargo clippy -- -D warnings

# 自动格式化代码
cargo fmt

# 检查未使用的依赖 (需要 `cargo-udeps`)
cargo udeps
```

### 调试

```bash
# 设置日志级别
RUST_LOG=debug cargo run

# 或使用 .env 文件
echo "RUST_LOG=debug" >> .env
```

---

## 📊 性能对比 (估算)

| 指标 | TypeScript 版本 | Rust 版本 | 改进 |
|------|-----------------|-------------|------|
| **启动时间** | ~2.5s | ~0.1s | **25x 更快** |
| **内存使用** | ~150MB | ~25MB | **6x 更低** |
| **请求延迟** | ~50ms | ~5ms | **10x 更低** |
| **并发连接** | ~1000 | ~10000 | **10x 更高** |

*注意: 这些是初步估算，实际性能取决于工作负载和硬件。*

---

## 🤝 贡献

欢迎贡献！请按以下步骤：

1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 代码规范

- 遵循 Rust 官方风格指南
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 为新功能添加测试
- 更新文档

---

## 📄 许可证

本项目采用 MIT 许可证。查看 [LICENSE](LICENSE) 文件了解详情。

---

## 🙏 致谢

- 原始 TypeScript 版本: [agent-runtime-integration-example](https://github.com/your-repo/agent-runtime-integration-example)
- Axum 框架: https://github.com/tokio-rs/axum
- Tokio 异步运行时: https://tokio.rs/
- Rust 社区: https://www.rust-lang.org/community
- MCP (Model Context Protocol): https://modelcontextprotocol.io/

---

## 📧 联系方式

如有问题或建议，请：
- 创建 GitHub Issue: https://github.com/your-repo/agent-runtime-rs/issues
- 联系维护者: your-email@example.com

---

## 🗺️ 路线图

- [x] **阶段 1**: 配置加载 + 类型定义
- [x] **阶段 2**: 工具管理器 + 内置工具 (8 个)
- [x] **阶段 3**: LLM 连接器 (通义千问 qwen-plus)
- [x] **阶段 4**: 会话管理器 (UUID, TTL, 历史)
- [x] **阶段 5**: MCP 集成 (JSON-RPC 2.0, stdio)
- [x] **阶段 6**: 技能系统 (Markdown + YAML)
- [x] **阶段 7**: 核心运行时 (AgentRuntime)
- [x] **阶段 8**: HTTP API 服务器 (Axum) + 技能管理 API
- [x] **阶段 8.5**: Agent 配置系统 (agent-config/) - 自动加载 + 按需加载 🆕
- [x] **阶段 8.7**: 会话持久化 (Session Persistence) - 自动保存/恢复，JSONL 格式，避免双重写入 🆕
- [ ] **阶段 9**: OpenAPI/Swagger 文档 (utoipa 集成进行中)
- [ ] **阶段 10**: Docker 镜像
- [ ] **阶段 11**: 性能优化 (异步改进)
- [ ] **阶段 12**: 生产部署指南
- [ ] **阶段 13**: 技能执行 API (需要解决 Arc 可变访问问题)
- [ ] **阶段 14**: 技能热重载
- [ ] **阶段 15**: Agent 配置热重载 🆕
- [ ] **阶段 16**: Web UI (可选)

---

## 📚 相关文档

- [Task Artifact: Rust Porting](./TASK_ARTIFACT_RUST_PORTING.md) - Rust 移植完整记录
- [Task Artifact: LLM Config Fix](./TASK_ARTIFACT_LLM_CONFIG_FIX.md) - LLM 配置加载修复
- [Task Artifact: Tool Registration Fix](./TASK_ARTIFACT_TOOL_REGISTRATION_FIX.md) - 工具注册修复
- [Task Artifact: MCP Integration](./TASK_ARTIFACT_MCP_INTEGRATION.md) - MCP 集成详解
- [Task Artifact: Skill System](./TASK_ARTIFACT_SKILL_SYSTEM.md) - 技能系统实现

---

**用 ❤️ 和 Rust 构建**

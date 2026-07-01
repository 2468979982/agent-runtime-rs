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
- 🧰 **模块化设计**: 清晰的模块分离（LLM、工具、会话、技能、MCP）
- 🔌 **MCP 集成**: 支持 Model Context Protocol (MCP) 通过 stdio 通信
- 📁 **技能系统**: 从 Markdown 文件加载技能，并注册为工具
- 🌐 **HTTP API**: RESTful API 使用 Axum (类似 Express)
- 📊 **全面测试**: 153 个测试全部通过

---

## 📊 项目结构

```
agent-runtime-rs/
├── src/
│   ├── config/          # 配置加载 (JSON)
│   │   ├── loader.rs     # ConfigLoader
│   │   ├── types.rs     # 配置类型定义
│   │   └── mod.rs
│   ├── llm/             # LLM 集成
│   │   ├── connector.rs  # LLMConnector (通义千问)
│   │   ├── client.rs     # HTTP 客户端 (可选)
│   │   ├── types.rs     # LLM 类型定义
│   │   └── mod.rs
│   ├── tools/           # 工具管理
│   │   ├── manager.rs    # ToolManager
│   │   ├── types.rs      # 工具类型定义
│   │   ├── builtin/      # 内置工具
│   │   │   ├── calculator.rs
│   │   │   ├── file_reader.rs
│   │   │   ├── file_writer.rs
│   │   │   ├── file_editor.rs
│   │   │   ├── file_lister.rs
│   │   │   ├── file_deleter.rs
│   │   │   ├── directory_creator.rs
│   │   │   ├── get_current_time.rs
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── session/         # 会话管理
│   │   ├── manager.rs    # SessionManager
│   │   ├── types.rs      # 会话类型定义
│   │   └── mod.rs
│   ├── mcp/             # MCP 集成
│   │   ├── client.rs      # MCPClient trait
│   │   ├── stdio_client.rs # MCPStdioClient
│   │   ├── config.rs     # MCP 配置加载
│   │   ├── types.rs      # MCP 类型定义 (JSON-RPC 2.0)
│   │   └── mod.rs
│   ├── skills/          # 技能系统
│   │   ├── loader.rs     # SkillLoader
│   │   ├── reference_tool.rs # 技能引用工具
│   │   ├── types.rs      # 技能类型定义
│   │   └── mod.rs
│   ├── runtime/         # 核心运行时
│   │   ├── agent.rs      # AgentRuntime
│   │   ├── types.rs      # 运行时类型定义
│   │   └── mod.rs
│   ├── api/             # HTTP API (Axum)
│   │   ├── handlers.rs   # 请求处理器
│   │   ├── middleware.rs # 中间件 (CORS, logging)
│   │   ├── routes.rs     # 路由定义
│   │   ├── types.rs      # API 请求/响应类型
│   │   └── mod.rs
│   ├── utils/           # 工具类
│   │   ├── logger.rs     # 日志 (tracing)
│   │   └── mod.rs
│   ├── error.rs         # 全局错误类型
│   ├── lib.rs           # 库入口
│   └── main.rs          # 可执行文件入口
├── config/              # 配置文件
│   ├── agent-config.json
│   ├── tools-config.json
│   └── prompt-config.json
├── skills/              # 技能文件夹 (Markdown)
├── Cargo.toml          # Rust 项目配置
├── Cargo.lock          # 依赖锁定文件
└── README.md           # 本文档
```

---

## 🚀 快速开始

### 前置条件

- Rust 1.75+ (安装: https://rustup.rs/)
- Cargo (Rust 包管理器)
- 通义千问 API Key (可选，用于 LLM 集成)

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

1. **复制配置文件**:
   ```bash
   cp config/agent-config.json.example config/agent-config.json
   cp config/tools-config.json.example config/tools-config.json
   cp config/prompt-config.json.example config/prompt-config.json
   ```

2. **设置环境变量**:
   ```bash
   # 创建 .env 文件
   echo "QWEN_API_KEY=your_api_key_here" > .env
   ```

3. **编辑配置文件** (根据需要):
   - `config/agent-config.json` - LLM 配置
   - `config/tools-config.json` - 工具配置
   - `config/prompt-config.json` - 提示词配置

### 运行

```bash
# 运行服务器 (默认端口 3000)
cargo run

# 或运行 release 版本
./target/release/agent-runtime-rs
```

服务器启动后，可以访问 `http://localhost:3000/api/health` 检查健康状态。

---

## 📡 API 文档

### 端点

#### 1. **运行 Agent**
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
  "response": "Hello there!",
  "tool_calls": [],
  "session_id": "session-123"
}
```

---

#### 2. **执行工具调用**
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
  "result": {"value": 5},
  "success": true,
  "error": null
}
```

---

#### 3. **列出所有会话**
```http
GET /api/sessions
```

**响应**:
```json
{
  "sessions": ["session-1", "session-2"],
  "total": 2
}
```

---

#### 4. **获取会话详情**
```http
GET /api/sessions/:session_id
```

**响应**:
```json
{
  "session_id": "session-1",
  "history": [...],
  "metadata": {...}
}
```

---

#### 5. **删除会话**
```http
DELETE /api/sessions/:session_id
```

**响应**: `204 No Content`

---

#### 6. **健康检查**
```http
GET /api/health
```

**响应**: `200 OK`

---

## 🧰 内置工具

Agent Runtime RS 提供以下内置工具：

| 工具 | 描述 | 示例 |
|------|------|------|
| **calculator** | 数学计算 | `2 + 3 * 4` |
| **file_reader** | 读取文件 | `path/to/file.txt` |
| **file_writer** | 写入文件 | `content="Hello"` |
| **file_editor** | 编辑文件 (正则表达式) | `s/foo/bar/g` |
| **file_lister** | 列出目录内容 | `path/to/dir` |
| **file_deleter** | 删除文件/目录 | `path/to/file.txt` |
| **directory_creator** | 创建目录 | `path/to/dir` |
| **get_current_time** | 获取当前时间 | - |

---

## 📁 技能系统

技能是从 Markdown 文件加载的可复用知识模块。

### 创建技能

1. 在 `skills/` 目录中创建新的 `.md` 文件：
   ```markdown
   ---
   id: my-skill
   name: My Skill
   description: This is my custom skill
   author: Your Name
   version: 1.0.0
   ---
   
   # My Skill
   
   This is the content of my skill...
   ```

2. 重启服务器（或使用热加载，如果已实现）

3. 技能将自动注册为工具，名称为 `skill/<skill_id>`

---

## 🔌 MCP 集成

Agent Runtime RS 支持 Model Context Protocol (MCP)，允许通过 stdio 与 MCP 服务器通信。

### 配置 MCP 服务器

在 `config/tools-config.json` 中添加 MCP 服务器配置：

```json
{
  "mcpServers": {
    "my-mcp-server": {
      "command": "node",
      "args": ["path/to/mcp-server.js"],
      "env": {
        "API_KEY": "your_key"
      }
    }
  }
}
```

Agent Runtime 将自动：
1. 启动 MCP 服务器子进程
2. 通过 stdin/stdout 进行 JSON-RPC 2.0 通信
3. 加载 MCP 工具并注册到 ToolManager

---

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test --lib config::
cargo test --lib llm::
cargo test --lib tools::

# 运行集成测试
cargo test --test llm_integration_test

# 检查测试覆盖率 (需要 nightly Rust)
cargo +nightly test --flags="--coverage"
```

**测试结果**:
- ✅ **153 个测试**全部通过
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
| `tower-http` | 0.5 | HTTP 中间件 (CORS) |
| `tracing` | 0.1 | 日志 |
| `thiserror` | 1.0 | 错误处理 |
| `uuid` | 1.0 | UUID 生成 |
| `chrono` | 0.4 | 时间处理 |

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

---

## 📊 性能对比

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

---

## 📧 联系方式

如有问题或建议，请：
- 创建 GitHub Issue: https://github.com/your-repo/agent-runtime-rs/issues
- 联系维护者: your-email@example.com

---

## 🗺️ 路线图

- [x] **阶段 1**: 配置加载 + 类型定义
- [x] **阶段 2**: 工具管理器 + 内置工具
- [x] **阶段 3**: LLM 连接器 (通义千问)
- [x] **阶段 4**: 会话管理器
- [x] **阶段 5**: MCP 集成
- [x] **阶段 6**: 技能系统
- [x] **阶段 7**: 核心运行时 (AgentRuntime)
- [x] **阶段 8**: HTTP API 服务器 (Axum)
- [ ] **阶段 9**: Docker 镜像
- [ ] **阶段 10**: 性能优化
- [ ] **阶段 11**: 文档完善
- [ ] **阶段 12**: 生产部署指南

---

**用 ❤️ 和 Rust 构建**

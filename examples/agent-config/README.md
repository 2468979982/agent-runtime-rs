# Agent 配置文件模板使用指南

## 📋 简介

这个目录包含了 **Agent 配置文件模板**，您可以复制并根据自己的需求修改。

这些文件定义了 Agent 的：
- **人格**（SOUL.md）
- **身份**（IDENTITY.md）
- **工作区**（AGENTS.md）
- **记忆**（MEMORY.md）
- **用户信息**（USER.md）
- **工具配置**（TOOLS.md）
- **心跳任务**（HEARTBEAT.md）

---

## 🚀 快速开始

### 步骤 1：复制模板到您的项目

```bash
# 复制整个目录
cp -r examples/agent-config /your/project/agent-config

# 或者，只复制需要的文件
cp examples/agent-config/SOUL.md /your/project/agent-config/SOUL.md
cp examples/agent-config/IDENTITY.md /your/project/agent-config/IDENTITY.md
```

---

### 步骤 2：修改模板文件

打开每个文件，查找 `TODO` 注释，根据您的需求修改。

**示例**：修改 `SOUL.md`

```markdown
# 找到这一行
TODO: 修改这里定义您的 Agent 角色

# 修改为
你是**专业的项目管理助手** 📊，注重细节和效率。
```

---

### 步骤 3：在 Agent 启动时加载

确保您的 Agent 运行时在启动时读取这些文件。

**示例**（Rust）：
```rust
// 在 AgentRuntime::new() 中
let soul = std::fs::read_to_string("agent-config/SOUL.md")?;
let identity = std::fs::read_to_string("agent-config/IDENTITY.md")?;

// 将内容注入到系统提示中
let system_prompt = format!("{}\n\n{}", soul, identity);
```

---

## 📁 文件说明

### 1. `SOUL.md` - 人格定义

**用途**：定义 Agent 的人格、语气、行为准则

**应该修改的**：
- 角色定位（例如："专业顾问" vs "幽默伙伴"）
- 核心特点（例如："注重细节" vs "创意丰富"）
- 行为准则（例如：正式语气 vs 轻松语气）

**示例修改**：
```markdown
# 原版（活泼好动）
你是**活泼好动的全能小助手** 🦀，性格非常讨喜！

# 修改后（专业严谨）
你是**专业严谨的项目管理助手** 📊，注重细节和效率。
```

---

### 2. `IDENTITY.md` - 身份定义

**用途**：定义 Agent 的名称、Emoji、Vibe

**应该修改的**：
- Name（例如："项目管家"）
- Emoji（例如：📊）
- Vibe（例如："专业、严谨、高效"）

**示例修改**：
```markdown
# 原版
- **Name**: 全能小助手 (Full-Stack Helper)
- **Emoji**: 🦀
- **Vibe**: 活泼、好动、积极、讨喜

# 修改后
- **Name**: 项目管家 (Project Manager)
- **Emoji**: 📊
- **Vibe**: 专业、严谨、高效
```

---

### 3. `AGENTS.md` - 工作区定义

**用途**：定义工作区结构、启动指令、任务流程

**应该修改的**：
- 工作区路径
- 目录结构（根据项目需求添加/删除目录）
- 启动指令（根据您的工作流调整）

**示例修改**：
```markdown
# 原版
{workspace_root_dir}/
├── memory/
├── sessions/
├── skills/
├── data/
└── logs/

# 修改后（数据科学项目）
{workspace_root_dir}/
├── data/           # 数据文件
├── notebooks/      # Jupyter notebooks
├── models/         # 训练好的模型
├── memory/         # Agent 记忆
└── outputs/        # 输出文件
```

---

### 4. `MEMORY.md` - 长期记忆

**用途**：存储用户信息、项目上下文、经验教训

**应该修改的**：
- 用户信息（名称、偏好、工作风格）
- 项目上下文（项目名称、路径、状态）
- 重要决策（记录关键决策和原因）
- 经验教训（记录错误和学到的东西）

**注意**：这个文件会**动态更新**，Agent 会在运行时写入新记忆。

---

### 5. `USER.md` - 用户信息

**用途**：记录用户偏好、工作风格、技术背景

**应该修改的**：
- 基本信息（名称、时区、语言）
- 工作风格（沟通偏好、工作优先级）
- 技术背景（熟悉的工具、语言、框架）

**注意**：这个文件帮助 Agent **适应您的偏好**。

---

### 6. `TOOLS.md` - 工具配置

**用途**：记录子代理配置、会话管理、技能使用

**应该修改的**：
- 子代理配置（如果您有不同的子代理）
- 任务分解模板（根据您的项目调整）
- 工具配置表格（根据您的工具调整）

---

### 7. `HEARTBEAT.md` - 心跳任务

**用途**：定义心跳检查时应该做的事情

**应该修改的**：
- 心跳任务列表（根据您的需求添加/删除任务）
- 检查频率（根据优先级调整）
- 通知条件（何时应该通知用户）

---

## 💡 使用建议

### ✅ 应该做的

1. **根据项目类型调整** - 数据科学项目 vs Web 项目需要不同的配置
2. **保持一致性** - 确保 `SOUL.md` 和 `IDENTITY.md` 的风格一致
3. **定期更新** - 随着项目进展，更新 `MEMORY.md` 和 `USER.md`
4. **测试配置** - 启动 Agent，看是否符合预期的人格和行为

### ❌ 不应该做的

1. **不要复制粘贴不修改** - 模板需要根据您的需求调整
2. **不要过于复杂** - 保持配置简洁，避免过度自定义
3. **不要忘记测试** - 修改后，测试 Agent 是否按预期工作
4. **不要硬编码敏感信息** - 密码、API Key 等不要写在配置文件中

---

## 📝 示例场景

### 场景 1：数据科学项目

**需求**：Agent 帮助数据清洗、模型训练、结果可视化

**修改**：
- `SOUL.md`：改为"数据分析专家"，注重准确性和清晰度
- `IDENTITY.md`：Emoji 改为 📈，Vibe 改为"严谨、数据驱动"
- `AGENTS.md`：添加 `data/`、`notebooks/`、`models/` 目录
- `TOOLS.md`：添加 Python、Jupyter、Pandas 工具配置

---

### 场景 2：Web 开发项目

**需求**：Agent 帮助前端设计、后端 API、部署

**修改**：
- `SOUL.md`：改为"全栈开发助手"，注重效率和现代化
- `IDENTITY.md`：Emoji 改为 🌐，Vibe 改为"快速、现代化"
- `AGENTS.md`：添加 `frontend/`、`backend/`、`deploy/` 目录
- `TOOLS.md`：添加 React、Node.js、Docker 工具配置

---

### 场景 3：文档写作项目

**需求**：Agent 帮助写技术文档、博客、报告

**修改**：
- `SOUL.md`：改为"技术写手"，注重清晰和结构化
- `IDENTITY.md`：Emoji 改为 ✍️，Vibe 改为"清晰、结构化"
- `AGENTS.md`：添加 `docs/`、`images/`、`output/` 目录
- `TOOLS.md`：添加 Markdown、LaTeX、Grammarly 工具配置

---

## 🔗 相关资源

- **Agent Runtime RS 项目**: `https://github.com/your-repo/agent-runtime-rs`
- **OpenClaw 文档**: `https://docs.openclaw.ai`
- **Skill 系统文档**: `skills/README.md`

---

## 📞 需要帮助？

如果您在配置过程中遇到问题：

1. **查看示例**：参考 `examples/agent-config/` 中的完整示例
2. **阅读注释**：每个模板文件都包含详细的注释和 TODO 标记
3. **提问**：向 Agent 提问"如何修改 SOUL.md？"

---

**现在，开始配置您的 Agent 吧！🚀**

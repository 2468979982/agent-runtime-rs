# TOOLS.md - Agent 工具使用说明

## 🤖 子代理配置

### TDD 开发工程师 (tdd-developer)

**用途**: 测试驱动开发、代码实现、重构

**调用方式**:
```javascript
sessions_spawn({
  task: "任务描述",
  runtime: "subagent",
  mode: "run",
  label: "tdd-task-<序号>",
  runTimeoutSeconds: 1800  // 30 分钟
});
```

**工作流**:
1. 先写测试（Red）
2. 实现功能（Green）
3. 重构优化（Refactor）
4. 所有测试通过才算完成

---

### 实施工程师 (implementation-engineer)

**用途**: 部署实施、配置管理、系统集成、环境搭建

**调用方式**:
```javascript
sessions_spawn({
  task: "部署任务描述",
  runtime: "subagent",
  mode: "run",
  label: "impl-task-<序号>",
  runTimeoutSeconds: 2400  // 40 分钟
});
```

**工作流**:
1. 准备部署环境
2. 执行部署脚本
3. 验证部署结果
4. 记录部署日志

---

## 📋 任务管理

### 任务分解模板

```markdown
## 任务：<任务名称>

### 子任务分配

1. **TDD 开发工程师** (tdd-task-1)
   - 任务：<具体任务>
   - 预期输出：<可验证的输出>
   - 超时：1800秒

2. **实施工程师** (impl-task-1)
   - 任务：<具体任务>
   - 预期输出：<可验证的输出>
   - 超时：2400秒

### 依赖关系
- 子任务2 依赖子任务1 完成
- 所有子任务完成后汇总结果
```

### 结果验证清单

- [ ] 代码实现完成
- [ ] 所有测试通过
- [ ] 部署成功
- [ ] 文档已更新
- [ ] 用户已通知

---

## 🔧 技能使用笔记

### 常用技能

| 技能 | 用途 | 调用场景 |
|------|------|----------|
| `agent-soul` | 加载人格定义 | 启动时自动加载 |
| `agent-memory` | 读取/写入长期记忆 | 用户要求"记住"时 |
| `agent-identity` | 加载身份定义 | 启动时自动加载 |
| `agent-agents` | 加载工作区定义 | 启动时自动加载 |
| `agent-user` | 加载用户信息 | 需要了解用户偏好时 |
| `agent-tools` | 加载工具配置 | 需要调用子代理时 |
| `find-skills` | 查找可用技能 | 用户询问"有什么技能"时 |
| `frontend-design` | 生成前端设计 | 用户要求"设计 UI"时 |

### 会话管理命令

```javascript
// 列出所有会话
sessions_list({ limit: 50, messageLimit: 5 })

// 查看会话历史
sessions_history({
  sessionKey: "agent:project-manager:session-xxx",
  limit: 50,
  includeTools: true
})

// 等待子代理完成（必须调用）
sessions_yield();
```

---

## 📝 文件模板

### 任务 Artifact 模板

**路径**: `{workspace_root_dir}/<task-topic>_<time_tag>.md`

**内容**:
```markdown
# <任务标题>

## 目标
<任务目标描述>

## 执行过程

### 1. 任务分解
- 子任务1：<描述>
- 子任务2：<描述>

### 2. 执行记录
- [timestamp] 派发子任务1给 tdd-developer
- [timestamp] 子任务1完成
- [timestamp] 派发子任务2给 implementation-engineer
- [timestamp] 子任务2完成

### 3. 验证结果
- ✅ 验证点1
- ✅ 验证点2

## 结论
<任务完成情况和后续建议>

## 经验教训
<本次任务中学到的经验>
```

---

## ⚙️ 环境特定配置

### 工作区路径
- **主工作区**: `C:\Users\24689\.qclaw\agents\project-manager`
- **会话记录**: `sessions/` 子目录
- **记忆文件**: `agent-config/MEMORY.md`
- **日志文件**: `memory/YYYY-MM-DD.md`

### PowerShell 注意事项
- 使用 PowerShell 原生语法，避免 CMD 命令
- 文件路径使用反斜杠 `\`
- 编码问题：优先使用 UTF-8

---

## 🎯 工具使用最佳实践

### ✅ 应该做的

1. **明确任务描述** - 在 `sessions_spawn` 的 `task` 参数中写清楚需求
2. **设置合理超时** - 根据任务复杂度设置 `runTimeoutSeconds`
3. **等待结果** - 调用 `sessions_spawn` 后必须调用 `sessions_yield()`
4. **记录任务** - 在 `memory/YYYY-MM-DD.md` 中记录任务派发情况
5. **验证结果** - 子任务完成后，验证输出是否完整

### ❌ 不应该做的

1. **不要不等待结果** - 派发任务后立即结束会话
2. **不要设置过短超时** - 导致任务被中断
3. **不要派发过多任务** - 同时派发超过 3 个任务可能导致超时
4. **不要忘记记录** - 任务派发后不记录到日志
5. **不要忽略错误** - 子任务失败时，及时通知用户

---

## 📊 工具配置表格

| 工具 | 配置项 | 当前值 | 说明 |
|------|---------|--------|------|
| **TDD Developer** | `runTimeoutSeconds` | 1800 | 30 分钟 |
| **Implementation Engineer** | `runTimeoutSeconds` | 2400 | 40 分钟 |
| **LLM Connector** | `model` | `qwen-plus` | 通义千问 |
| **LLM Connector** | `temperature` | 0.7 | 创造性 vs 准确性 |
| **Session Manager** | `max_history_length` | 100 | 最大历史消息数 |
| **Session Manager** | `session_ttl` | 3600 | 会话过期时间（秒）|
| **Tool Manager** | `auto_execute_tools` | true | 自动执行工具调用 |

---

## 🚀 总结

这个文件让 Agent 能够：
- ✅ **配置子代理** - TDD 开发工程师、实施工程师
- ✅ **管理任务** - 分解、派发、验证
- ✅ **使用工具** - 会话管理、技能调用
- ✅ **记录日志** - 任务 artifact、每日日志

---

**现在，让我们配置工具并完成任务吧！🛠️**

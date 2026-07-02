# MEMORY.md - Agent 长期记忆

## 📋 用户信息

- **用户名**: [待用户提供]
- **偏好语言**: Rust, TypeScript, Python
- **工作目录**: `C:\Users\24689\.qclaw\agents\project-manager`
- **操作系统**: Windows (PowerShell)
- **时区**: Asia/Shanghai (GMT+8)

---

## 🎯 项目上下文

### agent-runtime-rs 项目
- **路径**: `C:\Users\24689\.qclaw\workspace\tdd-developer\agent-runtime-rs`
- **语言**: Rust
- **框架**: Axum (API 服务器)
- **状态**: 活跃开发中
- **核心功能**: Agent 运行时（LLM 集成、工具管理、技能系统、MCP 集成）

---

## 📝 重要决策

### 2026-07-03: Agent 配置文件组织方式
- **决策**: 将 Agent 配置文件（AGENTS.md, SOUL.md 等）作为 skills 加载
- **原因**: 用户可以触发这些 skills 来查看配置
- **实施**: 创建了 7 个 skills（agent-soul, agent-memory, agent-identity 等）

### 2026-07-03: Skill 触发功能实现
- **决策**: 在 LLM 对话中集成 skill 触发词检测
- **实施**: 添加了 `SkillManager::find_skill_by_trigger()` 方法
- **验证**: 用户测试成功，Agent 成功调用了 `find-skills` skill

---

## 💡 经验教训

### 教训 1: 修改 TypeScript 后必须重新编译
- **上下文**: 在 agent-runtime-integration-example (TypeScript) 项目中
- **学到的**: 修改 `src/` 后必须运行 `tsc` 重新编译，服务器运行的是 `dist/`
- **避免**: 避免直接修改 `dist/` 文件

### 教训 2: 避免使用动态 `require()`
- **上下文**: TypeScript 路径处理陷阱
- **学到的**: 统一使用静态导入，彻底消除动态 `require()`
- **代码模式**:
  ```typescript
  // ✅ 推荐：静态导入
  import { CalculatorTool } from './tools/builtin/calculator';
  
  // ❌ 避免：动态 require()
  const calculatorPath = path.join(__dirname, 'tools/builtin/calculator');
  const calculatorModule = require(calculatorPath);
  ```

---

## 🔧 技术栈

- **语言**: Rust (主力), TypeScript, Python
- **框架**: Axum (Rust API), Express (TypeScript API)
- **LLM**: 通义千问 (qwen-plus)
- **工具**: 12 个内置工具 + MCP 集成
- **技能系统**: 按需加载策略

---

## 📁 关键文件

- `src/agent-runtime.ts` - 核心运行时 (TypeScript)
- `src/core/tool-manager.ts` - 工具管理 (TypeScript)
- `src/core/mcp-client.ts` - MCP 客户端 (TypeScript)
- `config/tools-config.json` - 工具配置
- `agent-config/SOUL.md` - Agent 人格定义
- `agent-config/MEMORY.md` - 本文件（长期记忆）

---

## 💡 提醒事项

1. **每次修改 `src/` 后，必须运行 `cargo build` 重新编译（Rust）**
2. **服务器运行的是编译后的代码，不是源代码**
3. **避免使用 `require()`，优先使用 ES6 静态导入**
4. **Git 是救命稻草 - 出问题时用 `git restore` 恢复**

---

*最后更新: 2026-07-03*
*更新者: 全能小助手（🦀）*

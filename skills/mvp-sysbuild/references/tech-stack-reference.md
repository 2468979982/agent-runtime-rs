# 技术选型参考模板

> 最后更新: 2026-05-08
> 用途: Phase 1 需求确认阶段，AI 向用户推荐技术方案时参考此模板

---

## 一、Web 前端（网页端）

### 1.1 全栈框架（推荐给想快速出 MVP 的用户）

| 框架 | 语言 | 包大小 | 适合场景 | 学习曲线 | 生态 |
|------|------|--------|----------|----------|------|
| **Next.js** | TypeScript | 中 | SSR/SSG 全栈应用、SEO 敏感 | 中 | 极大 |
| **Nuxt** | TypeScript | 中 | Vue 生态全栈应用 | 中 | 大 |
| **Remix** | TypeScript | 小 | 数据驱动 Web 应用 | 中 | 中 |
| **Astro** | TypeScript/多框架 | 极小 | 内容站/博客/文档 | 低 | 中 |
| **SvelteKit** | TypeScript | 极小 | 高性能轻量应用 | 低 | 中 |

### 1.2 前端 UI 框架

| 框架 | 特点 | 适合场景 |
|------|------|----------|
| **React + Tailwind CSS** | 灵活、社区最大 | 通用 Web 应用 |
| **Vue 3 + Element Plus** | 中文友好、开箱即用 | 后台管理系统 |
| **Ant Design (React)** | 企业级组件库 | 中后台系统 |
| **Shadcn/ui (React)** | 可定制、无依赖 | 现代设计风格 |
| **Vuetify (Vue)** | Material Design | 快速原型 |

### 1.3 新兴/小众但值得关注

| 框架 | 特点 |
|------|------|
| **HTMX** | 无需写 JS，HTML 属性驱动交互 |
| **Alpine.js** | 轻量替代 jQuery，直接写在 HTML 里 |
| **Solid.js** | 类 React 但无虚拟 DOM，性能极好 |
| **Qwik** | 可恢复性框架，首屏极快 |

---

## 二、桌面端（Windows / macOS / Linux）

### 2.1 跨平台桌面框架

| 框架 | 前端技术 | 包大小 | 启动速度 | 适合场景 | 学习曲线 |
|------|----------|--------|----------|----------|----------|
| **Tauri 2.x** | Web (React/Vue/Svelte) | 3-10 MB | 快 | 轻量桌面工具 | 中 (需 Rust) |
| **Electron** | Web (React/Vue) | 80-150 MB | 慢 | 复杂桌面应用 | 低 |
| **Wails** | Web (Go 后端) | 10-20 MB | 快 | Go 开发者 | 中 |
| **Neutralinojs** | Web | 3-5 MB | 快 | 极简桌面工具 | 低 |
| **Flutter Desktop** | Dart | 20-40 MB | 中 | 需要移动端+桌面一致 UI | 中 |

### 2.2 原生桌面框架

| 框架 | 语言 | 包大小 | 适合场景 |
|------|------|--------|----------|
| **WinUI 3 / .NET 9** | C# | 小 | Windows 原生体验 |
| **SwiftUI** | Swift | 小 | macOS/iOS 原生 |
| **Qt 6** | C++/Python | 中 | 工业级跨平台 |
| **PyQt6 / PySide6** | Python | 中 | Python 开发者做 GUI |
| **CustomTkinter** | Python | 小 | Python 快速原型 (UI 一般) |
| **Dear ImGui** | C++/Python | 极小 | 工具类/调试界面 |

### 2.3 选型决策树

```
需要桌面应用？
├─ 团队会 Rust？→ Tauri 2.x (最轻量)
├─ 只要 Windows？→ WinUI 3 / WPF (.NET)
├─ 团队会 Python？→ PyQt6 或 Wails+Python
├─ 要最大生态/最易上手？→ Electron
└─ 要移动端+桌面统一 UI？→ Flutter
```

---

## 三、Android 端

### 3.1 跨平台（Android + iOS）

| 框架 | 语言 | 性能 | 生态 | 适合场景 |
|------|------|------|------|----------|
| **Flutter** | Dart | 高 | 大 | 高保真 UI、动画密集型 |
| **React Native** | TypeScript | 中高 | 极大 | Web 团队转移动端 |
| **Kotlin Multiplatform** | Kotlin | 高 | 中 | 共享逻辑层，UI 各端原生 |
| **Capacitor (Ionic)** | Web | 中 | 中 | Web 应用包装成 App |
| **.NET MAUI** | C# | 中 | 中 | .NET 开发者 |
| **Compose Multiplatform** | Kotlin | 高 | 新 | Jetpack Compose 跨平台 |

### 3.2 Android 原生

| 框架 | 特点 | 适合场景 |
|------|------|----------|
| **Jetpack Compose** | 声明式 UI，Google 官方 | 新 Android 项目首选 |
| **Kotlin + XML Layout** | 传统方式 | 维护老项目 |

---

## 四、iOS 端

### 4.1 跨平台（见上方 Android 跨平台部分，通用）

### 4.2 iOS 原生

| 框架 | 特点 | 适合场景 |
|------|------|----------|
| **SwiftUI** | 声明式 UI，Apple 官方 | 新 iOS 项目首选 |
| **UIKit** | 传统命令式 | 复杂定制 UI、维护老项目 |

---

## 五、后端 / API

| 框架 | 语言 | 特点 | 适合场景 |
|------|------|------|----------|
| **FastAPI** | Python | 异步、自动文档、快 | AI/ML 相关、快速 API |
| **Flask** | Python | 极简灵活 | 小型 API、原型 |
| **Express / Fastify** | TypeScript | Node.js 生态 | 全栈 JS 团队 |
| **Spring Boot** | Java/Kotlin | 企业级 | 大型系统 |
| **Gin / Fiber** | Go | 高性能 | 高并发微服务 |
| **Axum / Actix** | Rust | 极致性能 | 性能敏感场景 |
| **Django** | Python | 全功能、ORM 内置 | 内容管理、CRUD 重型 |
| **Supabase** | - | BaaS (Firebase 替代) | 快速 MVP、无后端开发 |
| **PocketBase** | Go | 嵌入式后端、单文件 | 极简后端需求 |

---

## 六、数据库

| 类型 | 推荐 | 特点 |
|------|------|------|
| **SQLite** | 本地存储、桌面应用内嵌 | 零配置、单文件 |
| **PostgreSQL** | 关系型首选 | 功能最强、开源 |
| **MySQL** | 关系型 | 最广泛、运维成熟 |
| **MongoDB** | 文档型 | 灵活 Schema、快速迭代 |
| **Redis** | 缓存/KV | 高速读写、会话管理 |
| **Prisma** | ORM | TypeScript 友好、类型安全 |

---

## 七、打包/部署

| 场景 | 工具 | 说明 |
|------|------|------|
| Windows 安装包 | **NSIS / Inno Setup / WiX** | 传统安装包 |
| Windows 便携版 | **Tauri build / Electron Builder** | 单 exe 或绿色版 |
| Mac DMG | **create-dmg** | macOS 标准 |
| Docker | **Docker Compose** | 服务端部署 |
| 静态网站 | **Vercel / Cloudflare Pages / Netlify** | 零配置部署 |

---

## 八、2025-2026 新兴趋势

| 趋势 | 说明 |
|------|------|
| **AI-Native 应用** | LangChain/LlamaIndex 构建 AI Agent 应用 |
| **Edge Runtime** | Cloudflare Workers / Deno Deploy 边缘计算 |
| **Bun** | Node.js 替代品，启动快 3x、安装快 30x |
| **Tauri 2.0** | 移动端支持（Android/iOS），一套 Rust 后端全平台 |
| **HTMX + Go/Python** | 极简全栈，无需 SPA 框架 |
| **Local-First** | 本地优先架构（CRDTs、SQLite），离线可用 |
| **Zig / Rust 系统编程** | 替代 C/C++ 的安全选择 |

---

## 九、按用户画像推荐

### 完全不懂技术的小白
- Web: **Astro** 或 **Nuxt** (模板多)
- 桌面: **Electron** (资料多、坑少)
- 移动: **Capacitor** (Web 包装)

### 会一点编程
- Web: **Next.js** 或 **Vue 3**
- 桌面: **Tauri 2.x** (轻量美观)
- 移动: **Flutter** 或 **React Native**

### 专业开发者
- Web: 按团队技术栈选
- 桌面: **Tauri** (Rust 后端) 或 **Qt** (C++)
- 移动: **Kotlin Multiplatform** 或原生

---

## 使用方式

AI 在 Phase 1 需求确认阶段：
1. 根据用户选择的平台（Web/桌面/Android/iOS）筛选对应分类
2. 根据用户技术水平（小白/入门/专业）进一步缩小范围
3. 提供 2-3 个推荐方案，附简要对比
4. **由用户做最终决定**
5. 记录用户选择到项目上下文，后续变更必须再次确认

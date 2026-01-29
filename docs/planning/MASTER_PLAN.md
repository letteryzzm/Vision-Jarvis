# Vision-Jarvis Master Plan

**视觉驱动的AI秘书 - Tauri + Astro 架构**

---

## 📋 项目概览

### 核心目标
构建一个基于 Tauri + Astro 的桌面AI助手，实现：
- 🖥️ 实时屏幕内容捕获和理解
- 🧠 短期/长期记忆系统
- 💡 智能触发反馈
- ⌨️ 快捷键唤起 (cmd+shift+j)
- 🔒 本地数据存储，保护隐私

### 技术栈

**前端层 (Astro)**
- 框架：Astro 4.x
- UI库：Tailwind CSS + shadcn/ui (可选)
- 状态管理：Nanostores
- 图标：Lucide Icons

**桌面层 (Tauri)**
- Rust 后端：Tauri 2.x
- 屏幕捕获：`screenshots` crate (跨平台)
- 系统托盘：Tauri System Tray Plugin
- 快捷键：Tauri Global Shortcut Plugin
- 窗口管理：Tauri Window Plugin

**AI & 服务**
- AI Provider：Claude API (Anthropic)
- HTTP Client：`reqwest` (Rust)
- 图像处理：`image` crate

**数据存储**
- 配置：`tauri-plugin-store`
- 文件系统：本地 JSONL 格式
- 路径：`~/Library/Application Support/vision-jarvis/`

---

## 🗂️ 项目结构

```
vision-jarvis/
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── main.rs         # 应用入口
│   │   ├── commands/       # Tauri 命令
│   │   │   ├── mod.rs
│   │   │   ├── capture.rs  # 屏幕捕获命令
│   │   │   ├── ai.rs       # AI 调用命令
│   │   │   └── memory.rs   # 记忆操作命令
│   │   ├── core/           # 核心业务逻辑
│   │   │   ├── screen/     # 屏幕捕获
│   │   │   ├── ai/         # AI 客户端
│   │   │   ├── memory/     # 记忆系统
│   │   │   └── triggers/   # 触发器
│   │   ├── config.rs       # 配置管理
│   │   └── utils.rs        # 工具函数
│   ├── Cargo.toml
│   ├── tauri.conf.json     # Tauri 配置
│   └── icons/              # 应用图标
│
├── src/                    # Astro 前端
│   ├── components/         # 组件
│   │   ├── Chat.astro      # 对话组件
│   │   ├── MemoryView.astro # 记忆查看
│   ��   └── Settings.astro  # 设置面板
│   ├── layouts/
│   │   └── Layout.astro    # 主布局
│   ├── pages/
│   │   ├── index.astro     # 主页面
│   │   └── settings.astro  # 设置页
│   ├── stores/             # Nanostores 状态
│   │   ├── chat.ts
│   │   └── config.ts
│   └── styles/
│       └── global.css      # 全局样式
│
├── public/                 # 静态资源
├── astro.config.mjs        # Astro 配置
├── tailwind.config.mjs     # Tailwind 配置
├── package.json
└── README.md
```

---

## 📊 依赖索引表

| ID | Status | Primary Files | Depends | Blocks |
|----|--------|---------------|---------|--------|
| TASK-001 | 📋 TODO | `src-tauri/Cargo.toml` | - | TASK-002, TASK-003 |
| TASK-002 | 📋 TODO | `src-tauri/src/core/screen/` | TASK-001 | TASK-005 |
| TASK-003 | 📋 TODO | `src/`, `astro.config.mjs` | TASK-001 | TASK-006 |
| TASK-004 | 📋 TODO | `src-tauri/src/core/ai/` | TASK-001 | TASK-005 |
| TASK-005 | 📋 TODO | `src-tauri/src/commands/` | TASK-002, TASK-004 | TASK-008 |
| TASK-006 | 📋 TODO | `src/components/Chat.astro` | TASK-003 | TASK-008 |
| TASK-007 | 📋 TODO | `src-tauri/src/core/memory/` | TASK-001 | TASK-009 |
| TASK-008 | 📋 TODO | `src/`, `src-tauri/src/` | TASK-005, TASK-006 | - |
| TASK-009 | 📋 TODO | `src-tauri/tauri.conf.json` | TASK-007 | - |
| TASK-010 | 📋 TODO | `src-tauri/src/core/triggers/` | TASK-007 | - |

---

## 🎯 MVP 阶段任务

### TASK-001: 项目初始化和脚手架 (📋 TODO)

**优先级**: P0-CRITICAL

**目标**: 搭建 Tauri + Astro 混合项目基础

**待修改文件**:
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `astro.config.mjs`
- `tailwind.config.mjs`

**实施步骤**:
- [ ] 创建 Tauri 项目骨架
- [ ] 集成 Astro 作为前端
- [ ] 配置 Tailwind CSS
- [ ] 设置开发环境热重载
- [ ] 配置 TypeScript 和 Rust 代码检查工具

**验收标准**:
- `npm run dev` 成功启动开发服务器
- `npm run tauri dev` 成功打开桌面窗口
- 前端页面显示 "Hello Vision-Jarvis"

**依赖**: 无

**阻塞**: TASK-002, TASK-003

---

### TASK-002: Rust 屏幕捕获模块 (📋 TODO)

**优先级**: P1-HIGH

**目标**: 实现跨平台屏幕截图功能

**待修改文件**:
- `src-tauri/src/core/screen/mod.rs`
- `src-tauri/src/core/screen/capturer.rs`
- `src-tauri/src/core/screen/optimizer.rs`
- `src-tauri/Cargo.toml` (添加依赖)

**技术方案**:
```rust
// 依赖
screenshots = "0.7"
image = "0.24"
```

**关键功能**:
1. 捕获主屏幕全屏截图
2. 图像压缩和优化 (目标 <5MB)
3. macOS 权限检查和引导
4. 导出为 base64 编码

**实施步骤**:
- [ ] 创建 `ScreenCapturer` trait
- [ ] 实现 `capture_screen()` 函数
- [ ] 实现图像优化器 (resize + compress)
- [ ] 添加权限检查逻辑
- [ ] 编写单元测试

**验收标准**:
- 能够捕获屏幕并返回 base64 字符串
- 图像大小 <5MB
- macOS 权限检查正常

**依赖**: TASK-001

**阻塞**: TASK-005

---

### TASK-003: Astro 前端基础 (📋 TODO)

**优先级**: P1-HIGH

**目标**: 搭建前端UI框架和基础组件

**待修改文件**:
- `src/layouts/Layout.astro`
- `src/pages/index.astro`
- `src/components/Chat.astro`
- `src/styles/global.css`
- `astro.config.mjs`

**UI设计**:
- 简洁的对话界面
- 系统托盘图标和菜单
- 设置面板

**实施步骤**:
- [ ] 创建主布局 Layout.astro
- [ ] 实现对话组件 (输入框 + 消息列表)
- [ ] 配置 Tailwind CSS 主题
- [ ] 创建设置页面
- [ ] 实现深色/浅色主题切换

**验收标准**:
- 页面样式美观
- 对话界面可交互
- 主题切换正常

**依赖**: TASK-001

**阻塞**: TASK-006

---

### TASK-004: Claude AI 客户端 (📋 TODO)

**优先级**: P1-HIGH

**目标**: 实现 Claude API 调用和图像分析

**待修改文件**:
- `src-tauri/src/core/ai/mod.rs`
- `src-tauri/src/core/ai/client.rs`
- `src-tauri/src/core/ai/prompts.rs`
- `src-tauri/Cargo.toml` (添加依赖)

**技术方案**:
```rust
// 依赖
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
base64 = "0.21"
```

**关键功能**:
1. 异步调用 Claude API
2. 图像 base64 编码和发送
3. Prompt 模板管理
4. 错误处理和重试机制

**实施步骤**:
- [ ] 创建 `ClaudeClient` 结构体
- [ ] 实现 `analyze_screen()` 方法
- [ ] 创建 Prompt 模板管理器
- [ ] 添加 API 速率限制处理
- [ ] 编写集成测试

**验收标准**:
- 成功调用 Claude API 并返回分析结果
- 支持图像 + 文本混合输入
- 错误处理完善

**依赖**: TASK-001

**阻塞**: TASK-005

---

### TASK-005: Tauri Commands 层 (📋 TODO)

**优先级**: P1-HIGH

**目标**: 实现前后端通信的 Tauri 命令

**待修改文件**:
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/capture.rs`
- `src-tauri/src/commands/ai.rs`
- `src-tauri/src/main.rs`

**关键命令**:
```rust
#[tauri::command]
async fn capture_screen() -> Result<String, String>

#[tauri::command]
async fn analyze_with_ai(image: String, prompt: String) -> Result<String, String>

#[tauri::command]
async fn get_chat_history() -> Result<Vec<Message>, String>
```

**实施步骤**:
- [ ] 定义命令接口
- [ ] 实现屏幕捕获命令
- [ ] 实现 AI 分析命令
- [ ] 注册命令到 Tauri
- [ ] 添加错误处理

**验收标准**:
- 前端可成功调用 Rust 命令
- 命令返回正确的数据类型
- 错误信息清晰

**依赖**: TASK-002, TASK-004

**阻塞**: TASK-008

---

### TASK-006: 前端状态管理和 API 调用 (📋 TODO)

**优先级**: P1-HIGH

**目标**: 实现前端与后端的数据交互

**待修改文件**:
- `src/stores/chat.ts`
- `src/stores/config.ts`
- `src/components/Chat.astro`

**技术方案**:
```typescript
// 使用 Tauri invoke API
import { invoke } from '@tauri-apps/api/tauri';

// 调用 Rust 命令
const result = await invoke('analyze_with_ai', {
  image: base64Image,
  prompt: userMessage
});
```

**实施步骤**:
- [ ] 创建 Nanostores 状态管理
- [ ] 实现聊天历史状态
- [ ] 封装 Tauri invoke 调用
- [ ] 实现消息发送和接收
- [ ] 添加加载状态和错误处理

**验收标准**:
- 用户可发送消息并得到 AI 回复
- 聊天历史正确显示
- 加载和错误状态展示清晰

**依赖**: TASK-003

**阻塞**: TASK-008

---

### TASK-007: 记忆系统 (📋 TODO)

**优先级**: P2-MEDIUM

**目标**: 实现短期/长期记忆的文件系统存储

**待修改文件**:
- `src-tauri/src/core/memory/mod.rs`
- `src-tauri/src/core/memory/storage.rs`
- `src-tauri/src/core/memory/manager.rs`

**数据结构**:
```rust
struct MemoryEntry {
    id: String,
    timestamp: DateTime<Utc>,
    entry_type: MemoryType, // Screenshot, Conversation, Summary
    content: serde_json::Value,
    importance: f32,
    tags: Vec<String>,
}
```

**存储路径**:
```
~/Library/Application Support/vision-jarvis/
├── memory/
│   ├── short_term/
│   │   └── sessions/
│   │       └── 2026-01-29.jsonl
│   └── long_term/
│       ├── knowledge/
│       │   └── 2026-01.jsonl
│       └── summaries/
│           └── daily/
│               └── 2026-01-29.json
```

**实施步骤**:
- [ ] 定义 `MemoryEntry` 结构
- [ ] 实现 JSONL 文件读写
- [ ] 实现短期记忆 (24小时 TTL)
- [ ] 实现长期记忆归档
- [ ] 实现简��的检索功能

**验收标准**:
- 记忆可正确存储和读取
- 短期记忆自动过期
- 文件格式正确 (JSONL)

**依赖**: TASK-001

**阻塞**: TASK-009

---

### TASK-008: 端到端集成 (📋 TODO)

**优先级**: P1-HIGH

**目标**: 打通整个流程：截图 → AI 分析 → 显示结果

**待修改文件**:
- `src/pages/index.astro`
- `src/components/Chat.astro`
- `src-tauri/src/main.rs`

**集成流程**:
```
用户点击"分析屏幕"
  ↓
前端调用 capture_screen()
  ↓
获取 base64 图像
  ↓
前端调用 analyze_with_ai(image, prompt)
  ↓
后端调用 Claude API
  ↓
返回分析结果
  ↓
前端显示在对话界面
```

**实施步骤**:
- [ ] 添加"分析屏幕"按钮
- [ ] 实现完整调用链
- [ ] 添加进度指示器
- [ ] 处理各种错误情况
- [ ] 优化用户体验 (加载动画等)

**验收标准**:
- 点击按钮后成功捕获屏幕
- AI 返回对屏幕内容的分析
- 结果正确显示在界面上
- 错误处理完善

**依赖**: TASK-005, TASK-006

**阻塞**: 无

---

### TASK-009: 系统托盘和快捷键 (📋 TODO)

**优先级**: P2-MEDIUM

**目标**: 实现系统托盘菜单和全局快捷键

**待修改文件**:
- `src-tauri/src/main.rs`
- `src-tauri/tauri.conf.json`

**技术方案**:
```rust
// 使用 Tauri 插件
tauri-plugin-global-shortcut = "2.0"
```

**功能**:
1. 系统托盘图标
2. 右键菜单 (显示/隐藏、退出)
3. 全局快捷键 `cmd+shift+j` 唤起窗口

**实施步骤**:
- [ ] 配置系统托盘
- [ ] 添加托盘菜单项
- [ ] 注册全局快捷键
- [ ] 实现窗口显示/隐藏逻辑
- [ ] 测试快捷键冲突

**验收标准**:
- 托盘图标正常显示
- 快捷键可唤起/隐藏窗口
- 菜单功能正常

**依赖**: TASK-007

**阻塞**: 无

---

### TASK-010: 触发器系统框架 (📋 TODO)

**优先级**: P3-LOW

**目标**: 搭建触发器系统的基础架构

**待修改文件**:
- `src-tauri/src/core/triggers/mod.rs`
- `src-tauri/src/core/triggers/manager.rs`
- `src-tauri/src/core/triggers/base.rs`

**架构设计**:
```rust
trait Trigger {
    fn check(&self, context: &Context) -> Option<TriggerEvent>;
    fn execute(&self, event: TriggerEvent) -> Result<(), Error>;
}
```

**MVP 阶段仅实现**:
- 用户手动触发
- 简单的时间触发 (定时总结)

**实施步骤**:
- [ ] 定义 Trigger trait
- [ ] 实现 TriggerManager
- [ ] 实现用户触发器
- [ ] 实现时间触发器
- [ ] 预留智能触��器接口

**验收标准**:
- 触发器框架可扩展
- 用户触发正常工作
- 时间触发可配置

**依赖**: TASK-007

**阻塞**: 无

---

## 🚀 开发里程碑

### Milestone 1: 基础框架 (Week 1)
- ✅ 完成 TASK-001: 项目初始化
- ✅ 完成 TASK-003: Astro 前端基础
- **目标**: 可运行的空壳应用

### Milestone 2: 核心功能 (Week 2-3)
- ✅ 完成 TASK-002: 屏幕捕获
- ✅ 完成 TASK-004: AI 客户端
- ✅ 完成 TASK-005: Tauri Commands
- **目标**: 可截图并调用 AI 分析

### Milestone 3: MVP 完成 (Week 4)
- ✅ 完成 TASK-006: 前端交互
- ✅ 完成 TASK-008: 端到端集成
- ✅ 完成 TASK-007: 记忆系统
- **目标**: 可用的 MVP 版本

### Milestone 4: 完善功能 (Week 5-6)
- ✅ 完成 TASK-009: 系统托盘和快捷键
- ✅ 完成 TASK-010: 触发器框架
- **目标**: 功能完整的 v1.0

---

## 🛣️ 未来路线图

| ID | Feature | Priority | Status | Notes |
|----|---------|----------|--------|-------|
| ROAD-001 | 智能内容触发器 | P2 | TODO | 检测特定应用/关键词 |
| ROAD-002 | 工作模式识别 | P2 | TODO | 识别编码/写作/休息模式 |
| ROAD-003 | 多显示器支持 | P3 | TODO | 选择捕获哪个屏幕 |
| ROAD-004 | Windows 支持 | P3 | TODO | 跨平台扩展 |
| ROAD-005 | Linux 支持 | P3 | TODO | 跨平台扩展 |
| ROAD-006 | 插件系统 | P3 | TODO | 允许用户自定义触发器 |
| ROAD-007 | 云同步 (可选) | P4 | IDEA | 加密云端备份 |
| ROAD-008 | 多语言支持 | P3 | TODO | i18n 国际化 |

---

## 💡 技术决策记录

### TDR-001: 为什么选择 Tauri + Astro？

**决策**: 使用 Tauri 2.x + Astro 4.x 混合架构

**理由**:
1. **Tauri 优势**:
   - Rust 后端性能优异，内存占用小
   - 原生系统 API 访问 (屏幕捕获、托盘、快捷键)
   - 应用体积小 (相比 Electron)
   - 更好的安全性

2. **Astro 优势**:
   - 零 JavaScript 默认，性能最优
   - 灵活的组件岛架构
   - 优秀的开发体验
   - 支持 Tailwind CSS

3. **组合优势**:
   - 前端轻量快速
   - 后端功能强大
   - 开发效率高

**替代方案**:
- Electron + React: 体积大，性能差
- Flutter Desktop: 生态不成熟
- PyQt6: Python 性能瓶颈

**日期**: 2026-01-29

---

### TDR-002: 为什么选择文件系统存储而非数据库？

**决策**: 使用 JSONL 文件格式存储记忆

**理由**:
1. **简单性**: 无需数据库依赖
2. **可读性**: 用户可直接查看数据
3. **可移植性**: 轻松备份和迁移
4. **隐私性**: 完全本地，无需担心数据泄��

**权衡**:
- 性能: 对于 MVP 阶段数据量足够
- 扩展性: 未来可升级为 SQLite 或向量数据库

**日期**: 2026-01-29

---

## 📝 最近完成

暂无

---

## 🐛 已知问题

| ID | Description | Severity | Status |
|----|-------------|----------|--------|
| - | 暂无 | - | - |

---

## 📚 参考资源

### 官方文档
- [Tauri 官方文档](https://tauri.app/v1/guides/)
- [Astro 官方文档](https://docs.astro.build/)
- [Claude API 文档](https://docs.anthropic.com/claude/reference/messages_post)

### 技术参考
- [screenshots crate](https://crates.io/crates/screenshots)
- [reqwest crate](https://crates.io/crates/reqwest)
- [Nanostores](https://github.com/nanostores/nanostores)

### 示例项目
- [Tauri Examples](https://github.com/tauri-apps/tauri/tree/dev/examples)
- [Astro + Tauri Template](https://github.com/astro-community/astro-tauri)

---

**最后更新**: 2026-01-29
**版本**: 1.0.0
**维护者**: Vision-Jarvis Team

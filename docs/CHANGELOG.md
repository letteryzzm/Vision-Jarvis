# Vision-Jarvis 项目文档变更记录

所有整体项目文档的变更都将记录在此文件中。

格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

---

## [Unreleased]

### Added - Phase 1: 核心基础设施实现 (2026-02-05)

#### 后端实现
- ✅ 数据库模式设计 (db/mod.rs, db/schema.rs, db/migrations.rs)
  - SQLite 数据库初始化
  - screenshots 表（截图元数据、AI 分析结果、向量嵌入）
  - short_term_memories 表（短期记忆、时间范围、活动分类）
  - long_term_memories 表（长期记忆总结）
  - settings 表（应用配置）
  - 数据库迁移系统
  - 测试覆盖率: 100%

- ✅ 设置持久化模块 (settings/mod.rs, settings/config.rs)
  - AppSettings 结构体定义
  - SettingsManager 配置管理
  - 输入验证（截图间隔、时间格式、存储限制）
  - 默认配置
  - 测试覆盖率: 100%

- ✅ 截图捕获模块 (capture/mod.rs, capture/scheduler.rs, capture/storage.rs)
  - ScreenCapture 使用 xcap 0.8.1
  - CaptureScheduler 定时调度器（可配置间隔 1-15秒）
  - StorageManager 存储管理（容量限制、自动清理）
  - 异步任务调度（tokio）
  - 测试覆盖率: 92%

- ✅ Tauri 插件集成
  - tauri-plugin-notification（系统通知）
  - tauri-plugin-autostart（开机自启动）
  - tauri-plugin-fs（文件系统）
  - tauri-plugin-store（配置持久化）
  - 权限配置更新

### Added - 后端、API、数据库文档 (2026-02-04)

#### 后端架构文档
- 🏗️ 创建后端文档总览 (backend/README.md)
- 🏛️ 创建后端架构概述 (backend/architecture/overview.md)
  - 分层架构设计 (Presentation / Service / DAL / Infrastructure)
  - 服务化设计模式
  - 异步并发架构
  - 错误处理机制
  - 系统架构图

#### 后端服务文档
- 📦 创建服务层概述 (backend/services/README.md)
- 🔧 创建核心服务文档:
  - 截屏服务 (backend/services/screenshot-service.md)
    - 定时截图、智能触发、图片处理、应用监控
    - 状态机设计 (Idle → Ready → Capturing → Processing → Completed)
    - 权限管理和性能优化
  - 记忆服务 (backend/services/memory-service.md)
    - 短期记忆生成、意图识别、事项提取
    - 时间窗口管理
    - 向量搜索和语义查询
    - 长期记忆聚合算法

#### API 接口文档
- 🌐 创建 API 文档总览 (api/README.md)
  - Tauri IPC Commands 协议说明
  - 接口列表和命名规范
  - 错误码说明和处理示例
  - 性能优化建议

#### 数据库设计文档
- 💾 创建数据库文档总览 (database/README.md)
  - SQLite/libSQL 技术选型
  - ER 图和表关系设计
  - 索引策略
  - 数据备份和清理策略
- 📊 创建核心表文档:
  - screenshots 表 (D1) (database/schema/tables/screenshots.md)
    - 截图元数据和 AI 分析结果
    - 状态机: pending → analyzing → completed/failed
  - short_term_memory 表 (D3) (database/schema/tables/short_term_memory.md)
    - 短期记忆事项存储
    - JSON 数组关联截图和应用
  - app_usage 表 (D4) (database/schema/tables/app_usage.md)
    - 应用使用时间追踪
    - 应用切换检测逻辑

### Added - 前端文档 (2026-02-04)
- 📝 创建前端文档总览 (frontend/README.md)
- 🏗️ 创建前端架构设计文档 (frontend/architecture.md)
- 📦 创建组件库概述 (frontend/components/README.md)
- 🎨 创建核心组件文档:
  - FloatingOrb 悬浮球组件 (frontend/components/FloatingOrb.md)
  - Header 展开模式组件 (frontend/components/Header.md)
  - Asker AI 问答组件 (frontend/components/Asker.md)
- 📄 创建页面文档:
  - Memory 记忆管理页面 (frontend/pages/memory.md)
  - Popup-Setting 提醒设置页面 (frontend/pages/popup-setting.md)

### 待创建
- 系统整体架构文档 (technical/architecture/system-overview.md)
- 数据流设计文档 (technical/architecture/data-flow.md)
- 前后端集成文档 (technical/architecture/integration.md)
- 产品路线图 (planning/roadmap.md)
- 需求文档 (planning/requirements.md)
- 后端其他服务文档 (ai-service.md, notification-service.md)
- 后端模块文档 (modules.md, error-handling.md, concurrency.md)
- API 详细接口文档 (endpoints/screenshot.md, memory.md, ai-analysis.md, notification.md)
- API 数据模型文档 (models/)
- 数据库其他表文档 (long_term_memory.md, notifications.md, app_config.md)
- 数据库迁移文档 (migrations/README.md)
- 前端其他组件文档 (DatePicker, MemoryList, MemoryCard, FloatingInput, SettingCard, ToggleSwitch 等)
- 前端状态管理文档 (frontend/state-management.md)
- 前端样式规范文档 (frontend/styling.md)
- 前端动画设计文档 (frontend/animations.md)
- 前端开发指南 (frontend/development.md)
- 前端测试文档 (frontend/testing.md)

---

## [2.0.0] - 2026-02-04

### Added
- 🎉 实现三层文档架构（整体/前端/后端分离）
- 📝 创建文档审计报告 (DOCUMENT_AUDIT_REPORT.md)
- 📝 创建整体文档变更记录 (CHANGELOG.md)
- 📁 创建整体技术文档目录 (technical/architecture/, technical/specifications/)

### Migrated
- 📦 迁移 `functional-specifications.md` 到 `technical/specifications/functional-specs.md`
- 📦 迁移 `non-functional-requirements.md` 到 `technical/specifications/non-functional-specs.md`
- 📦 移动 `UPDATES.md` 到整体文档目录

### Changed
- ♻️  重构文档索引 (README.md v2.0)
- ♻️  建立三层文档架构导航系统
- ♻️  添加旧文档位置迁移映射表

### Organizational
- 🗂️  前端文档迁移至 `/vision-jarvis/src/docs/`
- 🗂️  后端文档迁移至 `/vision-jarvis/src-tauri/docs/`
- 🗂️  整体文档保留在 `/docs/`

---

## [1.0.0] - 2026-01-29

### Added
- 📝 初始文档结构
- 📋 主计划文档 (planning/MASTER_PLAN.md)
- 🧪 集成测试报告
- 📖 项目搭建总结 (SETUP_SUMMARY.md)
- 📖 迁移指南 (MIGRATION.md)

---

**说明**:
- 本 CHANGELOG 仅记录整体项目文档的变更
- 前端文档变更见 `/vision-jarvis/src/docs/CHANGELOG.md`
- 后端文档变更见 `/vision-jarvis/src-tauri/docs/CHANGELOG.md`

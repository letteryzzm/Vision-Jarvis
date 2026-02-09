# Vision-Jarvis 项目文档

> **最后更新**: 2026-02-06
> **文档版本**: v2.2 (Frontend V2: Multi-Window Architecture)
> **文档层级**: 层级 1 - 整体项目文档

---

## 📚 文档架构

Vision-Jarvis 采用**三层文档架构**（整体/前端/后端分离）:

1. **整体项目文档** (`/docs/`) - 项目规划、系统架构、测试、发布
2. **前端文档** (`/vision-jarvis/src/docs/`) - Astro 前端技术文档
3. **后端文档** (`/vision-jarvis/src-tauri/docs/`) - Rust/Tauri 后端技术文档

---

## 🎯 快速导航

### 架构师/技术负责人
- [系统整体架构](technical/architecture/system-overview.md)（待创建）
- [数据流设计](technical/architecture/data-flow.md)（待创建）
- [前后端集成](technical/architecture/integration.md)（待创建）
- [前端文档入口](../vision-jarvis/src/docs/README.md) ✅
- [后端文档入口](../vision-jarvis/src-tauri/docs/README.md) ✅
- **NEW** [Frontend V2: Multi-Window Architecture](frontend/architecture-v2-floating-windows.md) ✅

### 前端开发者
- [前端文档总览](frontend/README.md) ✅
- [前端架构设计 V1](frontend/architecture.md) ✅ (单页面架构，已废弃)
- **NEW** [前端架构设计 V2](frontend/architecture-v2-floating-windows.md) ✅ (多窗口架构，当前版本)
- [组件库概述](frontend/components/README.md) ✅
- [页面文档](frontend/pages/) ✅
  - [Memory 页面](frontend/pages/memory.md)
  - [Popup-Setting 页面](frontend/pages/popup-setting.md)

### 后端开发者
- [后端文档总览](../vision-jarvis/src-tauri/docs/README.md) ✅
- [后端架构](../vision-jarvis/src-tauri/docs/technical/architecture/backend-architecture.md) ✅
- [Tauri Commands API](../vision-jarvis/src-tauri/docs/technical/api/tauri-commands.md) ✅
- [数据库设计](../vision-jarvis/src-tauri/docs/technical/database/schema.md) ✅
- **Phase 6 & 7 新增**:
  - [Storage Service](backend/services/storage-service.md) ✅ - 文件存储管理
  - [AI Providers Service](backend/services/ai-providers-service.md) ✅ - AI提供商配置
  - [Storage API](api/endpoints/storage.md) ✅ - 存储管理接口
  - [AI Config API](api/endpoints/ai-config.md) ✅ - AI配置接口

### 项目管理
- [主计划 (MASTER_PLAN)](planning/MASTER_PLAN.md) ✅
- [产品路线图](planning/roadmap.md)（待创建）
- [需求文档](planning/requirements.md)（待创建）

---

### 项目级文档

#### 后端服务（backend/services/）
- [storage-service.md](backend/services/storage-service.md) ✅ - 文件存储管理服务
- [ai-providers-service.md](backend/services/ai-providers-service.md) ✅ - AI提供商配置服务

#### API 接口（api/endpoints/）
- [storage.md](api/endpoints/storage.md) ✅ - 存储管理API（5个接口）
- [ai-config.md](api/endpoints/ai-config.md) ✅ - AI配置API（8个接口）

### 规划文档（planning/）
- [MASTER_PLAN.md](planning/MASTER_PLAN.md) ✅ - 任务跟踪和主计划
- roadmap.md（待创建）- 产品路线图
- requirements.md（待创建）- 功能需求

### 整体技术文档（technical/）

#### 系统架构（architecture/）
- system-overview.md（待创建）- 系统总览
- data-flow.md（待创建）- 数据流设计
- integration.md（待创建）- 前后端集成

#### 功能规格（specifications/）
- [functional-specs.md](technical/specifications/functional-specs.md) ✅ - 功能规格
- [non-functional-specs.md](technical/specifications/non-functional-specs.md) ✅ - 非功能性需求

### 测试文档（testing/）
- [集成测试](testing/integration/) ✅
- [测试报告](testing/test-reports/) ✅

### 其他文档
- [MIGRATION.md](MIGRATION.md) ✅ - 迁移指南
- [SETUP_SUMMARY.md](SETUP_SUMMARY.md) ✅ - 项目搭建总结
- [UPDATES.md](UPDATES.md) ✅ - 更新记录
- [DOCUMENT_AUDIT_REPORT.md](DOCUMENT_AUDIT_REPORT.md) ✅ - 文档审计报告

### 笔记（notes/）
- [AGENTS.md](notes/AGENTS.md) ✅ - AI 代理笔记

---

## 📝 最近更新

| 日期 | 文档 | 层级 | 变更类型 | 说明 |
|------|------|------|---------|------|
| 2026-02-06 | Frontend V2 Architecture | 前端 | 重大重构 | 多窗口悬浮球架构，从单页面重构为多窗口系统 |
| 2026-02-06 | Phase 6 & 7 文档 | 后端/API | 新增 | 文件管理和AI配置系统文档 |
| 2026-02-06 | Storage Service | 后端 | 新增 | 文件存储管理服务文档（350行） |
| 2026-02-06 | AI Providers | 后端 | 新增 | AI提供商配置服务文档（480行） |
| 2026-02-06 | Storage API | API | 新增 | 5个存储管理接口文档 |
| 2026-02-06 | AI Config API | API | 新增 | 8个AI配置接口文档 |
| 2026-02-04 | 文档架构重构 | 整体 | 重构 | 实现三层文档架构（整体/前端/后端） |
| 2026-02-04 | 前端文档迁移 | 前端 | 新增 | 创建 /vision-jarvis/src/docs/ |
| 2026-02-04 | 后端文档迁移 | 后端 | 新增 | 创建 /vision-jarvis/src-tauri/docs/ |
| 2026-02-02 | API 扩展 | 后端 | 新增 | 添加 Todo、通知等 API |
| 2026-02-02 | 前端设计 | 前端 | 更新 | 新增双日期选择功能 |

查看完整变更记录: [CHANGELOG.md](CHANGELOG.md) ✅

---

## 📂 旧文档位置说明

**重要提示**: 2026-02-04 文档架构重组后，部分文档已迁移：

### 迁移映射

| 旧位置 | 新位置 | 状态 |
|--------|--------|------|
| `/vision-jarvis/docs/technical/frontend-*.md` | `/vision-jarvis/src/docs/technical/architecture/` | ✅ 已迁移 |
| `/vision-jarvis/docs/technical/backend-*.md` | `/vision-jarvis/src-tauri/docs/technical/architecture/` | ✅ 已迁移 |
| `/vision-jarvis/docs/technical/api-*.md` | `/vision-jarvis/src-tauri/docs/technical/api/` | ✅ 已迁移 |
| `/vision-jarvis/docs/technical/database-*.md` | `/vision-jarvis/src-tauri/docs/technical/database/` | ✅ 已迁移 |
| `/vision-jarvis/docs/development/component-*.md` | `/vision-jarvis/src/docs/technical/components/` | ✅ 已迁移 |
| `/vision-jarvis/docs/technical/*-specifications.md` | `/docs/technical/specifications/` | ✅ 已迁移 |

---

## 🔍 文档查找

### 我想了解...

- **项目整体规划?** → [主计划](planning/MASTER_PLAN.md)
- **新的多窗口架构?** → [Frontend V2 Architecture](frontend/architecture-v2-floating-windows.md) 🆕
- **前端如何开发?** → [前端文档](../vision-jarvis/src/docs/README.md)
- **后端如何开发?** → [后端文档](../vision-jarvis/src-tauri/docs/README.md)
- **API 如何调用?** → [Tauri Commands](../vision-jarvis/src-tauri/docs/technical/api/tauri-commands.md)
- **数据库设计?** → [数据库模型](../vision-jarvis/src-tauri/docs/technical/database/schema.md)
- **文件管理系统?** → [Storage Service](backend/services/storage-service.md) + [Storage API](api/endpoints/storage.md)
- **AI配置管理?** → [AI Providers Service](backend/services/ai-providers-service.md) + [AI Config API](api/endpoints/ai-config.md)
- **最新变更?** → [CHANGELOG.md](CHANGELOG.md)
- **最新测试结果?** → [测试报告](testing/test-reports/)
- **项目搭建过程?** → [SETUP_SUMMARY.md](SETUP_SUMMARY.md)

---

## 📖 文档规范

### 三层架构原则

1. **层级 1（整体文档）**: 跨前后端的系统级设计、项目规划、测试发布
2. **层级 2（前端文档）**: Astro 前端独立文档，前端开发者可独立阅读
3. **层级 3（后端文档）**: Rust/Tauri 后端独立文档，后端开发者可独立阅读

### 文档命名规范

- 使用小写和连字符: `system-overview.md`
- 描述性名称: `tauri-commands.md` 优于 `api.md`
- 日期前缀用于临时文档: `2026-01-29-test-result.md`

### Markdown 风格

- 使用清晰的标题层级 (`#`, `##`, `###`)
- 添加目录（如果文档较长）
- 使用代码块标注语言: ` ```rust ` ` ```typescript `
- 适度使用 emoji 提升可读性

---

## 🔗 相关链接

- [项目主页](../README.md)
- [前端文档](../vision-jarvis/src/docs/README.md)
- [后端文档](../vision-jarvis/src-tauri/docs/README.md)
- [Tauri 官方文档](https://tauri.app/)
- [Astro 官方文档](https://docs.astro.build/)
- [Claude API 文档](https://docs.anthropic.com/)

---

**文档中心维护者**: Vision-Jarvis 开发团队
**文档架构**: 三层架构（整体/前端/后端）
**最后更新**: 2026-02-06 (Frontend V2: Multi-Window Architecture + Phase 6 & 7 完成)
**当前架构版本**: Frontend V2.0 (Multi-Window Floating Ball System)

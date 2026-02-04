# Vision-Jarvis 项目文档变更记录

所有整体项目文档的变更都将记录在此文件中。

格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

---

## [Unreleased]

### 待创建
- 系统整体架构文档 (technical/architecture/system-overview.md)
- 数据流设计文档 (technical/architecture/data-flow.md)
- 前后端集成文档 (technical/architecture/integration.md)
- 产品路线图 (planning/roadmap.md)
- 需求文档 (planning/requirements.md)

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

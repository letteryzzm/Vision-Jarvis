# 📚 Vision-Jarvis 文档中心

欢迎来到 Vision-Jarvis 项目文档中心。所有项目文档统一存放在此目录。

## 📂 目录结构

```
docs/
├── planning/           # 📋 规划文档
├── development/        # 💻 开发文档
├── technical/          # 🔧 技术文档
├── testing/           # 🧪 测试文档
├── releases/          # 🚀 发布文档
└── notes/             # 📝 临时笔记
```

## 📋 规划文档 (planning/)

项目规划、架构设计、路线图等高层次文档。

- `MASTER_PLAN.md` - 主要实施计划
- `architecture.md` - 系统架构设计
- `roadmap.md` - 产品路线图

## 💻 开发文档 (development/)

开发环境、工作流程、编码规范等。

- `setup.md` - 环境搭建指南
- `workflow.md` - 开发工作流
- `conventions.md` - 代码和文档规范

## 🔧 技术文档 (technical/)

### API 文档 (technical/api/)
- API 设计文档
- 接口规范
- 数据模型

### 组件文档 (technical/components/)
- 前端组件说明
- Astro 组件库
- UI 设计指南

### 架构文档 (technical/architecture/)
- 系统架构图
- 数据流设计
- 技术选型说明

## 🧪 测试文档 (testing/)

### 测试报告 (testing/test-reports/)
- 集成测试报告
- 性能测试结果
- Bug 修复验证

示例: `2026-01-29-tauri-integration-test.md`

### 集成文档 (testing/integration/)
- 集成方案说明
- 第三方服务对接
- API 集成测试

## 🚀 发布文档 (releases/)

- `changelog.md` - 版本变更日志
- `migration/` - 版本迁移指南
- 发布说明和发布检查清单

## 📝 临时笔记 (notes/)

临时性的记录、会议纪要、调研笔记等。

使用日期命名: `YYYYMMDD-topic-name.md`

示例:
- `2026-01-29-rust-installation-notes.md`
- `2026-01-30-claude-api-research.md`

---

## 🔍 快速查找

### 我想了解...

- **如何开始开发?** → `development/setup.md`
- **项目整体规划?** → `planning/MASTER_PLAN.md`
- **API 如何调用?** → `technical/api/`
- **最新测试结果?** → `testing/test-reports/` (按日期排序)
- **版本更新内容?** → `releases/changelog.md`

### 我要创建...

- **测试报告** → `testing/test-reports/YYYYMMDD-*.md`
- **技术方案** → `technical/[category]/*.md`
- **临时笔记** → `notes/YYYYMMDD-*.md`
- **API 文档** → `technical/api/*.md`

---

## 📖 文档规范

### 命名规范

1. **使用小写和连字符**: `screen-capture-api.md`
2. **日期前缀用于临时文档**: `2026-01-29-test-result.md`
3. **描述性名称**: `tauri-astro-integration.md` 优于 `test1.md`

### 文档模板

每个文档应包含：

```markdown
# 文档标题

**创建日期**: YYYY-MM-DD
**作者**: [名称]
**状态**: [草稿/审核中/已完成]

## 概述

[简要说明文档目的]

## 内容

[主要内容]

---

**相关文档**:
- [链接到相关文档]

**最后更新**: YYYY-MM-DD
```

### Markdown 风格

- 使用清晰的标题层级 (`#`, `##`, `###`)
- 添加目录（如果文档较长）
- 使用代码块标注语言: ` ```python ` ` ```bash `
- 添加 emoji 提升可读性（适度使用）

---

## 🔗 外部链接

- [项目主 README](../README.md)
- [Tauri 官方文档](https://tauri.app/)
- [Astro 官方文档](https://docs.astro.build/)
- [Claude API 文档](https://docs.anthropic.com/)

---

**文档中心维护者**: Vision-Jarvis 开发团队
**最后更新**: 2026-01-29

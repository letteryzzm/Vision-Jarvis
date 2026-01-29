# 📦 文档迁移说明

**迁移日期**: 2026-01-29
**版本**: 1.0

## 🎯 迁移目的

统一项目文档管理，将所有分散的 Markdown 文档集中到 `docs/` 目录，提升项目可维护性。

## 📋 迁移清单

### 已迁移的文档

| 原位置 | 新位置 | 文档类型 |
|--------|--------|----------|
| `/MASTER_PLAN.md` | `docs/planning/MASTER_PLAN.md` | 主计划 |
| `/INTEGRATION_REPORT.md` | `docs/testing/integration/INTEGRATION_REPORT.md` | 集成报告 |
| `/AGENTS.md` | `docs/notes/AGENTS.md` | 代理说明 |
| `/vision-jarvis/TEST_RESULT.md` | `docs/testing/test-reports/2026-01-29-tauri-integration-test.md` | 测试报告 |

### 保留的文档

这些文档保留在原位置（符合约定）：

- `/README.md` - 项目主说明
- `/LICENSE` - 开源协议
- `/.env.example` - 环境变量示例
- `/vision-jarvis/README.md` - 子项目说明（可选）

## 📁 新目录结构

```
docs/
├── README.md                    # 文档中心说明
├── planning/                    # 规划文档
│   └── MASTER_PLAN.md          ✅ 已迁移
├── development/                 # 开发文档
├── technical/                   # 技术文档
│   ├── api/
│   ├── components/
│   └── architecture/
├── testing/                     # 测试文档
│   ├── test-reports/
│   │   └── 2026-01-29-tauri-integration-test.md  ✅ 已迁移
│   └── integration/
│       └── INTEGRATION_REPORT.md                 ✅ 已迁移
├── releases/                    # 发布文档
│   └── migration/
└── notes/                       # 临时笔记
    └── AGENTS.md               ✅ 已迁移
```

## 🔧 配置更新

### 1. 创建了 `.clinerules`

项目系统提示词文件，定义了：
- 📁 文档管理规范
- 📝 文档命名规则
- 🚫 禁止的行为
- ✅ AI 助手的文档创建规则

### 2. 更新了 `.gitignore`

确保 `docs/` 目录被正确追踪，只忽略构建输出 `docs/_build/`

### 3. 创建了 `docs/README.md`

文档中心索引，提供：
- 目录结构说明
- 快速查找指南
- 文档规范和模板

## 🎓 使用指南

### 对于开发者

1. **��找文档**: 所有文档现在都在 `docs/` 目录
2. **创建文档**: 参考 `docs/README.md` 中的命名规范
3. **更新文档**: 在对应的子目录中更新

### 对于 AI 助手 (Claude Code)

系统将自动读取 `.clinerules` 文件，遵循以下规则：

✅ **正确做法**:
```bash
# 创建测试报告
docs/testing/test-reports/2026-01-30-screen-capture-test.md

# 创建 API 文档
docs/technical/api/claude-integration.md

# 创建临时笔记
docs/notes/2026-01-30-quick-notes.md
```

❌ **错误做法**:
```bash
# 在根目录创建
/NEW_FEATURE.md

# 在代码目录创建
/vision-jarvis/DOCS.md
```

## 📌 重要提醒

### 引用更新

如果有代码或其他文档引用了旧的文档路径，需要更新引用：

**旧引用**:
```markdown
参见 [主计划](../MASTER_PLAN.md)
```

**新引用**:
```markdown
参见 [主计划](docs/planning/MASTER_PLAN.md)
```

### Git 提交

迁移后需要提交到版本控制：

```bash
git add .clinerules
git add docs/
git add .gitignore
git commit -m "docs: migrate all documentation to docs/ directory

- Created unified docs/ structure
- Added .clinerules for documentation management
- Migrated MASTER_PLAN.md to docs/planning/
- Migrated test reports to docs/testing/
- Updated .gitignore to track docs/
"
```

## 🔄 后续维护

### 定期检查

1. 确保新文档遵循命名规范
2. 及时归档临时笔记
3. 更新过时的文档

### 清理规则

`docs/notes/` 中的临时笔记应该：
- 在完成相关工作后移动到正式文档
- 或者在不再需要时删除
- 保留期限建议：30 天

## 📞 问题反馈

如果发现：
- 文档分类不合理
- 命名规范需要调整
- 新的文档类型需要支持

请在项目中提出讨论。

---

**迁移执行者**: Claude Code
**最后更新**: 2026-01-29

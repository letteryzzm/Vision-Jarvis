---
name: vision-jarvis-docs
description: 全栈文档管理 Skill（统一目录版）。所有文档统一放在根目录 docs/ 下，按前端/后端/API/数据库分类。优先更新现有文档，避免重复创建。当用户要求"更新文档"时自动触发。
---

# Vision-Jarvis 文档管理 Skill

---

## ⚠️ 核心规则（必须遵守）

### 规则 1: 只有 `plans/` 作为历史记录

**允许存在的文档类型：**
- `docs/plans/` — 功能实现计划（含实时完成度）
- `docs/CODEMAP.md` — 文件功能映射
- `docs/CHANGELOG.md` — 版本变更记录
- `docs/frontend/` — 前端技术文档
- `docs/backend/` — 后端技术文档
- `docs/api/` — API 接口文档
- `docs/database/` — 数据库文档
- `docs/planning/MASTER_PLAN.md` — 主计划

**严禁创建的文档类型：**
```
❌ IMPLEMENTATION_STATUS.md  — 用 plan 的完成度替代
❌ PHASE_PROGRESS.md         — 用 plan 的完成度替代
❌ UPDATES.md                — 用 CHANGELOG.md 替代
❌ SETUP_SUMMARY.md          — 一次性文档，无需保留
❌ DOCUMENT_AUDIT_REPORT.md  — 一次性文档，无需保留
❌ *_STATUS.md               — 所有状态报告
❌ *_SUMMARY.md              — 所有总结报告（除 CHANGELOG）
❌ DOCUMENTATION_SUMMARY.md  — 元文档
```

### 规则 2: 写代码时必须同步更新 plan 完成度

**每次完成一个功能点后，立即更新对应 plan 文件：**

```markdown
<!-- docs/plans/2026-02-16-memory-system-v3.md 示例 -->

## 任务清单

- [x] 截图分析器 (screenshot_analyzer.rs) — 完成于 2026-02-19
- [x] 活动分组器 (activity_grouper.rs) — 完成于 2026-02-19
- [ ] 向量搜索优化
- [ ] 前端记忆展示页面
```

**触发时机：**
- 新增一个 `.rs` / `.ts` / `.tsx` 文件后
- 完成一个功能模块后
- 修复一个 bug 后（如果 plan 中有对应任务）

### 规则 3: 优先更新现有文档，禁止新建版本文件

```
❌ architecture-v2.md
❌ memory-service-new.md
✅ 直接编辑 architecture.md（Git 保留历史）
```

---

## 📁 文档目录结构

```
docs/
├── CODEMAP.md                    # 文件→功能映射（唯一真相来源）
├── CHANGELOG.md                  # 版本变更记录
├── README.md                     # 文档总索引
│
├── plans/                        # 实现计划（含完成度，永久保留）
│   └── YYYY-MM-DD-feature.md    # 每个功能一个 plan
│
├── planning/
│   └── MASTER_PLAN.md           # 主计划和路线图
│
├── frontend/                     # 前端技术文档
│   ├── README.md
│   ├── architecture.md
│   ├── components/
│   └── pages/
│
├── backend/                      # 后端技术文档
│   ├── README.md
│   ├── architecture/
│   └── services/
│
├── api/                          # API 文档
│   ├── README.md
│   └── endpoints/
│
└── database/                     # 数据库文档
    ├── README.md
    └── schema/
```

---

## 🔄 Plan 文件格式规范

每个 plan 文件必须包含**实时完成度**：

```markdown
# [功能名称] 实现计划

> 创建日期: YYYY-MM-DD
> 最后更新: YYYY-MM-DD
> 状态: 进行中 / 已完成

## 目标

[一句话描述]

## 任务清单

### Phase 1: [阶段名]
- [x] 任务 1 — 完成于 YYYY-MM-DD
- [x] 任务 2 — 完成于 YYYY-MM-DD
- [ ] 任务 3

### Phase 2: [阶段名]
- [ ] 任务 4
- [ ] 任务 5

## 已知问题 / 备注

[实现过程中发现的问题，直接在此记录]
```

---

## 🔄 文档更新流程

### 写代码时（实时）

```
完成一个功能点
    ↓
找到对应 plan 文件（Glob: docs/plans/*.md）
    ↓
将对应任务标记为 [x]，加上完成日期
    ↓
如果是重大变更，更新 CODEMAP.md
```

### 用户要求"更新文档"时

```
步骤 1: Glob 查找现有文档
步骤 2: Read 读取现有内容
步骤 3: Edit 增量更新（不新建文件）
步骤 4: 更新 CHANGELOG.md
```

---

## 📋 场景更新清单

### 新增后端模块

| 文档 | 操作 |
|------|------|
| `docs/CODEMAP.md` | 添加新文件的功能描述 |
| `docs/plans/对应plan.md` | 标记任务完成 `[x]` |
| `docs/CHANGELOG.md` | 记录变更 |
| `docs/backend/services/[service].md` | 新建或更新服务文档 |

### 新增前端组件

| 文档 | 操作 |
|------|------|
| `docs/plans/对应plan.md` | 标记任务完成 `[x]` |
| `docs/CHANGELOG.md` | 记录变更 |
| `docs/frontend/components/[Component].md` | 新建组件文档 |

### 数据库变更

| 文档 | 操作 |
|------|------|
| `docs/CODEMAP.md` | 更新数据库表列表 |
| `docs/plans/对应plan.md` | 标记任务完成 `[x]` |
| `docs/database/schema/tables/[table].md` | 更新表结构 |
| `docs/CHANGELOG.md` | 记录变更 |

---

## ✅ 检查清单

### 写代码后
- [ ] 对应 plan 的任务已标记 `[x]`
- [ ] CODEMAP.md 已更新（如有新文件）
- [ ] 没有创建任何状态报告文档

### 文档更新后
- [ ] 使用 Edit 更新原文件（未新建版本文件）
- [ ] CHANGELOG.md 已记录
- [ ] 没有创建 *_STATUS / *_SUMMARY / *_REPORT 类文件

---

**Skill 版本**: v5.0
**最后更新**: 2026-02-19
**核心变更**: 禁止状态报告文档，plans 承担完成度追踪，写代码时实时更新 plan

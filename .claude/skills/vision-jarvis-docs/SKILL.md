---
name: vision-jarvis-docs
description: 全栈文档管理 Skill（统一目录版）。所有文档统一放在根目录 docs/ 下，按前端/后端/API/数据库分类。优先更新现有文档，避免重复创建。当用户要求"更新文档"时自动触发。
---

# Vision-Jarvis 全栈文档管理 Skill

统一管理 Vision-Jarvis 项目的所有技术文档，采用**统一目录结构**，所有文档集中在 `/docs/` 下，按**前端/后端/API/数据库**分类。

---

## 何时使用

**自动触发条件**（必须使用此 skill）:
- 用户要求"更新文档"、"同步文档"、"整理文档"
- 修改或新增功能后需要同步文档
- 用户提到 `docs/` 目录
- 需要检查文档完整性

**关键词触发**:
- "文档"、"docs"、"documentation"
- "更新文档"、"同步文档"
- "前端文档"、"后端文档"、"API 文档"、"数据库文档"

---

## ⚠️ 核心原则

### 原则 1: 优先更新现有文档，避免新建

**强制规则**:
- ✅ **更新现有文档** - 找到对应的现有文档并增量更新
- ❌ **避免新建版本** - 不创建 `xxx-v2.md`、`xxx-new.md`
- ✅ **增量更新** - 在原文档基础上修改
- ✅ **Git 版本控制** - 通过 Git 保留历史

**示例**:
```
❌ 错误:
  用户: "更新前端架构文档"
  → 创建 docs/frontend/architecture-v2.md

✅ 正确:
  用户: "更新前端架构文档"
  → 查找 docs/frontend/architecture.md
  → 在原文件上增量更新
  → 更新 CHANGELOG.md
```

### 原则 2: 统一文档目录（根目录 docs/）

所有文档统一放在 `/docs/` 目录下：
- 便于集中管理和查找
- 避免文档分散在代码目录
- 清晰的文档层级结构

### 原则 3: 文档查找优先

更新前必须先查找现有文档:
1. 使用 `Glob` 搜索相关文档
2. 使用 `Read` 读取现有内容
3. 确认需要更新的章节
4. 执行 `Edit` 增量更新

---

## 📁 统一文档目录结构

```
Vision-Jarvis/
│
├── docs/                                  # 【统一文档根目录】
│   ├── README.md                         # 文档总索引
│   ├── getting-started.md                # 快速开始
│   ├── CHANGELOG.md                      # 版本变更记录
│   ├── MIGRATION.md                      # 迁移指南
│   ├── SETUP_SUMMARY.md                  # 项目搭建总结
│   │
│   ├── planning/                         # 项目规划
│   │   ├── README.md
│   │   ├── MASTER_PLAN.md               # 主计划
│   │   ├── roadmap.md                   # 产品路线图
│   │   └── requirements.md              # 需求文档
│   │
│   ├── frontend/                         # 【前端文档】
│   │   ├── README.md                    # 前端概述
│   │   ├── development.md               # 开发指南
│   │   ├── architecture.md              # 前端架构
│   │   ├── components/                  # 组件文档
│   │   │   ├── README.md               # 组件库概述
│   │   │   ├── FloatingOrb.md          # 悬浮球组件
│   │   │   ├── Header.md               # Header 组件
│   │   │   ├── Asker.md                # Asker 组件
│   │   │   └── MemoryList.md           # 记忆列表组件
│   │   ├── pages/                       # 页面文档
│   │   │   ├── index.md                # 主页
│   │   │   ├── memory.md               # 记忆管理页
│   │   │   └── popup-setting.md        # 提醒设置页
│   │   ├── state-management.md         # 状态管理 (Nanostores)
│   │   ├── routing.md                  # 路由配置
│   │   ├── styling.md                  # 样式规范 (Tailwind CSS)
│   │   ├── animations.md               # 动画设计
│   │   ├── testing.md                  # 前端测试
│   │   └── build-deploy.md             # 构建部署
│   │
│   ├── backend/                          # 【后端文档】
│   │   ├── README.md                    # 后端概述
│   │   ├── development.md               # 开发指南
│   │   ├── architecture/                # 架构设计
│   │   │   ├── overview.md             # 架构概述
│   │   │   ├── modules.md              # 模块划分
│   │   │   ├── error-handling.md       # 错误处理
│   │   │   └── concurrency.md          # 并发设计
│   │   ├── services/                    # 服务文档
│   │   │   ├── README.md               # 服务概述
│   │   │   ├── memory-service.md       # 记忆服务
│   │   │   ├── screenshot-service.md   # 截屏服务
│   │   │   ├── ai-service.md           # AI 服务
│   │   │   └── notification-service.md # 通知服务
│   │   ├── middleware/                  # 中间件文档
│   │   │   └── logging.md              # 日志中间件
│   │   ├── testing.md                   # 后端测试
│   │   ├── security.md                  # 安全规范
│   │   └── monitoring.md                # 监控告警
│   │
│   ├── api/                              # 【API 文档】
│   │   ├── README.md                    # API 概述
│   │   ├── authentication.md            # 认证授权
│   │   ├── error-codes.md               # 错误码说明
│   │   ├── endpoints/                   # 接口详细文档
│   │   │   ├── screenshot.md           # 截屏接口
│   │   │   ├── memory.md               # 记忆接口
│   │   │   ├── ai-analysis.md          # AI 分析接口
│   │   │   └── notification.md         # 通知接口
│   │   └── models/                      # 数据模型
│   │       ├── Screenshot.md           # 截图模型
│   │       ├── ShortTermMemory.md      # 短期记忆模型
│   │       ├── LongTermMemory.md       # 长期记忆模型
│   │       └── Notification.md         # 通知模型
│   │
│   ├── database/                         # 【数据库文档】
│   │   ├── README.md                    # 数据库概述
│   │   ├── schema/                      # 数据库模式
│   │   │   ├── overview.md             # 模式概览
│   │   │   ├── erd.md                  # ER 图
│   │   │   └── tables/                 # 表文档
│   │   │       ├── screenshots.md      # 截图表
│   │   │       ├── short_term_memory.md # 短期记忆表
│   │   │       ├── long_term_memory.md  # 长期记忆表
│   │   │       └── notifications.md     # 通知表
│   │   ├── migrations/                  # 迁移文档
│   │   │   ├── README.md               # 迁移指南
│   │   │   └── history.md              # 迁移历史
│   │   ├── indexes/                     # 索引文档
│   │   │   └── strategy.md             # 索引策略
│   │   ├── queries/                     # 查询文档
│   │   │   ├── common-queries.md       # 常用查询
│   │   │   └── optimization.md         # 查询优化
│   │   └── backup-restore.md           # 备份恢复
│   │
│   ├── deployment/                       # 【部署文档】
│   │   ├── README.md                    # 部署概述
│   │   ├── environments.md              # 环境配置
│   │   ├── docker.md                    # Docker 部署
│   │   ├── ci-cd.md                     # CI/CD 流程
│   │   └── troubleshooting.md           # 故障排查
│   │
│   ├── testing/                          # 【测试文档】
│   │   ├── README.md                    # 测试概述
│   │   ├── unit-testing.md              # 单元测试
│   │   ├── integration-testing.md       # 集成测试
│   │   ├── e2e-testing.md               # 端到端测试
│   │   └── test-reports/                # 测试报告
│   │
│   ├── guides/                           # 【操作指南】
│   │   ├── code-review.md               # 代码审查
│   │   ├── git-workflow.md              # Git 工作流
│   │   ├── debugging.md                 # 调试指南
│   │   └── best-practices.md            # 最佳实践
│   │
│   ├── references/                       # 【参考资料】
│   │   ├── tech-stack.md                # 技术栈说明
│   │   ├── glossary.md                  # 术语表
│   │   └── faq.md                       # 常见问题
│   │
│   └── notes/                            # 【笔记和临时文档】
│       └── AGENTS.md
│
└── vision-jarvis/                        # 应用代码（不含文档）
    ├── src/                              # 前端代码
    └── src-tauri/                        # 后端代码
```

---

## 🔄 文档更新流程

### 标准更新流程

```
用户请求: "更新前端架构文档"

步骤 1: 查找现有文档
  ↓
  Glob: docs/frontend/architecture.md

步骤 2: 读取现有文档
  ↓
  Read: docs/frontend/architecture.md

步骤 3: 识别需要更新的章节
  ↓
  根据用户需求确定要修改的部分

步骤 4: 执行增量更新
  ↓
  Edit: 精确替换需要更新的段落

步骤 5: 更新 Frontmatter
  ↓
  Edit: 更新 version, updated 字段

步骤 6: 更新 CHANGELOG
  ↓
  Edit: docs/CHANGELOG.md 记录变更

步骤 7: （可选）更新索引
  ↓
  如果文档结构变化，更新 docs/README.md
```

---

## 📋 新需求文档更新清单

### 场景 1: ���加新前端组件

| 文档路径 | 更新内容 | 优先级 |
|---------|---------|--------|
| `docs/CHANGELOG.md` | 添加新组件条目 | 🔴 必须 |
| `docs/frontend/components/[Component].md` | 创建组件文档 | 🔴 必须 |
| `docs/frontend/components/README.md` | 更新组件列表 | 🔴 必须 |
| `docs/frontend/README.md` | 如果是主要功能，更新概述 | 🟡 建议 |

**操作步骤**:
1. 创建组件文档：`docs/frontend/components/[Component].md`
2. 更新组件索引：`docs/frontend/components/README.md`
3. 更新 CHANGELOG：`docs/CHANGELOG.md`

### 场景 2: 添加新 API 接口

| 文档路径 | 更新内容 | 优先级 |
|---------|---------|--------|
| `docs/CHANGELOG.md` | 添加新接口条目 | 🔴 必须 |
| `docs/api/endpoints/[resource].md` | 创建接口详细文档 | 🔴 必须 |
| `docs/api/models/[Model].md` | 创建数据模型文档 | 🔴 必须 |
| `docs/api/README.md` | 更新接口列表 | 🔴 必须 |
| `docs/backend/services/[service].md` | 更新相关服务文档 | 🟡 建议 |

**操作步骤**:
1. 创建接口文档：`docs/api/endpoints/[resource].md`
2. 创建模型文档：`docs/api/models/[Model].md`
3. 更新 API 索引：`docs/api/README.md`
4. 更新 CHANGELOG：`docs/CHANGELOG.md`
5. 如果涉及服务层，更新服务文档

### 场景 3: 修改数据库表结构

| 文档路径 | 更新内容 | 优先级 |
|---------|---------|--------|
| `docs/CHANGELOG.md` | 记录表结构变更 | 🔴 必须 |
| `docs/database/schema/tables/[table].md` | 更新表结构文档 | 🔴 必须 |
| `docs/database/schema/erd.md` | 更新 ER 图 | 🔴 必须 |
| `docs/database/migrations/history.md` | 记录迁移历史 | 🔴 必须 |
| `docs/api/models/[Model].md` | 如果影响 API，更新模型 | 🔴 必须 |
| `docs/api/endpoints/[resource].md` | 如果影响接口，更新接口 | 🔴 必须 |

**操作步骤**:
1. 更新表结构文档：`docs/database/schema/tables/[table].md`
2. 更新 ER 图：`docs/database/schema/erd.md`
3. 记录迁移历史：`docs/database/migrations/history.md`
4. 更新 CHANGELOG：`docs/CHANGELOG.md`
5. 如果影响 API，同步更新相关文档

### 场景 4: 添加新后端服务

| 文档路径 | 更新内容 | 优先级 |
|---------|---------|--------|
| `docs/CHANGELOG.md` | 添加新服务条目 | 🔴 必须 |
| `docs/backend/architecture/overview.md` | 更新架构图 | 🔴 必须 |
| `docs/backend/services/[service].md` | 创建服务文档 | 🔴 必须 |
| `docs/backend/services/README.md` | 更新服务列表 | 🔴 必须 |
| `docs/api/README.md` | 如果暴露 API，更新接口列表 | 🔴 必须 |

**操作步骤**:
1. 创建服务文档：`docs/backend/services/[service].md`
2. 更新架构概述：`docs/backend/architecture/overview.md`
3. 更新服务索引：`docs/backend/services/README.md`
4. 更新 CHANGELOG：`docs/CHANGELOG.md`
5. 如果暴露 API，更新 API 文档

---

## 📝 文档命名规范

### 文件命名

```
格式: <主题>.md（简洁描述性名称）

✅ 推荐:
- architecture.md（架构设计）
- development.md（开发指南）
- FloatingOrb.md（组件名）
- memory-service.md（服务名，kebab-case）

❌ 避免:
- architecture-v2.md（不要版本号）
- new-architecture.md（不要 new 前缀）
- 前端架构.md（不要中文文件名）
```

### 目录命名

```
✅ 推荐:
- docs/frontend/
- docs/backend/
- docs/api/
- docs/database/

❌ 避免:
- docs/frontend-v2/
- docs/backend-new/
```

---

## 🛠️ 核心功能

### 1. 文档查找（必须先执行）

```typescript
// 查找前端文档
Glob: "docs/frontend/**/*.md"
Grep: pattern="悬浮球" path="docs/frontend/"

// 查找 API 文档
Glob: "docs/api/endpoints/*.md"

// 查找数据库文档
Glob: "docs/database/schema/tables/*.md"
```

### 2. 文档更新（而非新建）

**更新策略**:
- ✅ 使用 `Edit` 工具精确替换
- ✅ 保留文档整体结构
- ✅ 更新 Frontmatter
- ✅ 记录到 CHANGELOG

### 3. CHANGELOG 维护

```markdown
# docs/CHANGELOG.md

## [Unreleased]

### Added
- 前端: 新增悬浮球组件文档 (2026-02-04)
- API: 新增通知接口文档 (2026-02-04)

### Changed
- 前端: 更新架构设计，改为悬浮球交互 (2026-02-04)
- 数据库: 更新 screenshots 表结构 (2026-02-04)

### Fixed
- 修正组件文档中的示例代码错误 (2026-02-04)
```

---

## 🎯 使用示例

### 示例 1: 更新前端架构文档

```
用户: "更新前端架构文档，改为悬浮球交互设计"

Skill 执行:
1. Glob 查找: docs/frontend/architecture.md
2. Read 读取现有内容
3. Edit 更新 "## UI 设计架构" 章节
4. Edit 更新 Frontmatter (version, updated)
5. Edit 更新 docs/CHANGELOG.md:
   - Changed: 前端架构更新为悬浮球交互设计
6. 完成（未创建新文件）
```

### 示例 2: 添加新 API 接口文档

```
用户: "添加通知接口文档"

Skill 执行:
1. 创建 docs/api/endpoints/notification.md
2. 创建 docs/api/models/Notification.md
3. Edit 更新 docs/api/README.md（添加接口列表）
4. Edit 更新 docs/backend/services/notification-service.md
5. Edit 更新 docs/CHANGELOG.md:
   - Added: API 通知接口文档
6. 完成
```

### 示例 3: 更新数据库表结构

```
用户: "更新数据库文档，screenshots 表新增 tags 字段"

Skill 执行:
1. Glob 查找: docs/database/schema/tables/screenshots.md
2. Read 读取现有表结构
3. Edit 在字段列表中新增 tags 字段定义
4. Edit 更新 docs/database/schema/erd.md（如需更新 ER 图）
5. Edit 更新 docs/database/migrations/history.md
6. Edit 更新 docs/CHANGELOG.md:
   - Changed: screenshots 表新增 tags 字段
7. 完成
```

---

## ✅ 文档质量检查

### 更新前检查
- [ ] 已使用 Glob 查找现有文档
- [ ] 已读取现有文档内容
- [ ] 确认要更新的具体章节

### 更新后检查
- [ ] 使用 Edit 工具更新原文件（而非新建）
- [ ] 更新了 Frontmatter
- [ ] 更新了 docs/CHANGELOG.md
- [ ] 如有必要，更新了 README 索引
- [ ] 文档结构清晰，无重复内容

---

## 📖 最佳实践

### 1. 更新而非新建
✅ 用户要求更新 → 查找现有文档 → 增量更新 → 记录 CHANGELOG

### 2. 文档查找优先
每次更新前：Glob 查找 → Read 读取 → Edit 更新

### 3. 保持文档唯一性
- 同一主题只有一个文档
- 通过 Git 管理历史
- 不需要 v2、v3 版本文件

### 4. 及时更新 CHANGELOG
每次文档变更都记录到 `docs/CHANGELOG.md`

### 5. 遵循更新清单
根据场景（新组件/新 API/数据库变更/新服务）执行对应的更新清单

---

## 🚀 快速参考

### 文档位置速查

| 文档类型 | 文件路径 | 更新时机 |
|---------|---------|---------|
| 前端架构 | `docs/frontend/architecture.md` | 前端架构变更 |
| 前端组件 | `docs/frontend/components/[Component].md` | 新增/更新组件 |
| API 接口 | `docs/api/endpoints/[resource].md` | 新增/更新接口 |
| 数据模型 | `docs/api/models/[Model].md` | 新增/更新模型 |
| 数据库表 | `docs/database/schema/tables/[table].md` | 数据库变更 |
| 后端服务 | `docs/backend/services/[service].md` | 新增/更新服务 |
| 测试文档 | `docs/testing/[test-type]-testing.md` | 测试策略变更 |

---

**Skill 版本**: v4.0
**最后更新**: 2026-02-04
**核心变更**:
- **统一文档目录** - 所有文档集中在 `/docs/` 下管理
- **前端/后端/API/数据库分类** - 清晰的文档层级
- **新需求文档更新清单** - 4 大场景的标准化更新流程
- **强制优先更新现有文档** - 避免创建重复文档

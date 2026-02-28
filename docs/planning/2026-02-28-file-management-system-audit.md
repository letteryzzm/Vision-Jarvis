# 文件管理系统审计与实施计划

> **日期**: 2026-02-28
> **范围**: 存储管理 + 记忆系统前端 + 文档同步
> **分支**: memory

---

## 一、文档与代码差异审计

### 1.1 storage-service.md vs storage/mod.rs

| 差异点 | 文档 | 代码 | 严重度 |
|--------|------|------|--------|
| FolderType 枚举 | 5 种（缺 Recordings） | 6 种（含 Recordings） | HIGH |
| StorageInfo 字段 | 缺 `recordings_bytes` | 含 `recordings_bytes` | HIGH |
| 行数 | 350 lines | 345 lines（含测试） | LOW |
| list_files 遍历 | 未说明 | 仅顶层目录，不递归子目录 | MEDIUM（Recordings 按日期分子目录，无法列出） |

### 1.2 CODEMAP.md vs 实际代码

| 差异点 | 文档 | 代码 | 严重度 |
|--------|------|------|--------|
| 截图分析间隔 | "每5分钟" | 实际 90 秒（pipeline.rs） | HIGH |
| `app_usage` 表 | 列出 | 不存在 | HIGH |
| `short_term_memory` 表 | 列出 | 存在但已被 V3 替代 | MEDIUM |
| `notifications` 表 | 列出 | 待确认 | LOW |
| V2 遗留文件 | 列出 short_term.rs / long_term.rs / scheduler.rs | 可能仍存在但无功能 | MEDIUM |

### 1.3 database/README.md vs migrations.rs

| 差异点 | 文档 | 代码 | 严重度 |
|--------|------|------|--------|
| V2 表名 | `activity_sessions` | `activities` | HIGH |
| V2 表名 | `tracked_files` | `memory_files` | HIGH |
| V2 screenshot_analyses 字段 | 列出 9 个 V2 新增字段 | 未迁移（需 V8+） | INFO（已标注） |

### 1.4 MASTER_PLAN.md vs 实际项目

| 差异点 | 说明 | 严重度 |
|--------|------|--------|
| 整体架构 | MASTER_PLAN 描述 JSONL 存储 + Claude-only，实际已是 SQLite + 多 Provider | CRITICAL |
| 项目结构 | 文档中的目录结构与实际差异巨大 | CRITICAL |
| 任务状态 | 所有 TASK 标为 TODO，实际大部分已完成 | HIGH |
| 技术栈 | 缺少 FFmpeg 录制、多 AI Provider、管道调度等核心组件 | CRITICAL |

**结论**: MASTER_PLAN.md 完全过时，需要重写或归档。

### 1.5 前端 MemoryPage.tsx 功能缺口

| 缺口 | 当前状态 | 说明 |
|------|---------|------|
| 短期记忆列表 | **硬编码** | 未连接后端 `get_activities` API |
| 搜索功能 | **仅 UI** | 未连接 `search_memories` API |
| 日期选择器 | **仅 UI** | 未实现日期切换加载活动 |
| 文件存储设置按钮 | **无功能** | 点击无响应 |
| 活动详情 | **缺失** | 无法查看活动详情 |
| 项目/习惯/总结展示 | **缺失** | 后端有 API，前端未接入 |

### 1.6 FileBrowser.tsx 功能缺口

| 缺口 | 说明 |
|------|------|
| Recordings 标签页 | `list_files` 不递归子目录，Recordings 内容无法显示 |
| 文件删除 | 无单个文件删除按钮 |
| 录制存储统计 | `StorageInfo` 有 `recordings_bytes` 但前端未展示 |

---

## 二、实施路径

### 路径 A：文档修复优先（低风险，文档准确性）

仅修复文档与代码的差异，不改代码。

**预计工作量**: 2-3 小时

### 路径 B：前端功能补全（中风险，用户可见价值最大）

将 MemoryPage 从静态 mockup 升级为功能完整的记忆浏览器。

**预计工作量**: 6-8 小时

### 路径 C：全栈补齐（高风险，最完整）

文档修复 + 前端功能补全 + 后端 bug 修复 + 遗留代码清理。

**预计工作量**: 10-15 小时

---

## 三、推荐实施计划（路径 C 分阶段）

### Phase 1: 文档修复（1-2 小时）

1. **更新 storage-service.md**
   - 添加 `Recordings` FolderType
   - 添加 `recordings_bytes` 字段
   - 修正行数

2. **更新 CODEMAP.md**
   - 修正截图分析间隔为 90 秒
   - 移除不存在的 `app_usage` 表
   - 标注 V2 遗留文件状态
   - 更新前端组件列表

3. **更新 database/README.md**
   - 修正 V2 表名（`activities`, `memory_files`）
   - 标注 V2 字段迁移状态

4. **归档 MASTER_PLAN.md**
   - 移至 `docs/archive/` 或添加明显过时标记
   - 可选：重写为反映当前架构的版本

### Phase 2: 后端修复（1-2 小时）

5. **修复 `list_files` 递归问题**
   - `storage/mod.rs`: `list_files` 改为递归遍历子目录
   - 影响：Recordings 文件夹（`recordings/YYYYMMDD/period/*.mp4`）可正常列出

6. **清理 V2 遗留代码**（可选）
   - 确认 `short_term.rs`, `long_term.rs`, `scheduler.rs` 无引用
   - 删除或标注 `#[deprecated]`

### Phase 3: MemoryPage 功能接入（3-4 小时）

7. **接入活动列表**
   - 日期选择器联动 `get_activities(date)` API
   - 替换硬编码短期记忆为真实活动数据
   - 按时段（早晨/下午/晚上）分组显示

8. **接入搜索功能**
   - 搜索框连接 `search_memories(query)` API
   - 实现 debounce 搜索
   - 显示搜索结果列表

9. **添加活动详情视图**
   - 点击活动项 → 显示详情面板
   - 展示 `screenshot_analyses` 列表
   - 显示标签、应用、时长等信息

10. **添加统计总览**
    - 接入 `get_recording_stats()` API
    - 在页面顶部显示总录制/分析/活动/项目/习惯计数

### Phase 4: FileBrowser 增强（1-2 小时）

11. **添加 Recordings 标签页**
    - 新增 `Recordings` 到 `folderTabs`
    - 利用修复后的递归 `list_files`

12. **添加录制存储统计**
    - 在 Storage Overview 中展示 `recordings_bytes`

13. **添加单文件删除功能**（可选）
    - 每个文件行增加删除按钮 + 确认弹窗

### Phase 5: 验证与测试（1-2 小时）

14. **后端单元测试**
    - 为递归 `list_files` 添加测试
    - 确保现有 6 个存储测试通过

15. **前端功能验证**
    - 验证活动列表正确加载
    - 验证搜索功能工作
    - 验证日期切换正确

---

## 四、优先级与依赖关系

```
Phase 1 (文档修复) ── 独立，无依赖
         ↓
Phase 2 (后端修复) ← Phase 4 依赖此修复
         ↓
Phase 3 (MemoryPage) ── 独立于 Phase 2
         ↓
Phase 4 (FileBrowser) ← 依赖 Phase 2
         ↓
Phase 5 (验证)
```

Phase 1 和 Phase 3 可**并行**执行。
Phase 2 和 Phase 3 可**并行**执行。
Phase 4 必须等 Phase 2 完成。

---

## 五、风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 递归 list_files 可能返回大量文件 | 性能 | 保持 limit 参数，前端分页 |
| V2 遗留代码有隐藏引用 | 编译失败 | 先 grep 确认引用关系再删 |
| MemoryPage 活动数据为空 | 用户困惑 | 添加空状态提示"开启记忆功能后将自动记录" |
| MASTER_PLAN 重写工作量大 | 时间 | 先加过时标记，不阻塞其他工作 |

---

**等待确认**: 是否按此计划执行？优先执行哪些 Phase？

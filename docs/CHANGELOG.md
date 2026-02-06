# Vision-Jarvis 变更日志

本文档记录 Vision-Jarvis 项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)。

---

## [Unreleased]

### Added - 2026-02-06

**Phase 5: Tauri Commands - 前后端通信层**

- **API 接口** (commands/):
  - `health_check()` - 健康检查接口
  - `capture_screenshot()` - 捕获屏幕截图
  - `get_screenshots(limit)` - 获取截图列表
  - `delete_screenshot(id)` - 删除指定截图
  - `search_memories(query, limit)` - 搜索记忆（支持SQL转义防注入）
  - `get_memories_by_date(date)` - 获取指定日期记忆（含日期验证）
  - `generate_memory(date)` - 生成指定日期的短期记忆
  - `get_pending_notifications()` - 获取待处理通知
  - `dismiss_notification(id)` - 关闭通知
  - `get_notification_history(limit)` - 获取通知历史
  - `get_settings()` - 获取应用设置
  - `update_settings(settings)` - 更新应用设置
  - `reset_settings()` - 重置设置为默认值

- **通用响应结构** (`ApiResponse<T>`):
  - `success: bool` - 操作是否成功
  - `data: Option<T>` - 返回数据
  - `error: Option<String>` - 错误信息

- **安全增强**:
  - SQL 注入防护：search_memories() 使用 ESCAPE 子句转义通配符
  - 输入验证：get_memories_by_date() 验证日期格式（YYYY-MM-DD）
  - 错误处理改进：generate_memory() 收集并返回所有保存错误

### Changed - 2026-02-06

- **SettingsManager 重构**:
  - 从 `settings: AppSettings` 改为 `settings: Arc<Mutex<AppSettings>>`
  - 支持线程安全的内部可变性
  - `get()` 方法返回克隆而非引用
  - `update()` 方法使用 `&self` 而非 `&mut self`
  - 测试更新以适配新 API

- **AppState 初始化**:
  - ScreenCapture 创建时传入 storage_path
  - 从 SettingsManager 获取存储路径

### Fixed - 2026-02-06

- 修复 capture_screenshot() 中的文件写入逻辑（ScreenCapture 已自动保存）
- 清理未使用的导入：tauri::State, VectorStore, Serialize, Deserialize, NotificationPriority
- 修复编译警告和类型错误

### Technical Details

**测试覆盖率**: 60 个测试全部通过
- 核心 API 响应结构测试
- 健康检查接口测试
- 所有既有模块测试保持通过

**架构改进**:
- 统一错误处理模式（Result -> ApiResponse 转换）
- 参数化 SQL 查询防止注入
- 线程安全的状态管理（Arc + Mutex）

---

## [0.1.0] - 2026-02-04

### Added

**Phase 1-3: 核心后端架构**

- 数据库模块 (db/)
- 设置管理 (settings/)
- 屏幕捕获 (capture/)
- AI 服务 (ai/)
- 记忆系统 (memory/)
- 通知系统 (notification/)

详细变更请参考 git commit 历史。

---

## 版本说明

- **[Unreleased]**: 开发中的功能
- **[0.1.0]**: 首个开发版本

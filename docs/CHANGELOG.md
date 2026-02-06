# Vision-Jarvis 变更日志

本文档记录 Vision-Jarvis 项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)。

---

## [Unreleased]

### Added - 2026-02-06

**Phase 6 & 7: File Management and API Management Systems**

- **Backend - Storage Module** (`src-tauri/src/storage/mod.rs` - 350 lines):
  - 文件存储管理模块，支持5种文件夹类型管理
  - FolderType 枚举: `Screenshots`, `Memories`, `Database`, `Logs`, `Temp`
  - StorageManager 提供完整的文件管理功能
  - 安全的路径遍历检查，防止删除存储目录外的文件
  - 存储信息查询（总使用量、各文件夹使用量、文件总数）
  - 文件列表功能（支持限制返回数量，按修改时间倒序）
  - 旧文件清理功能（按天数清理）
  - 单文件删除功能（带安全验证）

- **Backend - AI Providers Module** (`src-tauri/src/ai/providers.rs` - 480 lines):
  - 多AI提供商配置系统
  - 支持5种AI提供商: `OpenAI`, `Anthropic`, `Google`, `Local` (Ollama), `Custom`
  - API密钥使用 `secrecy` crate 进行内存保护（SecretString）
  - AIProviderConfig 包含: api_key, base_url, model, enabled 状态
  - AIConfigCollection 管理所有提供商配置和活动提供商
  - 配置验证功能（API密钥必填性、base_url格式）
  - 获取可用提供商列表（enabled=true）
  - 更新API密钥和提供商配置
  - Provider默认配置（各提供商的默认base_url和model）

- **Backend - Storage Commands** (`src-tauri/src/commands/storage.rs` - 133 lines):
  - `get_storage_info()` - 获取存储信息（总使用量、各文件夹使用量、文件总数、根路径）
  - `list_files(folder_type, limit)` - 列出指定文件夹中的文件（支持限制数量）
  - `cleanup_old_files(folder_type, days)` - 清理指定天数之前的旧文件
  - `open_folder(folder_type)` - 在系统文件管理器中打开文件夹
  - `delete_file(file_path)` - 删除单个文件（带路径安全检查）

- **Backend - AI Config Commands** (`src-tauri/src/commands/ai_config.rs` - 319 lines):
  - `get_ai_config_summary()` - 获取AI配置摘要
  - `get_ai_config()` - 获取完整的AI配置
  - `update_ai_api_key(provider, api_key)` - 更新AI提供商的API密钥
  - `update_ai_provider_config(provider, config)` - 更新AI提供商配置（带provider验证）
  - `set_active_ai_provider(provider)` - 设置活动的AI提供商
  - `test_ai_connection(provider)` - 测试AI提供商连接（支持所有5种provider）
  - `get_available_ai_providers()` - 获取可用的提供商列表
  - `reset_ai_config()` - 重置AI配置为默认值
  - 内置连接测试函数: test_openai_connection, test_anthropic_connection, test_google_connection, test_local_connection, test_custom_connection

- **Frontend - Pages**:
  - `/files.astro` - 文件管理页面（存储概览、文件浏览器、清理功能）
  - `/api-settings.astro` - API配置页面（提供商选择、API密钥配置、连接测试）

- **Testing**:
  - 新增 storage 模块测试（6个测试）
  - 新增 ai_config 模块测试（2个测试）
  - 新增 ai providers 模块测试（7个测试）
  - 总测试数: 76个测试通过（从60个增加到76个）

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

- **Commands Module**:
  - 新增 `storage` 子模块用于文件管理
  - 新增 `ai_config` 子模块用于AI配置管理
  - 导出 `AIConfigState` 以供 Tauri 状态管理
  - lib.rs 中注册所有新的 Tauri commands（storage和ai_config相关）

- **Application State**:
  - 添加 `AIConfigState` 到应用状态管理
  - 支持多AI提供商配置持久化
  - 线程安全的配置访问（Arc\<Mutex\<AIConfigCollection\>\>）

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

- **AI Config Commands Security Fix**:
  - 添加 provider 参数验证到 `update_ai_provider_config` 函数
  - 确保传入的 provider 参数与 config_update.provider 匹配
  - 防止配置混淆攻击

- 修复 capture_screenshot() 中的文件写入逻辑（ScreenCapture 已自动保存）
- 清理未使用的导入：tauri::State, VectorStore, Serialize, Deserialize, NotificationPriority
- 修复编译警告和类型错误

### Technical Details

**Security**:
- **Storage Security**:
  - 实现路径遍历保护：验证文件路径必须在存储根目录下（`path.starts_with(&self.root_path)`）
  - 防止删除系统关键文件和存储目录外的文件
  - 文件删除前检查文件是否存在

- **AI Config Security**:
  - 使用 `secrecy` crate 保护API密钥在内存中的存储（SecretString类型）
  - API密钥不会以明文形式记录到日志
  - Provider配置更新时验证provider参数匹配（防止配置混淆）
  - 配置验证：API密钥必填性检查、base_url格式验证

**测试覆盖率**: 76 个测试全部通过（从60个增加到76个）
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

# Vision-Jarvis 后端 API 设计文档

> **版本**: 1.0.0
> **更新日期**: 2026-02-09
> **架构**: Tauri v2 IPC Commands + Rust Backend

---

## 目录

1. [概述](#概述)
2. [技术栈](#技术栈)
3. [接口约定](#接口约定)
4. [核心模块](#核心模块)
   - [设置管理模块](#1-设置管理模块)
   - [文件系统模块](#2-文件系统模块)
   - [窗口管理模块](#3-窗口管理模块)
   - [自动启动模块](#4-自动启动模块)
   - [截图与记忆模块](#5-截图与记忆模块)
   - [提醒系统模块](#6-提醒系统模块)
5. [数据模型](#数据模型)
6. [错误处理](#错误处理)
7. [安全性](#安全性)
8. [性能优化](#性能优化)

---

## 概述

Vision-Jarvis 后端采用 **Tauri v2 架构**，使用 Rust 实现高性能的本地 API ���务。前端通过 Tauri IPC 机制调用后端命令，无需传统 HTTP 服务器。

### 主要特点

- **零网络开销**: 进程内通信，无 HTTP 延迟
- **类型安全**: Rust 类型系统保证数据安全
- **跨平台**: macOS/Windows/Linux 统一接口
- **高性能**: 原生性能，无 V8 开销

---

## 技术栈

| 组件 | 技术 | 版本 |
|------|------|------|
| 运行时 | Tauri | v2.1+ |
| 后端语言 | Rust | 1.75+ |
| 序列化 | serde_json | 1.0 |
| 文件存储 | tauri-plugin-store | 2.1 |
| 文件对话框 | tauri-plugin-dialog | 2.1 |
| 自动启动 | tauri-plugin-autostart | 2.0 |
| 截图 | screenshots | 0.7 |

---

## 接口约定

### 调用方式

前端通过 `@tauri-apps/api/core` 的 `invoke` 函数调用：

```typescript
import { invoke } from '@tauri-apps/api/core'

// 调用示例
const result = await invoke<Settings>('get_settings')
```

### 命名规范

- **命令名称**: 小写蛇形命名法 (snake_case)
- **参数/返回值**: 驼峰命名法 (camelCase)
- **错误信息**: 中文或英文，包含错误码

### 返回值格式

所有命令遵循统一的返回格式：

```typescript
// 成功返回
Result<T, String>

// Rust 端
Ok(data) // 成功
Err("错误信息".to_string()) // ��败

// TypeScript 端
try {
  const data = await invoke<T>('command_name', { params })
} catch (error) {
  // error 是字符串类型的错误信息
  console.error(error)
}
```

---

## 核心模块

---

## 1. 设置管理模块

### 1.1 获取所有设置

**命令**: `get_settings`

**描述**: 获取应用的所有配置信息

**参数**: 无

**返回值**:

```typescript
interface Settings {
  memory: MemorySettings
  popup: PopupSettings
  api: ApiSettings
  ui: UiSettings
}

interface MemorySettings {
  screenshotFrequency: number // 截图频率（秒），范围 1-15
  storageLocation: string // 记忆文件存储位置
  maxSize: number // 最大容量（MB）
  autoDelete: boolean // 是否自动删除
}

interface PopupSettings {
  startup: {
    enabled: boolean // 启动时弹出
    autoStart: boolean // 开机自启
    text: string // 弹出文本
  }
  timedReminder: {
    enabled: boolean // 定时提醒开关
    startTime: string // 开始时间 "HH:MM"
    endTime: string // 结束时间 "HH:MM"
    interval: number // 间隔（分钟）
  }
  idleReminder: {
    enabled: boolean // 空闲提醒开关
    idleThreshold: number // 空闲阈值（分钟）
    useAiSuggestion: boolean // 使用AI智能建议
  }
}

interface ApiSettings {
  providers: ApiProvider[]
}

interface ApiProvider {
  id: string // 唯一标识
  type: 'official' | 'custom'
  name: string // 提供商名称（如 "Claude"）
  key: string // API Key（加密存储）
  model: string // 模型名称
  endpoint?: string // 自定义API地址（仅 custom 类型）
  enabled: boolean // 是否启用
}

interface UiSettings {
  orbPosition: { x: number; y: number } // 悬浮球位置
  theme: 'light' | 'dark' | 'auto' // 主题
  language: 'zh-CN' | 'en-US' // 语言
}
```

**示例调用**:

```typescript
const settings = await invoke<Settings>('get_settings')
console.log(settings.memory.screenshotFrequency) // 5
```

**错误**:
- `"配置文件读取失败: {原因}"` - 文件损坏或权限问题
- `"配置文件格式错误"` - JSON 解析失败

---

### 1.2 更新设置

**命令**: `update_settings`

**描述**: 更新部分或全部设置（增量更新）

**参数**:

```typescript
interface UpdateSettingsParams {
  memory?: Partial<MemorySettings>
  popup?: {
    startup?: Partial<PopupSettings['startup']>
    timedReminder?: Partial<PopupSettings['timedReminder']>
    idleReminder?: Partial<PopupSettings['idleReminder']>
  }
  api?: {
    providers?: ApiProvider[]
  }
  ui?: Partial<UiSettings>
}
```

**返回值**: `Settings` (更新后的完整设置)

**示例调用**:

```typescript
// 仅更新截图频率
await invoke<Settings>('update_settings', {
  memory: {
    screenshotFrequency: 10
  }
})

// 更新多个设置
await invoke<Settings>('update_settings', {
  memory: {
    screenshotFrequency: 5,
    maxSize: 2048
  },
  popup: {
    startup: {
      enabled: true,
      text: "今天要做什么有意义的事情？"
    }
  }
})
```

**错误**:
- `"参数验证失败: {字段名}"` - 参数不符合约束
- `"配置保存失败: {原因}"` - 磁盘写入失败

**验证规则**:
- `screenshotFrequency`: 1-15 之间
- `maxSize`: > 0
- `startTime/endTime`: 格式 "HH:MM"，且 startTime < endTime
- `interval`: > 0
- `idleThreshold`: > 0

---

### 1.3 重置设置

**命令**: `reset_settings`

**描述**: 恢复默认设置

**参数**:

```typescript
interface ResetSettingsParams {
  section?: 'memory' | 'popup' | 'api' | 'ui' | 'all' // 要重置的部分
}
```

**返回值**: `Settings`

**示例调用**:

```typescript
// 重置所有设置
await invoke<Settings>('reset_settings', { section: 'all' })

// 仅重置记忆设置
await invoke<Settings>('reset_settings', { section: 'memory' })
```

**默认值**:

```typescript
const DEFAULT_SETTINGS: Settings = {
  memory: {
    screenshotFrequency: 5,
    storageLocation: '~/.vision-jarvis/memory',
    maxSize: 1024,
    autoDelete: true
  },
  popup: {
    startup: {
      enabled: false,
      autoStart: false,
      text: "If today were the last day of my life, would I want to do what I am about to do today?"
    },
    timedReminder: {
      enabled: false,
      startTime: "09:00",
      endTime: "21:00",
      interval: 30
    },
    idleReminder: {
      enabled: false,
      idleThreshold: 15,
      useAiSuggestion: true
    }
  },
  api: {
    providers: []
  },
  ui: {
    orbPosition: { x: -60, y: 20 }, // 右上角
    theme: 'auto',
    language: 'zh-CN'
  }
}
```

---

## 2. 文件系统模块

### 2.1 选择记忆文件夹

**命令**: `pick_memory_folder`

**描述**: 打开原生文件选择对话框，选择记忆文件存储位置

**参数**: 无

**返回值**: `string` (选中的文件夹路径)

**示例调用**:

```typescript
try {
  const path = await invoke<string>('pick_memory_folder')
  console.log('选中的路径:', path) // "/Users/xxx/Documents/vision-memory"
} catch (error) {
  // 用户取消选择
  console.log('用户取消选择')
}
```

**错误**:
- `"用户取消选择"` - 用户点击了取消按钮
- `"无权限访问该路径"` - 权限问题

---

### 2.2 获取记忆文件夹信息

**命令**: `get_memory_folder_info`

**描述**: 获取当前记忆文件夹的磁盘使用情况

**参数**:

```typescript
interface GetMemoryFolderInfoParams {
  path: string // 文件夹路径
}
```

**返回值**:

```typescript
interface MemoryFolderInfo {
  path: string // 文件夹路径
  totalSize: number // 总大小（字节）
  totalSizeMB: number // 总大小（MB）
  fileCount: number // 文件数量
  oldestFile?: {
    path: string
    createdAt: string // ISO 8601
  }
  newestFile?: {
    path: string
    createdAt: string
  }
}
```

**示例调用**:

```typescript
const info = await invoke<MemoryFolderInfo>('get_memory_folder_info', {
  path: '/Users/xxx/.vision-jarvis/memory'
})

console.log(`当前使用 ${info.totalSizeMB} MB，共 ${info.fileCount} 个文件`)
```

**错误**:
- `"文件夹不存在"` - 路径无效
- `"无权限访问"` - 权限问题

---

### 2.3 清理旧记忆文件

**命令**: `cleanup_memory_files`

**描述**: 根据设置的最大容量，自动删除最旧的记忆文件

**参数**:

```typescript
interface CleanupMemoryFilesParams {
  path: string // 文件夹路径
  maxSizeMB: number // 最大容量（MB）
}
```

**返回值**:

```typescript
interface CleanupResult {
  deletedCount: number // 删除的文件数量
  freedSizeMB: number // 释放的空间（MB）
  remainingCount: number // 剩余文件数量
  remainingSizeMB: number // 剩余大小（MB）
}
```

**示例调用**:

```typescript
const result = await invoke<CleanupResult>('cleanup_memory_files', {
  path: '/Users/xxx/.vision-jarvis/memory',
  maxSizeMB: 1024
})

console.log(`删除了 ${result.deletedCount} 个文件，释放了 ${result.freedSizeMB} MB`)
```

**错误**:
- `"清理失败: {原因}"` - 文件删除失败

---

## 3. 窗口管理模块

### 3.1 设置窗口位置

**命令**: `set_window_position`

**描述**: 设置悬浮窗位置

**参数**:

```typescript
interface SetWindowPositionParams {
  x: number // X 坐标（相对于屏幕）
  y: number // Y 坐标
}
```

**返回值**: `void`

**示例调用**:

```typescript
// 移动到右上角
await invoke('set_window_position', { x: -60, y: 20 })
```

---

### 3.2 获取屏幕信息

**命令**: `get_screen_info`

**描述**: 获取主屏幕和所有屏幕的信息

**参数**: 无

**返回值**:

```typescript
interface ScreenInfo {
  primary: {
    width: number
    height: number
    scaleFactor: number // DPI 缩放
  }
  all: Array<{
    id: number
    x: number
    y: number
    width: number
    height: number
    scaleFactor: number
    isPrimary: boolean
  }>
}
```

**示例调用**:

```typescript
const screenInfo = await invoke<ScreenInfo>('get_screen_info')
console.log(`主屏幕: ${screenInfo.primary.width}x${screenInfo.primary.height}`)
```

---

### 3.3 检测窗口是否在屏幕内

**命令**: `is_position_on_screen`

**描述**: 检查给定坐标是否在任意屏幕内

**参数**:

```typescript
interface IsPositionOnScreenParams {
  x: number
  y: number
  width: number
  height: number
}
```

**返回值**: `boolean`

**示例调用**:

```typescript
const isOnScreen = await invoke<boolean>('is_position_on_screen', {
  x: 1920,
  y: 0,
  width: 400,
  height: 500
})
```

---

### 3.4 设置窗口大小

**命令**: `set_window_size`

**描述**: 动态调整窗口大小（用于状态切换）

**参数**:

```typescript
interface SetWindowSizeParams {
  width: number
  height: number
  animate?: boolean // 是否使用动画过渡
}
```

**返回值**: `void`

**示例调用**:

```typescript
// 切换到 Asker 状态（400x500）
await invoke('set_window_size', {
  width: 400,
  height: 500,
  animate: true
})
```

---

## 4. 自动启动模块

### 4.1 启用开机自启

**命令**: `enable_auto_start`

**描述**: 将应用添加到系统启动项

**参数**: 无

**返回值**: `void`

**示例调用**:

```typescript
await invoke('enable_auto_start')
```

**错误**:
- `"权限不足"` - macOS 需要用户授权
- `"系统不支持"` - Linux 某些发行版

**平台差异**:
- **macOS**: 添加到 `~/Library/LaunchAgents/`
- **Windows**: 注册表 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
- **Linux**: `~/.config/autostart/`

---

### 4.2 禁用开机自启

**命令**: `disable_auto_start`

**参数**: 无

**返回值**: `void`

**示例调用**:

```typescript
await invoke('disable_auto_start')
```

---

### 4.3 检查自启状态

**命令**: `is_auto_start_enabled`

**参数**: 无

**返回值**: `boolean`

**示例调用**:

```typescript
const enabled = await invoke<boolean>('is_auto_start_enabled')
console.log('开机自启:', enabled)
```

---

## 5. 截图与记忆模块

### 5.1 开始截图任务

**命令**: `start_screenshot_task`

**描述**: 启动后台截图任务，按设置的频率自动截图

**参数**:

```typescript
interface StartScreenshotTaskParams {
  frequency: number // 截图间隔（秒）
  savePath: string // 保存路径
}
```

**返回值**: `string` (任务 ID)

**示例调用**:

```typescript
const taskId = await invoke<string>('start_screenshot_task', {
  frequency: 5,
  savePath: '/Users/xxx/.vision-jarvis/memory'
})
```

**错误**:
- `"截图权限未授予"` - macOS 需要屏幕录制权限
- `"任务已在运行"` - 重复启动

---

### 5.2 停止截图任务

**命令**: `stop_screenshot_task`

**参数**:

```typescript
interface StopScreenshotTaskParams {
  taskId: string
}
```

**返回值**: `void`

**示例调用**:

```typescript
await invoke('stop_screenshot_task', { taskId: 'task-123' })
```

---

### 5.3 获取最新截图

**命令**: `get_latest_screenshot`

**描述**: 获取最新的截图文件路径

**参数**: 无

**返回值**:

```typescript
interface LatestScreenshot {
  path: string
  timestamp: string // ISO 8601
  sizeMB: number
}
```

**示例调用**:

```typescript
const latest = await invoke<LatestScreenshot>('get_latest_screenshot')
console.log('最新截图:', latest.path)
```

**错误**:
- `"无截图记录"` - 还未截图

---

### 5.4 立即截图

**命令**: `take_screenshot_now`

**描述**: 立即执行一次截图

**参数**:

```typescript
interface TakeScreenshotNowParams {
  savePath: string
}
```

**返回值**: `string` (截图文件路径)

**示例调用**:

```typescript
const path = await invoke<string>('take_screenshot_now', {
  savePath: '/Users/xxx/.vision-jarvis/memory'
})
```

---

## 6. 提醒系统模块

### 6.1 启动定时提醒

**命令**: `start_timed_reminder`

**描述**: 根据时间范围和间隔启动定时提醒

**参数**:

```typescript
interface StartTimedReminderParams {
  startTime: string // "HH:MM"
  endTime: string // "HH:MM"
  interval: number // 间隔（分钟）
}
```

**返回值**: `string` (提醒任务 ID)

**示例调用**:

```typescript
const reminderId = await invoke<string>('start_timed_reminder', {
  startTime: "09:00",
  endTime: "21:00",
  interval: 30
})
```

---

### 6.2 停止定时提醒

**命令**: `stop_timed_reminder`

**参数**:

```typescript
interface StopTimedReminderParams {
  reminderId: string
}
```

**返回值**: `void`

---

### 6.3 启动空闲检测

**命令**: `start_idle_detection`

**描述**: 启动空闲检测，超过阈值后触发提醒

**参数**:

```typescript
interface StartIdleDetectionParams {
  idleThreshold: number // 空闲阈值（分钟）
}
```

**返回值**: `string` (检测任务 ID)

**示例调用**:

```typescript
const detectionId = await invoke<string>('start_idle_detection', {
  idleThreshold: 15
})
```

---

### 6.4 停止空闲检测

**命令**: `stop_idle_detection`

**参数**:

```typescript
interface StopIdleDetectionParams {
  detectionId: string
}
```

**返回值**: `void`

---

### 6.5 获取当前空闲时长

**命令**: `get_idle_duration`

**描述**: 获取用户当前已空闲的时长

**参数**: 无

**返回值**: `number` (空闲分钟数)

**示例调用**:

```typescript
const idleMinutes = await invoke<number>('get_idle_duration')
console.log(`已空闲 ${idleMinutes} 分钟`)
```

**平台实现**:
- **macOS**: `CGEventSource` API
- **Windows**: `GetLastInputInfo`
- **Linux**: X11 Screensaver Extension

---

## 7. API 提供商管理

### 7.1 测试 API 连接

**命令**: `test_api_connection`

**描述**: 测试 API 提供商的连接性和认证

**参数**:

```typescript
interface TestApiConnectionParams {
  provider: ApiProvider
}
```

**返回值**:

```typescript
interface ApiTestResult {
  success: boolean
  latency?: number // 响应延迟（毫秒）
  error?: string // 错误信息
  modelInfo?: {
    name: string
    version: string
  }
}
```

**示例调用**:

```typescript
const result = await invoke<ApiTestResult>('test_api_connection', {
  provider: {
    id: 'claude-1',
    type: 'official',
    name: 'Claude',
    key: 'sk-ant-xxx',
    model: 'claude-3-opus-20240229',
    enabled: true
  }
})

if (result.success) {
  console.log(`连接成功，延迟 ${result.latency}ms`)
} else {
  console.error('连接失败:', result.error)
}
```

**错误**:
- `"API Key 无效"` - 认证失败
- `"网络连接失败"` - 超时或无网络
- `"模型不存在"` - 模型名称错误

---

### 7.2 加密 API Key

**命令**: `encrypt_api_key`

**描述**: 使用系统密钥库加密 API Key（macOS Keychain / Windows Credential Manager）

**参���**:

```typescript
interface EncryptApiKeyParams {
  providerId: string
  apiKey: string
}
```

**返回值**: `string` (加密后的引用 ID)

**示例调用**:

```typescript
const encryptedRef = await invoke<string>('encrypt_api_key', {
  providerId: 'claude-1',
  apiKey: 'sk-ant-xxx'
})

// 存储 encryptedRef 而非明文 API Key
```

---

### 7.3 解密 API Key

**命令**: `decrypt_api_key`

**参数**:

```typescript
interface DecryptApiKeyParams {
  encryptedRef: string
}
```

**返回值**: `string` (原始 API Key)

**示例调用**:

```typescript
const apiKey = await invoke<string>('decrypt_api_key', {
  encryptedRef: 'keychain:claude-1'
})
```

**错误**:
- `"引用不存在"` - 密钥已被删除
- `"解密失败"` - 权限问题

---

## 数据模型

### 配置文件存储位置

**路径**: `~/.vision-jarvis/settings.json`

**格式**: JSON

**结构**:

```json
{
  "version": "1.0.0",
  "memory": {
    "screenshotFrequency": 5,
    "storageLocation": "/Users/xxx/.vision-jarvis/memory",
    "maxSize": 1024,
    "autoDelete": true
  },
  "popup": {
    "startup": {
      "enabled": false,
      "autoStart": false,
      "text": "If today were the last day of my life..."
    },
    "timedReminder": {
      "enabled": false,
      "startTime": "09:00",
      "endTime": "21:00",
      "interval": 30
    },
    "idleReminder": {
      "enabled": false,
      "idleThreshold": 15,
      "useAiSuggestion": true
    }
  },
  "api": {
    "providers": [
      {
        "id": "claude-1",
        "type": "official",
        "name": "Claude",
        "key": "keychain:claude-1",
        "model": "claude-3-opus-20240229",
        "enabled": true
      }
    ]
  },
  "ui": {
    "orbPosition": { "x": -60, "y": 20 },
    "theme": "auto",
    "language": "zh-CN"
  }
}
```

---

## 错误处理

### 错误码规范

所有错误信息遵循以下格式：

```
[ERR_{模块}_{编号}] 错误描述
```

**示例**:

```
[ERR_SETTINGS_001] 配置文件读取失败: 权限不足
[ERR_FILE_002] 文件夹不存在: /invalid/path
[ERR_API_003] API Key 无效
```

### 常见错误

| 错误码 | 描述 | 解决方案 |
|--------|------|----------|
| ERR_SETTINGS_001 | 配置文件读���失败 | 检查文件权限 |
| ERR_SETTINGS_002 | 配置文件格式错误 | 删除配置文件重新生成 |
| ERR_FILE_001 | 无权限访问路径 | 授予应用文件访问权限 |
| ERR_FILE_002 | 文件夹不存在 | 选择有效路径 |
| ERR_WINDOW_001 | 屏幕信息获取失败 | 重启应用 |
| ERR_AUTO_001 | 自启动注册失败 | 管理员权限 |
| ERR_SCREENSHOT_001 | 截图权限未授予 | 系统设置中授予屏幕录制权限 |
| ERR_API_001 | API Key 无效 | 检查密钥格式 |
| ERR_API_002 | 网络连接失败 | 检查网络连接 |

---

## 安全性

### API Key 保护

1. **永不明文存储**: API Key 仅存储加密引用
2. **系统密钥库**: 使用 macOS Keychain / Windows Credential Manager
3. **内存清理**: 使用后立即清除内存中的密钥
4. **传输加密**: 进程内通信，无网络传输风险

### 文件系统安全

1. **路径验证**: 所有路径必须验证，防止路径遍历攻击
2. **权限检查**: 操作前检查读写权限
3. **沙箱限制**: Tauri 沙箱限制文件系统访问范围

### 输入验证

所有命令参数必须通过以下验证：

```rust
// 示例：验证截图频率
fn validate_screenshot_frequency(freq: u32) -> Result<(), String> {
    if freq < 1 || freq > 15 {
        return Err("[ERR_SETTINGS_003] 截图频率必须在 1-15 秒之间".to_string());
    }
    Ok(())
}
```

---

## 性能优化

### 1. 批量操作

**原则**: 合并多次小请求为一次大请求

```typescript
// ❌ 不推荐：多次调用
await invoke('update_settings', { memory: { screenshotFrequency: 5 } })
await invoke('update_settings', { popup: { startup: { enabled: true } } })

// ✅ 推荐：一次调用
await invoke('update_settings', {
  memory: { screenshotFrequency: 5 },
  popup: { startup: { enabled: true } }
})
```

### 2. 缓存策略

**设置缓存**: 前端缓存设置对象，减少 IPC 调用

```typescript
// 设置管理器
class SettingsManager {
  private cache: Settings | null = null

  async get(): Promise<Settings> {
    if (!this.cache) {
      this.cache = await invoke<Settings>('get_settings')
    }
    return this.cache
  }

  async update(partial: UpdateSettingsParams): Promise<Settings> {
    this.cache = await invoke<Settings>('update_settings', partial)
    return this.cache
  }

  invalidate() {
    this.cache = null
  }
}
```

### 3. 异步任务

**长时间任务**: 使用后台任务 + 事件通知

```rust
// Rust 端发送事件
app.emit_all("screenshot_completed", payload);

// TypeScript 端监听
import { listen } from '@tauri-apps/api/event'

await listen('screenshot_completed', (event) => {
  console.log('截图完成:', event.payload)
})
```

### 4. 防抖/节流

**高频操作**: 窗口位置变化等使用防抖

```typescript
import { debounce } from 'lodash-es'

const savePosition = debounce(async (x: number, y: number) => {
  await invoke('set_window_position', { x, y })
}, 500)
```

---

## 附录

### Tauri 命令实现示例

```rust
// src-tauri/src/commands/settings.rs

use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub memory: MemorySettings,
    pub popup: PopupSettings,
    pub api: ApiSettings,
    pub ui: UiSettings,
}

#[tauri::command]
pub async fn get_settings(
    state: State<'_, Mutex<Settings>>
) -> Result<Settings, String> {
    let settings = state.lock()
        .map_err(|e| format!("[ERR_SETTINGS_001] 获取设置失败: {}", e))?;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn update_settings(
    partial: serde_json::Value,
    state: State<'_, Mutex<Settings>>
) -> Result<Settings, String> {
    let mut settings = state.lock()
        .map_err(|e| format!("[ERR_SETTINGS_001] 获取设置失败: {}", e))?;

    // 合并更新
    merge_settings(&mut settings, partial)?;

    // 保存到磁盘
    save_settings_to_disk(&settings)?;

    Ok(settings.clone())
}
```

### 前端 TypeScript Wrapper 示例

```typescript
// src/lib/tauri.ts

import { invoke } from '@tauri-apps/api/core'

export class TauriAPI {
  // 设置管理
  static async getSettings(): Promise<Settings> {
    return invoke<Settings>('get_settings')
  }

  static async updateSettings(partial: UpdateSettingsParams): Promise<Settings> {
    return invoke<Settings>('update_settings', partial)
  }

  static async resetSettings(section: string = 'all'): Promise<Settings> {
    return invoke<Settings>('reset_settings', { section })
  }

  // 文件系统
  static async pickMemoryFolder(): Promise<string> {
    return invoke<string>('pick_memory_folder')
  }

  static async getMemoryFolderInfo(path: string): Promise<MemoryFolderInfo> {
    return invoke<MemoryFolderInfo>('get_memory_folder_info', { path })
  }

  // ... 其他方法
}
```

---

## 更新日志

### v1.0.0 (2026-02-09)

- 初始版本
- 完整的设置管理 API
- 文件系统操作
- 窗口管理
- 自动启动
- 截图与记忆
- 提醒系统
- API 提供商管理

---

**文档维护**: 随着开发进展，此文档将持续更新
**反馈**: 如有 API 设计问题或建议，请提交 Issue

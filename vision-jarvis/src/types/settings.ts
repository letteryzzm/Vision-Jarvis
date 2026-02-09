/**
 * Vision-Jarvis 设置类型定义
 *
 * 匹配后端 API 设计文档 (docs/API-DESIGN.md)
 * 所有类型与 Rust 后端保持一致
 */

// ============================================================================
// 主设置接口
// ============================================================================

export interface Settings {
  memory: MemorySettings
  popup: PopupSettings
  api: ApiSettings
  ui: UiSettings
}

// ============================================================================
// 记忆设置
// ============================================================================

export interface MemorySettings {
  /** 截图频率（秒），范围 1-15 */
  screenshotFrequency: number
  /** 记忆文件存储位置 */
  storageLocation: string
  /** 最大容量（MB） */
  maxSize: number
  /** 是否自动删除 */
  autoDelete: boolean
}

// ============================================================================
// 弹窗设置
// ============================================================================

export interface PopupSettings {
  /** 启动设置 */
  startup: StartupSettings
  /** 定时提醒 */
  timedReminder: TimedReminderSettings
  /** 空闲提醒 */
  idleReminder: IdleReminderSettings
}

export interface StartupSettings {
  /** 启动时弹出 */
  enabled: boolean
  /** 开机自启 */
  autoStart: boolean
  /** 弹出文本 */
  text: string
}

export interface TimedReminderSettings {
  /** 定时提醒开关 */
  enabled: boolean
  /** 开始时间 "HH:MM" */
  startTime: string
  /** 结束时间 "HH:MM" */
  endTime: string
  /** 间隔（分钟） */
  interval: number
}

export interface IdleReminderSettings {
  /** 空闲提醒开关 */
  enabled: boolean
  /** 空闲阈值（分钟） */
  idleThreshold: number
  /** 使用AI智能建议 */
  useAiSuggestion: boolean
}

// ============================================================================
// API 设置
// ============================================================================

export interface ApiSettings {
  providers: ApiProvider[]
}

export interface ApiProvider {
  /** 唯一标识 */
  id: string
  /** 提供商类型 */
  type: 'official' | 'custom'
  /** 提供商名称（如 "Claude"） */
  name: string
  /** API Key（加密存储） */
  key: string
  /** 模型名称 */
  model: string
  /** 自定义API地址（仅 custom 类型） */
  endpoint?: string
  /** 是否启用 */
  enabled: boolean
}

// ============================================================================
// UI 设置
// ============================================================================

export interface UiSettings {
  /** 悬浮球位置 */
  orbPosition: { x: number; y: number }
  /** 主题 */
  theme: 'light' | 'dark' | 'auto'
  /** 语言 */
  language: 'zh-CN' | 'en-US'
}

// ============================================================================
// 部分更新类型
// ============================================================================

export interface UpdateSettingsParams {
  memory?: Partial<MemorySettings>
  popup?: {
    startup?: Partial<StartupSettings>
    timedReminder?: Partial<TimedReminderSettings>
    idleReminder?: Partial<IdleReminderSettings>
  }
  api?: {
    providers?: ApiProvider[]
  }
  ui?: Partial<UiSettings>
}

// ============================================================================
// 重置设置参数
// ============================================================================

export interface ResetSettingsParams {
  section?: 'memory' | 'popup' | 'api' | 'ui' | 'all'
}

// ============================================================================
// 文件系统相关
// ============================================================================

export interface MemoryFolderInfo {
  /** 文件夹路径 */
  path: string
  /** 总大小（字节） */
  totalSize: number
  /** 总大小（MB） */
  totalSizeMB: number
  /** 文件数量 */
  fileCount: number
  /** 最旧文件 */
  oldestFile?: {
    path: string
    createdAt: string // ISO 8601
  }
  /** 最新文件 */
  newestFile?: {
    path: string
    createdAt: string
  }
}

export interface CleanupResult {
  /** 删除的文件数量 */
  deletedCount: number
  /** 释放的空间（MB） */
  freedSizeMB: number
  /** 剩余文件数量 */
  remainingCount: number
  /** 剩余大小（MB） */
  remainingSizeMB: number
}

// ============================================================================
// 屏幕信息
// ============================================================================

export interface ScreenInfo {
  primary: {
    width: number
    height: number
    scaleFactor: number
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

// ============================================================================
// 截图相关
// ============================================================================

export interface LatestScreenshot {
  path: string
  timestamp: string // ISO 8601
  sizeMB: number
}

// ============================================================================
// API 测试结果
// ============================================================================

export interface ApiTestResult {
  success: boolean
  latency?: number // 响应延迟（毫秒）
  error?: string
  modelInfo?: {
    name: string
    version: string
  }
}

// ============================================================================
// 默认值
// ============================================================================

export const DEFAULT_SETTINGS: Settings = {
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
    orbPosition: { x: -60, y: 20 },
    theme: 'auto',
    language: 'zh-CN'
  }
}

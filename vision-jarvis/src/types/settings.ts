/**
 * Vision-Jarvis 设置类型定义
 *
 * 与后端 Rust AppSettings 结构体保持一致（扁平结构）
 */

// 重新导出 tauri-api 中的类型
export type {
  AppSettings,
  UpdateSettingsParams,
  ResetSettingsParams,
  SchedulerStatus,
  ScreenshotInfo,
  AIProviderConfig,
  AIConfig,
  AIConfigSummary,
  ModelInfo,
  StorageInfo,
  FileInfo,
  CleanupResult,
} from '../lib/tauri-api'

// 为了向后兼容，保留 Settings 别名
export type { AppSettings as Settings } from '../lib/tauri-api'

// ============================================================================
// 默认值（与后端 AppSettings::default() 一致）
// ============================================================================

import type { AppSettings } from '../lib/tauri-api'

export const DEFAULT_SETTINGS: AppSettings = {
  memory_enabled: true,
  capture_interval_seconds: 60,
  storage_path: '',
  storage_limit_mb: 1024,
  auto_start: false,
  app_launch_text: 'If today were the last day of my life, would I want to do what I am about to do today?',

  // 固定提醒
  morning_reminder_enabled: false,
  morning_reminder_time: '08:00',
  morning_reminder_message: 'If today is the last day of my life, would I want to do what I am about to do today?',

  water_reminder_enabled: false,
  water_reminder_start: '09:00',
  water_reminder_end: '21:00',
  water_reminder_interval_minutes: 60,
  water_reminder_message: '该喝喝水了',

  sedentary_reminder_enabled: false,
  sedentary_reminder_start: '09:00',
  sedentary_reminder_end: '21:00',
  sedentary_reminder_threshold_minutes: 60,
  sedentary_reminder_message: '你已经连续工作很久了，再厉害的人也需要休息放松，是时候站起来走动走了',

  // 智能提醒
  screen_inactivity_reminder_enabled: false,
  screen_inactivity_minutes: 10,
  screen_inactivity_message: '',

  openai_api_key: null,
}

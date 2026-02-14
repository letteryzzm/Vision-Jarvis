/**
 * Settings 状态管理 (Nanostores)
 *
 * 使用 Nanostores 管理应用设置状态
 * 支持跨组件共享、自动保存、错误处理
 */

import { atom } from 'nanostores'
import type { AppSettings } from '../types/settings'
import { TauriAPI } from '../lib/tauri-api'
import { DEFAULT_SETTINGS } from '../types/settings'

// ============================================================================
// 状态原子
// ============================================================================

export const $settings = atom<AppSettings | null>(null)
export const $settingsLoading = atom<boolean>(false)
export const $settingsError = atom<string | null>(null)
export const $settingsInitialized = atom<boolean>(false)

// ============================================================================
// 操作函数
// ============================================================================

export async function loadSettings(): Promise<void> {
  $settingsLoading.set(true)
  $settingsError.set(null)

  try {
    const settings = await TauriAPI.getSettings()
    $settings.set(settings)
    $settingsInitialized.set(true)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    $settings.set(DEFAULT_SETTINGS)
    $settingsInitialized.set(true)
  } finally {
    $settingsLoading.set(false)
  }
}

export async function updateSettings(
  updates: Partial<AppSettings>
): Promise<void> {
  const current = $settings.get()
  if (!current) {
    throw new Error('Settings not initialized')
  }

  $settingsLoading.set(true)
  $settingsError.set(null)

  try {
    const updated = await TauriAPI.updateSettings(updates)
    $settings.set(updated)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  } finally {
    $settingsLoading.set(false)
  }
}

// ============================================================================
// 便捷操作
// ============================================================================

export async function toggleMemory(enabled: boolean): Promise<void> {
  await updateSettings({ memory_enabled: enabled })
}

export async function updateCaptureInterval(seconds: number): Promise<void> {
  if (seconds < 1 || seconds > 15) {
    throw new Error('截图间隔必须在 1-15 秒之间')
  }
  await updateSettings({ capture_interval_seconds: seconds })
}

export async function updateStoragePath(path: string): Promise<void> {
  await updateSettings({ storage_path: path })
}

export async function toggleAutoStart(enabled: boolean): Promise<void> {
  try {
    if (enabled) {
      await TauriAPI.enableAutoStart()
    } else {
      await TauriAPI.disableAutoStart()
    }
    await updateSettings({ auto_start: enabled })
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  }
}

export async function updateLaunchText(text: string): Promise<void> {
  await updateSettings({ app_launch_text: text })
}

// ========== 固定提醒 ==========

export async function toggleMorningReminder(enabled: boolean): Promise<void> {
  await updateSettings({ morning_reminder_enabled: enabled })
}

export async function toggleWaterReminder(enabled: boolean): Promise<void> {
  await updateSettings({ water_reminder_enabled: enabled })
}

export async function toggleSedentaryReminder(enabled: boolean): Promise<void> {
  await updateSettings({ sedentary_reminder_enabled: enabled })
}

// ========== 智能提醒 ==========

export async function toggleScreenInactivityReminder(enabled: boolean): Promise<void> {
  await updateSettings({ screen_inactivity_reminder_enabled: enabled })
}

// ============================================================================
// 重置和刷新
// ============================================================================

export async function resetSettings(): Promise<void> {
  $settingsLoading.set(true)
  $settingsError.set(null)

  try {
    const updated = await TauriAPI.resetSettings()
    $settings.set(updated)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  } finally {
    $settingsLoading.set(false)
  }
}

export async function refreshSettings(): Promise<void> {
  TauriAPI.invalidateCache()
  await loadSettings()
}

export function clearError(): void {
  $settingsError.set(null)
}

// ============================================================================
// 辅助函数
// ============================================================================

export function getCurrentSettings(): AppSettings {
  const settings = $settings.get()
  return settings || DEFAULT_SETTINGS
}

export function isSettingsLoaded(): boolean {
  return $settingsInitialized.get()
}

export function getError(): string | null {
  return $settingsError.get()
}

export function isLoading(): boolean {
  return $settingsLoading.get()
}

/**
 * Settings 状态管理 (Nanostores)
 *
 * 使用 Nanostores 管理应用设置状态
 * 支持跨组件共享、自动保存、错误处理
 */

import { atom } from 'nanostores'
import type { Settings } from '../types/settings'
import { TauriAPI } from '../lib/tauri-api'
import { DEFAULT_SETTINGS } from '../types/settings'

// ============================================================================
// 状态原子
// ============================================================================

/**
 * 设置数据
 * null 表示尚未加载
 */
export const $settings = atom<Settings | null>(null)

/**
 * 加载状态
 */
export const $settingsLoading = atom<boolean>(false)

/**
 * 错误信息
 */
export const $settingsError = atom<string | null>(null)

/**
 * 是否已初始化
 */
export const $settingsInitialized = atom<boolean>(false)

// ============================================================================
// 操作函数
// ============================================================================

/**
 * 加载设置
 * 从后端获取设置并更新状态
 */
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
    console.error('Failed to load settings:', error)

    // 使用默认值
    $settings.set(DEFAULT_SETTINGS)
    $settingsInitialized.set(true)
  } finally {
    $settingsLoading.set(false)
  }
}

/**
 * 更新记忆设置
 */
export async function updateMemorySettings(
  updates: Partial<Settings['memory']>
): Promise<void> {
  const current = $settings.get()
  if (!current) {
    throw new Error('Settings not initialized')
  }

  $settingsLoading.set(true)
  $settingsError.set(null)

  try {
    const updated = await TauriAPI.updateSettings({
      memory: updates
    })
    $settings.set(updated)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  } finally {
    $settingsLoading.set(false)
  }
}

/**
 * 更新启动设置
 */
export async function updateStartupSettings(
  updates: Partial<Settings['popup']['startup']>
): Promise<void> {
  const current = $settings.get()
  if (!current) {
    throw new Error('Settings not initialized')
  }

  $settingsLoading.set(true)
  $settingsError.set(null)

  try {
    const updated = await TauriAPI.updateSettings({
      popup: {
        startup: updates
      }
    })
    $settings.set(updated)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  } finally {
    $settingsLoading.set(false)
  }
}

/**
 * 更新定时提醒设置
 */
export async function updateTimedReminderSettings(
  updates: Partial<Settings['popup']['timedReminder']>
): Promise<void> {
  const current = $settings.get()
  if (!current) {
    throw new Error('Settings not initialized')
  }

  $settingsLoading.set(true)
  $settingsError.set(null)

  try {
    const updated = await TauriAPI.updateSettings({
      popup: {
        timedReminder: updates
      }
    })
    $settings.set(updated)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  } finally {
    $settingsLoading.set(false)
  }
}

/**
 * 更新空闲提醒设置
 */
export async function updateIdleReminderSettings(
  updates: Partial<Settings['popup']['idleReminder']>
): Promise<void> {
  const current = $settings.get()
  if (!current) {
    throw new Error('Settings not initialized')
  }

  $settingsLoading.set(true)
  $settingsError.set(null)

  try {
    const updated = await TauriAPI.updateSettings({
      popup: {
        idleReminder: updates
      }
    })
    $settings.set(updated)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  } finally {
    $settingsLoading.set(false)
  }
}

/**
 * 更新 API 提供商
 */
export async function updateApiProviders(
  providers: Settings['api']['providers']
): Promise<void> {
  const current = $settings.get()
  if (!current) {
    throw new Error('Settings not initialized')
  }

  $settingsLoading.set(true)
  $settingsError.set(null)

  try {
    const updated = await TauriAPI.updateSettings({
      api: {
        providers
      }
    })
    $settings.set(updated)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  } finally {
    $settingsLoading.set(false)
  }
}

/**
 * 更新 UI 设置
 */
export async function updateUiSettings(
  updates: Partial<Settings['ui']>
): Promise<void> {
  const current = $settings.get()
  if (!current) {
    throw new Error('Settings not initialized')
  }

  $settingsLoading.set(true)
  $settingsError.set(null)

  try {
    const updated = await TauriAPI.updateSettings({
      ui: updates
    })
    $settings.set(updated)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  } finally {
    $settingsLoading.set(false)
  }
}

/**
 * 重置设置
 */
export async function resetSettings(
  section: 'memory' | 'popup' | 'api' | 'ui' | 'all' = 'all'
): Promise<void> {
  $settingsLoading.set(true)
  $settingsError.set(null)

  try {
    const updated = await TauriAPI.resetSettings({ section })
    $settings.set(updated)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  } finally {
    $settingsLoading.set(false)
  }
}

/**
 * 刷新设置
 * 从后端重新加载，清除缓存
 */
export async function refreshSettings(): Promise<void> {
  TauriAPI.invalidateCache()
  await loadSettings()
}

/**
 * 清除错误
 */
export function clearError(): void {
  $settingsError.set(null)
}

// ============================================================================
// 自动启动管理
// ============================================================================

/**
 * 切换开机自启
 */
export async function toggleAutoStart(enabled: boolean): Promise<void> {
  try {
    if (enabled) {
      await TauriAPI.enableAutoStart()
    } else {
      await TauriAPI.disableAutoStart()
    }

    // 更新设置
    await updateStartupSettings({ autoStart: enabled })
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    $settingsError.set(errorMessage)
    throw error
  }
}

// ============================================================================
// 辅助函数
// ============================================================================

/**
 * 获取当前设��（同步）
 * 如果未初始化，返回默认值
 */
export function getCurrentSettings(): Settings {
  const settings = $settings.get()
  return settings || DEFAULT_SETTINGS
}

/**
 * 检查是否已加载
 */
export function isSettingsLoaded(): boolean {
  return $settingsInitialized.get()
}

/**
 * 获取错误信息
 */
export function getError(): string | null {
  return $settingsError.get()
}

/**
 * 检查是否正在加载
 */
export function isLoading(): boolean {
  return $settingsLoading.get()
}

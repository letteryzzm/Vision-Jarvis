# Popup-Setting 页面 (提醒设置)

> **页面路由**: `/popup-setting`
> **页面文件**: `src/pages/popup-setting.astro`
> **状态**: 🚧 开发中
> **最后更新**: 2026-02-04

---

## 📋 概述

Popup-Setting 页面用于配置 Vision-Jarvis 的各类提醒功能,包括启动提醒、定时提醒和无变化提醒。采用**卡片组件**的布局形式,支持后续扩展更多设置项。

---

## 🎨 页面布局

### 整体设计

```
┌─────────────────────────────────────────────────────┐
│            Popup-Setting 页面布局                    │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  ⚙️ 提醒设置                                   [返回] │ ← Header
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────────┐  ┌──────────────────┐       │
│  │  🚀 启动设置      │  │  ⏰ 定时提醒      │       │
│  │                  │  │                  │       │
│  │  [开关]          │  │  [开关]          │       │
│  │  启动文本输入框   │  │  时间范围选择     │       │
│  │                  │  │  间隔设置        │       │
│  └──────────────────┘  └──────────────────┘       │
│                                                     │
│  ┌──────────────────┐  ┌──────────────────┐       │
│  │  💤 无变化提醒    │  │  ➕ 其他设置      │       │
│  │                  │  │  (待扩展)        │       │
│  │  [开关]          │  │                  │       │
│  │  时长设置        │  │                  │       │
│  │                  │  │                  │       │
│  └──────────────────┘  └──────────────────┘       │
│                                                     │
└─────────────────────────────────────────────────────┘

布局:
- Grid 布局: 2 列（响应式）
- 卡片间距: 24px
- 卡片圆角: 12px
- 卡片阴影: 0 2px 8px rgba(0,0,0,0.1)
```

---

## 🏗️ 页面结构

### Astro 页面文件

```astro
---
// src/pages/popup-setting.astro
import Layout from '@/layouts/Layout.astro';
import SettingCard from '@/components/settings/SettingCard';
import StartupSetting from '@/components/settings/StartupSetting';
import TimedReminderSetting from '@/components/settings/TimedReminderSetting';
import IdleReminderSetting from '@/components/settings/IdleReminderSetting';
---

<Layout title="提醒设置 - Vision-Jarvis">
  <div class="setting-page">
    <!-- Header -->
    <header class="setting-header">
      <h1 class="setting-title">⚙️ 提醒设置</h1>
      <button
        onclick="window.history.back()"
        class="back-btn"
      >
        返回
      </button>
    </header>

    <!-- 设置卡片网格 -->
    <div class="setting-grid">
      <!-- 启动设置卡片 -->
      <StartupSetting client:load />

      <!-- 定时提醒卡片 -->
      <TimedReminderSetting client:load />

      <!-- 无变化提醒卡片 -->
      <IdleReminderSetting client:load />

      <!-- 预留扩展卡片 -->
      <SettingCard
        title="➕ 更多设置"
        description="即将推出更多功能"
        client:visible
      >
        <p class="text-gray-500 text-sm">敬请期待...</p>
      </SettingCard>
    </div>
  </div>
</Layout>

<style>
  .setting-page {
    min-height: 100vh;
    background: #f9fafb;
    padding: 24px;
  }

  .setting-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 32px;
  }

  .setting-title {
    font-size: 1.875rem;
    font-weight: 700;
    color: #111827;
  }

  .back-btn {
    padding: 8px 16px;
    border-radius: 8px;
    border: 1px solid #d1d5db;
    background: #ffffff;
    cursor: pointer;
    transition: all 0.2s;
  }

  .back-btn:hover {
    background: #f3f4f6;
    border-color: #9ca3af;
  }

  .setting-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 24px;
  }

  @media (max-width: 768px) {
    .setting-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
```

---

## 📦 设置卡片组件

### SettingCard 基础组件

```typescript
// components/settings/SettingCard.tsx
import { motion } from 'framer-motion';
import type { FC, ReactNode } from 'react';

interface SettingCardProps {
  title: string;
  description?: string;
  icon?: string;
  children: ReactNode;
  className?: string;
}

export const SettingCard: FC<SettingCardProps> = ({
  title,
  description,
  icon,
  children,
  className = ''
}) => {
  return (
    <motion.div
      className={`setting-card ${className}`}
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      style={{
        background: '#ffffff',
        borderRadius: '12px',
        padding: '24px',
        boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)'
      }}
    >
      <div className="card-header">
        <h3 className="card-title">
          {icon && <span className="card-icon">{icon}</span>}
          {title}
        </h3>
        {description && (
          <p className="card-description">{description}</p>
        )}
      </div>

      <div className="card-content">
        {children}
      </div>
    </motion.div>
  );
};
```

---

## 🚀 启动设置卡片

```typescript
// components/settings/StartupSetting.tsx
import { useStore } from '@nanostores/react';
import { settings } from '@/stores/settingStore';
import { SettingCard } from './SettingCard';
import { ToggleSwitch } from '@/components/ui/ToggleSwitch';
import { Input } from '@/components/ui/Input';

export function StartupSetting() {
  const currentSettings = useStore(settings);

  const handleToggleAutoStart = async (enabled: boolean) => {
    settings.setKey('autoStart', enabled);

    // 调用 Tauri 后端设置开机启动
    await invoke('set_auto_start', { enabled });
  };

  const handleUpdateMessage = (message: string) => {
    settings.setKey('startupMessage', message);
  };

  return (
    <SettingCard
      title="🚀 启动设置"
      description="配置应用启动行为"
    >
      <div className="setting-group">
        {/* 开机自动启动开关 */}
        <div className="setting-item">
          <ToggleSwitch
            enabled={currentSettings.autoStart}
            onChange={handleToggleAutoStart}
            label="开机自动启动"
          />
          <p className="setting-hint">
            启用后，应用将在系统启动时自动运行
          </p>
        </div>

        {/* 启动弹出文本设置 */}
        <div className="setting-item">
          <label className="setting-label">启动提示文本</label>
          <Input
            type="textarea"
            value={currentSettings.startupMessage}
            onChange={handleUpdateMessage}
            placeholder="If today were the last day of my life..."
            rows={3}
          />
          <p className="setting-hint">
            应用启动时显示的激励文本
          </p>
        </div>
      </div>
    </SettingCard>
  );
}
```

---

## ⏰ 定时提醒卡片

```typescript
// components/settings/TimedReminderSetting.tsx
import { useStore } from '@nanostores/react';
import { settings } from '@/stores/settingStore';
import { SettingCard } from './SettingCard';
import { ToggleSwitch } from '@/components/ui/ToggleSwitch';
import { TimeRangePicker } from '@/components/ui/TimeRangePicker';
import { Input } from '@/components/ui/Input';

export function TimedReminderSetting() {
  const currentSettings = useStore(settings);
  const { enabled, timeRange, interval } = currentSettings.timedReminder;

  return (
    <SettingCard
      title="⏰ 定时提醒"
      description="设置定时提醒规则"
    >
      <div className="setting-group">
        {/* 功能开关 */}
        <div className="setting-item">
          <ToggleSwitch
            enabled={enabled}
            onChange={(value) => {
              settings.setKey('timedReminder.enabled', value);
            }}
            label="启用定时提醒"
          />
        </div>

        {/* 时间范围选择 */}
        {enabled && (
          <>
            <div className="setting-item">
              <label className="setting-label">提醒时间范围</label>
              <TimeRangePicker
                start={timeRange.start}
                end={timeRange.end}
                onChange={(start, end) => {
                  settings.setKey('timedReminder.timeRange', { start, end });
                }}
              />
              <p className="setting-hint">
                在此时间段内每隔一定时间提醒一次
              </p>
            </div>

            {/* 间隔时长设置 */}
            <div className="setting-item">
              <label className="setting-label">提醒间隔</label>
              <div className="flex items-center gap-2">
                <Input
                  type="number"
                  value={interval}
                  onChange={(value) => {
                    settings.setKey('timedReminder.interval', Number(value));
                  }}
                  min={1}
                  max={120}
                  className="w-24"
                />
                <span className="text-gray-600">分钟</span>
              </div>
              <p className="setting-hint">
                每隔 {interval} 分钟提醒一次
              </p>
            </div>
          </>
        )}
      </div>
    </SettingCard>
  );
}
```

---

## 💤 无变化提醒卡片

```typescript
// components/settings/IdleReminderSetting.tsx
import { useStore } from '@nanostores/react';
import { settings } from '@/stores/settingStore';
import { SettingCard } from './SettingCard';
import { ToggleSwitch } from '@/components/ui/ToggleSwitch';
import { Input } from '@/components/ui/Input';

export function IdleReminderSetting() {
  const currentSettings = useStore(settings);
  const { enabled, idleThreshold } = currentSettings.idleReminder;

  return (
    <SettingCard
      title="💤 无变化提醒"
      description="屏幕无操作时的提醒设置"
    >
      <div className="setting-group">
        {/* 功能开关 */}
        <div className="setting-item">
          <ToggleSwitch
            enabled={enabled}
            onChange={(value) => {
              settings.setKey('idleReminder.enabled', value);
            }}
            label="启用无变化提醒"
          />
        </div>

        {/* 无变化时长判断 */}
        {enabled && (
          <div className="setting-item">
            <label className="setting-label">无变化判断时长</label>
            <div className="flex items-center gap-2">
              <Input
                type="number"
                value={idleThreshold}
                onChange={(value) => {
                  settings.setKey('idleReminder.idleThreshold', Number(value));
                }}
                min={1}
                max={60}
                className="w-24"
              />
              <span className="text-gray-600">分钟</span>
            </div>
            <p className="setting-hint">
              屏幕 {idleThreshold} 分钟内无变化时认为用户离开，触发提醒
            </p>
          </div>
        )}
      </div>
    </SettingCard>
  );
}
```

---

## 🔗 状态管理

### Nanostores 定义

```typescript
// stores/settingStore.ts
import { map } from 'nanostores';
import { invoke } from '@tauri-apps/api/core';

export const settings = map({
  // 启动设置
  autoStart: true,
  startupMessage: "If today were the last day of my life, would I want to do what I am about to do today?",

  // 定时提醒
  timedReminder: {
    enabled: false,
    timeRange: { start: '09:00', end: '21:00' },
    interval: 30 // 分钟
  },

  // 无变化提醒
  idleReminder: {
    enabled: false,
    idleThreshold: 15 // 分钟
  },

  // 截屏设置
  screenshot: {
    frequency: 5, // 秒
    storagePath: '/default/path',
    maxSize: 1024 // MB
  }
});

// 初始化设置（从后端加载）
export async function loadSettings() {
  try {
    const savedSettings = await invoke<typeof settings>('get_settings');
    settings.set(savedSettings);
  } catch (error) {
    console.error('Failed to load settings:', error);
  }
}

// 保存设置到后端
export async function saveSettings() {
  try {
    await invoke('save_settings', { settings: settings.get() });
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
}

// 监听设置变化，自动保存
settings.subscribe((value) => {
  saveSettings();
});
```

---

## 🎨 UI 组件

### TimeRangePicker 组件

```typescript
// components/ui/TimeRangePicker.tsx
interface TimeRangePickerProps {
  start: string;
  end: string;
  onChange: (start: string, end: string) => void;
}

export function TimeRangePicker({ start, end, onChange }: TimeRangePickerProps) {
  return (
    <div className="time-range-picker">
      <input
        type="time"
        value={start}
        onChange={(e) => onChange(e.target.value, end)}
        className="time-input"
      />
      <span className="separator">至</span>
      <input
        type="time"
        value={end}
        onChange={(e) => onChange(start, e.target.value)}
        className="time-input"
      />
    </div>
  );
}
```

---

## 📱 响应式设计

```css
/* 移动端适配 */
@media (max-width: 768px) {
  .setting-page {
    padding: 16px;
  }

  .setting-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }

  .setting-grid {
    grid-template-columns: 1fr;
    gap: 16px;
  }

  .setting-card {
    padding: 16px;
  }
}
```

---

## 🧪 测试用例

```typescript
// pages/popup-setting.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import PopupSettingPage from './popup-setting.astro';

describe('Popup Setting Page', () => {
  it('renders all setting cards', () => {
    render(<PopupSettingPage />);

    expect(screen.getByText('启动设置')).toBeInTheDocument();
    expect(screen.getByText('定时提醒')).toBeInTheDocument();
    expect(screen.getByText('无变化提醒')).toBeInTheDocument();
  });

  it('toggles auto start', async () => {
    render(<PopupSettingPage />);

    const toggle = screen.getByLabelText('开机自动启动');
    fireEvent.click(toggle);

    // 验证状态更新
    expect(settings.get().autoStart).toBe(false);
  });
});
```

---

## 📝 待实现功能

- [ ] 提醒音效选择
- [ ] 提醒样式自定义
- [ ] 提醒历史记录
- [ ] 批量导入/导出设置
- [ ] 设置预设模板
- [ ] 提醒测试功能

---

## 🔗 相关文档

- [SettingCard 组件](../components/SettingCard.md)
- [ToggleSwitch 组件](../components/ToggleSwitch.md)
- [TimeRangePicker 组件](../components/TimeRangePicker.md)
- [状态管理](../state-management.md)

---

**页面维护者**: 前端团队
**最后更新**: 2026-02-04

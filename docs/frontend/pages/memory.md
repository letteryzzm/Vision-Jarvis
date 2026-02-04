# Memory 页面 (记忆管理)

> **页面路由**: `/memory`
> **页面文件**: `src/pages/memory.astro`
> **状态**: 🚧 开发中
> **最后更新**: 2026-02-04

---

## 📋 概述

Memory 页面是 Vision-Jarvis 的核心功能页面,用于查看、管理和检索用户的短期和长期记忆。采用**左侧 Sidebar + 右侧主内容**的经典布局。

---

## 🎨 页面布局

### 整体设计

```
┌─────────────────────────────────────────────────────────┐
│                    Memory 页面布局                       │
└─────────────────────────────────────────────────────────┘

┌─────────────┬─────────────────────────────────────────┐
│             │                                         │
│             │  🔍 搜索框/问答框                        │
│             │  ┌──────────────────────────────────┐  │
│  Left       │  │ 想找哪段记忆，我都记着呢随便问    │  │
│  Sidebar    │  └──────────────────────────────────┘  │
│             │                                         │
│  280px      │                                         │
│             │  📋 记忆卡片 / 总结内容                  │
│  - 开关     │  ┌──────────────────────────────────┐  │
│  - 日期选择  │  │ 事项标题: 团队晨会                │  │
│  - 记忆列表  │  │ 时间: 09:00-10:00               │  │
│  - 截屏设置  │  │ 内容: ...                       │  │
│             │  │ 建议: ...                       │  │
│             │  └──────────────────────────────────┘  │
│             │                                         │
└─────────────┴─────────────────────────────────────────┘
   Sidebar          Main Content (flex-1)
```

---

## 🏗️ 页面结构

### Astro 页面文件

```astro
---
// src/pages/memory.astro
import Layout from '@/layouts/Layout.astro';
import MemorySidebar from '@/components/memory/MemorySidebar';
import MemoryContent from '@/components/memory/MemoryContent';
import FloatingInput from '@/components/FloatingInput';
---

<Layout title="记忆管理 - Vision-Jarvis">
  <div class="memory-page">
    <!-- 左侧 Sidebar -->
    <aside class="memory-sidebar">
      <MemorySidebar client:load />
    </aside>

    <!-- 右侧主内容 -->
    <main class="memory-content">
      <FloatingInput
        client:load
        placeholder="想找哪段记忆，我都记着呢随便问"
        className="memory-search"
      />

      <MemoryContent client:visible />
    </main>
  </div>
</Layout>

<style>
  .memory-page {
    display: flex;
    height: 100vh;
    background: #f9fafb;
  }

  .memory-sidebar {
    width: 280px;
    border-right: 1px solid #e5e7eb;
    background: #ffffff;
    overflow-y: auto;
  }

  .memory-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .memory-search {
    position: sticky;
    top: 0;
    z-index: 10;
  }
</style>
```

---

## 📦 左侧 Sidebar 组件

### MemorySidebar 结构

```typescript
// components/memory/MemorySidebar.tsx
import { useState } from 'react';
import { useStore } from '@nanostores/react';
import { memoryEnabled, shortTermMemories } from '@/stores/memoryStore';
import { ToggleSwitch } from '@/components/ui/ToggleSwitch';
import { DatePicker } from '@/components/DatePicker';
import { MemoryList } from '@/components/MemoryList';
import { ScreenshotSettings } from './ScreenshotSettings';

export function MemorySidebar() {
  const isMemoryEnabled = useStore(memoryEnabled);
  const memories = useStore(shortTermMemories);
  const [selectedDate, setSelectedDate] = useState(new Date());

  return (
    <div className="sidebar-container">
      {/* 1. 记忆功能开关 */}
      <section className="sidebar-section">
        <h3 className="sidebar-title">���忆功能</h3>
        <ToggleSwitch
          enabled={isMemoryEnabled}
          onChange={(enabled) => memoryEnabled.set(enabled)}
          label="启用全局记忆"
        />
      </section>

      {/* 2. 日期选择器 */}
      <section className="sidebar-section">
        <h3 className="sidebar-title">日期选择</h3>
        <DatePicker
          value={selectedDate}
          onChange={setSelectedDate}
          mode="single" // 单日期模式
        />
        <button
          onClick={() => {
            // 触发日期范围选择
            openDateRangeModal();
          }}
          className="date-range-btn"
        >
          选择日期范围分析
        </button>
      </section>

      {/* 3. 短期记忆列表 */}
      <section className="sidebar-section flex-1">
        <h3 className="sidebar-title">记忆事项</h3>
        <MemoryList
          date={selectedDate}
          memories={memories.get().items}
        />
      </section>

      {/* 4. 截屏设置 */}
      <section className="sidebar-section">
        <ScreenshotSettings />
      </section>
    </div>
  );
}
```

### 日期选择器

```typescript
// components/DatePicker/DatePicker.tsx
interface DatePickerProps {
  value: Date;
  onChange: (date: Date) => void;
  mode: 'single' | 'range';
}

export function DatePicker({ value, onChange, mode }: DatePickerProps) {
  return (
    <div className="date-picker">
      <input
        type="date"
        value={formatDate(value)}
        onChange={(e) => onChange(new Date(e.target.value))}
        className="date-input"
      />
      {/* 使用第三方日历组件如 react-day-picker */}
    </div>
  );
}
```

### 短期记忆列表

```typescript
// components/MemoryList/MemoryList.tsx
interface MemoryListProps {
  date: Date;
  memories: ShortTermMemory[];
}

export function MemoryList({ date, memories }: MemoryListProps) {
  // 按时间段分组
  const groupedMemories = groupByTimeOfDay(memories);

  return (
    <div className="memory-list">
      {/* 早晨 (06:00-12:00) */}
      {groupedMemories.morning.length > 0 && (
        <div className="time-group">
          <div className="time-divider">早晨</div>
          {groupedMemories.morning.map((memory) => (
            <MemoryListItem key={memory.id} memory={memory} />
          ))}
        </div>
      )}

      {/* 下午 (12:00-18:00) */}
      {groupedMemories.afternoon.length > 0 && (
        <div className="time-group">
          <div className="time-divider">下午</div>
          {groupedMemories.afternoon.map((memory) => (
            <MemoryListItem key={memory.id} memory={memory} />
          ))}
        </div>
      )}

      {/* 晚上 (18:00-24:00) */}
      {groupedMemories.evening.length > 0 && (
        <div className="time-group">
          <div className="time-divider">晚上</div>
          {groupedMemories.evening.map((memory) => (
            <MemoryListItem key={memory.id} memory={memory} />
          ))}
        </div>
      )}
    </div>
  );
}

// 记忆列表项
function MemoryListItem({ memory }: { memory: ShortTermMemory }) {
  return (
    <button
      className="memory-list-item"
      onClick={() => {
        // 切换右侧显示该事项详情
        selectedMemory.set(memory);
      }}
    >
      <div className="item-time">{memory.timeRange}</div>
      <div className="item-title">{memory.title}</div>
    </button>
  );
}
```

### 截屏设置

```typescript
// components/memory/ScreenshotSettings.tsx
import { SliderInput } from '@/components/ui/SliderInput';
import { settings } from '@/stores/settingStore';

export function ScreenshotSettings() {
  const screenshotFrequency = settings.get().screenshot.frequency;

  return (
    <div className="screenshot-settings">
      <h4 className="setting-title">截屏设置</h4>

      {/* 截屏频率滑动条 */}
      <SliderInput
        label="截屏频率"
        value={screenshotFrequency}
        min={1}
        max={15}
        step={1}
        unit="秒"
        onChange={(value) => {
          settings.setKey('screenshot.frequency', value);
        }}
      />

      {/* 文件存储位置 */}
      <div className="setting-item">
        <label>存储位置</label>
        <button
          onClick={async () => {
            const path = await openFileDialog();
            settings.setKey('screenshot.storagePath', path);
          }}
          className="file-path-btn"
        >
          选择文件夹
        </button>
      </div>

      {/* 内存上限设置 */}
      <div className="setting-item">
        <label>自动删除上限</label>
        <input
          type="number"
          value={settings.get().screenshot.maxSize}
          onChange={(e) => {
            settings.setKey('screenshot.maxSize', Number(e.target.value));
          }}
          className="number-input"
        />
        <span className="unit">MB</span>
      </div>
    </div>
  );
}
```

---

## 📋 右侧主内容区

### MemoryContent 组件

```typescript
// components/memory/MemoryContent.tsx
import { useStore } from '@nanostores/react';
import { selectedMemory } from '@/stores/memoryStore';
import { MemoryCard } from '@/components/MemoryCard';
import { LongTermSummary } from './LongTermSummary';

export function MemoryContent() {
  const memory = useStore(selectedMemory);
  const viewMode = useStore(memoryViewMode); // 'short' | 'long'

  if (!memory && viewMode === 'short') {
    return (
      <div className="content-empty">
        <p className="empty-message">
          想找哪段记忆，我都记着呢随便问
        </p>
      </div>
    );
  }

  return (
    <div className="content-area">
      {viewMode === 'short' && memory && (
        <MemoryCard memory={memory} />
      )}

      {viewMode === 'long' && (
        <LongTermSummary />
      )}
    </div>
  );
}
```

### 短期记忆卡片

```typescript
// components/MemoryCard/MemoryCard.tsx
interface MemoryCardProps {
  memory: ShortTermMemory;
}

export function MemoryCard({ memory }: MemoryCardProps) {
  return (
    <div className="memory-card">
      {/* 标题 */}
      <header className="card-header">
        <h2 className="card-title">{memory.title}</h2>
        <span className="card-time">{memory.timeRange}</span>
      </header>

      {/* 正文: 时间线 + 图片索引 + 分析总结 */}
      <main className="card-content">
        <div className="timeline">
          {memory.screenshots.map((screenshot, index) => (
            <div key={screenshot.id} className="timeline-item">
              <div className="timeline-time">
                {formatTime(screenshot.timestamp)}
              </div>
              <div className="timeline-content">
                <img
                  src={screenshot.thumbnailPath}
                  alt={`Screenshot ${index + 1}`}
                  className="screenshot-thumb"
                  onClick={() => openImagePreview(screenshot.path)}
                />
                <p className="screenshot-analysis">
                  {screenshot.analysis}
                </p>
              </div>
            </div>
          ))}
        </div>
      </main>

      {/* 结尾: 事项建议与分析 */}
      <footer className="card-footer">
        <h3 className="footer-title">💡 建议与分析</h3>
        <p className="footer-content">{memory.suggestion}</p>
      </footer>
    </div>
  );
}
```

### 长期记忆总结

```typescript
// components/memory/LongTermSummary.tsx
import { useStore } from '@nanostores/react';
import { longTermSummary } from '@/stores/memoryStore';

export function LongTermSummary() {
  const summary = useStore(longTermSummary);

  return (
    <div className="long-term-summary">
      {/* 标题 */}
      <header className="summary-header">
        <h2 className="summary-title">
          {formatDateRange(summary.dateRange)} 事项总结
        </h2>
      </header>

      {/* 主要事项概览 */}
      <main className="summary-content">
        <div className="summary-text">
          {summary.summary}
        </div>

        <div className="events-list">
          <h3 className="events-title">主要事项</h3>
          {summary.events.map((event) => (
            <div key={event.id} className="event-item">
              <div className="event-date">{formatDate(event.date)}</div>
              <div className="event-title">{event.title}</div>
              <div className="event-description">{event.description}</div>
            </div>
          ))}
        </div>
      </main>
    </div>
  );
}
```

---

## 🔗 状态管理

### Nanostores 定义

```typescript
// stores/memoryStore.ts
import { atom, map } from 'nanostores';

// 全局记忆开关
export const memoryEnabled = atom<boolean>(true);

// 短期记忆列表
export const shortTermMemories = map<{
  date: string;
  items: ShortTermMemory[];
}>({
  date: new Date().toISOString(),
  items: []
});

// 选中的记忆项
export const selectedMemory = atom<ShortTermMemory | null>(null);

// 长期记忆总结
export const longTermSummary = map<{
  dateRange: [string, string];
  summary: string;
  events: Event[];
}>({
  dateRange: ['', ''],
  summary: '',
  events: []
});

// 视图模式
export const memoryViewMode = atom<'short' | 'long'>('short');
```

---

## 📱 响应式设计

```css
/* 移动端适配 */
@media (max-width: 768px) {
  .memory-page {
    flex-direction: column;
  }

  .memory-sidebar {
    width: 100%;
    border-right: none;
    border-bottom: 1px solid #e5e7eb;
    max-height: 40vh;
  }

  .memory-content {
    flex: 1;
  }
}
```

---

## 🧪 测试用例

```typescript
// pages/memory.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import MemoryPage from './memory.astro';

describe('Memory Page', () => {
  it('renders sidebar and main content', () => {
    render(<MemoryPage />);

    expect(screen.getByText('记忆功能')).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/想找哪段记忆/)).toBeInTheDocument();
  });

  it('selects a memory and displays it', async () => {
    render(<MemoryPage />);

    const memoryItem = screen.getByText('团队晨会');
    fireEvent.click(memoryItem);

    expect(screen.getByText(/09:00-10:00/)).toBeInTheDocument();
  });
});
```

---

## 📝 待实现功能

- [ ] 日期范围选择弹窗
- [ ] 长期记忆分析确认对话框
- [ ] 记忆导出功能（PDF/JSON）
- [ ] 记忆删除和编辑
- [ ] 标签筛选功能
- [ ] 全文搜索
- [ ] 时间线可视化图表

---

## 🔗 相关文档

- [MemoryList 组件](../components/MemoryList.md)
- [MemoryCard 组件](../components/MemoryCard.md)
- [DatePicker 组件](../components/DatePicker.md)
- [FloatingInput 组件](../components/FloatingInput.md)

---

**页面维护者**: 前端团队
**最后更新**: 2026-02-04

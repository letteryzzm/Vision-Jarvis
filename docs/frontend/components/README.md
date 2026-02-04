# 前端组件库

> **最后更新**: 2026-02-04
> **版本**: v1.0
> **组件总数**: 12+

---

## 📦 组件分类

### 核心交互组件 (Core)
- [FloatingOrb](FloatingOrb.md) - 悬浮球组件（入口）
- [Header](Header.md) - Header 展开模式
- [Asker](Asker.md) - AI 问答对话组件

### 记忆管理组件 (Memory)
- [MemoryList](MemoryList.md) - 记忆列表（左侧 Sidebar）
- [MemoryCard](MemoryCard.md) - 记忆卡片
- [DatePicker](DatePicker.md) - 日期选择器
- [FloatingInput](FloatingInput.md) - 悬浮输入框

### 设置组件 (Settings)
- [SettingCard](SettingCard.md) - 设置卡片容器
- [ToggleSwitch](ToggleSwitch.md) - 开关切换
- [TimeRangePicker](TimeRangePicker.md) - 时间范围选择器
- [SliderInput](SliderInput.md) - 滑动输入（截屏频率）

### 通用 UI 组件 (UI)
- Button - 按钮
- Input - 输入框
- Card - 卡片容器
- Modal - 模态框
- Tooltip - 工具提示
- Spinner - 加载动画

---

## 🏗️ 组件架构图

```
┌─────────────────────────────────────────────┐
│              Vision-Jarvis 组件树            │
└─────────────────────────────────────────────┘

App (Layout.astro)
│
├── Page: index.astro (主页/悬浮窗)
│   └── FloatingOrb
│       ├── Header (悬停态)
│       │   ├── ToggleSwitch (全局记忆开关)
│       │   ├── Button (记忆管理)
│       │   └── Button (提醒设置)
│       │
│       └── Asker (点击态)
│           ├── FloatingInput (搜索框)
│           └── ChatHistory (对话历史)
│
├── Page: memory.astro (记忆管理)
│   ├── Sidebar
│   │   ├── ToggleSwitch (记忆开关)
│   │   ├── DatePicker (日期选择器)
│   │   ├── MemoryList (记忆列表)
│   │   ├── SliderInput (截屏频率)
│   │   └── FilePathSetting (文件设置)
│   │
│   └── MainContent
│       ├── FloatingInput (搜索框)
│       └── MemoryCard (记忆卡片)
│           ├── CardHeader (标题)
│           ├── Timeline (时间线)
│           └── CardFooter (建议)
│
└── Page: popup-setting.astro (设置)
    ├── SettingCard (启动设置)
    │   ├── ToggleSwitch (自动启动)
    │   └── Input (启动文本)
    │
    ├── SettingCard (定时提醒)
    │   ├── ToggleSwitch (功能开关)
    │   ├── TimeRangePicker (时间范围)
    │   └── Input (间隔时长)
    │
    └── SettingCard (无变化提醒)
        ├── ToggleSwitch (功能开关)
        └── Input (无变化时长)
```

---

## 🎨 组件设计原则

### 1. 单一职责
每个组件只负责一个明确的功能。

```typescript
// ✅ 好的设计
<DatePicker onDateChange={handleDateChange} />

// ❌ 不好的设计（组件职责过多）
<DatePickerWithMemoryList onDateChange={handleDateChange} />
```

### 2. Props 类型化
所有组件必须有严格的 TypeScript 类型定义。

```typescript
interface MemoryCardProps {
  memory: ShortTermMemory;
  onSelect?: (id: string) => void;
  className?: string;
}

export function MemoryCard({ memory, onSelect, className }: MemoryCardProps) {
  // ...
}
```

### 3. 可组合性
通过组合小组件构建大组件。

```typescript
<SettingCard title="定时提醒">
  <ToggleSwitch enabled={enabled} onChange={handleToggle} />
  <TimeRangePicker range={timeRange} onChange={handleRangeChange} />
  <Input type="number" label="间隔（分钟）" value={interval} />
</SettingCard>
```

### 4. 受控 vs 非受控
优先使用受控组件，状态由父组件管理。

```typescript
// ✅ 受控组件（推荐）
<DatePicker value={selectedDate} onChange={setSelectedDate} />

// ⚠️ 非受控组件（仅特殊情况）
<DatePicker defaultValue={new Date()} />
```

---

## 📝 组件开发规范

### 文件结构

```
components/
├── FloatingOrb/
│   ├── FloatingOrb.tsx       # 组件主文件
│   ├── FloatingOrb.test.tsx  # 单元测试
│   ├── FloatingOrb.stories.tsx # Storybook（可选）
│   └── index.ts              # 导出
```

### 组件模板

```typescript
// components/MemoryCard/MemoryCard.tsx
import { type FC } from 'react';
import { motion } from 'framer-motion';

interface MemoryCardProps {
  memory: ShortTermMemory;
  onSelect?: (id: string) => void;
  className?: string;
}

export const MemoryCard: FC<MemoryCardProps> = ({
  memory,
  onSelect,
  className = ''
}) => {
  const handleClick = () => {
    onSelect?.(memory.id);
  };

  return (
    <motion.div
      className={`memory-card ${className}`}
      onClick={handleClick}
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
    >
      <h3 className="memory-card__title">{memory.title}</h3>
      <p className="memory-card__content">{memory.content}</p>
      <span className="memory-card__time">{memory.timeRange}</span>
    </motion.div>
  );
};
```

### 样式规范

优先使用 Tailwind CSS，复杂样式使用 CSS Modules。

```typescript
// ✅ Tailwind CSS（简单样式）
<div className="flex items-center gap-4 p-4 rounded-lg bg-gray-100">
  ...
</div>

// ✅ CSS Modules（复杂样式）
import styles from './MemoryCard.module.css';

<div className={styles.memoryCard}>
  ...
</div>
```

---

## 🧪 组件测试

### 单元测试示例

```typescript
// components/MemoryCard/MemoryCard.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryCard } from './MemoryCard';

describe('MemoryCard', () => {
  const mockMemory = {
    id: '1',
    title: '晨会',
    content: '团队同步进度',
    timeRange: '09:00-10:00'
  };

  it('renders memory data correctly', () => {
    render(<MemoryCard memory={mockMemory} />);

    expect(screen.getByText('晨会')).toBeInTheDocument();
    expect(screen.getByText('团队同步进度')).toBeInTheDocument();
    expect(screen.getByText('09:00-10:00')).toBeInTheDocument();
  });

  it('calls onSelect when clicked', () => {
    const handleSelect = vi.fn();
    render(<MemoryCard memory={mockMemory} onSelect={handleSelect} />);

    fireEvent.click(screen.getByText('晨会'));
    expect(handleSelect).toHaveBeenCalledWith('1');
  });
});
```

---

## 📊 组件状态

| 组件 | 状态 | 测试覆盖率 | 文档 | 负责人 |
|------|------|-----------|------|-------|
| FloatingOrb | ✅ 完成 | 85% | ✅ | - |
| Header | 🚧 开发中 | 0% | ✅ | - |
| Asker | 🚧 开发中 | 0% | ✅ | - |
| MemoryList | 📝 规划中 | - | ✅ | - |
| MemoryCard | 📝 规划中 | - | ✅ | - |
| DatePicker | 📝 规划中 | - | ✅ | - |
| FloatingInput | 📝 规划中 | - | ✅ | - |
| SettingCard | 📝 规划中 | - | ✅ | - |

---

## 🔗 相关文档

- [前端架构](../architecture.md)
- [状态管理](../state-management.md)
- [样式规范](../styling.md)
- [动画设计](../animations.md)

---

**组件库维护者**: Vision-Jarvis 前端团队
**最后更新**: 2026-02-04

# Header 组件

> **组件名称**: Header (悬停展开模式)
> **状态**: 🚧 开发中
> **最后更新**: 2026-02-04

---

## 📋 概述

Header 组件是 FloatingOrb 的**悬停态**，当用户鼠标悬停在悬浮球上时展开，提供三个快捷操作入口：
1. **全局记忆开关** - 一键开启/关闭记忆功能
2. **记忆管理** - 跳转到记忆管理页面
3. **提醒设置** - 跳转到提醒设置页面

---

## 🎨 UI 设计

### 视觉规范

```
┌─────────────────────────────────────────┐
│         Header 展开布局                  │
└─────────────────────────────────────────┘

┌───────────────────────────────────────┐
│ 🧠 [全局记忆: ON]  📋 记忆  ⚙️ 设置    │
└───────────────────────────────────────┘
 │                    │        │
 └─ Toggle            │        └─ 设置按钮
                      └─ 记忆管理按钮

尺寸:
- 宽度: 300px
- 高度: 50px
- 圆角: 25px

背景:
- 渐变: linear-gradient(135deg, #667eea, #764ba2)
- 透明度: 95%
- 阴影: 0 8px 24px rgba(0,0,0,0.2)

布局:
- Display: flex
- Gap: 16px
- Padding: 0 20px
- Align: center
```

---

## 🔧 组件 API

### Props

```typescript
interface HeaderProps {
  /** 全局记忆开关状态 */
  memoryEnabled?: boolean;

  /** 全局记忆开关回调 */
  onToggleMemory?: (enabled: boolean) => void;

  /** 点击记忆管理回调 */
  onMemoryClick?: () => void;

  /** 点击设置回调 */
  onSettingClick?: () => void;

  /** 自定义类名 */
  className?: string;
}
```

### 默认值

```typescript
const defaultProps: HeaderProps = {
  memoryEnabled: true,
  onToggleMemory: undefined,
  onMemoryClick: undefined,
  onSettingClick: undefined,
  className: ''
};
```

---

## 💻 使用示例

### 基础用法

```typescript
// components/FloatingOrb/FloatingOrb.tsx
import { Header } from './Header';
import { useStore } from '@nanostores/react';
import { memoryEnabled } from '@/stores/memoryStore';
import { navigate } from 'astro:transitions/client';

export function FloatingOrb() {
  const isMemoryEnabled = useStore(memoryEnabled);

  const handleToggleMemory = async (enabled: boolean) => {
    memoryEnabled.set(enabled);
    await invoke('toggle_memory', { enabled });
  };

  const handleMemoryClick = () => {
    navigate('/memory');
  };

  const handleSettingClick = () => {
    navigate('/popup-setting');
  };

  return (
    <Header
      memoryEnabled={isMemoryEnabled}
      onToggleMemory={handleToggleMemory}
      onMemoryClick={handleMemoryClick}
      onSettingClick={handleSettingClick}
    />
  );
}
```

---

## 🏗️ 内部实现

### 组件结构

```typescript
// components/FloatingOrb/Header.tsx
import { motion } from 'framer-motion';
import { ToggleSwitch } from '@/components/ui/ToggleSwitch';
import type { FC } from 'react';

export const Header: FC<HeaderProps> = ({
  memoryEnabled = true,
  onToggleMemory,
  onMemoryClick,
  onSettingClick,
  className = ''
}) => {
  return (
    <motion.div
      className={`header ${className}`}
      initial={{ x: 100, opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      exit={{ x: 100, opacity: 0 }}
      transition={{ duration: 0.3, ease: 'easeOut' }}
      style={{
        width: '300px',
        height: '50px',
        borderRadius: '25px',
        background: 'linear-gradient(135deg, #667eea, #764ba2)',
        boxShadow: '0 8px 24px rgba(0, 0, 0, 0.2)',
        display: 'flex',
        alignItems: 'center',
        gap: '16px',
        padding: '0 20px'
      }}
    >
      {/* 全局记忆开关 */}
      <div className="flex items-center gap-2">
        <span className="text-white text-lg">🧠</span>
        <ToggleSwitch
          enabled={memoryEnabled}
          onChange={onToggleMemory}
          label="全局记忆"
          size="sm"
        />
      </div>

      {/* 分割线 */}
      <div className="w-px h-6 bg-white/30" />

      {/* 记忆管理按钮 */}
      <button
        onClick={onMemoryClick}
        className="header-btn"
        aria-label="记忆管理"
      >
        <span className="text-lg">📋</span>
        <span className="text-white text-sm">记忆</span>
      </button>

      {/* 设置按钮 */}
      <button
        onClick={onSettingClick}
        className="header-btn"
        aria-label="提醒设置"
      >
        <span className="text-lg">⚙️</span>
        <span className="text-white text-sm">设置</span>
      </button>
    </motion.div>
  );
};
```

### 样式定义

```css
/* components/FloatingOrb/Header.module.css */
.header-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  cursor: pointer;
  transition: all 0.2s ease;
}

.header-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(255, 255, 255, 0.3);
  transform: translateY(-1px);
}

.header-btn:active {
  transform: translateY(0);
}
```

---

## 🎬 动画效果

### 入场动画

```typescript
const headerAnimation = {
  initial: { x: 100, opacity: 0 },
  animate: {
    x: 0,
    opacity: 1,
    transition: {
      duration: 0.3,
      ease: 'easeOut'
    }
  },
  exit: {
    x: 100,
    opacity: 0,
    transition: {
      duration: 0.2,
      ease: 'easeIn'
    }
  }
};
```

### 按钮悬停动画

```typescript
<motion.button
  whileHover={{
    scale: 1.05,
    backgroundColor: 'rgba(255, 255, 255, 0.25)'
  }}
  whileTap={{ scale: 0.95 }}
  transition={{ duration: 0.15 }}
>
```

---

## 🔗 状态同步

### 与 Nanostores 集成

```typescript
// stores/memoryStore.ts
import { atom } from 'nanostores';
import { invoke } from '@tauri-apps/api/core';

export const memoryEnabled = atom<boolean>(true);

// 初始化时从后端获取状态
export async function initMemoryState() {
  const enabled = await invoke<boolean>('get_memory_status');
  memoryEnabled.set(enabled);
}

// 切换记忆状态
export async function toggleMemory(enabled: boolean) {
  memoryEnabled.set(enabled);

  try {
    await invoke('toggle_memory', { enabled });
  } catch (error) {
    // 失败时回滚
    memoryEnabled.set(!enabled);
    console.error('Failed to toggle memory:', error);
  }
}
```

### Header 组件使用 Store

```typescript
import { useStore } from '@nanostores/react';
import { memoryEnabled, toggleMemory } from '@/stores/memoryStore';

export const Header: FC<HeaderProps> = () => {
  const isMemoryEnabled = useStore(memoryEnabled);

  return (
    <Header
      memoryEnabled={isMemoryEnabled}
      onToggleMemory={toggleMemory}
    />
  );
};
```

---

## 📱 响应式设计

```typescript
// 根据屏幕宽度调整布局
const getHeaderLayout = () => {
  const width = window.innerWidth;

  if (width < 768) {
    return {
      width: '250px',
      height: '45px',
      fontSize: '0.875rem',
      gap: '12px'
    };
  }

  return {
    width: '300px',
    height: '50px',
    fontSize: '1rem',
    gap: '16px'
  };
};
```

---

## ♿ 可访问性

### 键盘导航

```typescript
// 支持 Tab 键切换焦点
<div role="toolbar" aria-label="快捷操作">
  <ToggleSwitch tabIndex={0} />
  <button tabIndex={0}>记忆</button>
  <button tabIndex={0}>设置</button>
</div>
```

### ARIA 标签

```typescript
<button
  onClick={onMemoryClick}
  aria-label="打开记忆管理页面"
  aria-describedby="memory-btn-desc"
>
  <span id="memory-btn-desc" className="sr-only">
    查看和管理您的短期和长期记忆
  </span>
  📋 记忆
</button>
```

---

## 🧪 测试

### 单元测试

```typescript
// components/FloatingOrb/Header.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { Header } from './Header';

describe('Header', () => {
  it('renders all action buttons', () => {
    render(<Header />);

    expect(screen.getByLabelText(/全局记忆/)).toBeInTheDocument();
    expect(screen.getByText('记忆')).toBeInTheDocument();
    expect(screen.getByText('设置')).toBeInTheDocument();
  });

  it('calls onToggleMemory when switch is clicked', () => {
    const handleToggle = vi.fn();
    render(<Header onToggleMemory={handleToggle} memoryEnabled={true} />);

    const toggle = screen.getByRole('switch');
    fireEvent.click(toggle);

    expect(handleToggle).toHaveBeenCalledWith(false);
  });

  it('calls onMemoryClick when memory button is clicked', () => {
    const handleClick = vi.fn();
    render(<Header onMemoryClick={handleClick} />);

    fireEvent.click(screen.getByText('记忆'));
    expect(handleClick).toHaveBeenCalled();
  });

  it('calls onSettingClick when setting button is clicked', () => {
    const handleClick = vi.fn();
    render(<Header onSettingClick={handleClick} />);

    fireEvent.click(screen.getByText('设置'));
    expect(handleClick).toHaveBeenCalled();
  });
});
```

---

## 🎨 主题定制

### 支持自定义主题

```typescript
interface HeaderTheme {
  background: string;
  textColor: string;
  borderRadius: string;
  shadow: string;
}

const defaultTheme: HeaderTheme = {
  background: 'linear-gradient(135deg, #667eea, #764ba2)',
  textColor: '#ffffff',
  borderRadius: '25px',
  shadow: '0 8px 24px rgba(0, 0, 0, 0.2)'
};

// 使用主题
<Header theme={customTheme} />
```

---

## 📝 待实现功能

- [ ] 支持自定义按钮顺序
- [ ] 添加更多快捷操作（可配置）
- [ ] 支持键盘快捷键（Ctrl+M 打开记忆，Ctrl+S 打开设置）
- [ ] 添加通知红点提示
- [ ] 支持主题切换（亮色/暗色）

---

## 🔗 相关组件

- [FloatingOrb](FloatingOrb.md) - 父组件
- [ToggleSwitch](ToggleSwitch.md) - 开关组件
- [Asker](Asker.md) - 问答组件

---

**组件维护者**: 前端团队
**测试覆盖率**: 0% (待开发)
**最后更新**: 2026-02-04

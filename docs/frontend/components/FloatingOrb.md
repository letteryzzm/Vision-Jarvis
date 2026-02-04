# FloatingOrb 组件

> **组件名称**: FloatingOrb (悬浮球)
> **状态**: ✅ 已完成
> **最后更新**: 2026-02-04

---

## 📋 概述

FloatingOrb 是 Vision-Jarvis 的核心入口组件，以**圆形悬浮球**的形式常驻桌面，支持三种交互状态：
1. **Idle** - 默认态（圆形悬浮球）
2. **Header** - 悬停态（展开快捷操作）
3. **Asker** - 点击态（展开问答界面）

---

## 🎨 UI 设计

### 视觉规范

```
┌───────────────────────────────────────────┐
│          FloatingOrb 三态设计              │
└───────────────────────────────────────────┘

1. Idle 态（默认）
   ┌────┐
   │ ●  │  40x40px 圆形
   └────┘
   - 背景: rgba(59, 130, 246, 0.8)
   - 阴影: 0 4px 12px rgba(0,0,0,0.15)
   - 位置: 屏幕右侧中央
   - 可拖动

2. Header 态（鼠标悬停）
   ┌─────────────────────────────────┐
   │ 🧠 [全局记忆: ON] 📋 记忆 ⚙️ 设置 │
   └─────────────────────────────────┘
   - 宽度: 300px
   - 高度: 50px
   - 动画: slideInFromRight (300ms)
   - 背景: linear-gradient(135deg, #667eea, #764ba2)

3. Asker 态（点击）
   ┌───────────────────────────────┐
   │ 🤖 问我任何记忆相关问题...     │
   │                               │
   │ ┌─────────────────────────┐   │
   │ │ 搜索框                   │   │
   │ └─────────────────────────┘   │
   │                               │
   │ 历史对话:                      │
   │ Q: 上午做了什么?               │
   │ A: 您上午主要进行了...         │
   └───────────────────────────────┘
   - 宽度: 400px
   - 高度: 500px
   - 动画: slideInFromBottom (400ms)
```

---

## 🔧 组件 API

### Props

```typescript
interface FloatingOrbProps {
  /** 初始位置 */
  initialPosition?: { x: number; y: number };

  /** 是否可拖动 */
  draggable?: boolean;

  /** 默认状态 */
  defaultState?: 'idle' | 'header' | 'asker';

  /** 状态改变回调 */
  onStateChange?: (state: 'idle' | 'header' | 'asker') => void;

  /** 自定义类名 */
  className?: string;
}
```

### 默认值

```typescript
const defaultProps: FloatingOrbProps = {
  initialPosition: { x: window.innerWidth - 60, y: window.innerHeight / 2 },
  draggable: true,
  defaultState: 'idle',
  onStateChange: undefined,
  className: ''
};
```

---

## 💻 使用示例

### 基础用法

```typescript
// src/pages/index.astro
---
import Layout from '@/layouts/Layout.astro';
import FloatingOrb from '@/components/FloatingOrb';
---

<Layout title="Vision-Jarvis">
  <FloatingOrb client:load />
</Layout>
```

### 自定义初始位置

```typescript
<FloatingOrb
  client:load
  initialPosition={{ x: 100, y: 200 }}
  draggable={true}
/>
```

### 监听状态变化

```typescript
<FloatingOrb
  client:load
  onStateChange={(state) => {
    console.log('FloatingOrb state changed to:', state);
  }}
/>
```

---

## 🏗️ 内部实现

### 状态管理

```typescript
// components/FloatingOrb/FloatingOrb.tsx
import { useState, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Header } from './Header';
import { Asker } from './Asker';

type FloatingState = 'idle' | 'header' | 'asker';

export function FloatingOrb({
  initialPosition = { x: window.innerWidth - 60, y: window.innerHeight / 2 },
  draggable = true,
  defaultState = 'idle',
  onStateChange
}: FloatingOrbProps) {
  const [state, setState] = useState<FloatingState>(defaultState);
  const [position, setPosition] = useState(initialPosition);
  const isDragging = useRef(false);

  // 状态转换逻辑
  const handleMouseEnter = () => {
    if (state === 'idle') {
      setState('header');
      onStateChange?.('header');
    }
  };

  const handleMouseLeave = () => {
    if (state === 'header' && !isDragging.current) {
      setState('idle');
      onStateChange?.('idle');
    }
  };

  const handleClick = () => {
    if (state !== 'asker') {
      setState('asker');
      onStateChange?.('asker');
    }
  };

  const handleClickOutside = () => {
    if (state === 'asker') {
      setState('idle');
      onStateChange?.('idle');
    }
  };

  // 拖动逻辑
  const handleDragEnd = (event: any, info: any) => {
    setPosition({
      x: position.x + info.offset.x,
      y: position.y + info.offset.y
    });
    isDragging.current = false;
  };

  const handleDragStart = () => {
    isDragging.current = true;
  };

  return (
    <motion.div
      className="floating-orb-container"
      style={{
        position: 'fixed',
        left: position.x,
        top: position.y,
        zIndex: 9999
      }}
      drag={draggable}
      dragMomentum={false}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onClick={handleClick}
    >
      <AnimatePresence mode="wait">
        {state === 'idle' && <IdleOrb key="idle" />}
        {state === 'header' && <Header key="header" />}
        {state === 'asker' && <Asker key="asker" onClose={handleClickOutside} />}
      </AnimatePresence>
    </motion.div>
  );
}
```

### 子组件: IdleOrb (默认态)

```typescript
// components/FloatingOrb/IdleOrb.tsx
import { motion } from 'framer-motion';

export function IdleOrb() {
  return (
    <motion.div
      className="idle-orb"
      initial={{ scale: 0.8, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      exit={{ scale: 0.8, opacity: 0 }}
      transition={{ duration: 0.2 }}
      style={{
        width: '40px',
        height: '40px',
        borderRadius: '50%',
        background: 'rgba(59, 130, 246, 0.8)',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'pointer'
      }}
    >
      <span className="text-white text-xl">🧠</span>
    </motion.div>
  );
}
```

---

## 🎬 动画效果

### 状态转换动画

```typescript
const animations = {
  idle: {
    initial: { scale: 0.8, opacity: 0 },
    animate: { scale: 1, opacity: 1 },
    exit: { scale: 0.8, opacity: 0 },
    transition: { duration: 0.2 }
  },

  header: {
    initial: { x: 100, opacity: 0 },
    animate: { x: 0, opacity: 1 },
    exit: { x: 100, opacity: 0 },
    transition: { duration: 0.3, ease: 'easeOut' }
  },

  asker: {
    initial: { y: 100, opacity: 0 },
    animate: { y: 0, opacity: 1 },
    exit: { y: 100, opacity: 0 },
    transition: { duration: 0.4, ease: 'easeOut' }
  }
};
```

### 拖动反馈

```typescript
// 拖动时添加视觉反馈
<motion.div
  drag
  whileDrag={{
    scale: 1.1,
    boxShadow: '0 8px 24px rgba(0, 0, 0, 0.3)'
  }}
  dragElastic={0.1}
  dragConstraints={{
    left: 0,
    right: window.innerWidth - 40,
    top: 0,
    bottom: window.innerHeight - 40
  }}
>
```

---

## 📱 响应式设计

```typescript
// 根据屏幕尺寸调整大小
const getOrbSize = () => {
  const width = window.innerWidth;
  if (width < 768) return { idle: 32, header: 250, asker: 320 };
  if (width < 1024) return { idle: 36, header: 280, asker: 360 };
  return { idle: 40, header: 300, asker: 400 };
};
```

---

## ♿ 可访问性

### 键盘导航

```typescript
// 支持 ESC 键关闭 Asker
useEffect(() => {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && state === 'asker') {
      setState('idle');
    }
  };

  window.addEventListener('keydown', handleKeyDown);
  return () => window.removeEventListener('keydown', handleKeyDown);
}, [state]);
```

### ARIA 标签

```typescript
<motion.div
  role="button"
  tabIndex={0}
  aria-label="Vision-Jarvis 悬浮助手"
  aria-expanded={state !== 'idle'}
  aria-haspopup="dialog"
>
```

---

## 🧪 测试

### 单元测试

```typescript
// components/FloatingOrb/FloatingOrb.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { FloatingOrb } from './FloatingOrb';

describe('FloatingOrb', () => {
  it('renders in idle state by default', () => {
    render(<FloatingOrb />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('switches to header state on mouse enter', () => {
    render(<FloatingOrb />);
    const orb = screen.getByRole('button');

    fireEvent.mouseEnter(orb);
    expect(screen.getByText(/全局记忆/)).toBeInTheDocument();
  });

  it('switches to asker state on click', () => {
    render(<FloatingOrb />);
    const orb = screen.getByRole('button');

    fireEvent.click(orb);
    expect(screen.getByPlaceholderText(/问我任何/)).toBeInTheDocument();
  });

  it('calls onStateChange callback', () => {
    const handleStateChange = vi.fn();
    render(<FloatingOrb onStateChange={handleStateChange} />);

    fireEvent.mouseEnter(screen.getByRole('button'));
    expect(handleStateChange).toHaveBeenCalledWith('header');
  });
});
```

---

## 🐛 已知问题

| 问题 | 优先级 | 状态 | 计划修复版本 |
|------|-------|------|-------------|
| 多显示器拖动位置计算错误 | 🟡 中 | 已知 | v1.1 |
| 拖动时鼠标指针偏移 | 🟢 低 | 已知 | v1.2 |

---

## 📝 待优化

- [ ] 添加拖动边界检测（防止拖出屏幕）
- [ ] 支持贴边收起功能
- [ ] 添加双击快速切换状态
- [ ] 支持自定义主题颜色
- [ ] 添加悬浮球动效（呼吸灯、脉冲等）

---

## 🔗 相关组件

- [Header](Header.md) - Header 展开模式
- [Asker](Asker.md) - 问答对话组件

---

## 📚 参考资料

- [Framer Motion Drag API](https://www.framer.com/motion/gestures/#drag)
- [React useRef Hook](https://react.dev/reference/react/useRef)
- [AnimatePresence](https://www.framer.com/motion/animate-presence/)

---

**组件维护者**: 前端团队
**测试覆盖率**: 85%
**最后更新**: 2026-02-04

# Asker 组件

> **组件名称**: Asker (AI 问答对话组件)
> **状态**: 🚧 开发中
> **最后更新**: 2026-02-04

---

## 📋 概述

Asker 组件是 FloatingOrb 的**点击态**，当用户点击悬浮球时展开，提供基于向量搜索的记忆问答功能。支持多轮对话，帮助用户快速检索和理解历史记忆。

---

## 🎨 UI 设计

### 视觉规范

```
┌───────────────────────────────────────┐
│        Asker 问答界面布局              │
└───────────────────────────────────────┘

┌─────────────────────────────────────┐
│  🤖 Vision-Jarvis 记忆助手     [×]  │ ← Header
├─────────────────────────────────────┤
│                                     │
│  💬 对话历史区域                     │
│                                     │
│  ┌───────────────────────────────┐ │
│  │ 👤 我: 上午做了什么?          │ │
│  └───────────────────────────────┘ │
│                                     │
│  ┌───────────────────────────────┐ │
│  │ 🤖 AI: 您上午主要进行了以下   │ │
│  │      活动：                   │ │
│  │      1. 09:00-10:00 团队晨会  │ │
│  │      2. 10:15-11:30 代码开发  │ │
│  └───────────────────────────────┘ │
│                                     │
├─────────────────────────────────────┤
│  ┌───────────────────────────────┐ │ ← Input Area
│  │ 输入框...              [发送] │ │
│  └───────────────────────────────┘ │
└─────────────────────────────────────┘

尺寸:
- 宽度: 400px
- 高度: 500px
- 圆角: 16px

背景:
- 背景色: #ffffff (亮色) / #1f2937 (暗色)
- 阴影: 0 20px 40px rgba(0,0,0,0.3)

布局:
- Header: 60px
- Chat Area: flex-1 (auto)
- Input Area: 80px
```

---

## 🔧 组件 API

### Props

```typescript
interface AskerProps {
  /** 关闭回调 */
  onClose?: () => void;

  /** 初始对话历史 */
  initialMessages?: Message[];

  /** 是否显示欢迎消息 */
  showWelcome?: boolean;

  /** 自定义占位符文本 */
  placeholder?: string;

  /** 最大高度 */
  maxHeight?: number;

  /** 自定义类名 */
  className?: string;
}

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}
```

### 默认值

```typescript
const defaultProps: AskerProps = {
  onClose: undefined,
  initialMessages: [],
  showWelcome: true,
  placeholder: '问我任何记忆相关的问题...',
  maxHeight: 500,
  className: ''
};
```

---

## 💻 使用示例

### 基础用法

```typescript
// components/FloatingOrb/FloatingOrb.tsx
import { Asker } from './Asker';

export function FloatingOrb() {
  const [state, setState] = useState<'idle' | 'header' | 'asker'>('idle');

  const handleClose = () => {
    setState('idle');
  };

  return (
    <>
      {state === 'asker' && <Asker onClose={handleClose} />}
    </>
  );
}
```

### 带初始消息

```typescript
const initialMessages: Message[] = [
  {
    id: '1',
    role: 'assistant',
    content: '你好！我是 Vision-Jarvis 记忆助手，有什么可以帮你的吗？',
    timestamp: new Date()
  }
];

<Asker
  initialMessages={initialMessages}
  showWelcome={false}
  onClose={handleClose}
/>
```

---

## 🏗️ 内部实现

### 组件结构

```typescript
// components/FloatingOrb/Asker.tsx
import { useState, useRef, useEffect } from 'react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { FloatingInput } from '@/components/FloatingInput';
import { ChatMessage } from './ChatMessage';
import type { FC } from 'react';

export const Asker: FC<AskerProps> = ({
  onClose,
  initialMessages = [],
  showWelcome = true,
  placeholder = '问我任何记忆相关的问题...',
  maxHeight = 500,
  className = ''
}) => {
  const [messages, setMessages] = useState<Message[]>(
    showWelcome
      ? [
          {
            id: 'welcome',
            role: 'assistant',
            content: '你好！我能帮你检索和分析历史记忆，请问有什么问题吗？',
            timestamp: new Date()
          },
          ...initialMessages
        ]
      : initialMessages
  );
  const [isLoading, setIsLoading] = useState(false);
  const chatEndRef = useRef<HTMLDivElement>(null);

  // 自动滚动到底部
  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // 发送消息
  const handleSendMessage = async (content: string) => {
    if (!content.trim()) return;

    // 添加用户消息
    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content,
      timestamp: new Date()
    };
    setMessages((prev) => [...prev, userMessage]);

    // 调用 AI 接口
    setIsLoading(true);
    try {
      const response = await invoke<string>('ask_memory', {
        query: content,
        history: messages
      });

      // 添加 AI 回复
      const aiMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: response,
        timestamp: new Date()
      };
      setMessages((prev) => [...prev, aiMessage]);
    } catch (error) {
      console.error('Failed to ask memory:', error);

      // 添加错误消息
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: '抱歉，查询记忆时出现错误，请稍后再试。',
        timestamp: new Date()
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <motion.div
      className={`asker ${className}`}
      initial={{ y: 100, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      exit={{ y: 100, opacity: 0 }}
      transition={{ duration: 0.4, ease: 'easeOut' }}
      style={{
        width: '400px',
        maxHeight: `${maxHeight}px`,
        borderRadius: '16px',
        background: '#ffffff',
        boxShadow: '0 20px 40px rgba(0, 0, 0, 0.3)',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden'
      }}
    >
      {/* Header */}
      <div className="asker-header">
        <div className="flex items-center gap-2">
          <span className="text-2xl">🤖</span>
          <h3 className="text-lg font-semibold">记忆助手</h3>
        </div>
        <button
          onClick={onClose}
          className="close-btn"
          aria-label="关闭"
        >
          ×
        </button>
      </div>

      {/* Chat Area */}
      <div className="asker-chat-area">
        {messages.map((message) => (
          <ChatMessage key={message.id} message={message} />
        ))}
        {isLoading && (
          <div className="loading-indicator">
            <span className="dot"></span>
            <span className="dot"></span>
            <span className="dot"></span>
          </div>
        )}
        <div ref={chatEndRef} />
      </div>

      {/* Input Area */}
      <FloatingInput
        placeholder={placeholder}
        onSend={handleSendMessage}
        disabled={isLoading}
      />
    </motion.div>
  );
};
```

### ChatMessage 子组件

```typescript
// components/FloatingOrb/ChatMessage.tsx
import { motion } from 'framer-motion';
import { formatTime } from '@/utils/time';

interface ChatMessageProps {
  message: Message;
}

export function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === 'user';

  return (
    <motion.div
      className={`chat-message ${isUser ? 'user' : 'assistant'}`}
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
    >
      <div className="message-avatar">
        {isUser ? '👤' : '🤖'}
      </div>
      <div className="message-content">
        <p className="message-text">{message.content}</p>
        <span className="message-time">
          {formatTime(message.timestamp)}
        </span>
      </div>
    </motion.div>
  );
}
```

---

## 🎬 动画效果

### 入场动画

```typescript
const askerAnimation = {
  initial: { y: 100, opacity: 0, scale: 0.95 },
  animate: {
    y: 0,
    opacity: 1,
    scale: 1,
    transition: {
      duration: 0.4,
      ease: 'easeOut'
    }
  },
  exit: {
    y: 100,
    opacity: 0,
    scale: 0.95,
    transition: {
      duration: 0.3,
      ease: 'easeIn'
    }
  }
};
```

### 消息动画

```typescript
const messageAnimation = {
  initial: { opacity: 0, y: 10 },
  animate: {
    opacity: 1,
    y: 0,
    transition: {
      duration: 0.3,
      ease: 'easeOut'
    }
  }
};
```

### 加载动画

```css
/* Loading dots animation */
.loading-indicator {
  display: flex;
  gap: 4px;
  padding: 16px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #667eea;
  animation: bounce 1.4s infinite ease-in-out both;
}

.dot:nth-child(1) {
  animation-delay: -0.32s;
}

.dot:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes bounce {
  0%, 80%, 100% {
    transform: scale(0);
  }
  40% {
    transform: scale(1);
  }
}
```

---

## 🔗 后端集成

### Tauri Command 调用

```typescript
// 向量搜索问答
async function askMemory(query: string, history: Message[]) {
  try {
    const response = await invoke<string>('ask_memory', {
      query,
      history: history.map(m => ({
        role: m.role,
        content: m.content
      }))
    });
    return response;
  } catch (error) {
    throw new Error(`Failed to ask memory: ${error}`);
  }
}
```

### Rust 后端示例

```rust
// src-tauri/src/commands/memory.rs
#[tauri::command]
pub async fn ask_memory(
    query: String,
    history: Vec<ChatMessage>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. 向量搜索相关记忆
    let memories = state.vector_db
        .search(&query, 5)
        .await
        .map_err(|e| e.to_string())?;

    // 2. 构建 AI 提示词
    let context = format_memories_context(&memories);
    let prompt = build_prompt(&query, &context, &history);

    // 3. 调用 AI API
    let response = state.ai_client
        .chat(&prompt)
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}
```

---

## 📱 响应式设计

```typescript
// 根据屏幕尺寸调整布局
const getAskerLayout = () => {
  const width = window.innerWidth;
  const height = window.innerHeight;

  if (width < 768) {
    return {
      width: '320px',
      maxHeight: `${Math.min(height - 100, 400)}px`
    };
  }

  if (width < 1024) {
    return {
      width: '360px',
      maxHeight: `${Math.min(height - 100, 450)}px`
    };
  }

  return {
    width: '400px',
    maxHeight: `${Math.min(height - 100, 500)}px`
  };
};
```

---

## ♿ 可访问性

### 键盘导航

```typescript
// 支持 ESC 键关闭
useEffect(() => {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose?.();
    }
  };

  window.addEventListener('keydown', handleKeyDown);
  return () => window.removeEventListener('keydown', handleKeyDown);
}, [onClose]);

// 支持 Enter 发送消息
<input
  onKeyDown={(e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }}
/>
```

### ARIA 标签

```typescript
<div
  role="dialog"
  aria-label="记忆问答对话框"
  aria-modal="true"
>
  <div
    role="log"
    aria-live="polite"
    aria-relevant="additions"
  >
    {/* 对话历史 */}
  </div>
</div>
```

---

## 🧪 测试

### 单元测试

```typescript
// components/FloatingOrb/Asker.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Asker } from './Asker';
import { vi } from 'vitest';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('Asker', () => {
  it('renders welcome message by default', () => {
    render(<Asker />);
    expect(screen.getByText(/你好/)).toBeInTheDocument();
  });

  it('sends message and displays response', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    (invoke as any).mockResolvedValue('这是 AI 的回复');

    render(<Asker />);

    const input = screen.getByPlaceholderText(/问我任何/);
    const sendBtn = screen.getByText('发送');

    fireEvent.change(input, { target: { value: '上午做了什么?' } });
    fireEvent.click(sendBtn);

    await waitFor(() => {
      expect(screen.getByText('上午做了什么?')).toBeInTheDocument();
      expect(screen.getByText('这是 AI 的回复')).toBeInTheDocument();
    });
  });

  it('calls onClose when close button is clicked', () => {
    const handleClose = vi.fn();
    render(<Asker onClose={handleClose} />);

    fireEvent.click(screen.getByLabelText('关闭'));
    expect(handleClose).toHaveBeenCalled();
  });
});
```

---

## 📝 待实现功能

- [ ] 消息引用功能（点击记忆卡片跳转详情）
- [ ] 消息编辑和删除
- [ ] 导出对话历史
- [ ] 语音输入支持
- [ ] 代码高亮显示
- [ ] Markdown 渲染
- [ ] 消息搜索功能
- [ ] 对话分支功能

---

## 🔗 相关组件

- [FloatingOrb](FloatingOrb.md) - 父组件
- [FloatingInput](FloatingInput.md) - 输入框组件
- [Header](Header.md) - Header 组件

---

## 📚 参考资料

- [ChatGPT UI Design](https://chat.openai.com/)
- [Framer Motion Variants](https://www.framer.com/motion/animation/#variants)
- [React useRef for Scroll](https://react.dev/reference/react/useRef)

---

**组件维护者**: 前端团队
**测试覆盖率**: 0% (待开发)
**最后更新**: 2026-02-04

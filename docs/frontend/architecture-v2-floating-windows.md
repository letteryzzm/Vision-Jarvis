# Frontend Architecture V2 - Floating Windows System

> **文档版本**: v2.0
> **创建日期**: 2026-02-06
> **状态**: Active
> **架构类型**: Multi-Window Floating Ball Architecture

---

## 目录

- [概览 (Overview)](#概览-overview)
- [架构演进](#架构演进)
- [窗口结构 (Window Structure)](#窗口结构-window-structure)
- [状态管理 (State Management)](#状态管理-state-management)
- [交互流程 (Interaction Flow)](#交互流程-interaction-flow)
- [技术实现细节 (Technical Implementation)](#技术实现细节-technical-implementation)
- [性能优化 (Performance Optimizations)](#性能优化-performance-optimizations)
- [文件组织 (File Organization)](#文件组织-file-organization)
- [关键设计决策](#关键设计决策)

---

## 概览 (Overview)

Vision-Jarvis V2 采用**多窗口悬浮球架构**（Multi-Window Floating Ball Architecture），从单页面应用重构为独立的多窗口系统，实现 macOS 风格的悬浮窗口交互。

### 核心特性

- **悬浮球主窗口**: 64x64 像素圆形球体，始终置顶，支持三种状态切换
- **独立功能窗口**: Memory 和 Popup-Setting 作为独立窗口运行
- **渐进式交互**: 悬停展开 Header，点击展开 Asker，实现无干扰的用户体验
- **窗口通信**: 基于 Tauri 2 的多窗口 API 和窗口事件管理

### 架构示意图

```
┌─────────────────────────────────────────────────────────────┐
│                   Vision-Jarvis V2 架构                      │
│              (Multi-Window Floating Ball System)             │
└─────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ Floating Ball│   │    Memory    │   │Popup-Setting │
│   Window     │   │   Window     │   │   Window     │
│  (Main)      │   │ (Independent)│   │ (Independent)│
└──────────────┘   └──────────────┘   └──────────────┘
│                   │                   │
│ • 64x64px        │ • 1200x800px     │ • 900x700px
│ • Always on top  │ • Resizable      │ • Resizable
│ • Transparent    │ • Full UI        │ • Card Layout
│ • 3 States:      │ • Standalone     │ • Settings
│   - Ball         │                   │
│   - Header       │                   │
│   - Asker        │                   │
└──────────────────┴───────────────────┴──────────────┘
```

---

## 架构演进

### V1 (单页面应用) → V2 (多窗口系统)

| 维度 | V1 架构 | V2 架构 | 改进点 |
|------|---------|---------|--------|
| **窗口模式** | 单窗口，所有功能在一个页面 | 多窗口，功能独立分离 | 更好的关注点分离 |
| **主界面** | 全屏应用 | 悬浮球（64x64px） | 减少视觉干扰 |
| **交互方式** | 点击导航链接 | 渐进式展开（悬停/点击） | 更流畅的用户体验 |
| **记忆管理** | 页面内组件 | 独立窗口（1200x800） | 可同时操作多个窗口 |
| **设置页面** | 页面内组件 | 独立窗口（900x700） | 并行配置和使用 |
| **置顶能力** | 无 | 主窗口始终置顶 | 随时可访问 |
| **透明效果** | 无 | 支持 macOS 透明 | 更好的视觉融合 |

### 重构范围

**Phase 1**: Tauri 窗口配置
- 配置多窗口系统（`tauri.conf.json`）
- 添加窗口管理 Commands（`window.rs`）

**Phase 2**: 悬浮球主窗口
- Ball 组件（初始状态 64x64）
- Header 组件（悬停展开 360x72）
- Asker 组件（点击展开 360x480）
- 状态管理和交互逻辑

**Phase 3**: Memory 窗口重构
- 从页面组件改为独立窗口布局
- 完整的记忆管理 UI（1200x800）

**Phase 4**: Popup-Setting 窗口重构
- 从页面组件改为独立窗口布局
- 卡片式设置页面（900x700）

**Phase 5**: 集成测试和优化
- 端到端测试（11项测试清单）
- 性能优化（防抖、GPU加速）
- Bug 修复（窗口标签、透明度）

---

## 窗口结构 (Window Structure)

### 主窗口: Floating Ball

#### 配置（tauri.conf.json）

```json
{
  "label": "floating-ball",
  "title": "Vision Jarvis",
  "url": "/floating-ball",
  "width": 64,
  "height": 64,
  "resizable": false,
  "decorations": false,
  "transparent": true,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "visible": true,
  "x": 1800,
  "y": 50
}
```

#### 三种状态

| 状态 | 尺寸 | 触发方式 | 内容 | 动画时长 |
|------|------|----------|------|----------|
| **Ball** | 64x64 | 默认状态 | 圆形悬浮球 + 脑图标 | - |
| **Header** | 360x72 | 鼠标悬停 (200ms 延迟) | 记忆开关、记忆按钮、提醒按钮 | 300ms ease-out |
| **Asker** | 360x480 | 点击悬浮球 | AI 对话界面（输入框、历史记录） | 400ms ease-out |

#### 特性

- **始终置顶** (`alwaysOnTop: true`): 浮于所有窗口之上
- **透明背景** (`transparent: true`): 与桌面融合
- **无边框** (`decorations: false`): 自定义拖动
- **跳过任务栏** (`skipTaskbar: true`): 不显示在任务栏

---

### Memory Window

#### 配置

```rust
WebviewWindowBuilder::new(
    &app,
    "memory",
    WebviewUrl::App("/memory".into())
)
.title("记忆管理 - Vision Jarvis")
.inner_size(1200.0, 800.0)
.resizable(true)
.center()
.build()
```

#### 布局结构

```
┌─────────────────────────────────────────────────┐
│          Memory Window (1200x800)               │
├────────────┬────────────────────────────────────┤
│  Sidebar   │      Main Content Area             │
│  (320px)   │      (880px)                       │
│            │                                     │
│ • 记忆开关 │  Search Bar (Always on top)        │
│ • 日期选择 │  ┌──────────────────────────────┐  │
│ • 短期记忆 │  │ 🔍 搜索记忆...                │  │
│   列表     │  └──────────────────────────────┘  │
│ • 设置项   │                                     │
│   - 截屏频率│  Default State:                    │
│   - 存储设置│  ┌──────────────────────────────┐  │
│            │  │      🧠                       │  │
│            │  │  想找哪段记忆                 │  │
│            │  │  我都记着呢，随便问           │  │
│            │  └──────────────────────────────┘  │
└────────────┴────────────────────────────────────┘
```

#### 功能

- **左侧边栏**: 日期选择、短期记忆列表、设置控件
- **右侧主区域**: 搜索栏、记忆内容展示、时间线视图
- **可独立操作**: 不影响悬浮球窗口，可同时运行

---

### Popup-Setting Window

#### 配置

```rust
WebviewWindowBuilder::new(
    &app,
    "popup-setting",
    WebviewUrl::App("/popup-setting".into())
)
.title("提醒设置 - Vision Jarvis")
.inner_size(900.0, 700.0)
.resizable(true)
.center()
.build()
```

#### 布局结构

```
┌─────────────────────────────────────────────────┐
│      Popup-Setting Window (900x700)             │
├─────────────────────────────────────────────────┤
│  Header                                         │
│  ┌───────────────────────────────────────────┐ │
│  │ 提醒设置                                   │ │
│  │ 配置您的智能提醒和通知                     │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  Card 1: 启动提醒                               │
│  ┌───────────────────────────────────────────┐ │
│  │ [Toggle] 开机自动启动                     │ │
│  │ [TextArea] 启动提醒文本                   │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  Card 2: 定时提醒                               │
│  ┌───────────────────────────────────────────┐ │
│  │ [Time Range] 工作时间段                   │ │
│  │ [Slider] 提醒间隔 (5-120分钟)             │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  Card 3: 空闲检测                               │
│  ┌───────────────────────────────────────────┐ │
│  │ [Slider] 空闲判定时长 (5-60分钟)          │ │
│  │ [Select] 提醒内容类型                     │ │
│  └───────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

#### 设置分类

1. **启动提醒**: 开机自启、启动消息
2. **定时提醒**: 工作时段、提醒间隔
3. **空闲检测**: 空闲时长、提醒类型

---

## 状态管理 (State Management)

### WindowState 类型定义

```typescript
type WindowState = 'ball' | 'header' | 'asker';
```

### 状态机设计

```
     ┌─────────────────────────────────┐
     │        Initial State            │
     │          (Ball)                 │
     └──────────┬──────────────────────┘
                │
    ┌───────────┴───────────┐
    │                       │
    │ Mouse Enter           │ Click
    │ (200ms delay)         │
    │                       │
    ▼                       ▼
┌──────────┐           ┌──────────┐
│  Header  │           │  Asker   │
│ (360x72) │           │(360x480) │
└────┬─────┘           └────┬─────┘
     │                      │
     │ Mouse Leave          │ Click Outside
     │ (300ms delay)        │ or ESC
     │                      │
     └──────────┬───────────┘
                │
                ▼
         ┌──────────┐
         │   Ball   │
         │ (64x64)  │
         └──────────┘
```

### 状态转换逻辑

```typescript
async function switchTo(state: WindowState) {
  if (currentState === state) return;

  currentState = state;

  // 隐藏所有状态
  ballState?.classList.add('hidden');
  headerState?.classList.add('hidden');
  askerState?.classList.add('hidden');

  // 显示目标状态并调用后端命令
  switch (state) {
    case 'ball':
      ballState?.classList.remove('hidden');
      await invoke('collapse_to_ball');
      break;
    case 'header':
      headerState?.classList.remove('hidden');
      await invoke('expand_to_header');
      break;
    case 'asker':
      askerState?.classList.remove('hidden');
      await invoke('expand_to_asker');
      break;
  }
}
```

### 事件处理

| 事件 | 当前状态 | 目标状态 | 延迟 | Tauri Command |
|------|----------|----------|------|---------------|
| `mouseenter` | ball | header | 200ms | `expand_to_header` |
| `mouseleave` | header | ball | 300ms | `collapse_to_ball` |
| `click` (球) | ball/header | asker | 0ms | `expand_to_asker` |
| `click` (外部) | asker | ball | 0ms | `collapse_to_ball` |
| `ESC` 键 | asker | ball | 0ms | `collapse_to_ball` |

---

## 交互流程 (Interaction Flow)

### 1. 应用启动流程

```
User launches app
    ↓
Tauri creates floating-ball window
    ↓
Load /floating-ball route
    ↓
Show Ball state (64x64)
    ↓
Position at top-right (x:1800, y:50)
    ↓
Apply: transparent, alwaysOnTop, skipTaskbar
    ↓
Ready for interaction
```

### 2. 悬浮球交互流程

```
User hovers over Ball
    ↓
Wait 200ms (debounced)
    ↓
invoke('expand_to_header')
    ↓
Tauri resizes window to 360x72
    ↓
Show Header component
    ↓
User can:
  - Toggle memory
  - Click "记忆" → open_memory_window
  - Click "提醒" → open_popup_setting_window
    ↓
User moves mouse away
    ↓
Wait 300ms
    ↓
invoke('collapse_to_ball')
    ↓
Tauri resizes window to 64x64
    ↓
Show Ball component
```

### 3. 打开 Memory 窗口流程

```
User clicks "记忆" button in Header
    ↓
invoke('open_memory_window')
    ↓
Rust checks if window exists
    ↓
IF exists:
  window.set_focus() → Bring to front
ELSE:
  WebviewWindowBuilder::new(...)
  .label("memory")
  .url("/memory")
  .inner_size(1200, 800)
  .build()
    ↓
Memory window opens/focuses
    ↓
Floating ball remains in current state
```

### 4. 打开 Popup-Setting 窗口流程

```
User clicks "提醒" button in Header
    ↓
invoke('open_popup_setting_window')
    ↓
Rust checks if window exists
    ↓
IF exists:
  window.set_focus() → Bring to front
ELSE:
  WebviewWindowBuilder::new(...)
  .label("popup-setting")
  .url("/popup-setting")
  .inner_size(900, 700)
  .build()
    ↓
Popup-Setting window opens/focuses
    ↓
Floating ball remains in current state
```

### 5. Asker 交互流程

```
User clicks Ball
    ↓
invoke('expand_to_asker')
    ↓
Tauri resizes window to 360x480
    ↓
Show Asker component
    ↓
User types question
    ↓
User clicks send or presses Enter
    ↓
invoke('search_memories', { query })
    ↓
Rust performs vector search
    ↓
Return results to frontend
    ↓
Display AI response in chat
    ↓
User clicks outside or presses ESC
    ↓
invoke('collapse_to_ball')
    ↓
Back to Ball state
```

---

## 技术实现细节 (Technical Implementation)

### Tauri Commands

#### 窗口管理命令

```rust
// src-tauri/src/commands/window.rs

#[tauri::command]
pub async fn expand_to_header(app: AppHandle) -> Result<ApiResponse<bool>, String> {
    if let Some(window) = app.get_webview_window("floating-ball") {
        window.set_size(PhysicalSize::new(360, 72))
            .map_err(|e| e.to_string())?;
        Ok(ApiResponse::success(true))
    } else {
        Err("Floating ball window not found".to_string())
    }
}

#[tauri::command]
pub async fn expand_to_asker(app: AppHandle) -> Result<ApiResponse<bool>, String> {
    if let Some(window) = app.get_webview_window("floating-ball") {
        window.set_size(PhysicalSize::new(360, 480))
            .map_err(|e| e.to_string())?;
        Ok(ApiResponse::success(true))
    } else {
        Err("Floating ball window not found".to_string())
    }
}

#[tauri::command]
pub async fn collapse_to_ball(app: AppHandle) -> Result<ApiResponse<bool>, String> {
    if let Some(window) = app.get_webview_window("floating-ball") {
        window.set_size(PhysicalSize::new(64, 64))
            .map_err(|e| e.to_string())?;
        Ok(ApiResponse::success(true))
    } else {
        Err("Floating ball window not found".to_string())
    }
}

#[tauri::command]
pub async fn open_memory_window(app: AppHandle) -> Result<ApiResponse<bool>, String> {
    if let Some(window) = app.get_webview_window("memory") {
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(ApiResponse::success(true));
    }

    WebviewWindowBuilder::new(
        &app,
        "memory",
        WebviewUrl::App("/memory".into())
    )
    .title("记忆管理 - Vision Jarvis")
    .inner_size(1200.0, 800.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(true))
}

#[tauri::command]
pub async fn open_popup_setting_window(app: AppHandle) -> Result<ApiResponse<bool>, String> {
    if let Some(window) = app.get_webview_window("popup-setting") {
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(ApiResponse::success(true));
    }

    WebviewWindowBuilder::new(
        &app,
        "popup-setting",
        WebviewUrl::App("/popup-setting".into())
    )
    .title("提醒设置 - Vision Jarvis")
    .inner_size(900.0, 700.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(true))
}
```

### 前端页面实现

#### Floating Ball 页面结构

```astro
---
// vision-jarvis/src/pages/floating-ball.astro
import Layout from '../layouts/Layout.astro';
import Ball from '../components/FloatingBall/Ball.astro';
import Header from '../components/FloatingBall/Header.astro';
import Asker from '../components/FloatingBall/Asker.astro';
---

<Layout>
  <div id="floating-container" class="w-full h-full">
    <!-- Ball State (default) -->
    <div id="ball-state">
      <Ball />
    </div>

    <!-- Header State (on hover) -->
    <div id="header-state" class="hidden">
      <Header />
    </div>

    <!-- Asker State (on click) -->
    <div id="asker-state" class="hidden">
      <Asker />
    </div>
  </div>

  <script>
    // State management and event handlers
    // (See State Management section for details)
  </script>
</Layout>
```

#### Ball 组件

```astro
<!-- vision-jarvis/src/components/FloatingBall/Ball.astro -->
<div
  id="floating-ball"
  class="w-16 h-16 rounded-full gradient-primary flex items-center justify-center cursor-pointer transition-all duration-300 hover:scale-110 pulse-glow"
>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="32"
    height="32"
    viewBox="0 0 24 24"
    fill="none"
    stroke="white"
    stroke-width="2"
    class="lucide lucide-brain"
  >
    <!-- Brain icon paths -->
  </svg>
</div>

<style>
  #floating-ball {
    -webkit-app-region: drag;
  }
</style>
```

---

## 性能优化 (Performance Optimizations)

### 1. 防抖 (Debounce)

```typescript
// 悬停事件防抖，避免频繁调用
let hoverTimer: NodeJS.Timeout | null = null;

floatingBall?.addEventListener('mouseenter', () => {
  if (currentState !== 'ball') return;

  hoverTimer = setTimeout(() => {
    switchTo('header');
  }, 200); // 200ms 延迟
});

floatingBall?.addEventListener('mouseleave', () => {
  if (hoverTimer) {
    clearTimeout(hoverTimer);
    hoverTimer = null;
  }
});
```

### 2. GPU 加速

```css
.transition-all {
  will-change: transform, opacity;
  transform: translateZ(0); /* 强制GPU加速 */
}
```

### 3. 窗口复用

```rust
// 检查窗口是否已存在，避免重复创建
if let Some(window) = app.get_webview_window("memory") {
    window.set_focus().map_err(|e| e.to_string())?;
    return Ok(ApiResponse::success(true));
}

// 仅在窗口不存在时创建新窗口
WebviewWindowBuilder::new(...)
```

### 4. 按需渲染

```astro
<!-- 仅在需要时渲染复杂组件 -->
<div id="asker-state" class="hidden">
  <Asker />
</div>

<!-- 通过显示/隐藏而非销毁/创建来切换状态 -->
<script>
  askerState?.classList.remove('hidden'); // Show
  askerState?.classList.add('hidden');    // Hide
</script>
```

### 5. CSS 动画优化

```css
/* 使用 transform 和 opacity，避免 layout reflow */
.expand-animation {
  transition: transform 300ms ease-out, opacity 300ms ease-out;
}

/* 避免使用 width/height 动画 */
/* ❌ Bad */
.bad {
  transition: width 300ms;
}

/* ✅ Good */
.good {
  transition: transform 300ms;
  transform: scaleX(1.5);
}
```

### 性能指标

| 指标 | 目标 | 实际 |
|------|------|------|
| 首次启动时间 | < 1s | ~140ms (路由加载) |
| 状态切换时间 | < 300ms | 200-400ms |
| 空闲时 CPU 占用 | < 5% | < 5% (已验证) |
| 内存占用 | < 100MB | 待测量 |
| 窗口响应时间 | < 100ms | 待测量 |

---

## 文件组织 (File Organization)

### 项目结构

```
vision-jarvis/
├── src/
│   ├── pages/
│   │   ├── floating-ball.astro       # 悬浮球主页面
│   │   ├── memory.astro              # 记忆管理窗口
│   │   └── popup-setting.astro       # 设置窗口
│   │
│   ├── components/
│   │   ├── FloatingBall/
│   │   │   ├── Ball.astro            # 球状态组件
│   │   │   ├── Header.astro          # Header状态组件
│   │   │   └── Asker.astro           # Asker状态组件
│   │   │
│   │   ├── Header/
│   │   │   └── (existing components)
│   │   │
│   │   └── Asker/
│   │       └── (existing components)
│   │
│   └── styles/
│       └── global.css                # 全局样式
│
└── src-tauri/
    ├── src/
    │   ├── commands/
    │   │   ├── window.rs             # 窗口管理命令
    │   │   └── mod.rs
    │   │
    │   └── lib.rs                    # 注册 commands
    │
    └── tauri.conf.json               # 窗口配置
```

### 关键文件说明

| 文件 | 用途 | 行数 |
|------|------|------|
| `tauri.conf.json` | Tauri 窗口配置 | ~46 |
| `commands/window.rs` | 窗口管理命令 | ~130 |
| `pages/floating-ball.astro` | 悬浮球主页面 | ~200+ |
| `components/FloatingBall/Ball.astro` | 球状态 UI | ~50 |
| `components/FloatingBall/Header.astro` | Header 状态 UI | ~70 |
| `components/FloatingBall/Asker.astro` | Asker 状态 UI | ~100 |
| `pages/memory.astro` | 记忆窗口（重构） | ~200 |
| `pages/popup-setting.astro` | 设置窗口（重构） | ~180 |

---

## 关键设计决策

### 1. 为什么选择多窗口而非单窗口?

| 方案 | 优势 | 劣势 | 决策 |
|------|------|------|------|
| **单窗口** | 简单，易管理 | 功能耦合，视觉干扰大 | ❌ |
| **多窗口** | 关注点分离，可并行操作 | 需要窗口通信 | ✅ 选择 |

**原因**:
- 悬浮球需要始终可见且不干扰
- Memory 和 Settings 是重型功能，独立窗口体验更好
- 用户可能需要同时查看记忆和配置设置

### 2. 为什么是 64x64 → 360x72 → 360x480?

| 尺寸 | 设计理由 |
|------|----------|
| **64x64** | 最小可见尺寸，不遮挡内容，符合 macOS 风格 |
| **360x72** | 足够容纳 3 个按钮 + Toggle，单行布局 |
| **360x480** | 常见聊天窗口高度，避免滚动 |

**黄金比例考量**:
- 64:360 = 1:5.625 (接近黄金比例)
- 72:480 = 1:6.67 (合理的高宽比)

### 3. 为什么悬停用 200ms，离开用 300ms?

**研究依据**:
- **200ms**: 人类感知延迟阈值，感觉即时但不误触
- **300ms**: 给用户足够时间移动到 Header 按钮，避免过早折叠

**用户体验**:
- 悬停 → 快速响应
- 离开 → 容错时间

### 4. 为什么 Memory 是 1200x800，Settings 是 900x700?

| 窗口 | 尺寸 | 设计理由 |
|------|------|----------|
| **Memory** | 1200x800 | 双栏布局（Sidebar + Content），需要横向空间展示时间线 |
| **Settings** | 900x700 | 单栏卡片布局，更窄即可，避免卡片过宽 |

**参考标准**:
- 1200x800: 接近 15" MacBook 的 80% 屏幕宽度
- 900x700: 接近 13" MacBook 的 70% 屏幕宽度

### 5. 为什么选择 Astro 而非纯 React?

| 方案 | 打包体积 | 首屏性能 | 交互能力 | 决策 |
|------|----------|----------|----------|------|
| **纯 React** | ~150KB | 慢 | 强 | ❌ |
| **Astro + React Islands** | ~50KB | 快 | 强 | ✅ 选择 |

**原因**:
- 悬浮球需要极快加载
- 大部分内容是静态的（Ball UI）
- 仅交互部分需要 JS

---

## 测试和验证

### E2E 测试清单

已通过测试（2026-02-06）:

1. ✅ 应用启动显示悬浮球（右上角）
2. ✅ 悬浮球可以拖动
3. ✅ 鼠标悬停展开为 Header
4. ✅ Header 显示记忆toggle、记忆按钮、提醒按钮
5. ✅ 点击悬浮球展开为 Asker
6. ✅ 点击外部区域折叠回悬浮球
7. ✅ 点击"记忆"按钮打开 Memory 窗口
8. ✅ 点击"提醒"按钮打开 Popup-Setting 窗口
9. ✅ Memory 窗口可以独立操作
10. ✅ Popup-Setting 窗口可以独立操作
11. ✅ 悬浮球始终保持在最顶层

### 已修复的关键 Bug

**Bug #1: 窗口标签不匹配** (CRITICAL - 已修复)
- **问题**: tauri.conf.json 使用 "floating-ball"，但 window.rs 引用 "main"
- **影响**: 所有窗口 resize 命令失败
- **修复**: 更新所有 window.rs 中的窗口引用为 "floating-ball"

**Bug #2: macOS 透明度警告** (MEDIUM - 已修复)
- **问题**: `transparent: true` 但未启用 macOS private API
- **影响**: 透明效果可能无法正常工作
- **修复**: 添加 `"macOSPrivateApi": true` 到 tauri.conf.json

### 测试报告

详见: [E2E Test Report](../../testing/test-reports/2026-02-06-floating-ball-e2e.md)

---

## 未来改进方向

### 计划中的功能

1. **快捷键支持**: 全局快捷键显示/隐藏悬浮球
2. **多屏支持**: 记住每个屏幕的悬浮球位置
3. **主题切换**: 支持浅色/深色主题
4. **动画增强**: 更流畅的状态切换动画
5. **窗口记忆**: 记住用户调整的窗口大小和位置

### 性能优化方向

1. **虚拟滚动**: Memory 窗口的长列表优化
2. **懒加载**: Asker 组件按需加载
3. **缓存策略**: 记忆数据本地缓存
4. **增量更新**: 仅更新变化的数据

---

## 相关文档

- [Frontend Architecture V1](architecture.md) - 单页面架构（已废弃）
- [FloatingOrb Component](components/FloatingOrb.md)
- [Header Component](components/Header.md)
- [Asker Component](components/Asker.md)
- [Memory Page](pages/memory.md)
- [Popup-Setting Page](pages/popup-setting.md)
- [E2E Test Report](../testing/test-reports/2026-02-06-floating-ball-e2e.md)
- [Implementation Plan](../plans/2026-02-06-floating-ball-multi-window-architecture.md)

---

**文档维护者**: Vision-Jarvis 架构团队
**审核状态**: ✅ 已审核
**最后更新**: 2026-02-06
**架构版本**: V2.0

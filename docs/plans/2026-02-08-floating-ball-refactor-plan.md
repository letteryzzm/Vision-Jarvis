# 悬浮球系统重构开发计划

> **创建日期**: 2026-02-08
> **状态**: Planning
> **优先级**: High
> **目标**: 确保丝滑的用户交互体验和良好的可扩展性

---

## 📋 目录

1. [现状分析](#现状分析)
2. [核心问题](#核心问题)
3. [重构目标](#重构目标)
4. [技术方案](#技术方案)
5. [实施计划](#实施计划)
6. [测试验证](#测试验证)
7. [可扩展性设计](#可扩展性设计)

---

## 现状分析

### ✅ 已完成的部分

1. **Tauri 窗口配置** (基础完成)
   - 主窗口 64x64 配置 ✅
   - 透明背景 ✅
   - 始终置顶 ✅
   - macOS 私有 API 启用 ✅

2. **窗口管理命令** (基础完成)
   - `expand_to_header` ✅
   - `expand_to_asker` ✅
   - `collapse_to_ball` ✅
   - `open_memory_window` ✅
   - `open_popup_setting_window` ✅

3. **组件结构** (基础完成)
   - Ball.astro ✅
   - Header.astro ✅
   - Asker.astro ✅
   - floating-ball.astro 主页面 ✅

4. **状态管理** (基础完成)
   - WindowState 类型定义 ✅
   - switchTo 函数 ✅
   - 防抖处理 ✅

---

## 核心问题

### 🔴 Critical Issues (必须修复)

#### 1. 窗口位置问题
**现状**: 窗口使用 `center: true`，启动时在屏幕中央
**期望**: 右上角固定位置
**影响**: 违背设计初衷，用户体验差

**问题代码**:
```json
// tauri.conf.json
{
  "center": true  // ❌ 错误
}
```

#### 2. 布局容器问题
**现状**: 使用 `100vw x 100vh` 容器包裹组件
**期望**: 组件直接填充窗口，无额外容器
**影响**: 可能导致透明区域响应事件

**问题代码**:
```html
<!-- floating-ball.astro -->
<div id="floating-container" style="width: 100vw; height: 100vh;">
  <!-- ❌ 不必要的容器 -->
</div>
```

#### 3. 组件定位问题
**现状**: Ball、Header、Asker 三个状态需要正确定位和尺寸匹配
**期望**:
- Ball: 64x64，窗口完全填充
- Header: 360x72，窗口调整后完全填充
- Asker: 360x480，窗口调整后完全填充

**影响**: 当前可能出现组件偏移或尺寸不匹配

#### 4. 交互状态管理问题
**现状**: 没有处理窗口大小变化时的位置保持
**期望**: 窗口展开/收起时，保持相对位置（例如右上角）
**影响**: 用户体验不佳，窗口可能跳动

### 🟡 Important Issues (需要优化)

#### 5. 动画缺失
**现状**: 只有 CSS transition，没有平滑的尺寸变化动画
**期望**: 丝滑的展开/收起动画
**影响**: 用户体验不够流畅

#### 6. 事件处理优化
**现状**:
- 双重延迟（hoverTimer + debounce）
- Header 的 mouseleave 判断逻辑复杂

**期望**: 简化事件处理逻辑，确保响应灵敏

#### 7. Toggle 开关未实现功能
**现状**: Header 中的 memory-toggle 只是 UI，没有功能
**期望**: 点击后真正控制全局记忆功能

#### 8. 缺少加载状态
**现状**: 窗口打开时没有加载指示
**期望**: 显示加载状态，避免用户重复点击

### 🟢 Enhancement Issues (可扩展性)

#### 9. 组件耦合度高
**现状**: floating-ball.astro 包含所有逻辑
**期望**: 提取状态管理到独立模块

#### 10. 缺少错误处理
**现状**: invoke 调用没有错误处理
**期望**: 完善的错误处理和用户提示

#### 11. 缺少键盘快捷键
**现状**: 只能鼠标操作
**期望**: 支持键盘快捷键（ESC关闭等）

---

## 重构目标

### 核心目标

1. **丝滑的交互体验**
   - 平滑的展开/收起动画（300ms ease-out）
   - 无闪烁、无跳动
   - 快速响应（<100ms）

2. **稳定的窗口定位**
   - 启动时固定右上角
   - 展开/收起保持位置
   - 多显示器支持

3. **完善的功能实现**
   - Toggle 开关真实功能
   - 错误处理和重试机制
   - 加载状态显示

4. **良好的可扩展性**
   - 模块化设计
   - 状态管理独立
   - 易于添加新状态/组件

---

## 技术方案

### 方案 A: 窗口定位策略（推荐）

**问题**: Tauri 2 移除了窗口配置中的 `position` 字段

**解决方案**: 使用 Rust 后端代码设置窗口位置

```rust
// src-tauri/src/lib.rs

use tauri::{LogicalPosition, Manager};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 获取主窗口
            let window = app.get_webview_window("floating-ball").unwrap();

            // 获取主显示器尺寸
            if let Some(monitor) = window.primary_monitor()? {
                if let Some(size) = monitor.size() {
                    // 计算右上角位置（距离右边缘 20px，距离顶部 50px）
                    let x = size.width as f64 - 64.0 - 20.0;
                    let y = 50.0;

                    window.set_position(LogicalPosition::new(x, y))?;
                }
            }

            Ok(())
        })
        // ... rest of setup
}
```

**优点**:
- 动态计算，适配不同分辨率
- 支持多显示器
- 可以保存用户自定义位置

### 方案 B: 组件布局优化

**当前问题**: 容器嵌套导致复杂性

**解决方案**: 简化 HTML 结构

```html
<!-- 优化后的 floating-ball.astro -->
<body style="margin: 0; padding: 0; background: transparent; overflow: hidden;">
  <!-- Ball State -->
  <div id="ball-state">
    <Ball />
  </div>

  <!-- Header State -->
  <div id="header-state" class="hidden">
    <Header />
  </div>

  <!-- Asker State -->
  <div id="asker-state" class="hidden">
    <Asker />
  </div>
</body>
```

**组件 CSS**:
```css
#ball-state,
#header-state,
#asker-state {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

#ball-state > *,
#header-state > *,
#asker-state > * {
  /* 组件自身负责尺寸 */
}
```

### 方案 C: 窗口尺寸变化动画

**问题**: Tauri 窗口 resize 没有动画

**解决方案**:
1. 后端快速调整窗口大小
2. 前端 CSS 动画处理内容过渡

```typescript
async function switchTo(state: WindowState) {
  if (currentState === state) return;

  const previousState = currentState;
  currentState = state;

  // 1. 先隐藏当前状态（淡出）
  const currentElement = getStateElement(previousState);
  currentElement?.classList.add('fade-out');

  await new Promise(resolve => setTimeout(resolve, 150));

  // 2. 调整窗口大小
  await invoke(getResizeCommand(state));

  // 3. 显示新状态（淡入）
  currentElement?.classList.add('hidden');
  currentElement?.classList.remove('fade-out');

  const nextElement = getStateElement(state);
  nextElement?.classList.remove('hidden');
  nextElement?.classList.add('fade-in');

  await new Promise(resolve => setTimeout(resolve, 150));
  nextElement?.classList.remove('fade-in');
}
```

```css
.fade-out {
  opacity: 0;
  transform: scale(0.95);
  transition: opacity 150ms ease-out, transform 150ms ease-out;
}

.fade-in {
  opacity: 0;
  transform: scale(1.05);
  animation: fadeIn 150ms ease-out forwards;
}

@keyframes fadeIn {
  to {
    opacity: 1;
    transform: scale(1);
  }
}
```

### 方案 D: 状态管理重构

**问题**: 状态逻辑分散，难以维护

**解决方案**: 创建独立的状态管理模块

```typescript
// src/lib/floating-ball-state.ts

type WindowState = 'ball' | 'header' | 'asker';

interface StateTransition {
  from: WindowState;
  to: WindowState;
  animation: 'expand' | 'collapse' | 'none';
  duration: number;
}

class FloatingBallStateManager {
  private currentState: WindowState = 'ball';
  private isTransitioning: boolean = false;
  private listeners: Map<string, Function[]> = new Map();

  async transitionTo(newState: WindowState): Promise<void> {
    if (this.isTransitioning || this.currentState === newState) {
      return;
    }

    this.isTransitioning = true;

    const transition = this.getTransition(this.currentState, newState);

    // Emit beforeTransition event
    this.emit('beforeTransition', { from: this.currentState, to: newState });

    try {
      // Perform transition
      await this.performTransition(transition);

      const oldState = this.currentState;
      this.currentState = newState;

      // Emit afterTransition event
      this.emit('afterTransition', { from: oldState, to: newState });
    } catch (error) {
      this.emit('transitionError', { error, from: this.currentState, to: newState });
      throw error;
    } finally {
      this.isTransitioning = false;
    }
  }

  private async performTransition(transition: StateTransition): Promise<void> {
    // Hide current state
    await this.hideState(transition.from);

    // Resize window
    await this.resizeWindow(transition.to);

    // Show new state
    await this.showState(transition.to);
  }

  getCurrentState(): WindowState {
    return this.currentState;
  }

  isInTransition(): boolean {
    return this.isTransitioning;
  }

  on(event: string, callback: Function): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(callback);
  }

  private emit(event: string, data: any): void {
    const callbacks = this.listeners.get(event) || [];
    callbacks.forEach(cb => cb(data));
  }

  // ... more methods
}

export const stateManager = new FloatingBallStateManager();
```

### 方案 E: Toggle 功能实现

**需求**: Header 中的���忆开关需要真实控制功能

**后端 Tauri Command**:
```rust
// src-tauri/src/commands/memory.rs

use std::sync::Mutex;
use tauri::State;

pub struct MemoryState {
    enabled: Mutex<bool>,
}

#[tauri::command]
pub async fn toggle_memory(state: State<'_, MemoryState>) -> ApiResponse<bool> {
    let mut enabled = state.enabled.lock().unwrap();
    *enabled = !*enabled;

    let new_state = *enabled;

    // TODO: 启动或停止截屏和分析服务
    // if new_state {
    //     start_capture_service();
    // } else {
    //     stop_capture_service();
    // }

    ApiResponse::success(new_state)
}

#[tauri::command]
pub async fn get_memory_state(state: State<'_, MemoryState>) -> ApiResponse<bool> {
    let enabled = state.enabled.lock().unwrap();
    ApiResponse::success(*enabled)
}
```

**前端实现**:
```typescript
// Toggle 点击事件
const memoryToggle = document.querySelector('.memory-toggle');
let isMemoryEnabled = false;

// 初始化：获取当前状态
async function initMemoryToggle() {
  const response = await invoke('get_memory_state');
  isMemoryEnabled = response.data;
  updateToggleUI(isMemoryEnabled);
}

memoryToggle?.addEventListener('click', async () => {
  try {
    const response = await invoke('toggle_memory');
    isMemoryEnabled = response.data;
    updateToggleUI(isMemoryEnabled);

    // 显示提示
    showToast(isMemoryEnabled ? '记忆功能已开启' : '记忆功能已关闭');
  } catch (error) {
    showToast('操作失败，请重试', 'error');
  }
});

function updateToggleUI(enabled: boolean) {
  if (enabled) {
    memoryToggle?.classList.remove('justify-end');
    memoryToggle?.classList.add('justify-start');
    memoryToggle?.classList.add('gradient-success');
  } else {
    memoryToggle?.classList.remove('justify-start');
    memoryToggle?.classList.add('justify-end');
    memoryToggle?.classList.remove('gradient-success');
  }
}
```

---

## 实施计划

### Phase 1: 核心问题修复 (优先级: Critical)

**目标**: 修复窗口定位和布局问题

#### Task 1.1: 窗口位置修复
**文件**: `src-tauri/src/lib.rs`, `tauri.conf.json`

**步骤**:
1. 移除 `tauri.conf.json` 中的 `center: true`
2. 在 `lib.rs` 的 `setup` 钩子中添加位置计算逻辑
3. 实现窗口位置保存/恢复功能（可选）

**代码**:
```rust
.setup(|app| {
    let window = app.get_webview_window("floating-ball").unwrap();

    // 获取主显示器
    if let Ok(Some(monitor)) = window.primary_monitor() {
        if let Some(size) = monitor.size() {
            // 右上角: 距右边缘 20px，距顶部 50px
            let x = (size.width as f64 - 84.0).max(0.0); // 64 + 20
            let y = 50.0;

            let _ = window.set_position(LogicalPosition::new(x, y));
        }
    }

    Ok(())
})
```

**验证**:
- [ ] 应用启动时悬浮球出现在右上角
- [ ] 多显示器环境下位置正确
- [ ] 窗口展开/收起后位置不变

---

#### Task 1.2: 布局容器简化
**文件**: `src/pages/floating-ball.astro`

**步骤**:
1. 移除 `floating-container` div
2. 直接在 body 下放置状态容器
3. 调整 CSS 确保组件正确填充

**Before**:
```html
<body>
  <div id="floating-container" style="width: 100vw; height: 100vh;">
    <div id="ball-state">...</div>
  </div>
</body>
```

**After**:
```html
<body style="margin: 0; padding: 0; overflow: hidden; background: transparent;">
  <div id="ball-state">
    <Ball />
  </div>
  <div id="header-state" class="hidden">
    <Header />
  </div>
  <div id="asker-state" class="hidden">
    <Asker />
  </div>
</body>

<style>
  #ball-state,
  #header-state,
  #asker-state {
    width: 100%;
    height: 100%;
  }
</style>
```

**验证**:
- [ ] Ball 状态完全填充 64x64 窗口
- [ ] Header 状态完全填充 360x72 窗口
- [ ] Asker 状态完全填充 360x480 窗口
- [ ] 无透明区域误触问题

---

#### Task 1.3: 组件尺寸匹配
**文件**: `src/components/FloatingBall/*.astro`

**步骤**:
1. Ball.astro: 确保 64x64
2. Header.astro: 确保 360x72
3. Asker.astro: 确保 360x480

**Ball.astro**:
```astro
<div
  id="floating-ball"
  class="w-16 h-16 rounded-full gradient-primary flex items-center justify-center cursor-pointer"
  style="width: 64px; height: 64px;"
>
  <!-- SVG icon -->
</div>

<style>
  #floating-ball {
    -webkit-app-region: drag;
  }
</style>
```

**Header.astro**:
```astro
<div
  id="header-expanded"
  class="bg-card rounded-[40px] border border-primary flex items-center gap-4 px-5"
  style="width: 360px; height: 72px;"
>
  <!-- Toggle + Buttons -->
</div>
```

**Asker.astro**:
```astro
<div
  id="asker-expanded"
  class="bg-card rounded-[32px] border border-primary flex flex-col"
  style="width: 360px; height: 480px;"
>
  <!-- Header + Messages + Input -->
</div>
```

**验证**:
- [ ] 每个组件尺寸与窗口完全匹配
- [ ] 无滚动条
- [ ] 无裁剪或溢出

---

### Phase 2: 动画和交互优化 (优先级: Important)

#### Task 2.1: 丝滑展开/收起动画
**文件**: `src/pages/floating-ball.astro`

**步骤**:
1. 添加状态过渡动画
2. 优化窗口尺寸变化时机
3. 添加淡入淡出效果

**实现**:
```typescript
// 优化后的 switchTo 函数
async function switchTo(state: WindowState) {
  if (currentState === state || isTransitioning) return;

  isTransitioning = true;
  const previousState = currentState;

  try {
    // 1. 淡出当前状态
    const currentEl = getStateElement(previousState);
    currentEl?.classList.add('transitioning-out');

    await delay(150);

    // 2. 调整窗口大小
    await invoke(getResizeCommand(state));

    // 3. 切换 DOM 显示
    currentEl?.classList.add('hidden');
    currentEl?.classList.remove('transitioning-out');

    const nextEl = getStateElement(state);
    nextEl?.classList.remove('hidden');
    nextEl?.classList.add('transitioning-in');

    await delay(150);

    nextEl?.classList.remove('transitioning-in');

    currentState = state;
  } catch (error) {
    console.error('Transition failed:', error);
    // Rollback
    currentState = previousState;
  } finally {
    isTransitioning = false;
  }
}

function getStateElement(state: WindowState): HTMLElement | null {
  return document.getElementById(`${state}-state`);
}

function getResizeCommand(state: WindowState): string {
  const commands = {
    'ball': 'collapse_to_ball',
    'header': 'expand_to_header',
    'asker': 'expand_to_asker'
  };
  return commands[state];
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
```

**CSS**:
```css
.transitioning-out {
  opacity: 0;
  transform: scale(0.95);
  transition: opacity 150ms ease-out, transform 150ms ease-out;
}

.transitioning-in {
  opacity: 0;
  transform: scale(1.05);
  animation: fadeIn 150ms ease-out forwards;
}

@keyframes fadeIn {
  to {
    opacity: 1;
    transform: scale(1);
  }
}
```

**验证**:
- [ ] 展开动画流畅，无闪烁
- [ ] 收起动画自然
- [ ] 总动画时长约 300ms
- [ ] 不同状态切换都有动画

---

#### Task 2.2: 事件处理优化
**���件**: `src/pages/floating-ball.astro`

**步骤**:
1. 移除双重延迟（简化为单一延迟）
2. 优化 mouseleave 逻辑
3. 添加鼠标进入 Header 区域的处理

**Before** (双重延迟):
```typescript
// ❌ 有问题的代码
floatingBall?.addEventListener('mouseenter', () => {
  hoverTimer = setTimeout(() => {
    debouncedExpand();  // 这里又有 200ms debounce
  }, 200);
});
```

**After** (单一延迟):
```typescript
// ✅ 优化后的代码
let hoverTimer: number | null = null;

// Ball 悬停
document.getElementById('ball-state')?.addEventListener('mouseenter', () => {
  if (currentState !== 'ball') return;

  hoverTimer = window.setTimeout(() => {
    switchTo('header');
  }, 200);
});

document.getElementById('ball-state')?.addEventListener('mouseleave', () => {
  if (hoverTimer) {
    clearTimeout(hoverTimer);
    hoverTimer = null;
  }
});

// Header 区域
document.getElementById('header-state')?.addEventListener('mouseenter', () => {
  // 鼠标进入 Header，取消折叠计时器
  if (hoverTimer) {
    clearTimeout(hoverTimer);
    hoverTimer = null;
  }
});

document.getElementById('header-state')?.addEventListener('mouseleave', () => {
  // 鼠标离开 Header，300ms 后折叠
  hoverTimer = window.setTimeout(() => {
    if (currentState === 'header') {
      switchTo('ball');
    }
  }, 300);
});
```

**验证**:
- [ ] 悬停响应时间精确 200ms
- [ ] 鼠标在 Header 内移动不会折叠
- [ ] 离开 Header 300ms 后折叠
- [ ] 无意外的状态切换

---

#### Task 2.3: 键盘快捷键支持
**文件**: `src/pages/floating-ball.astro`

**步骤**:
1. 监听 ESC 键关闭 Asker
2. 支持 Enter 发送消息（Asker 状态）

**实现**:
```typescript
// 全局键盘事件
document.addEventListener('keydown', (e) => {
  // ESC: 折叠回 Ball
  if (e.key === 'Escape') {
    if (currentState !== 'ball') {
      switchTo('ball');
    }
  }

  // Enter: 发送消息（仅在 Asker 状态且输入框聚焦时）
  if (e.key === 'Enter' && currentState === 'asker') {
    const input = document.getElementById('asker-input') as HTMLInputElement;
    if (input && document.activeElement === input && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }
});
```

**验证**:
- [ ] ESC 键能关闭 Asker 和 Header
- [ ] Enter 发送消息
- [ ] Shift+Enter 换行（textarea 的话）

---

### Phase 3: 功能完善 (优先级: Important)

#### Task 3.1: Toggle 开关功能实现
**文件**:
- `src-tauri/src/commands/memory.rs` (新建)
- `src-tauri/src/commands/mod.rs`
- `src/components/FloatingBall/Header.astro`

**后端实现** (`memory.rs`):
```rust
use std::sync::Mutex;
use tauri::State;
use super::ApiResponse;

pub struct MemoryState {
    pub enabled: Mutex<bool>,
}

impl Default for MemoryState {
    fn default() -> Self {
        Self {
            enabled: Mutex::new(false),
        }
    }
}

#[tauri::command]
pub async fn toggle_memory(state: State<'_, MemoryState>) -> ApiResponse<bool> {
    let mut enabled = state.enabled.lock().unwrap();
    *enabled = !*enabled;
    let new_state = *enabled;

    // TODO: 启动/停止截屏服务
    // if new_state {
    //     start_capture_service();
    // } else {
    //     stop_capture_service();
    // }

    ApiResponse::success(new_state)
}

#[tauri::command]
pub async fn get_memory_state(state: State<'_, MemoryState>) -> ApiResponse<bool> {
    let enabled = state.enabled.lock().unwrap();
    ApiResponse::success(*enabled)
}
```

**注册命令** (`lib.rs`):
```rust
mod commands;
use commands::memory::MemoryState;

pub fn run() {
    tauri::Builder::default()
        .manage(MemoryState::default())
        .invoke_handler(tauri::generate_handler![
            // ... existing commands
            commands::memory::toggle_memory,
            commands::memory::get_memory_state,
        ])
        // ...
}
```

**前端实现** (Header.astro):
```typescript
<script>
  import { invoke } from '@tauri-apps/api/core';

  let isMemoryEnabled = false;

  // 初始化
  async function initToggle() {
    try {
      const response = await invoke('get_memory_state');
      isMemoryEnabled = response.data;
      updateToggleUI();
    } catch (error) {
      console.error('Failed to get memory state:', error);
    }
  }

  // Toggle 点击事件
  document.querySelector('.memory-toggle')?.addEventListener('click', async () => {
    try {
      const response = await invoke('toggle_memory');
      isMemoryEnabled = response.data;
      updateToggleUI();
    } catch (error) {
      console.error('Failed to toggle memory:', error);
    }
  });

  function updateToggleUI() {
    const toggle = document.querySelector('.memory-toggle');
    const indicator = toggle?.querySelector('div');

    if (isMemoryEnabled) {
      toggle?.classList.add('gradient-success');
      toggle?.classList.remove('bg-gray-400');
      indicator?.classList.remove('translate-x-0');
      indicator?.classList.add('translate-x-full');
    } else {
      toggle?.classList.remove('gradient-success');
      toggle?.classList.add('bg-gray-400');
      indicator?.classList.add('translate-x-0');
      indicator?.classList.remove('translate-x-full');
    }
  }

  initToggle();
</script>
```

**验证**:
- [ ] Toggle 点击能切换状态
- [ ] UI 正确反映状态（颜色、位置）
- [ ] 刷新后状态保持
- [ ] 后端状态持久化

---

#### Task 3.2: 错误处理和加载状态
**文件**: `src/pages/floating-ball.astro`

**步骤**:
1. 添加 Toast 通知组件
2. 处理 invoke 错误
3. 显示加载状态

**Toast 组件**:
```typescript
// src/lib/toast.ts

interface ToastOptions {
  type: 'success' | 'error' | 'info';
  duration: number;
}

export function showToast(message: string, options: Partial<ToastOptions> = {}) {
  const { type = 'info', duration = 2000 } = options;

  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.textContent = message;

  document.body.appendChild(toast);

  // 动画进入
  requestAnimationFrame(() => {
    toast.classList.add('show');
  });

  // 自动移除
  setTimeout(() => {
    toast.classList.remove('show');
    setTimeout(() => toast.remove(), 300);
  }, duration);
}
```

**CSS**:
```css
.toast {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%) translateY(-100px);
  padding: 12px 24px;
  border-radius: 24px;
  font-size: 14px;
  font-weight: 500;
  opacity: 0;
  transition: all 300ms ease-out;
  z-index: 10000;
  pointer-events: none;
}

.toast.show {
  transform: translateX(-50%) translateY(0);
  opacity: 1;
}

.toast-success {
  background: var(--gradient-success);
  color: white;
}

.toast-error {
  background: #ff4444;
  color: white;
}

.toast-info {
  background: var(--bg-card);
  color: var(--text-primary);
  border: 1px solid var(--border-primary);
}
```

**使用示例**:
```typescript
import { showToast } from '../lib/toast';

// Memory 按钮点击
document.getElementById('memory-btn')?.addEventListener('click', async () => {
  try {
    await invoke('open_memory_window');
    // 成功时不需要 toast，窗口已经打开
  } catch (error) {
    showToast('无法打开记忆窗口，请重试', { type: 'error' });
  }
});
```

**验证**:
- [ ] 错误时显示友好提示
- [ ] Toast 自动消失
- [ ] 多个 Toast 不重叠

---

### Phase 4: 可扩展性设计 (优先级: Enhancement)

#### Task 4.1: 状态管理模块化
**文件**: `src/lib/state-manager.ts` (新建)

**实现**: (见技术方案 D)

**集成**:
```typescript
// floating-ball.astro
import { stateManager } from '../lib/state-manager';

// 监听状态变化
stateManager.on('afterTransition', ({ from, to }) => {
  console.log(`Transitioned from ${from} to ${to}`);
});

// 切换状态
document.getElementById('floating-ball')?.addEventListener('click', () => {
  stateManager.transitionTo('asker');
});
```

**验证**:
- [ ] 状态切换逻辑独立
- [ ] 事件系统工作正常
- [ ] 易于扩展新状态

---

#### Task 4.2: 配置系统
**文件**: `src/lib/config.ts` (新建)

**目的**: 集中管理可配置项

```typescript
// src/lib/config.ts

export const FLOATING_BALL_CONFIG = {
  // 窗口尺寸
  windows: {
    ball: { width: 64, height: 64 },
    header: { width: 360, height: 72 },
    asker: { width: 360, height: 480 },
  },

  // 动画时长
  animations: {
    fadeOut: 150,
    fadeIn: 150,
    hoverDelay: 200,
    collapseDelay: 300,
  },

  // 位置
  position: {
    offsetX: 20,  // 距右边缘
    offsetY: 50,  // 距顶部
  },

  // 功能开关
  features: {
    keyboardShortcuts: true,
    animations: true,
    toast: true,
  },
};

export type FloatingBallConfig = typeof FLOATING_BALL_CONFIG;
```

**使用**:
```typescript
import { FLOATING_BALL_CONFIG as CONFIG } from '../lib/config';

setTimeout(() => {
  switchTo('header');
}, CONFIG.animations.hoverDelay);
```

**验证**:
- [ ] 配置集中管理
- [ ] 易于调整参数
- [ ] 类型安全

---

## 测试验证

### 手动测试清单

#### 基础功能测试
- [ ] 应用启动后悬浮球出现在右上角
- [ ] 悬浮球可以拖动
- [ ] 鼠标悬停 200ms 后展开为 Header
- [ ] Header 显示 Toggle、记忆按钮、提醒按钮
- [ ] 点击悬浮球展开为 Asker
- [ ] 点击外部折叠回 Ball
- [ ] ESC 键折叠回 Ball

#### 窗口管理测试
- [ ] 点击记忆按钮打开 Memory 窗口
- [ ] 点击提醒按钮打开 Popup-Setting 窗口
- [ ] Memory 窗口可独立操作
- [ ] Popup-Setting 窗口可独立操作
- [ ] 重复点击按钮聚焦已有窗口（不重复创建）

#### 交互体验测试
- [ ] 展开/收起动画流畅（无闪烁）
- [ ] 鼠标在 Header 内移动不触发折叠
- [ ] 离开 Header 300ms 后自动折叠
- [ ] Toggle 开关点击响应灵敏
- [ ] 状态切换无卡顿

#### 边界情况测试
- [ ] 多显示器环境下位置正确
- [ ] 快速切换状态不出错
- [ ] 网络错误时有友好提示
- [ ] 窗口最小化后恢复正常

### 性能测试

- [ ] CPU 使用率 < 5% (空闲时)
- [ ] 内存使用 < 50MB
- [ ] 状态切换响应时间 < 100ms
- [ ] 动画帧率 >= 60fps

---

## 可扩展性设计

### 1. 新增状态支持

**示例：添加 "Settings" 状态**

```typescript
// 1. 更新类型定义
type WindowState = 'ball' | 'header' | 'asker' | 'settings';

// 2. 添加组件
// src/components/FloatingBall/Settings.astro

// 3. 在 floating-ball.astro 中添加状态容器
<div id="settings-state" class="hidden">
  <Settings />
</div>

// 4. 添加 Tauri 命令
#[tauri::command]
pub async fn expand_to_settings(app: AppHandle) -> ApiResponse<String> {
    // resize to 360x600
}

// 5. 更新状态管理器
stateManager.registerState('settings', {
  size: { width: 360, height: 600 },
  command: 'expand_to_settings',
});
```

### 2. 插件系统（未来）

**设计思路**:
```typescript
interface FloatingBallPlugin {
  name: string;
  version: string;

  // 添加新状态
  states?: StateDefinition[];

  // 添加 UI 组件
  components?: ComponentDefinition[];

  // 注册事件监听器
  listeners?: EventListener[];

  // 初始化
  init(): void;

  // 销毁
  destroy(): void;
}

// 使用
import myPlugin from './plugins/my-plugin';
stateManager.use(myPlugin);
```

### 3. 主题系统

**支持动态主题切换**:
```typescript
// src/lib/theme.ts
export const themes = {
  default: {
    gradientPrimary: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    gradientSuccess: 'linear-gradient(135deg, #06FF00 0%, #00FFD6 100%)',
    bgCard: '#1E1E1E',
    textPrimary: '#FFFFFF',
  },
  light: {
    // light theme colors
  },
  dark: {
    // dark theme colors
  },
};

export function applyTheme(themeName: keyof typeof themes) {
  const theme = themes[themeName];
  Object.entries(theme).forEach(([key, value]) => {
    document.documentElement.style.setProperty(
      `--${key}`,
      value
    );
  });
}
```

---

## 总结

### 重构优先级

**P0 (必须完成)**:
- Task 1.1: 窗口位置修复
- Task 1.2: 布局容器简化
- Task 1.3: 组件尺寸匹配
- Task 2.1: 丝滑动画

**P1 (高优先级)**:
- Task 2.2: 事件处理优化
- Task 3.1: Toggle 功能实现
- Task 3.2: 错误处理

**P2 (增强功能)**:
- Task 2.3: 键盘快捷键
- Task 4.1: 状态管理模块化
- Task 4.2: 配置系统

### 预估工时

- **Phase 1**: 4-6 小时
- **Phase 2**: 4-6 小时
- **Phase 3**: 3-4 小时
- **Phase 4**: 3-4 小时

**总计**: 14-20 小时（2-3 个工作日）

### 成功标准

✅ **交互体验**:
- 动画流畅，帧率 60fps
- 响应时间 < 100ms
- 无闪烁、无卡顿

✅ **功能完整**:
- Toggle 开关可用
- 所有窗口正常打开
- 错误处理完善

✅ **代码质量**:
- 模块化设计
- 类型安全
- 易于扩展

✅ **用户体验**:
- 符合设计稿
- 操作直观
- 性能优秀

---

**下一步**: 开始 Phase 1 的实施，优先修复核心问题。

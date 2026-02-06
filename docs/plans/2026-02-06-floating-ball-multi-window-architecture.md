# Floating Ball Multi-Window Architecture Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 重构前端架构从单页面应用到悬浮球+多窗口系统，实现 macOS 风格的悬浮窗口交互

**Architecture:**
- 主窗口：64x64 悬浮球，始终置顶，支持鼠标悬停展开 Header、点击展开 Asker
- Memory 窗口：独立窗口，显示记忆管理界面
- Popup-Setting 窗口：独立窗口，显示提醒设置界面
- 使用 Tauri 2 的多窗口 API 和窗口事件管理

**Tech Stack:**
- Tauri 2 (多窗口管理、窗口置顶、窗口事件)
- Astro 5.16 (每个窗口独立页面)
- Tailwind CSS 4 (Vision Jarvis 设计系统)
- TypeScript (窗口通信和状态管理)

**设计参考:** `/Users/lettery/Documents/code/Vision-Jarvis/frontend.pen`

---

## Phase 1: Tauri 窗口配置

### Task 1.1: 配置多窗口系统

**Files:**
- Modify: `vision-jarvis/src-tauri/tauri.conf.json`

**Step 1: 更新 tauri.conf.json 配置主悬浮球窗口**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "vision-jarvis",
  "version": "0.1.0",
  "identifier": "com.lettery.vision-jarvis",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:4321",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
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
        "x": null,
        "y": null,
        "center": false,
        "position": {
          "type": "Physical",
          "x": 1800,
          "y": 50
        }
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

**Step 2: 验证配置**

Run: `cat vision-jarvis/src-tauri/tauri.conf.json | grep -A 20 "windows"`
Expected: 看到悬浮球窗口配置

**Step 3: Commit**

```bash
git add vision-jarvis/src-tauri/tauri.conf.json
git commit -m "feat: configure floating ball main window"
```

---

### Task 1.2: 添加窗口管理 Commands

**Files:**
- Create: `vision-jarvis/src-tauri/src/commands/window.rs`
- Modify: `vision-jarvis/src-tauri/src/commands/mod.rs`
- Modify: `vision-jarvis/src-tauri/src/lib.rs`

**Step 1: 创建窗口管理 commands**

Create `vision-jarvis/src-tauri/src/commands/window.rs`:

```rust
/// 窗口管理 Commands
use super::ApiResponse;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

/// 创建 Memory 窗口
#[tauri::command]
pub async fn open_memory_window(app: AppHandle) -> Result<ApiResponse<bool>, String> {
    // 检查窗口是否已存在
    if let Some(window) = app.get_webview_window("memory") {
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(ApiResponse::success(true));
    }

    // 创建新窗口
    let window = WebviewWindowBuilder::new(
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

/// 创建 Popup Setting 窗口
#[tauri::command]
pub async fn open_popup_setting_window(app: AppHandle) -> Result<ApiResponse<bool>, String> {
    // 检查窗口是否已存在
    if let Some(window) = app.get_webview_window("popup-setting") {
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(ApiResponse::success(true));
    }

    // 创建新窗口
    let window = WebviewWindowBuilder::new(
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

/// 展开悬浮球到 Header 模式
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

/// 展开悬浮球到 Asker 模式
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

/// 折叠悬浮球到原始大小
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

#[cfg(test)]
mod tests {
    use super::*;

    // Note: 窗口测试需要在集成测试中进行
}
```

**Step 2: 更新 commands/mod.rs**

Modify `vision-jarvis/src-tauri/src/commands/mod.rs`:

```rust
pub mod window;
```

Add after line 17.

**Step 3: 注册 commands 到 lib.rs**

Modify `vision-jarvis/src-tauri/src/lib.rs`, add to invoke_handler:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::window::open_memory_window,
    commands::window::open_popup_setting_window,
    commands::window::expand_to_header,
    commands::window::expand_to_asker,
    commands::window::collapse_to_ball,
])
```

**Step 4: 编译检查**

Run: `cd vision-jarvis/src-tauri && cargo check`
Expected: 编译通过，无错误

**Step 5: Commit**

```bash
git add vision-jarvis/src-tauri/src/commands/window.rs \
        vision-jarvis/src-tauri/src/commands/mod.rs \
        vision-jarvis/src-tauri/src/lib.rs
git commit -m "feat: add window management commands"
```

---

## Phase 2: 悬浮球主窗口

### Task 2.1: 创建悬浮球页面

**Files:**
- Create: `vision-jarvis/src/pages/floating-ball.astro`
- Create: `vision-jarvis/src/components/FloatingBall/Ball.astro`
- Create: `vision-jarvis/src/components/FloatingBall/Header.astro`
- Create: `vision-jarvis/src/components/FloatingBall/Asker.astro`

**Step 1: 创建 Ball 组件（初始状态）**

Create `vision-jarvis/src/components/FloatingBall/Ball.astro`:

```astro
---
// 64x64 圆形悬浮球
---

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
    stroke-linecap="round"
    stroke-linejoin="round"
    class="lucide lucide-brain"
  >
    <path d="M12 5a3 3 0 1 0-5.997.125 4 4 0 0 0-2.526 5.77 4 4 0 0 0 .556 6.588A4 4 0 1 0 12 18Z" />
    <path d="M12 5a3 3 0 1 1 5.997.125 4 4 0 0 1 2.526 5.77 4 4 0 0 1-.556 6.588A4 4 0 1 1 12 18Z" />
    <path d="M15 13a4.5 4.5 0 0 1-3-4 4.5 4.5 0 0 1-3 4" />
    <path d="M17.599 6.5a3 3 0 0 0 .399-1.375" />
    <path d="M6.003 5.125A3 3 0 0 0 6.401 6.5" />
    <path d="M3.477 10.896a4 4 0 0 1 .585-.396" />
    <path d="M19.938 10.5a4 4 0 0 1 .585.396" />
    <path d="M6 18a4 4 0 0 1-1.967-.516" />
    <path d="M19.967 17.484A4 4 0 0 1 18 18" />
  </svg>
</div>

<style>
  #floating-ball {
    -webkit-app-region: drag;
  }
</style>
```

**Step 2: 创建 Header 组件（鼠标悬停展开）**

Create `vision-jarvis/src/components/FloatingBall/Header.astro`:

```astro
---
// 360x72 Header 展开状态
---

<div
  id="header-expanded"
  class="w-[360px] h-[72px] bg-card rounded-[40px] border border-primary flex items-center gap-4 px-5 hidden opacity-0 transition-all duration-300"
>
  <!-- Memory Toggle -->
  <div class="memory-toggle w-[100px] h-10 gradient-success rounded-[20px] flex items-center justify-end px-1 cursor-pointer">
    <div class="w-8 h-8 bg-white rounded-full"></div>
  </div>

  <!-- Memory Button -->
  <button
    id="memory-btn"
    class="h-10 bg-secondary rounded-[20px] border border-secondary px-4 flex items-center gap-2 hover:border-glow transition-colors"
  >
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#00D4FF" stroke-width="2">
      <ellipse cx="12" cy="5" rx="9" ry="3"/>
      <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>
    </svg>
    <span class="text-info text-sm font-medium">记忆</span>
  </button>

  <!-- Popup Button -->
  <button
    id="popup-btn"
    class="h-10 bg-secondary rounded-[20px] border border-secondary px-4 flex items-center gap-2 hover:border-glow transition-colors"
  >
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#00D4FF" stroke-width="2">
      <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/>
      <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>
    </svg>
    <span class="text-info text-sm font-medium">提醒</span>
  </button>
</div>
```

**Step 3: 创建 Asker 组件（点击展开）**

Create `vision-jarvis/src/components/FloatingBall/Asker.astro`:

```astro
---
// 360x480 Asker 展开状态
---

<div
  id="asker-expanded"
  class="w-[360px] h-[480px] bg-card rounded-[32px] border border-primary hidden opacity-0 transition-all duration-300 flex flex-col"
>
  <!-- Header -->
  <div class="h-16 px-6 flex items-center justify-between border-b border-primary">
    <h2 class="text-lg font-semibold text-primary">AI 助手</h2>
    <button
      id="close-asker"
      class="w-8 h-8 rounded-full hover:bg-secondary transition-colors flex items-center justify-center"
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
        <path d="M18 6 6 18M6 6l12 12"/>
      </svg>
    </button>
  </div>

  <!-- Messages -->
  <div class="flex-1 p-4 overflow-y-auto custom-scrollbar" id="asker-messages">
    <div class="text-center text-muted py-12">
      向我提问关于你的记忆...
    </div>
  </div>

  <!-- Input -->
  <div class="p-4 border-t border-primary">
    <div class="flex gap-2">
      <input
        type="text"
        id="asker-input"
        placeholder="输入问题..."
        class="flex-1 h-10 px-4 bg-input rounded-full border border-secondary focus:border-glow outline-none text-sm text-primary placeholder:text-placeholder"
      />
      <button
        id="asker-send"
        class="w-10 h-10 rounded-full gradient-primary flex items-center justify-center hover:opacity-90 transition-opacity"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
          <path d="m3 3 3 9-3 9 19-9Z"/>
          <path d="M6 12h16"/>
        </svg>
      </button>
    </div>
  </div>
</div>
```

**Step 4: 创建主页面**

Create `vision-jarvis/src/pages/floating-ball.astro`:

```astro
---
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
    import { invoke } from '@tauri-apps/api/core';

    type WindowState = 'ball' | 'header' | 'asker';
    let currentState: WindowState = 'ball';
    let hoverTimer: NodeJS.Timeout | null = null;

    const ballState = document.getElementById('ball-state');
    const headerState = document.getElementById('header-state');
    const askerState = document.getElementById('asker-state');
    const floatingBall = document.getElementById('floating-ball');

    // 切换到指定状态
    async function switchTo(state: WindowState) {
      if (currentState === state) return;

      currentState = state;

      // 隐藏所有状态
      ballState?.classList.add('hidden');
      headerState?.classList.add('hidden');
      askerState?.classList.add('hidden');

      // 显示目标状态
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

    // 鼠标悬停 -> Header
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

    // Header 鼠标离开 -> Ball
    headerState?.addEventListener('mouseleave', () => {
      if (currentState === 'header') {
        setTimeout(() => {
          if (currentState === 'header') {
            switchTo('ball');
          }
        }, 300);
      }
    });

    // 点击悬浮球 -> Asker
    floatingBall?.addEventListener('click', (e) => {
      e.stopPropagation();
      switchTo('asker');
    });

    // 点击外部 -> Ball
    document.addEventListener('click', (e) => {
      const target = e.target as HTMLElement;
      if (!askerState?.contains(target) && !headerState?.contains(target)) {
        if (currentState !== 'ball') {
          switchTo('ball');
        }
      }
    });

    // Memory 按钮
    document.getElementById('memory-btn')?.addEventListener('click', async () => {
      await invoke('open_memory_window');
    });

    // Popup 按钮
    document.getElementById('popup-btn')?.addEventListener('click', async () => {
      await invoke('open_popup_setting_window');
    });

    // 关闭 Asker
    document.getElementById('close-asker')?.addEventListener('click', () => {
      switchTo('ball');
    });
  </script>
</Layout>
```

**Step 5: 测试页面**

Run: `cd vision-jarvis && npm run dev`
Navigate: 打开开发工具，访问 http://localhost:4321/floating-ball
Expected: 看到 64x64 悬浮球

**Step 6: Commit**

```bash
git add vision-jarvis/src/pages/floating-ball.astro \
        vision-jarvis/src/components/FloatingBall/
git commit -m "feat: create floating ball main window UI"
```

---

## Phase 3: Memory 窗口

### Task 3.1: 重构 Memory 页面为独立窗口

**Files:**
- Modify: `vision-jarvis/src/pages/memory.astro`

**Step 1: 更新 Memory 页面为完整窗口布局**

Update `vision-jarvis/src/pages/memory.astro`:

```astro
---
import Layout from '../layouts/Layout.astro';
---

<Layout>
  <div class="flex h-screen bg-app">
    <!-- Left Sidebar -->
    <div class="w-80 bg-sidebar border-r border-primary p-6 flex flex-col gap-6 overflow-y-auto custom-scrollbar">
      <!-- Memory Toggle -->
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium text-secondary">全局记忆</span>
        <div class="memory-toggle w-[60px] h-8 gradient-success rounded-full flex items-center justify-end px-1 cursor-pointer">
          <div class="w-6 h-6 bg-white rounded-full transition-all duration-300"></div>
        </div>
      </div>

      <!-- Date Selector -->
      <div>
        <button
          id="date-btn"
          class="w-full h-12 bg-input rounded-xl border border-secondary hover:border-glow transition-colors flex items-center justify-between px-4"
        >
          <span class="text-sm text-primary" id="selected-date">2026-02-06</span>
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#00D4FF" stroke-width="2">
            <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
            <line x1="16" y1="2" x2="16" y2="6"/>
            <line x1="8" y1="2" x2="8" y2="6"/>
            <line x1="3" y1="10" x2="21" y2="10"/>
          </svg>
        </button>
      </div>

      <!-- Short-term Memory List -->
      <div class="flex-1 flex flex-col gap-4">
        <h3 class="text-sm font-semibold text-primary">短期记忆</h3>

        <div class="flex flex-col gap-2">
          <!-- Morning Section -->
          <div class="text-xs text-muted px-2 py-1">早晨</div>
          <div class="memory-item p-3 bg-item rounded-lg hover:bg-secondary cursor-pointer transition-colors">
            <div class="text-xs text-info mb-1">08:00-09:30</div>
            <div class="text-sm text-primary">开发 Vision-Jarvis 项目</div>
          </div>

          <!-- Afternoon Section -->
          <div class="text-xs text-muted px-2 py-1 mt-2">下午</div>
          <div class="memory-item p-3 bg-item rounded-lg hover:bg-secondary cursor-pointer transition-colors">
            <div class="text-xs text-info mb-1">14:00-15:30</div>
            <div class="text-sm text-primary">设计前端架构</div>
          </div>
        </div>
      </div>

      <!-- Settings -->
      <div class="border-t border-primary pt-4 space-y-4">
        <!-- Capture Frequency -->
        <div>
          <div class="flex justify-between text-xs mb-2">
            <span class="text-secondary">截屏频率</span>
            <span class="text-info">5秒</span>
          </div>
          <input
            type="range"
            min="1"
            max="15"
            value="5"
            class="w-full"
          />
        </div>

        <!-- Storage Settings -->
        <button class="w-full h-10 bg-input rounded-lg border border-secondary hover:border-glow transition-colors text-sm text-primary">
          文件存储设置
        </button>
      </div>
    </div>

    <!-- Right Main Content -->
    <div class="flex-1 flex flex-col">
      <!-- Search Bar (always on top) -->
      <div class="h-20 px-8 flex items-center border-b border-primary">
        <div class="flex-1 relative">
          <input
            type="text"
            placeholder="搜索记忆..."
            class="w-full h-12 pl-12 pr-4 bg-input rounded-full border border-secondary focus:border-glow outline-none text-sm text-primary placeholder:text-placeholder"
          />
          <svg
            class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="#888899"
            stroke-width="2"
          >
            <circle cx="11" cy="11" r="8"/>
            <path d="m21 21-4.35-4.35"/>
          </svg>
        </div>
      </div>

      <!-- Content Area -->
      <div class="flex-1 overflow-y-auto custom-scrollbar p-8">
        <!-- Default State -->
        <div id="default-content" class="flex items-center justify-center h-full">
          <div class="text-center">
            <div class="text-6xl mb-4">🧠</div>
            <h2 class="text-2xl font-semibold text-primary mb-2">想找哪段记忆</h2>
            <p class="text-lg text-muted">我都记着呢，随便问</p>
          </div>
        </div>

        <!-- Memory Detail Content (hidden by default) -->
        <div id="memory-content" class="hidden">
          <h1 class="text-3xl font-bold text-primary mb-6">开发 Vision-Jarvis 项目</h1>

          <div class="prose prose-invert max-w-none">
            <h3 class="text-xl font-semibold text-secondary mb-4">时间线</h3>
            <div class="space-y-4">
              <div class="p-4 bg-card rounded-lg border border-primary">
                <div class="text-sm text-info mb-2">08:00 - 08:30</div>
                <p class="text-primary">查看项目需求，分析技术栈</p>
              </div>
              <div class="p-4 bg-card rounded-lg border border-primary">
                <div class="text-sm text-info mb-2">08:30 - 09:30</div>
                <p class="text-primary">实现悬浮窗口架构设计</p>
              </div>
            </div>

            <h3 class="text-xl font-semibold text-secondary mt-8 mb-4">建议与分析</h3>
            <div class="p-4 bg-card rounded-lg border border-primary">
              <p class="text-primary">本次开发进展顺利，建议继续保持专注...</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</Layout>
```

**Step 2: 测试 Memory 窗口**

Run: `cd vision-jarvis && npm run dev`
Navigate: http://localhost:4321/memory
Expected: 看到完整的 Memory 管理界面

**Step 3: Commit**

```bash
git add vision-jarvis/src/pages/memory.astro
git commit -m "feat: refactor memory page as standalone window"
```

---

## Phase 4: Popup-Setting 窗口

### Task 4.1: 重构 Popup-Setting 页面

**Files:**
- Modify: `vision-jarvis/src/pages/popup-setting.astro`

**Step 1: 更新 Popup-Setting 为卡片布局**

Update `vision-jarvis/src/pages/popup-setting.astro`:

```astro
---
import Layout from '../layouts/Layout.astro';
---

<Layout>
  <div class="min-h-screen bg-app p-12">
    <div class="max-w-4xl mx-auto">
      <!-- Header -->
      <div class="mb-12">
        <h1 class="text-4xl font-bold text-primary mb-2">提醒设置</h1>
        <p class="text-lg text-muted">配置您的智能提醒和通知</p>
      </div>

      <!-- Cards Grid -->
      <div class="grid gap-6">
        <!-- Card 1: 启动提醒 -->
        <div class="p-8 bg-card rounded-[24px] border border-primary">
          <div class="flex items-start justify-between mb-6">
            <div>
              <h2 class="text-2xl font-semibold text-primary mb-2">启动提醒</h2>
              <p class="text-sm text-muted">应用启动时的提醒设置</p>
            </div>
            <div class="memory-toggle w-16 h-8 bg-secondary rounded-full flex items-center px-1 cursor-pointer">
              <div class="w-6 h-6 bg-white rounded-full transition-all duration-300"></div>
            </div>
          </div>

          <div class="space-y-4">
            <!-- Auto Start -->
            <div class="flex items-center justify-between p-4 bg-input rounded-xl">
              <span class="text-sm text-secondary">开机自动启动</span>
              <div class="memory-toggle w-12 h-6 bg-secondary rounded-full flex items-center px-0.5 cursor-pointer">
                <div class="w-5 h-5 bg-white rounded-full transition-all duration-300"></div>
              </div>
            </div>

            <!-- Startup Message -->
            <div>
              <label class="text-sm text-secondary block mb-2">启动提醒文本</label>
              <textarea
                class="w-full h-24 p-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary resize-none"
                placeholder="输入启动提醒文本..."
              >If today were the last day of my life, would I want to do what I am about to do today?</textarea>
            </div>
          </div>
        </div>

        <!-- Card 2: 定时提醒 -->
        <div class="p-8 bg-card rounded-[24px] border border-primary">
          <div class="flex items-start justify-between mb-6">
            <div>
              <h2 class="text-2xl font-semibold text-primary mb-2">定时提醒</h2>
              <p class="text-sm text-muted">设置定期提醒通知</p>
            </div>
            <div class="memory-toggle w-16 h-8 gradient-success rounded-full flex items-center justify-end px-1 cursor-pointer">
              <div class="w-6 h-6 bg-white rounded-full"></div>
            </div>
          </div>

          <div class="space-y-4">
            <!-- Time Range -->
            <div>
              <label class="text-sm text-secondary block mb-3">工作时间段</label>
              <div class="flex items-center gap-4">
                <input
                  type="time"
                  value="09:00"
                  class="flex-1 h-12 px-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary"
                />
                <span class="text-muted">至</span>
                <input
                  type="time"
                  value="21:00"
                  class="flex-1 h-12 px-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary"
                />
              </div>
            </div>

            <!-- Interval -->
            <div>
              <div class="flex justify-between text-sm mb-3">
                <span class="text-secondary">提醒间隔</span>
                <span class="text-info">每 30 分钟</span>
              </div>
              <input
                type="range"
                min="5"
                max="120"
                value="30"
                step="5"
                class="w-full"
              />
              <div class="flex justify-between text-xs text-muted mt-1">
                <span>5分钟</span>
                <span>120分钟</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Card 3: 空闲检测 -->
        <div class="p-8 bg-card rounded-[24px] border border-primary">
          <div class="flex items-start justify-between mb-6">
            <div>
              <h2 class="text-2xl font-semibold text-primary mb-2">空闲检测</h2>
              <p class="text-sm text-muted">检测屏幕无变化并提醒</p>
            </div>
            <div class="memory-toggle w-16 h-8 gradient-success rounded-full flex items-center justify-end px-1 cursor-pointer">
              <div class="w-6 h-6 bg-white rounded-full"></div>
            </div>
          </div>

          <div class="space-y-4">
            <!-- Idle Duration -->
            <div>
              <div class="flex justify-between text-sm mb-3">
                <span class="text-secondary">空闲判定时长</span>
                <span class="text-info">15 分钟</span>
              </div>
              <input
                type="range"
                min="5"
                max="60"
                value="15"
                step="5"
                class="w-full"
              />
              <div class="flex justify-between text-xs text-muted mt-1">
                <span>5分钟</span>
                <span>60分钟</span>
              </div>
            </div>

            <!-- Reminder Type -->
            <div>
              <label class="text-sm text-secondary block mb-2">提醒内容</label>
              <select class="w-full h-12 px-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary">
                <option>AI 智能建议</option>
                <option>休息提醒</option>
                <option>自定义文本</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</Layout>
```

**Step 2: 测试 Popup-Setting 窗口**

Run: `cd vision-jarvis && npm run dev`
Navigate: http://localhost:4321/popup-setting
Expected: 看到卡片式布局的设置页面

**Step 3: Commit**

```bash
git add vision-jarvis/src/pages/popup-setting.astro
git commit -m "feat: refactor popup-setting page as standalone window"
```

---

## Phase 5: 集成测试和优化

### Task 5.1: 端到端测试

**Files:**
- Test manually

**Step 1: 启动应用测试完整流程**

Run: `cd vision-jarvis/src-tauri && cargo tauri dev`

**测试清单**:
- [ ] 应用启动显示悬浮球（右上角）
- [ ] 悬浮球可以拖动
- [ ] 鼠标悬停展开为 Header
- [ ] Header 显示记忆toggle、记忆按钮、提醒按钮
- [ ] 点击悬浮球展开为 Asker
- [ ] 点击外部区域折叠回悬浮球
- [ ] 点击"记忆"按钮打开 Memory 窗口
- [ ] 点击"提醒"按钮打开 Popup-Setting 窗口
- [ ] Memory 窗口可以独立操作
- [ ] Popup-Setting 窗口可以独���操作
- [ ] 悬浮球始终保持在最顶层

**Step 2: 记录测试结果**

Create test report: `docs/testing/test-reports/2026-02-06-floating-ball-e2e.md`

**Step 3: 修复发现的问题**

If bugs found:
1. Create issue in GitHub/notes
2. Fix bugs
3. Re-test
4. Commit fixes

---

### Task 5.2: 性能优化

**Files:**
- Modify: `vision-jarvis/src/pages/floating-ball.astro`

**Step 1: 优化窗口切换动画**

Add transition classes and debounce:

```typescript
// 防抖函数
function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | null = null;
  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}

// 使用防抖优化 hover 事件
const debouncedExpand = debounce(() => {
  if (currentState === 'ball') {
    switchTo('header');
  }
}, 200);
```

**Step 2: 优化窗口渲染**

Add `will-change` for GPU acceleration:

```css
.transition-all {
  will-change: transform, opacity;
}
```

**Step 3: 测试性能**

Run: `cargo tauri dev`
Monitor: CPU usage should be < 5% when idle

**Step 4: Commit**

```bash
git add vision-jarvis/src/pages/floating-ball.astro
git commit -m "perf: optimize window transitions and animations"
```

---

## Phase 6: 文档更新

### Task 6.1: 更新技术文档

**Files:**
- Create: `docs/frontend/architecture-v2-floating-windows.md`
- Update: `docs/CHANGELOG.md`
- Update: `docs/README.md`

**Step 1: 创建架构文档**

Create `docs/frontend/architecture-v2-floating-windows.md`:

```markdown
# Frontend Architecture V2 - Floating Windows System

## Overview

Vision-Jarvis 采用多窗口悬浮架构，主窗口为64x64悬浮球，始终置顶。

## Window Structure

### Main Window: Floating Ball
- Size: 64x64 → 360x72 → 360x480
- States: Ball → Header → Asker
- Always on top: true
- Transparent: true
- Decorations: false

### Memory Window
- Size: 1200x800
- Independent window
- Full memory management UI

### Popup-Setting Window
- Size: 900x700
- Independent window
- Card-based settings layout

## State Management

```typescript
type WindowState = 'ball' | 'header' | 'asker';
```

## Interaction Flow

1. App starts → Floating Ball (top-right)
2. Mouse hover → Expand to Header
3. Click ball → Expand to Asker
4. Click "记忆" → Open Memory Window
5. Click "提醒" → Open Popup-Setting Window
6. Click outside → Collapse to Ball
```

**Step 2: 更新 CHANGELOG**

Add to `docs/CHANGELOG.md`:

```markdown
## [Unreleased]

### Changed - 2026-02-06

**Frontend Architecture Redesign**:
- 重构前端为多窗口悬浮架构
- 主窗口：64x64 悬浮球，支持 3 种展开状态
- Memory 窗口：独立窗口，1200x800
- Popup-Setting 窗口：独立窗口，900x700
- 实现窗口置顶、透明、无边框等特性
```

**Step 3: Commit**

```bash
git add docs/
git commit -m "docs: update architecture documentation for floating windows"
```

---

## Summary

**Implementation Phases**:
1. ✅ Tauri 窗口配置（多窗口系统）
2. ✅ 悬浮球主窗口（Ball → Header → Asker）
3. ✅ Memory 独立窗口
4. ✅ Popup-Setting 独立窗口
5. ✅ 集成测试和优化
6. ✅ 文档更新

**Key Technologies**:
- Tauri 2 多窗口 API
- 窗口置顶和透明
- 状态机管理
- 防抖和性能优化

**Testing Strategy**:
- 手动端到端测试
- 性能监控
- 交互流程验证

**Estimated Time**: 4-6 hours

---

Plan complete and saved to `docs/plans/2026-02-06-floating-ball-multi-window-architecture.md`.

**Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**

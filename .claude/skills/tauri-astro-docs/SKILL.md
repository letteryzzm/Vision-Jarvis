# Tauri + Astro 官方文档索引 Skill

自动索引 Tauri、Astro、Rust 等官方文档，确保使用正确的 API 和最佳实践。

## 何时使用

**自动触发条件**（必须使用此 skill）：

- 创建 Tauri commands 或 Rust 后端代码
- 实现前后端通信（invoke API）
- 创建 Astro 组件或页面
- 配置 Tauri 插件（托盘、快捷键等）
- 使用 Rust crates（screenshots、reqwest 等）
- 配置构建和打包
- 实现系统 API 调用

**关键词触发**：

- "Tauri"、"Astro"、"Rust"
- "invoke"、"command"、"plugin"
- "component"、"layout"、"page"
- "前后端通信"、"API 调用"
- "官方文档"、"最佳实践"

## 核心功能

### 1. 文档索引映射

```yaml
技术栈文档索引:
  Tauri:
    官方文档: https://v2.tauri.app/
    API 参考: https://tauri.app/v1/api/
    核心概念:
      - Commands: https://tauri.app/v1/guides/features/command
      - Events: https://tauri.app/v1/guides/features/events
      - Window: https://tauri.app/v1/api/js/window
      - System Tray: https://tauri.app/v1/guides/features/system-tray
      - Global Shortcut: https://tauri.app/v1/guides/features/global-shortcut
    配置:
      - tauri.conf.json: https://tauri.app/v1/api/config
      - Cargo.toml: https://tauri.app/v1/guides/building/

  Astro:
    官方文档: https://docs.astro.build/
    核心概念:
      - Components: https://docs.astro.build/en/core-concepts/astro-components/
      - Layouts: https://docs.astro.build/en/core-concepts/layouts/
      - Pages: https://docs.astro.build/en/core-concepts/astro-pages/
      - Islands: https://docs.astro.build/en/concepts/islands/
    集成:
      - Tailwind: https://docs.astro.build/en/guides/integrations-guide/tailwind/
      - TypeScript: https://docs.astro.build/en/guides/typescript/

  Rust Crates:
    screenshots: https://docs.rs/screenshots/latest/screenshots/
    reqwest: https://docs.rs/reqwest/latest/reqwest/
    serde: https://serde.rs/
    tokio: https://tokio.rs/
    image: https://docs.rs/image/latest/image/
```

### 2. 使用工作流

```
用户请求开发任务
    ↓
检测是否涉及 Tauri/Astro/Rust
    ↓
[是] → 触发此 skill
    ↓
识别具体技术点（如 "Tauri command"）
    ↓
使用 WebFetch 获取相关官方文档
    ↓
提取关键 API 和示例代码
    ↓
基于官方文档生成代码
    ↓
附上文档引用链接
```

### 3. 文档查询模板

#### Tauri Commands

```
当创建 Tauri command 时：
1. 查询: https://tauri.app/v1/guides/features/command
2. 提取关键点:
   - #[tauri::command] 宏用法
   - 参数序列化/反序列化
   - 错误处理模式
   - 异步 command 写法
3. 生成符合官方规范的代码
```

#### 前后端通信

```
当实现 invoke 调用时：
1. 查询: https://tauri.app/v1/api/js/tauri#invoke
2. 提取关键点:
   - TypeScript 类型定义
   - 错误处理
   - Promise 用法
3. 生成类型安全的调用代码
```

#### Astro 组件

```
当创建 Astro 组件时：
1. 查询: https://docs.astro.build/en/core-concepts/astro-components/
2. 提取关键点:
   - 组件结构 (--- 分隔符)
   - Props 定义
   - 客户端脚本 (<script> 标签)
3. 生成符合规范的组件
```

## 具体使用场景

### 场景 1: 创建 Tauri Command

**用户请求**: "创建一个捕获屏幕的 Tauri command"

**Skill 执行流程**:

```
1. WebFetch https://tauri.app/v1/guides/features/command
   提取: #[tauri::command] 宏、异步函数、错误处理

2. WebFetch https://docs.rs/screenshots/latest/screenshots/
   提取: Screen::all()、capture() API

3. 生成代码（基于官方示例）:
```rust
use screenshots::Screen;
use tauri::command;

#[command]
async fn capture_screen() -> Result<String, String> {
    let screens = Screen::all()
        .map_err(|e| e.to_string())?;

    let screen = screens
        .first()
        .ok_or("No screen found")?;

    let image = screen
        .capture()
        .map_err(|e| e.to_string())?;

    // 转换为 base64
    let base64 = todo!();
    Ok(base64)
}
```

4. 附上参考文档:
   - Tauri Commands: https://tauri.app/v1/guides/features/command
   - screenshots crate: https://docs.rs/screenshots/

### 场景 2: 前端调用 Tauri Command

**用户请求**: "在 Astro 组件中调用 capture_screen"

**Skill 执行流程**:

```
1. WebFetch https://tauri.app/v1/api/js/tauri#invoke
   提取: invoke() API、TypeScript 类型

2. WebFetch https://docs.astro.build/en/core-concepts/astro-components/
   提取: 客户端脚本、事件处理

3. 生成代码:
```typescript
---
// Astro frontmatter
const title = "Screen Capture";
---

<button id="capture-btn">捕获屏幕</button>

<script>
  import { invoke } from '@tauri-apps/api/tauri';

  const btn = document.getElementById('capture-btn');

  btn?.addEventListener('click', async () => {
    try {
      const base64Image = await invoke<string>('capture_screen');
      console.log('Captured:', base64Image);
    } catch (error) {
      console.error('Error:', error);
    }
  });
</script>
```

4. 附上参考文档:
   - Tauri invoke: https://tauri.app/v1/api/js/tauri#invoke
   - Astro client scripts: https://docs.astro.build/en/guides/client-side-scripts/

### 场景 3: 配置系统托盘

**用户请求**: "添加系统托盘图标和菜单"

**Skill 执行流程**:

```
1. WebFetch https://tauri.app/v1/guides/features/system-tray
   提取: SystemTray API、菜单配置

2. WebFetch https://tauri.app/v1/api/config#tauri.systemTray
   提取: tauri.conf.json 配置项

3. 生成配置和代码:
```json
// tauri.conf.json
{
  "tauri": {
    "systemTray": {
      "iconPath": "icons/tray-icon.png",
      "iconAsTemplate": true
    }
  }
}
```

```rust
// main.rs
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent};

fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", "显示"))
        .add_item(CustomMenuItem::new("quit", "退出"));

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                match id.as_str() {
                    "show" => {
                        // 显示窗口
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

4. 附上参考文档:
   - System Tray Guide: https://tauri.app/v1/guides/features/system-tray

### 场景 4: 全局快捷键

**用户请求**: "注册 cmd+shift+j 快捷键"

**Skill 执行流程**:

```
1. WebFetch https://tauri.app/v1/guides/features/global-shortcut
   提取: register() API、快捷键格式

2. 生成代码:
```rust
use tauri::GlobalShortcutManager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let mut shortcut = app.global_shortcut_manager();

            shortcut.register("CmdOrCtrl+Shift+J", move || {
                println!("Shortcut triggered!");
                // 显示/隐藏窗口
            })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

3. 附上参考文档:
   - Global Shortcut: https://tauri.app/v1/guides/features/global-shortcut

## 文档版本管理

```yaml
版本追踪:
  Tauri:
    当前使用: v1.x
    文档地址: https://tauri.app/v1/
    更新检查: 每月一次

  Astro:
    当前使用: v4.x
    文档地址: https://docs.astro.build/
    更新检查: 每月一次

  注意事项:
    - Tauri v2 与 v1 API 不兼容
    - 优先使用项目声明的版本文档
    - 如遇 API 不存在，提示用户检查版本
```

## 代码生成规范

### 必须遵循的原则

1. **完全基于官方文档**

   - 不编造 API
   - 使用官方示例作为模板
   - 保持最佳实践
2. **提供文档引用**

   - 每段代码附上官方文档链接
   - 注明 API 版本
   - 说明关键参数含义
3. **错误处理**

   - 使用 Result<T, E> 模式
   - 提供清晰的错误信息
   - 遵循 Rust 错误处理规范
4. **类型安全**

   - TypeScript 类型定义
   - Rust 类型注解
   - Serde 序列化规范

## 工作检查清单

使用此 skill 时，必须：

- [ ] 识别涉及的技术（Tauri/Astro/Rust）
- [ ] 使用 WebFetch 获取相关官方文档
- [ ] 提取关键 API 和用法
- [ ] 生成符合官方规范的代码
- [ ] 附上文档链接作为参考
- [ ] 验证 API 版本兼容性
- [ ] 检查错误处理是否完善

## 示例对话流程

```
用户: "创建一个 Tauri command 来捕获屏幕"

Assistant (内部思考):
1. 触发 tauri-astro-docs skill
2. 需要查询: Tauri Commands 文档 + screenshots crate 文档
3. WebFetch 获取官方示例
4. 生成代码

Assistant (回复):
我会基于 Tauri 官方文档创建屏幕捕获 command。

[使用 WebFetch 获取文档...]
[生成代码...]

参考文档:
- Tauri Commands: https://tauri.app/v1/guides/features/command
- screenshots crate: https://docs.rs/screenshots/
```

## 错误处理

如果文档获取失败：

```
1. 尝试备用文档源（GitHub、crates.io）
2. 使用已知的 API 模式（基于 skill 内置知识）
3. 明确告知用户使用了备用方案
4. 建议用户验证代码
```

## 持续改进

收集常见查询模式，优化文档索引：

```yaml
常见查询:
  - Tauri command 创建: 累计 XX 次
  - invoke 调用: 累计 XX 次
  - 系统托盘配置: 累计 XX 次

优化方向:
  - 预缓存高频文档
  - 提供快速参考模板
  - 自动版本检测
```

---

**重要提醒**:

- ⚠️ 永远不要编造或猜测 API
- ✅ 优先使用官方文档
- 📚 保持文档链接有效性
- 🔄 定期更新技术栈版本

---

**Skill 版本**: 1.0.0
**最后更新**: 2026-01-29
**维护者**: Vision-Jarvis Team

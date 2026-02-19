# Vision-Jarvis 前端文档

> **最后更新**: 2026-02-19
> **技术栈**: Astro + TypeScript + Tauri v2 + Tailwind CSS

---

## 实际文件结构

```
vision-jarvis/src/
├── pages/
│   ├── floating-ball.astro   # 悬浮球窗口（Ball/Header/Asker 三态）
│   ├── memory.astro          # 记忆管理页
│   ├── popup-setting.astro   # 弹窗设置页
│   ├── api-settings.astro    # API 设置页
│   ├── files.astro           # 文件管理页
│   └── index.astro           # 主页
├── components/
│   ├── FloatingBall/
│   │   ├── Ball.astro        # 64x64 圆形悬浮球
│   │   ├── Header.astro      # 360x72 Header 展开态
│   │   └── Asker.astro       # 360x480 Asker 展开态
│   ├── FloatingOrb/
│   │   └── FloatingOrb.astro # 旧版悬浮球（遗留）
│   ├── Asker/
│   │   └── AskerExpanded.astro
│   ├── Header/
│   │   └── HeaderExpanded.astro
│   └── Welcome.astro
├── stores/
│   └── settingsStore.ts      # 设置状态（Nanostores）
├── types/
│   └── settings.ts           # 设置类型定义
├── lib/
│   └── tauri-api.ts          # Tauri IPC 调用封装
└── layouts/
    └── Layout.astro
```

---

## 窗口架构

Vision-Jarvis 采用多窗口架构，每个窗口对应一个 Tauri WebviewWindow：

| 窗口 ID | 页面 | 说明 |
|---------|------|------|
| `floating-ball` | `floating-ball.astro` | 常驻悬浮球，支持三种展开态 |
| `memory` | `memory.astro` | 记忆管理，通过 `open_memory_window` 打开 |
| `popup-setting` | `popup-setting.astro` | 设置弹窗，通过 `open_popup_setting_window` 打开 |

---

## 悬浮球三态

`floating-ball.astro` 包含三个子组件，通过 JS 切换显示：

```
Ball（默认）→ expand_to_header → Header
Ball（默认）→ expand_to_asker  → Asker
Header/Asker → collapse_to_ball → Ball
```

---

## 相关文档

- [组件文档](components/README.md)
- [页面文档](pages/)
- [API 接口](../../api/endpoints/commands.md)

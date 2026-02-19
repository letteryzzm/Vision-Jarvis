# 前端组件库

> **最后更新**: 2026-02-19

---

## 实际组件（Astro）

### FloatingBall/（悬浮球三态）

| 文件 | 尺寸 | 说明 |
|------|------|------|
| `Ball.astro` | 64×64 | 圆形悬浮球，默认态 |
| `Header.astro` | 360×72 | Header 展开态，含记忆开关/记忆管理/设置按钮 |
| `Asker.astro` | 360×480 | Asker 展开态，AI 助手对话界面 |

### 其他组件

| 文件 | 说明 |
|------|------|
| `FloatingOrb/FloatingOrb.astro` | 旧版悬浮球（遗留，待清理） |
| `Asker/AskerExpanded.astro` | Asker 展开态（另一版本） |
| `Header/HeaderExpanded.astro` | Header 展开态（另一版本） |
| `Welcome.astro` | 欢迎页组件 |

---

## 注意

- 所有组件为 **Astro 组件**（`.astro`），不是 React 组件
- 状态管理使用 `settingsStore.ts`（Nanostores）
- 组件间通信通过 Tauri IPC（`tauri-api.ts`）

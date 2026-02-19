# 实施计划：前端代码质量改进

## 需求重述

修复前端 `src/` 目录中已识别的代码质量问题：
1. `floating-ball.astro` 中的 `console.log` 调试日志
2. `popup-setting.astro` 中的 `any` 类型
3. `memory.astro` 中的硬编码日期
4. `showNotification` 函数在两个文件中重复
5. `floating-ball.astro` 中的 `is:global` 样式分散
6. `popup-setting.astro` 超过 800 行限制（1193 行）

---

## Phase 1：简单修复（低风险）

### 1.1 清理 `floating-ball.astro` 中的 console.log
- 删除所有 `console.log` 和 `console.warn` 调试语句（约 10 处）
- 保留 `console.error`（错误处理需要）

### 1.2 修复 `memory.astro` 硬编码日期
- `line 23`：将 `"2026-02-06"` 替换为动态生成的今日日期

### 1.3 修复 `popup-setting.astro` 的 `any` 类型
- `line 568`：`updateGeneralSettingsUI(settings: any)` → `updateGeneralSettingsUI(settings: AppSettings)`
- 添加对应 import

---

## Phase 2：提取共享工具

### 2.1 创建 `src/lib/notification.ts`
提取重复的 `showNotification` 函数：
```typescript
export function showNotification(message: string, type: 'success' | 'error' | 'info'): void
```
- 在 `memory.astro` 和 `popup-setting.astro` 中替换为 import

### 2.2 将 `floating-ball.astro` 的全局样式移到 `global.css`
- 将 `<style is:global>` 中的样式（过渡动画、布局样式）迁移到 `global.css`
- 删除 `floating-ball.astro` 中的 `<style is:global>` 块

---

## Phase 3：拆分 `popup-setting.astro`

将 1193 行的文件拆分为：

```
src/components/Settings/
  GeneralSettings.astro    # 通用设置 tab 内容（启动、提醒卡片）
  AISettings.astro         # AI 配置 tab 内容
```

`popup-setting.astro` 保留 tab 切换逻辑和 Layout，引用两个子组件。

**注意**：子组件中的事件绑定和 store 交互逻辑需要保留在各自的 `<script>` 块中。

---

## 风险评估

| 风险 | 级别 | 说明 |
|------|------|------|
| Phase 1 | 低 | 纯删除/替换，无逻辑变更 |
| Phase 2.1 | 低 | 函数提取，行为不变 |
| Phase 2.2 | 低 | 样式迁移，视觉不变 |
| Phase 3 | 中 | 组件拆分需确保 script 作用域正确 |

---

## 执行顺序

Phase 1 → Phase 2.1 → Phase 2.2 → Phase 3

**等待确认后开始执行。**

# Tauri Commands API

Vision-Jarvis 前后端通信接口文档。

> **最后更新**: 2026-02-19

---

## 概述

所有命令使用统一的 `ApiResponse<T>` 响应格式：

```rust
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}
```

---

## 健康检查

### `health_check`

```typescript
await invoke('health_check')
// { success: true, data: "OK" }
```

---

## 截图管理

### `capture_screenshot`

捕获当前屏幕截图并保存。

```typescript
await invoke('capture_screenshot')
// { success: true, data: { id, path, captured_at, analyzed } }
```

### `get_screenshots`

```typescript
await invoke('get_screenshots', { limit?: number })
// { success: true, data: Screenshot[] }
```

### `delete_screenshot`

```typescript
await invoke('delete_screenshot', { id: string })
```

---

## 记忆管理（V2 遗留）

### `search_memories`

关键词搜索短期记忆。

```typescript
await invoke('search_memories', { query: string, limit?: number })
```

### `get_memories_by_date`

```typescript
await invoke('get_memories_by_date', { date: 'YYYY-MM-DD' })
```

### `generate_memory`

从已分析截图生成短期记忆。

```typescript
await invoke('generate_memory', { date: 'YYYY-MM-DD' })
```

---

## 记忆管理（V3）

### `search_activities_v2`

搜索活动（关键词匹配，TODO: 后续集成语义搜索）。

```typescript
await invoke('search_activities_v2', { query: string, limit?: number })
// data: ActivitySearchResult[]
// ActivitySearchResult: { id, title, start_time, duration_minutes, application, score }
```

### `get_activity_detail_v2`

获取活动详情。

```typescript
await invoke('get_activity_detail_v2', { activity_id: string })
// data: { id, title, start_time, end_time, duration_minutes, application, summary, screenshot_count }
```

### `get_activities_by_date_v2`

按时间范围获取活动列表。

```typescript
await invoke('get_activities_by_date_v2', {
  start_time: number,  // Unix timestamp (ms)
  end_time: number
})
// data: ActivitySearchResult[]
```

---

## 通知管理

### `get_pending_notifications`

```typescript
await invoke('get_pending_notifications')
// data: Notification[]
// Notification: { id, notification_type, priority, title, message, created_at, dismissed }
```

### `dismiss_notification`

```typescript
await invoke('dismiss_notification', { id: string })
```

### `get_notification_history`

```typescript
await invoke('get_notification_history', { limit?: number })
```

### `respond_to_suggestion`

响应主动建议（接受/拒绝）。

```typescript
await invoke('respond_to_suggestion', { id: string, accepted: boolean })
```

### `get_suggestion_history`

获取主动建议历史。

```typescript
await invoke('get_suggestion_history', { limit?: number })
```

---

## 设置管理

### `get_settings`

```typescript
await invoke('get_settings')
// data: AppSettings
```

### `update_settings`

```typescript
await invoke('update_settings', { settings: AppSettings })
```

**验证规则**:
- `capture_interval_seconds`: 1-15 秒
- `timed_reminder_start/end`: HH:MM 格式

### `reset_settings`

```typescript
await invoke('reset_settings')
// data: AppSettings（默认值）
```

---

## 窗口管理

### `open_memory_window`

打开记忆管理窗口。

```typescript
await invoke('open_memory_window')
```

### `open_popup_setting_window`

打开弹窗设置窗口。

```typescript
await invoke('open_popup_setting_window')
```

### `expand_to_header`

悬浮球展开为 Header 模式。

```typescript
await invoke('expand_to_header')
```

### `expand_to_asker`

悬浮球展开为 Asker 模式。

```typescript
await invoke('expand_to_asker')
```

### `collapse_to_ball`

收起为悬浮球。

```typescript
await invoke('collapse_to_ball')
```

---

## AI 配置

### `get_ai_config_summary`

获取 AI 配置摘要（不含敏感信息）。

```typescript
await invoke('get_ai_config_summary')
```

### `get_ai_config`

获取完整 AI 配置。

```typescript
await invoke('get_ai_config')
```

### `update_ai_api_key`

```typescript
await invoke('update_ai_api_key', { provider: string, api_key: string })
```

### `update_ai_provider_config`

```typescript
await invoke('update_ai_provider_config', { provider: string, config: object })
```

### `set_active_ai_provider`

```typescript
await invoke('set_active_ai_provider', { provider: string })
```

### `test_ai_connection`

测试 AI 连接是否正常。

```typescript
await invoke('test_ai_connection', { provider: string })
// data: { success: boolean, latency_ms?: number, error?: string }
```

### `get_available_ai_providers`

获取支持的 AI 提供商列表。

```typescript
await invoke('get_available_ai_providers')
// data: ModelInfo[]
```

### `delete_ai_provider`

```typescript
await invoke('delete_ai_provider', { provider: string })
```

### `reset_ai_config`

重置 AI 配置为默认值。

```typescript
await invoke('reset_ai_config')
```

### `connect_ai_to_pipeline`

将已配置的 AI 连接到记忆管道（启用 AI 分析）。

```typescript
await invoke('connect_ai_to_pipeline')
```

### `get_pipeline_status`

获取记忆管道运行状态。

```typescript
await invoke('get_pipeline_status')
// data: { running: boolean, ai_enabled: boolean, last_run?: number }
```

---

**文档版本**: v2.0
**最后更新**: 2026-02-19

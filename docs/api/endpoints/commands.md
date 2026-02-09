# Tauri Commands API

Vision-Jarvis 前后端通信接口文档。

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

### health_check

检查应用是否正常运行。

**调用方式**:
```typescript
import { invoke } from '@tauri-apps/api/core';

const response = await invoke('health_check');
// { success: true, data: "OK", error: null }
```

**响应**:
- `ApiResponse<String>` - 成功返回 "OK"

---

## 截图管理

### capture_screenshot

捕获当前屏幕截图并保存到数据库。

**调用方式**:
```typescript
const response = await invoke('capture_screenshot');
```

**响应**:
```typescript
{
  success: true,
  data: {
    id: "uuid-string",
    path: "/path/to/screenshot.png",
    captured_at: 1704556800,
    analyzed: false
  },
  error: null
}
```

**错误示例**:
```typescript
{
  success: false,
  data: null,
  error: "截图失败: 未找到显示器"
}
```

---

### get_screenshots

获取截图列表，按时间倒序排列。

**参数**:
- `limit?: number` - 返回数量限制（默认 50）

**调用方式**:
```typescript
const response = await invoke('get_screenshots', { limit: 20 });
```

**响应**:
```typescript
{
  success: true,
  data: [
    {
      id: "uuid-1",
      path: "/path/to/screenshot1.png",
      captured_at: 1704556800,
      analyzed: true
    },
    // ...
  ],
  error: null
}
```

---

### delete_screenshot

删除指定的截图（同时删除文件和数据库记录）。

**参数**:
- `id: string` - 截图 ID

**调用方式**:
```typescript
const response = await invoke('delete_screenshot', { id: 'uuid-string' });
```

**响应**:
```typescript
{
  success: true,
  data: true,
  error: null
}
```

---

## 记忆管理

### search_memories

搜索短期记忆（支持关键词匹配）。

**安全特性**: 自动转义 SQL 通配符，防止注入攻击。

**参数**:
- `query: string` - 搜索关键词
- `limit?: number` - 返回数量限制（默认 10）

**调用方式**:
```typescript
const response = await invoke('search_memories', {
  query: '编程',
  limit: 20
});
```

**响应**:
```typescript
{
  success: true,
  data: [
    {
      id: "uuid-1",
      date: "2026-02-06",
      time_start: "09:00",
      time_end: "10:30",
      period: "Morning",
      activity: "编程",
      summary: "开发 Vision-Jarvis 项目"
    },
    // ...
  ],
  error: null
}
```

---

### get_memories_by_date

获取指定日期的所有短期记忆。

**输入验证**: 自动验证日期格式（YYYY-MM-DD），无效日期将返回错误。

**参数**:
- `date: string` - 日期（格式: YYYY-MM-DD）

**调用方式**:
```typescript
const response = await invoke('get_memories_by_date', {
  date: '2026-02-06'
});
```

**响应**: 同 `search_memories`

**错误示例**:
```typescript
{
  success: false,
  data: null,
  error: "日期格式错误，请使用YYYY-MM-DD格式: invalid digit found in string"
}
```

---

### generate_memory

生成指定日期的短期记忆（从已分析的截图）。

**错误处理**: 收集所有保存错误并返回，确保用户了解部分失败情况。

**参数**:
- `date: string` - 日期（格式: YYYY-MM-DD）

**调用方式**:
```typescript
const response = await invoke('generate_memory', {
  date: '2026-02-06'
});
```

**响应**: 同 `search_memories`

**错误示例**:
```typescript
{
  success: false,
  data: null,
  error: "保存记忆 uuid-1 失败: UNIQUE constraint failed; 保存记忆 uuid-2 失败: ..."
}
```

---

## 通知管理

### get_pending_notifications

获取所有待处理的通知。

**调用方式**:
```typescript
const response = await invoke('get_pending_notifications');
```

**响应**:
```typescript
{
  success: true,
  data: [
    {
      id: "uuid-1",
      notification_type: "RestReminder",
      priority: 2,
      title: "休息提醒",
      message: "您已经工作了60分钟，建议休息一下",
      created_at: 1704556800,
      dismissed: false
    },
    // ...
  ],
  error: null
}
```

---

### dismiss_notification

关闭（标记为已读）指定通知。

**参数**:
- `id: string` - 通知 ID

**调用方式**:
```typescript
const response = await invoke('dismiss_notification', {
  id: 'uuid-string'
});
```

**响应**:
```typescript
{
  success: true,
  data: true,
  error: null
}
```

---

### get_notification_history

获取通知历史（包括已关闭的通知）。

**参数**:
- `limit?: number` - 返回数量限制（默认 50）

**调用方式**:
```typescript
const response = await invoke('get_notification_history', { limit: 100 });
```

**响应**: 同 `get_pending_notifications`

---

## 设置管理

### get_settings

获取当前应用设置。

**调用方式**:
```typescript
const response = await invoke('get_settings');
```

**响应**:
```typescript
{
  success: true,
  data: {
    capture_interval_seconds: 5,
    storage_path: "/Users/user/Library/Application Support/vision-jarvis/screenshots",
    storage_limit_mb: 1024,
    memory_enabled: true,
    openai_api_key: "",
    timed_reminder_enabled: false,
    timed_reminder_start: "09:00",
    timed_reminder_end: "18:00",
    timed_reminder_interval_minutes: 60,
    inactivity_reminder_enabled: false,
    inactivity_threshold_minutes: 120
  },
  error: null
}
```

---

### update_settings

更新应用设置（含验证）。

**参数**:
- `settings: AppSettings` - 完整的设置对象

**验证规则**:
- `capture_interval_seconds`: 1-15 秒
- `storage_limit_mb`: > 0
- `timed_reminder_start/end`: HH:MM 格式，小时 0-23，分钟 0-59
- `timed_reminder_interval_minutes`: > 0（如果启用）
- `inactivity_threshold_minutes`: > 0（如果启用）

**调用方式**:
```typescript
const newSettings = { ...currentSettings, capture_interval_seconds: 10 };
const response = await invoke('update_settings', { settings: newSettings });
```

**响应**:
```typescript
{
  success: true,
  data: true,
  error: null
}
```

**错误示例**:
```typescript
{
  success: false,
  data: null,
  error: "更新设置失败: 截图间隔必须在 1-15 秒之间"
}
```

---

### reset_settings

重置所有设置为默认值。

**调用方式**:
```typescript
const response = await invoke('reset_settings');
```

**响应**:
```typescript
{
  success: true,
  data: { /* 默认设置对象 */ },
  error: null
}
```

---

## 错误处理

所有命令遵循统一的错误处理模式：

1. **成功响应**: `success: true, data: T, error: null`
2. **失败响应**: `success: false, data: null, error: "错误描述"`

**前端示例**:
```typescript
const response = await invoke('capture_screenshot');

if (response.success) {
  console.log('截图成功:', response.data);
} else {
  console.error('截图失败:', response.error);
  // 显示错误提示给用户
}
```

---

## 安全特性

### SQL 注入防护

`search_memories()` 自动转义 SQL 通配符：
```rust
// 用户输入: "%"
// 转义后: "\%"
// SQL: WHERE activity LIKE '%\%%' ESCAPE '\'
```

### 输入验证

- 日期格式验证（YYYY-MM-DD）
- 设置范围验证
- 参数类型检查

### 线程安全

- 使用 `Arc<Mutex<>>` 保护共享状态
- 无数据竞争
- 原子操作保证

---

## 性能考虑

- **数据库连接池**: 使用 `with_connection()` 管理连接
- **参数化查询**: 预编译 SQL 语句
- **批量操作**: generate_memory() 批量插入记忆
- **限制查询**: 默认限制返回数量（防止 OOM）

---

**文档版本**: v1.0  
**最后更新**: 2026-02-06  
**对应代码**: Phase 5 - Tauri Commands

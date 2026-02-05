# API 接口文档

> **最后更新**: 2026-02-04
> **版本**: v1.0
> **协议**: Tauri IPC Commands

---

## 目录

- [API 概述](#api-概述)
- [接口列表](#接口列表)
- [认证授权](#认证授权)
- [错误码说明](#错误码说明)
- [接口详细文档](#接口详细文档)

---

## API 概述

Vision-Jarvis 使用 **Tauri IPC Commands** 作为前后端通信协议。前端通过 `invoke()` 方法调用后端命令。

### 技术栈

- **前端**: Astro + TypeScript
- **后端**: Rust + Tauri 2.x
- **通信**: IPC (Inter-Process Communication)
- **序列化**: serde_json

### 调用示例

```typescript
// 前端调用示例
import { invoke } from '@tauri-apps/api/core';

// 捕获截图
const screenshot = await invoke<Screenshot>('capture_screenshot');

// 生成记忆
const memory = await invoke<ShortTermMemory>('generate_memory', {
  startTime: '2026-02-04T10:00:00Z',
  endTime: '2026-02-04T11:00:00Z',
});
```

---

## 接口列表

### 截图相关

| 命令 | 方法 | 描述 | 文档 |
|------|------|------|------|
| `capture_screenshot` | - | 立即捕获屏幕截图 | [详细](endpoints/screenshot.md#capture_screenshot) |
| `get_screenshot` | `{ id }` | 获取截图详情 | [详细](endpoints/screenshot.md#get_screenshot) |
| `list_screenshots` | `{ start, end, limit }` | 获取截图列表 | [详细](endpoints/screenshot.md#list_screenshots) |
| `delete_screenshot` | `{ id }` | 删除指定截图 | [详细](endpoints/screenshot.md#delete_screenshot) |

### 记忆相关

| 命令 | 方法 | 描述 | 文档 |
|------|------|------|------|
| `generate_memory` | `{ startTime, endTime }` | 生成短期记忆 | [详细](endpoints/memory.md#generate_memory) |
| `get_memory` | `{ id }` | 获取记忆详情 | [详细](endpoints/memory.md#get_memory) |
| `list_memories_by_date` | `{ date }` | 获取指定日期的记忆 | [详细](endpoints/memory.md#list_memories_by_date) |
| `search_memory` | `{ query, limit }` | 向量搜索记忆 | [详细](endpoints/memory.md#search_memory) |
| `generate_long_term_memory` | `{ startDate, endDate }` | 生成长期记忆 | [详细](endpoints/memory.md#generate_long_term_memory) |

### AI 分析相关

| 命令 | 方法 | 描述 | 文档 |
|------|------|------|------|
| `analyze_screenshot` | `{ id }` | 触发截图分析 | [详细](endpoints/ai-analysis.md#analyze_screenshot) |
| `get_analysis_result` | `{ id }` | 获取分析结果 | [详细](endpoints/ai-analysis.md#get_analysis_result) |
| `retry_failed_analysis` | `{ id }` | 重试失败的分析 | [详细](endpoints/ai-analysis.md#retry_failed_analysis) |

### 通知相关

| 命令 | 方法 | 描述 | 文档 |
|------|------|------|------|
| `get_notification_settings` | - | 获取通知设置 | [详细](endpoints/notification.md#get_notification_settings) |
| `update_notification_settings` | `{ settings }` | 更新通知设置 | [详细](endpoints/notification.md#update_notification_settings) |
| `get_notification_history` | `{ limit }` | 获取通知历史 | [详细](endpoints/notification.md#get_notification_history) |

### 配置相关

| 命令 | 方法 | 描述 | 文档 |
|------|------|------|------|
| `get_app_config` | - | 获取应用配置 | - |
| `update_app_config` | `{ config }` | 更新应用配置 | - |
| `get_storage_info` | - | 获取存储信息 | - |
| `cleanup_old_data` | `{ beforeDate }` | 清理旧数据 | - |

---

## 认证授权

### 当前版本

**本地应用，无需认证**

Vision-Jarvis 是本地应用，前后端运行在同一进程中，无需额外认证。

### 未来版本 (云同步)

如果未来支持云同步功能，将采用以下认证方案：

```typescript
// JWT 认证示例
const token = await invoke<string>('login', {
  username: 'user@example.com',
  password: 'password',
});

// 后续请求携带 token
await invoke('upload_memory', {
  token,
  memory: memoryData,
});
```

---

## 错误码说明

### 错误响应格式

```typescript
interface ErrorResponse {
  code: string;        // 错误码
  message: string;     // 错误描述
  details?: any;       // 详细信息
}
```

### 错误码列表

| 错误码 | 描述 | HTTP 状态 | 处理建议 |
|--------|------|-----------|---------|
| `PERMISSION_DENIED` | 权限被拒绝 | 403 | 引导用户授权屏幕录制权限 |
| `SCREENSHOT_CAPTURE_FAILED` | 截图捕获失败 | 500 | 重试或检查系统权限 |
| `IMAGE_PROCESSING_FAILED` | 图片处理失败 | 500 | 重试 |
| `AI_SERVICE_ERROR` | AI 服务错误 | 500 | 稍后重试 |
| `DATABASE_ERROR` | 数据库错误 | 500 | 联系支持 |
| `NOT_FOUND` | 资源不存在 | 404 | 检查 ID 是否正确 |
| `INVALID_PARAMS` | 参数无效 | 400 | 检查参数格式 |
| `TIME_WINDOW_INVALID` | 时间窗口无效 | 400 | 确保 start < end |
| `INSUFFICIENT_DATA` | 数据不足 | 400 | 至少需要 3 张截图 |

### 错误处理示例

```typescript
try {
  const screenshot = await invoke<Screenshot>('capture_screenshot');
} catch (error) {
  const err = error as ErrorResponse;

  switch (err.code) {
    case 'PERMISSION_DENIED':
      // 显示权限引导弹窗
      showPermissionModal();
      break;

    case 'SCREENSHOT_CAPTURE_FAILED':
      // Toast 提示并重试
      toast.error('截图失败，正在重试...');
      setTimeout(() => retryCapture(), 1000);
      break;

    default:
      // 通用错误处理
      toast.error(err.message);
  }
}
```

---

## 接口详细文档

### 详细文档目录

- **截图接口**: [endpoints/screenshot.md](endpoints/screenshot.md)
- **记忆接口**: [endpoints/memory.md](endpoints/memory.md)
- **AI 分析接口**: [endpoints/ai-analysis.md](endpoints/ai-analysis.md)
- **通知接口**: [endpoints/notification.md](endpoints/notification.md)

### 数据模型

- **Screenshot**: [models/Screenshot.md](models/Screenshot.md)
- **ShortTermMemory**: [models/ShortTermMemory.md](models/ShortTermMemory.md)
- **LongTermMemory**: [models/LongTermMemory.md](models/LongTermMemory.md)
- **Notification**: [models/Notification.md](models/Notification.md)

---

## 接口设计原则

### 1. 命名规范

- 使用 snake_case: `capture_screenshot`
- 动词开头: `get_`, `list_`, `update_`, `delete_`
- 清晰描述: `generate_long_term_memory` 优于 `gen_ltm`

### 2. 参数设计

- 使用结构体封装复杂参数
- 必填参数在前，可选参数在后
- 时间使用 ISO 8601 格式字符串

### 3. 响应设计

- 成功返回具体数据模型
- 失败返回标准错误结构
- 避免返回 `null`，使用 `Option<T>`

### 4. 版本控制

- 当前版本: v1
- 未来如有破坏性变更，使用 `v2_command_name` 前缀

---

## 性能优化

### 1. 批量操作

```typescript
// ❌ 不推荐：多次调用
for (const id of ids) {
  await invoke('delete_screenshot', { id });
}

// ✅ 推荐：批量删除
await invoke('batch_delete_screenshots', { ids });
```

### 2. 分页查询

```typescript
// 使用 limit 和 offset 分页
const screenshots = await invoke<Screenshot[]>('list_screenshots', {
  limit: 20,
  offset: 0,
});
```

### 3. 异步处理

```typescript
// 对于耗时操作，使用异步模式
const taskId = await invoke<string>('start_long_term_analysis', {
  dateRange: { start: '2026-01-01', end: '2026-01-31' },
});

// 轮询任务状态
const result = await pollTask(taskId);
```

---

## 相关文档

- [后端服务文档](../backend/services/README.md)
- [数据库设计](../database/README.md)
- [前端集成指南](../frontend/api-integration.md)

---

**维护者**: API 设计组
**最后更新**: 2026-02-04

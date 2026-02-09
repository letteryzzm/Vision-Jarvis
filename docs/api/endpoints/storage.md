# Storage API

文件存储管理 API 接口

## 概述

Storage API 提供文件系统管理功能，包括存储信息查询、文件列表、清理和删除操作。

## 位置

- **Commands 模块**: `src-tauri/src/commands/storage.rs`
- **Service 模块**: `src-tauri/src/storage/mod.rs`
- **行数**: 133 lines (commands)

## API 列表

| Command | 功能 | 参数 | 返回 |
|---------|------|------|------|
| `get_storage_info` | 获取存储信息 | - | `ApiResponse<StorageInfo>` |
| `list_files` | 列出文件 | folder_type, limit | `ApiResponse<Vec<FileInfo>>` |
| `cleanup_old_files` | 清理旧文件 | folder_type, days | `ApiResponse<usize>` |
| `open_folder` | 打开文件夹 | folder_type | `ApiResponse<String>` |
| `delete_file` | 删除文件 | file_path | `ApiResponse<bool>` |

---

## get_storage_info

获取存储信息，包括总使用量和各文件夹使用量。

### 函数签名

```rust
#[tauri::command]
pub async fn get_storage_info(
    state: State<'_, AppState>,
) -> Result<ApiResponse<StorageInfo>, String>
```

### 参数

无

### 返回值

```typescript
interface StorageInfo {
  total_used_bytes: number;     // 总使用量（字节）
  screenshots_bytes: number;     // 截图文件夹使用量
  memories_bytes: number;        // 记忆文件夹使用量
  database_bytes: number;        // 数据库使用量
  logs_bytes: number;            // 日志使用量
  temp_bytes: number;            // 临时文件使用量
  total_files: number;           // 文件总数
  root_path: string;             // 根路径
}
```

### 前端调用

```typescript
import { invoke } from '@tauri-apps/api/core';

const getStorageInfo = async () => {
  const response = await invoke<ApiResponse<StorageInfo>>('get_storage_info');

  if (response.success && response.data) {
    const info = response.data;
    console.log(`Total files: ${info.total_files}`);
    console.log(`Total size: ${(info.total_used_bytes / 1024 / 1024).toFixed(2)} MB`);
    console.log(`Screenshots: ${(info.screenshots_bytes / 1024 / 1024).toFixed(2)} MB`);
  }
};
```

### 示例响应

```json
{
  "success": true,
  "data": {
    "total_used_bytes": 524288000,
    "screenshots_bytes": 314572800,
    "memories_bytes": 104857600,
    "database_bytes": 52428800,
    "logs_bytes": 26214400,
    "temp_bytes": 26214400,
    "total_files": 1250,
    "root_path": "/Users/app/Vision-Jarvis/storage"
  },
  "error": null
}
```

---

## list_files

列出指定文件夹中的文件。

### 函数签名

```rust
#[tauri::command]
pub async fn list_files(
    state: State<'_, AppState>,
    folder_type: FolderType,
    limit: Option<usize>,
) -> Result<ApiResponse<Vec<FileInfo>>, String>
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| folder_type | FolderType | 是 | 文件夹类型 |
| limit | number \| null | 否 | 限制返回数量 |

**FolderType 枚举**:
```typescript
type FolderType =
  | "Screenshots"
  | "Memories"
  | "Database"
  | "Logs"
  | "Temp";
```

### 返回值

```typescript
interface FileInfo {
  name: string;           // 文件名
  path: string;           // 完整路径
  size_bytes: number;     // 文件大小（字节）
  created_at: number;     // 创建时间（Unix时间戳）
  modified_at: number;    // 修改时间（Unix时间戳）
  extension: string | null; // 文件扩展名
}
```

### 前端调用

```typescript
// 获取所有截图
const getAllScreenshots = async () => {
  const response = await invoke<ApiResponse<FileInfo[]>>('list_files', {
    folderType: 'Screenshots',
    limit: null
  });

  if (response.success && response.data) {
    console.log(`Found ${response.data.length} screenshots`);
  }
};

// 获取最新10个截图
const getRecentScreenshots = async () => {
  const response = await invoke<ApiResponse<FileInfo[]>>('list_files', {
    folderType: 'Screenshots',
    limit: 10
  });

  if (response.success && response.data) {
    response.data.forEach(file => {
      const date = new Date(file.modified_at * 1000);
      console.log(`${file.name} - ${file.size_bytes} bytes - ${date.toLocaleString()}`);
    });
  }
};
```

### 示例响应

```json
{
  "success": true,
  "data": [
    {
      "name": "screenshot_2026-02-06_15-30-45.png",
      "path": "/Users/app/storage/screenshots/screenshot_2026-02-06_15-30-45.png",
      "size_bytes": 524288,
      "created_at": 1738851045,
      "modified_at": 1738851045,
      "extension": "png"
    },
    {
      "name": "screenshot_2026-02-06_15-25-30.png",
      "path": "/Users/app/storage/screenshots/screenshot_2026-02-06_15-25-30.png",
      "size_bytes": 612352,
      "created_at": 1738850730,
      "modified_at": 1738850730,
      "extension": "png"
    }
  ],
  "error": null
}
```

**注意**: 返回的文件列表按修改时间倒序排列（最新的在前）。

---

## cleanup_old_files

清理指定天数之前的旧文件。

### 函数签名

```rust
#[tauri::command]
pub async fn cleanup_old_files(
    state: State<'_, AppState>,
    folder_type: FolderType,
    days: u64,
) -> Result<ApiResponse<usize>, String>
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| folder_type | FolderType | 是 | 文件夹类型 |
| days | number | 是 | 天数阈值 |

### 返回值

`number` - 删除的文件数量

### 前端调用

```typescript
// 清理30天前的临时文件
const cleanupTempFiles = async () => {
  const response = await invoke<ApiResponse<number>>('cleanup_old_files', {
    folderType: 'Temp',
    days: 30
  });

  if (response.success && response.data !== undefined) {
    console.log(`Deleted ${response.data} old files`);
  }
};

// 清理90天前的截图
const cleanupOldScreenshots = async () => {
  const response = await invoke<ApiResponse<number>>('cleanup_old_files', {
    folderType: 'Screenshots',
    days: 90
  });

  if (response.success && response.data !== undefined) {
    alert(`已删除 ${response.data} 个旧截图`);
  } else if (response.error) {
    alert(`清理失败: ${response.error}`);
  }
};
```

### 示例响应

```json
{
  "success": true,
  "data": 15,
  "error": null
}
```

---

## open_folder

在系统文件管理器中打开指定文件夹。

### 函数签名

```rust
#[tauri::command]
pub async fn open_folder(
    state: State<'_, AppState>,
    folder_type: FolderType,
) -> Result<ApiResponse<String>, String>
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| folder_type | FolderType | 是 | 文件夹类型 |

### 返回值

`string` - 成功消息

### 前端调用

```typescript
// 打开截图文件夹
const openScreenshotsFolder = async () => {
  const response = await invoke<ApiResponse<string>>('open_folder', {
    folderType: 'Screenshots'
  });

  if (response.success) {
    console.log(response.data); // "文件夹已打开"
  } else {
    alert(`打开失败: ${response.error}`);
  }
};
```

### 示例响应

```json
{
  "success": true,
  "data": "文件夹已打开",
  "error": null
}
```

**平台行为**:
- macOS: 使用 `open` 命令
- Windows: 使用 `explorer` 命令
- Linux: 使用 `xdg-open` 命令

---

## delete_file

删除单个文件（带安全检查）。

### 函数签名

```rust
#[tauri::command]
pub async fn delete_file(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ApiResponse<bool>, String>
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| file_path | string | 是 | 文件完整路径 |

### 返回值

`boolean` - 删除是否成功

### 安全检查

1. **路径验证**: 文件路径必须在存储根目录下
2. **存在性检查**: 文件必须存在
3. **权限检查**: 必须有删除权限

### 前端调用

```typescript
// 删除指定文件
const deleteFile = async (filePath: string) => {
  const response = await invoke<ApiResponse<boolean>>('delete_file', {
    filePath
  });

  if (response.success) {
    console.log('文件已删除');
  } else {
    alert(`删除失败: ${response.error}`);
  }
};

// 示例：删除截图
const deleteScreenshot = async (file: FileInfo) => {
  if (confirm(`确定要删除 ${file.name} 吗？`)) {
    await deleteFile(file.path);
  }
};
```

### 示例响应

**成功**:
```json
{
  "success": true,
  "data": true,
  "error": null
}
```

**失败（路径不安全）**:
```json
{
  "success": false,
  "data": null,
  "error": "不允许删除存储目录外的文件"
}
```

**失败（文件不存在）**:
```json
{
  "success": false,
  "data": null,
  "error": "文件不存在"
}
```

---

## 类型定义

完整的 TypeScript 类型定义：

```typescript
// API Response
interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

// Folder Type
type FolderType =
  | "Screenshots"
  | "Memories"
  | "Database"
  | "Logs"
  | "Temp";

// Storage Info
interface StorageInfo {
  total_used_bytes: number;
  screenshots_bytes: number;
  memories_bytes: number;
  database_bytes: number;
  logs_bytes: number;
  temp_bytes: number;
  total_files: number;
  root_path: string;
}

// File Info
interface FileInfo {
  name: string;
  path: string;
  size_bytes: number;
  created_at: number;
  modified_at: number;
  extension: string | null;
}
```

## 错误处理

所有 Storage API 都返回统一的 `ApiResponse` 格式：

```typescript
const handleStorageOperation = async () => {
  try {
    const response = await invoke<ApiResponse<StorageInfo>>('get_storage_info');

    if (response.success && response.data) {
      // 处理成功数据
      console.log(response.data);
    } else if (response.error) {
      // 处理错误
      console.error(response.error);
      alert(`操作失败: ${response.error}`);
    }
  } catch (error) {
    // 处理异常
    console.error('Invoke failed:', error);
    alert('调用失败，请检查后端服务');
  }
};
```

## 最佳实践

1. **定期清理**:
   ```typescript
   // 每周清理30天前的临时文件
   setInterval(async () => {
     await invoke('cleanup_old_files', {
       folderType: 'Temp',
       days: 30
     });
   }, 7 * 24 * 60 * 60 * 1000);
   ```

2. **存储监控**:
   ```typescript
   const checkStorage = async () => {
     const response = await invoke<ApiResponse<StorageInfo>>('get_storage_info');
     if (response.success && response.data) {
       const usedGB = response.data.total_used_bytes / 1024 / 1024 / 1024;
       if (usedGB > 10) {
         alert('存储空间超过10GB，建议清理');
       }
     }
   };
   ```

3. **安全删除**:
   ```typescript
   // 始终使用 delete_file API，不要直接操作文件系统
   const safeDelete = async (filePath: string) => {
     const response = await invoke<ApiResponse<boolean>>('delete_file', {
       filePath
     });
     return response.success;
   };
   ```

## 相关文档

- [Storage Service](../../backend/services/storage-service.md) - Backend service reference
- [File Management Page](../../frontend/pages/files.md) - Frontend implementation

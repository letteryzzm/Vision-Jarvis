# Storage Service

文件存储管理服务

## 概述

Storage Service 提供完整的文件系统管理功能，支持多种文件夹类型的组织和管理。

## 位置

- **模块路径**: `src-tauri/src/storage/mod.rs`
- **行数**: 350 lines
- **测试**: 6 个单元测试

## 核心类型

### FolderType

文件夹类型枚举，定义了5种标准文件夹：

```rust
pub enum FolderType {
    Screenshots,  // 截图文件夹
    Memories,     // 记忆文件夹
    Database,     // 数据库文件夹
    Logs,         // 日志文件夹
    Temp,         // 临时文件夹
}
```

### StorageInfo

存储信息结构体：

```rust
pub struct StorageInfo {
    pub total_used_bytes: u64,      // 总使用量（字节）
    pub screenshots_bytes: u64,      // 截图文件夹使用量
    pub memories_bytes: u64,         // 记忆文件夹使用量
    pub database_bytes: u64,         // 数据库使用量
    pub logs_bytes: u64,             // 日志使用量
    pub temp_bytes: u64,             // 临时文件使用量
    pub total_files: u64,            // 文件总数
    pub root_path: String,           // 根路径
}
```

### FileInfo

文件信息结构体：

```rust
pub struct FileInfo {
    pub name: String,           // 文件名
    pub path: String,           // 完整路径
    pub size_bytes: u64,        // 文件大小（字节）
    pub created_at: i64,        // 创建时间（Unix时间戳）
    pub modified_at: i64,       // 修改时间（Unix时间戳）
    pub extension: Option<String>, // 文件扩展名
}
```

## StorageManager

### 初始化

```rust
pub fn new(root_path: PathBuf) -> Result<Self>
```

创建新的存储管理器，自动创建根目录。

**参数**:
- `root_path`: 存储根目录路径

**返回**: `Result<Self>`

**示例**:
```rust
let manager = StorageManager::new(PathBuf::from("/Users/app/storage"))?;
```

### 方法

#### ensure_folder

```rust
pub fn ensure_folder(&self, folder_type: &FolderType) -> Result<PathBuf>
```

确保指定文件夹存在，如果不存在则创建。

**参数**:
- `folder_type`: 文件夹类型

**返回**: `Result<PathBuf>` - 文件夹路径

#### get_folder_path

```rust
pub fn get_folder_path(&self, folder_type: &FolderType) -> PathBuf
```

获取文件夹路径（不检查是否存在）。

#### get_storage_info

```rust
pub fn get_storage_info(&self) -> Result<StorageInfo>
```

获取存储信息，包括总使用量和各文件夹使用量。

**返回**: `Result<StorageInfo>`

**示例**:
```rust
let info = manager.get_storage_info()?;
println!("Total used: {} MB", info.total_used_bytes / 1024 / 1024);
```

#### list_files

```rust
pub fn list_files(
    &self,
    folder_type: &FolderType,
    limit: Option<usize>,
) -> Result<Vec<FileInfo>>
```

列出指定文件夹中的文件。

**参数**:
- `folder_type`: 文件夹类型
- `limit`: 限制返回数量（None表示返回所有）

**返回**: `Result<Vec<FileInfo>>` - 按修改时间倒序排列的文件列表

**示例**:
```rust
// 获取最新的10个截图
let files = manager.list_files(&FolderType::Screenshots, Some(10))?;
```

#### cleanup_old_files

```rust
pub fn cleanup_old_files(&self, folder_type: &FolderType, days: u64) -> Result<usize>
```

清理指定天数之前的旧文件。

**参数**:
- `folder_type`: 文件夹类型
- `days`: 天数阈值

**返回**: `Result<usize>` - 删除的文件数量

**示例**:
```rust
// 删除30天前的临时文件
let deleted = manager.cleanup_old_files(&FolderType::Temp, 30)?;
println!("Deleted {} old files", deleted);
```

#### delete_file

```rust
pub fn delete_file(&self, file_path: &str) -> Result<()>
```

删除单个文件（带安全检查）。

**参数**:
- `file_path`: 文件路径

**安全检查**:
- 验证文件路径在存储根目录下
- 验证文件存在

**错误**:
- 如果文件不在存储根目录下，返回错误
- 如果文件不存在，返回错误

## 安全特性

### 路径遍历保护

在 `delete_file` 方法中实现了路径遍历保护：

```rust
// 安全检查：确保文件在存储根目录下
if !path.starts_with(&self.root_path) {
    anyhow::bail!("不允许删除存储目录外的文件");
}
```

这防止了恶意用户通过构造特殊路径（如 `../../etc/passwd`）删除系统文件。

### 文件存在性检查

��除前验证文件是否存在：

```rust
if !path.exists() {
    anyhow::bail!("文件不存在");
}
```

## 单元测试

测试覆盖：

1. `test_storage_manager_creation` - 存储管理器创建
2. `test_ensure_folder` - 文件夹创建
3. `test_get_storage_info_empty` - 空存储信息
4. `test_list_files` - 文件列表
5. `test_delete_file` - 文件删除
6. `test_delete_file_outside_root` - 路径遍历保护

所有测试使用 `tempfile::TempDir` 创建临时目录进行隔离测试。

## 使用示例

### 完整示例

```rust
use vision_jarvis::storage::{StorageManager, FolderType};
use std::path::PathBuf;

// 创建存储管理器
let manager = StorageManager::new(PathBuf::from("/app/storage"))?;

// 确保截图文件夹存在
let screenshots_path = manager.ensure_folder(&FolderType::Screenshots)?;

// 获取存储信息
let info = manager.get_storage_info()?;
println!("Total files: {}", info.total_files);
println!("Total size: {} MB", info.total_used_bytes / 1024 / 1024);

// 列出最新的20个文件
let files = manager.list_files(&FolderType::Screenshots, Some(20))?;
for file in files {
    println!("{}: {} bytes", file.name, file.size_bytes);
}

// 清理30天前的临时文件
let deleted = manager.cleanup_old_files(&FolderType::Temp, 30)?;
println!("Cleaned up {} old files", deleted);
```

## 最佳实践

1. **定期清理**: 使用 `cleanup_old_files` 定期清理旧文件
2. **监控存储**: 定期调用 `get_storage_info` 监控磁盘使用
3. **安全删除**: 始终使用 `delete_file` 方法而不是直接文件系统操作
4. **错误处理**: 正确处理所有 `Result` 类型的返回值

## 相关文档

- [Storage Commands](../../api/endpoints/storage.md) - Tauri commands for storage
- [File Management API](../../api/endpoints/storage.md) - Frontend API reference

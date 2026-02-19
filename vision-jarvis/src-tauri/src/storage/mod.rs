/// 文件存储管理模块
///
/// 提供文件系统管理、存储信息查询和文件清理功能

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 文件夹类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FolderType {
    Screenshots,
    Recordings,
    Memories,
    Database,
    Logs,
    Temp,
}

impl FolderType {
    pub fn folder_name(&self) -> &str {
        match self {
            FolderType::Screenshots => "screenshots",
            FolderType::Recordings => "recordings",
            FolderType::Memories => "memories",
            FolderType::Database => "database",
            FolderType::Logs => "logs",
            FolderType::Temp => "temp",
        }
    }
}

/// 存储信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub total_used_bytes: u64,
    pub screenshots_bytes: u64,
    pub recordings_bytes: u64,
    pub memories_bytes: u64,
    pub database_bytes: u64,
    pub logs_bytes: u64,
    pub temp_bytes: u64,
    pub total_files: u64,
    pub root_path: String,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// 文件名
    pub name: String,
    /// 完整路径
    pub path: String,
    /// 文件大小（字节）
    pub size_bytes: u64,
    /// 创建时间（Unix时间戳）
    pub created_at: i64,
    /// 修改时间（Unix时间戳）
    pub modified_at: i64,
    /// 文件扩展名
    pub extension: Option<String>,
}

/// 存储管理器
pub struct StorageManager {
    root_path: PathBuf,
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new(root_path: PathBuf) -> Result<Self> {
        // 确保根目录存在
        fs::create_dir_all(&root_path)
            .context("创建存储根目录失败")?;

        Ok(Self { root_path })
    }

    /// 确保指定文件夹存在
    pub fn ensure_folder(&self, folder_type: &FolderType) -> Result<PathBuf> {
        let folder_path = self.root_path.join(folder_type.folder_name());
        fs::create_dir_all(&folder_path)
            .context(format!("创建 {} 文件夹失败", folder_type.folder_name()))?;
        Ok(folder_path)
    }

    /// 获取文件夹路径
    pub fn get_folder_path(&self, folder_type: &FolderType) -> PathBuf {
        self.root_path.join(folder_type.folder_name())
    }

    /// 计算目录大小
    fn calculate_dir_size(&self, path: &Path) -> Result<(u64, u64)> {
        let mut total_size = 0u64;
        let mut file_count = 0u64;

        if !path.exists() {
            return Ok((0, 0));
        }

        for entry in fs::read_dir(path).context("读取目录失败")? {
            let entry = entry.context("读取目录项失败")?;
            let metadata = entry.metadata().context("读取元数据失败")?;

            if metadata.is_file() {
                total_size += metadata.len();
                file_count += 1;
            } else if metadata.is_dir() {
                let (sub_size, sub_count) = self.calculate_dir_size(&entry.path())?;
                total_size += sub_size;
                file_count += sub_count;
            }
        }

        Ok((total_size, file_count))
    }

    /// 获取存储信息
    pub fn get_storage_info(&self) -> Result<StorageInfo> {
        let folders = [
            FolderType::Screenshots,
            FolderType::Recordings,
            FolderType::Memories,
            FolderType::Database,
            FolderType::Logs,
            FolderType::Temp,
        ];
        let sizes: Vec<(u64, u64)> = folders.iter()
            .map(|f| self.calculate_dir_size(&self.get_folder_path(f)).unwrap_or((0, 0)))
            .collect();

        let total_used_bytes: u64 = sizes.iter().map(|(b, _)| b).sum();
        let total_files: u64 = sizes.iter().map(|(_, c)| c).sum();

        Ok(StorageInfo {
            total_used_bytes,
            screenshots_bytes: sizes[0].0,
            recordings_bytes: sizes[1].0,
            memories_bytes: sizes[2].0,
            database_bytes: sizes[3].0,
            logs_bytes: sizes[4].0,
            temp_bytes: sizes[5].0,
            total_files,
            root_path: self.root_path.to_string_lossy().to_string(),
        })
    }

    /// 列出指定文件夹中的文件
    pub fn list_files(
        &self,
        folder_type: &FolderType,
        limit: Option<usize>,
    ) -> Result<Vec<FileInfo>> {
        let folder_path = self.get_folder_path(folder_type);

        if !folder_path.exists() {
            return Ok(Vec::new());
        }

        let mut files: Vec<FileInfo> = Vec::new();

        for entry in fs::read_dir(&folder_path).context("读取目录失败")? {
            let entry = entry.context("读取目录项失败")?;
            let metadata = entry.metadata().context("读取元数据失败")?;

            if metadata.is_file() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let created_at = metadata
                    .created()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                let modified_at = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                let extension = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string());

                files.push(FileInfo {
                    name,
                    path: path.to_string_lossy().to_string(),
                    size_bytes: metadata.len(),
                    created_at,
                    modified_at,
                    extension,
                });
            }
        }

        // 按修改时间倒序排列
        files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

        // 限制返回数量
        if let Some(limit) = limit {
            files.truncate(limit);
        }

        Ok(files)
    }

    /// 清理指定天数之前的旧文件（递归遍历子目录）
    pub fn cleanup_old_files(&self, folder_type: &FolderType, days: u64) -> Result<usize> {
        let folder_path = self.get_folder_path(folder_type);
        if !folder_path.exists() {
            return Ok(0);
        }
        let threshold = SystemTime::now() - Duration::from_secs(days * 24 * 60 * 60);
        self.cleanup_recursive(&folder_path, threshold)
    }

    fn cleanup_recursive(&self, dir: &Path, threshold: SystemTime) -> Result<usize> {
        let mut deleted = 0;
        for entry in fs::read_dir(dir).context("读取目录失败")? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                if let Ok(modified) = metadata.modified() {
                    if modified < threshold {
                        fs::remove_file(entry.path())?;
                        deleted += 1;
                    }
                }
            } else if metadata.is_dir() {
                deleted += self.cleanup_recursive(&entry.path(), threshold)?;
                // 删除空子目录
                if fs::read_dir(entry.path())?.next().is_none() {
                    let _ = fs::remove_dir(entry.path());
                }
            }
        }
        Ok(deleted)
    }

    /// 删除单个文件
    pub fn delete_file(&self, file_path: &str) -> Result<()> {
        let path = Path::new(file_path);

        // 安全检查：确保文件在存储根目录下
        if !path.starts_with(&self.root_path) {
            anyhow::bail!("不允许删除存储目录外的文件");
        }

        if !path.exists() {
            anyhow::bail!("文件不存在");
        }

        fs::remove_file(path).context("删除文件失败")?;

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_storage_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_ensure_folder() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf()).unwrap();

        let path = manager.ensure_folder(&FolderType::Screenshots).unwrap();
        assert!(path.exists());
        assert!(path.ends_with("screenshots"));
    }

    #[test]
    fn test_get_storage_info_empty() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf()).unwrap();

        let info = manager.get_storage_info().unwrap();
        assert_eq!(info.total_used_bytes, 0);
        assert_eq!(info.total_files, 0);
    }

    #[test]
    fn test_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 创建截图文件夹和测试文件
        let screenshots_path = manager.ensure_folder(&FolderType::Screenshots).unwrap();
        let test_file = screenshots_path.join("test.png");
        let mut f = fs::File::create(&test_file).unwrap();
        f.write_all(b"test content").unwrap();

        let files = manager.list_files(&FolderType::Screenshots, None).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "test.png");
    }

    #[test]
    fn test_delete_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 创建测试文件
        let screenshots_path = manager.ensure_folder(&FolderType::Screenshots).unwrap();
        let test_file = screenshots_path.join("to_delete.png");
        fs::File::create(&test_file).unwrap();

        assert!(test_file.exists());

        manager
            .delete_file(test_file.to_str().unwrap())
            .unwrap();

        assert!(!test_file.exists());
    }

    #[test]
    fn test_delete_file_outside_root() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf()).unwrap();

        let result = manager.delete_file("/etc/passwd");
        assert!(result.is_err());
    }
}

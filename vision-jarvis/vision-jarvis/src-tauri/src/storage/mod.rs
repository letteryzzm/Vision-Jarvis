/// 文件存储管理模块

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// 文件夹类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FolderType {
    Screenshots,
    ShortTermMemory,
    LongTermMemory,
}

impl FolderType {
    /// 获取文件夹名称
    pub fn folder_name(&self) -> &str {
        match self {
            FolderType::Screenshots => "screenshots",
            FolderType::ShortTermMemory => "short_term_memory",
            FolderType::LongTermMemory => "long_term_memory",
        }
    }
}

/// 存储信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub total_size_mb: f64,
    pub screenshots_size_mb: f64,
    pub short_term_size_mb: f64,
    pub long_term_size_mb: f64,
    pub screenshots_count: usize,
    pub short_term_count: usize,
    pub long_term_count: usize,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: i64,
    pub modified_at: i64,
}

/// 存储管理器
pub struct StorageManager {
    base_path: PathBuf,
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new(base_path: PathBuf) -> Result<Self> {
        // 确保基础目录存在
        fs::create_dir_all(&base_path)
            .context("创建存储目录失败")?;

        Ok(Self { base_path })
    }

    /// 获取文件夹路径
    pub fn get_folder_path(&self, folder_type: &FolderType) -> PathBuf {
        self.base_path.join(folder_type.folder_name())
    }

    /// 确保文件夹存在
    pub fn ensure_folder(&self, folder_type: &FolderType) -> Result<PathBuf> {
        let folder_path = self.get_folder_path(folder_type);
        fs::create_dir_all(&folder_path)
            .context(format!("创建{}文件夹失败", folder_type.folder_name()))?;
        Ok(folder_path)
    }

    /// 获取存储信息
    pub fn get_storage_info(&self) -> Result<StorageInfo> {
        let screenshots_info = self.get_folder_info(&FolderType::Screenshots)?;
        let short_term_info = self.get_folder_info(&FolderType::ShortTermMemory)?;
        let long_term_info = self.get_folder_info(&FolderType::LongTermMemory)?;

        let total_size_bytes = screenshots_info.0 + short_term_info.0 + long_term_info.0;

        Ok(StorageInfo {
            total_size_mb: bytes_to_mb(total_size_bytes),
            screenshots_size_mb: bytes_to_mb(screenshots_info.0),
            short_term_size_mb: bytes_to_mb(short_term_info.0),
            long_term_size_mb: bytes_to_mb(long_term_info.0),
            screenshots_count: screenshots_info.1,
            short_term_count: short_term_info.1,
            long_term_count: long_term_info.1,
        })
    }

    /// 获取文件夹信息 (总大小, 文件数量)
    fn get_folder_info(&self, folder_type: &FolderType) -> Result<(u64, usize)> {
        let folder_path = self.get_folder_path(folder_type);

        if !folder_path.exists() {
            return Ok((0, 0));
        }

        let mut total_size = 0u64;
        let mut count = 0usize;

        for entry in fs::read_dir(&folder_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                total_size += metadata.len();
                count += 1;
            }
        }

        Ok((total_size, count))
    }

    /// 列出文件
    pub fn list_files(&self, folder_type: &FolderType, limit: Option<usize>) -> Result<Vec<FileInfo>> {
        let folder_path = self.get_folder_path(folder_type);

        if !folder_path.exists() {
            return Ok(Vec::new());
        }

        let mut files: Vec<FileInfo> = Vec::new();

        for entry in fs::read_dir(&folder_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                let created_at = metadata.created()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                let modified_at = metadata.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                files.push(FileInfo {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: entry.path().to_string_lossy().to_string(),
                    size_bytes: metadata.len(),
                    created_at,
                    modified_at,
                });
            }
        }

        // 按修改时间倒序排序
        files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

        // 应用限制
        if let Some(limit) = limit {
            files.truncate(limit);
        }

        Ok(files)
    }

    /// 清理旧文件
    pub fn cleanup_old_files(&self, folder_type: &FolderType, days: u64) -> Result<usize> {
        let folder_path = self.get_folder_path(folder_type);

        if !folder_path.exists() {
            return Ok(0);
        }

        let cutoff_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() - (days * 24 * 3600);

        let mut deleted_count = 0;

        for entry in fs::read_dir(&folder_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                let modified = metadata.modified()?
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();

                if modified < cutoff_time {
                    fs::remove_file(entry.path())?;
                    deleted_count += 1;
                    log::info!("删除旧文件: {:?}", entry.path());
                }
            }
        }

        Ok(deleted_count)
    }

    /// 删除单个文件
    pub fn delete_file(&self, file_path: &str) -> Result<()> {
        let path = Path::new(file_path);

        // 安全检查：确保文件在允许的目录下
        if !path.starts_with(&self.base_path) {
            anyhow::bail!("不允许删除此路径的文件");
        }

        if path.exists() {
            fs::remove_file(path)
                .context("删除文件失败")?;
            log::info!("删除文件: {}", file_path);
        }

        Ok(())
    }

    /// 获取基础路径
    pub fn get_base_path(&self) -> &Path {
        &self.base_path
    }
}

/// 字节转换为MB
fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let folder = manager.ensure_folder(&FolderType::Screenshots).unwrap();
        assert!(folder.exists());
    }

    #[test]
    fn test_bytes_to_mb() {
        assert_eq!(bytes_to_mb(1024 * 1024), 1.0);
        assert_eq!(bytes_to_mb(512 * 1024), 0.5);
    }
}

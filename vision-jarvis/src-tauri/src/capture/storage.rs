/// 截图存储管理
///
/// 负责存储容量管理和自动清理

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;

/// 存储管理器
pub struct StorageManager {
    storage_path: PathBuf,
    limit_bytes: u64,
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new(storage_path: PathBuf, limit_mb: u64) -> Self {
        Self {
            storage_path,
            limit_bytes: limit_mb * 1024 * 1024,
        }
    }

    /// 获取当前存储使用量（字节）
    pub fn get_current_usage(&self) -> Result<u64> {
        let mut total_size = 0u64;

        if !self.storage_path.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(&self.storage_path)
            .context("读取存储目录失败")? {
            let entry = entry.context("读取目录项失败")?;
            let metadata = entry.metadata()
                .context("读取文件元数据失败")?;

            if metadata.is_file() {
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }

    /// 检查是否超出存储限制
    pub fn is_over_limit(&self) -> Result<bool> {
        let usage = self.get_current_usage()?;
        Ok(usage > self.limit_bytes)
    }

    /// 清理旧截图直到低于限制
    pub fn cleanup_old_screenshots(&self) -> Result<u64> {
        let mut files_info: Vec<(PathBuf, u64, i64)> = Vec::new();

        // 收集所有文件信息
        for entry in fs::read_dir(&self.storage_path)
            .context("读取存储目录失败")? {
            let entry = entry.context("读取目录项失败")?;
            let path = entry.path();
            let metadata = entry.metadata()
                .context("读取文件元数据失败")?;

            if metadata.is_file() && path.extension().map_or(false, |ext| ext == "png") {
                // 从文件名提取时间戳
                if let Some(filename) = path.file_stem() {
                    if let Some(timestamp_str) = filename.to_str()
                        .and_then(|s| s.split('_').next()) {
                        if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                            files_info.push((path, metadata.len(), timestamp));
                        }
                    }
                }
            }
        }

        // 按时间戳排序（旧的在前）
        files_info.sort_by_key(|(_, _, timestamp)| *timestamp);

        let mut current_usage = self.get_current_usage()?;
        let mut deleted_count = 0u64;

        // 删除旧文件直到低于限制的80%
        let target = (self.limit_bytes as f64 * 0.8) as u64;

        for (path, size, _) in files_info {
            if current_usage <= target {
                break;
            }

            fs::remove_file(&path)
                .context(format!("删除文件失败: {:?}", path))?;

            current_usage -= size;
            deleted_count += 1;
        }

        Ok(deleted_count)
    }

    /// 获取存储限制（字节）
    pub fn get_limit_bytes(&self) -> u64 {
        self.limit_bytes
    }

    /// 获取存储路径
    pub fn storage_path(&self) -> &Path {
        &self.storage_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;

    #[test]
    fn test_storage_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf(), 100);

        assert_eq!(manager.get_limit_bytes(), 100 * 1024 * 1024);
        assert_eq!(manager.storage_path(), temp_dir.path());
    }

    #[test]
    fn test_get_current_usage_empty() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf(), 100);

        let usage = manager.get_current_usage().unwrap();
        assert_eq!(usage, 0);
    }

    #[test]
    fn test_get_current_usage_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf(), 100);

        // 创建测试文件
        let file1 = temp_dir.path().join("test1.png");
        let mut f1 = fs::File::create(&file1).unwrap();
        f1.write_all(&vec![0u8; 1024]).unwrap(); // 1KB

        let file2 = temp_dir.path().join("test2.png");
        let mut f2 = fs::File::create(&file2).unwrap();
        f2.write_all(&vec![0u8; 2048]).unwrap(); // 2KB

        let usage = manager.get_current_usage().unwrap();
        assert_eq!(usage, 3072); // 3KB
    }

    #[test]
    fn test_is_over_limit() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf(), 1); // 1MB 限制

        // 创建小文件
        let file = temp_dir.path().join("small.png");
        let mut f = fs::File::create(&file).unwrap();
        f.write_all(&vec![0u8; 1024]).unwrap(); // 1KB

        assert!(!manager.is_over_limit().unwrap());

        // 创建大文件超出限制
        let file2 = temp_dir.path().join("large.png");
        let mut f2 = fs::File::create(&file2).unwrap();
        f2.write_all(&vec![0u8; 2 * 1024 * 1024]).unwrap(); // 2MB

        assert!(manager.is_over_limit().unwrap());
    }

    #[test]
    fn test_cleanup_old_screenshots() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf(), 1); // 1MB 限制

        // 创建多个带时间戳的文件
        for i in 0..5 {
            let filename = format!("{}_{}.png", 1000000 + i, uuid::Uuid::new_v4());
            let path = temp_dir.path().join(filename);
            let mut f = fs::File::create(&path).unwrap();
            f.write_all(&vec![0u8; 500 * 1024]).unwrap(); // 每个 500KB
        }

        // 总共 2.5MB，超出 1MB 限制
        assert!(manager.is_over_limit().unwrap());

        // 执行清理
        let deleted = manager.cleanup_old_screenshots().unwrap();
        assert!(deleted > 0);

        // 清理后应低于限制的 80%
        let usage = manager.get_current_usage().unwrap();
        assert!(usage <= (manager.get_limit_bytes() as f64 * 0.8) as u64);
    }
}

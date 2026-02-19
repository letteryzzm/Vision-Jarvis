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

/// 递归计算目录大小
fn dir_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    if !path.exists() {
        return Ok(0);
    }
    for entry in fs::read_dir(path).context("读取目录失败")? {
        let entry = entry.context("读取目录项失败")?;
        let metadata = entry.metadata().context("读取元数据失败")?;
        if metadata.is_dir() {
            total += dir_size(&entry.path())?;
        } else {
            total += metadata.len();
        }
    }
    Ok(total)
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new(storage_path: PathBuf, limit_mb: u64) -> Self {
        Self {
            storage_path,
            limit_bytes: limit_mb * 1024 * 1024,
        }
    }

    /// shots 目录路径
    fn shots_path(&self) -> PathBuf {
        self.storage_path.join("shots")
    }

    /// 获取当前存储使用量（字节）- 递归计算 shots/ 下所有文件
    pub fn get_current_usage(&self) -> Result<u64> {
        dir_size(&self.shots_path())
    }

    /// 检查是否超出存储限制
    pub fn is_over_limit(&self) -> Result<bool> {
        let usage = self.get_current_usage()?;
        Ok(usage > self.limit_bytes)
    }

    /// 清理旧截图直到低于���制
    /// 按日期目录从旧到新删除
    pub fn cleanup_old_screenshots(&self) -> Result<u64> {
        let shots_dir = self.shots_path();
        if !shots_dir.exists() {
            return Ok(0);
        }

        // 收集日期目录并排序（旧的在前）
        let mut date_dirs: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(&shots_dir).context("读取 shots 目录失败")? {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                date_dirs.push(entry.path());
            }
        }
        date_dirs.sort();

        let mut current_usage = self.get_current_usage()?;
        let target = (self.limit_bytes as f64 * 0.8) as u64;
        let mut deleted_count = 0u64;

        for date_dir in date_dirs {
            if current_usage <= target {
                break;
            }

            // 收集该日期目录下所有文件
            let mut files: Vec<(PathBuf, u64)> = Vec::new();
            Self::collect_files_recursive(&date_dir, &mut files)?;

            for (path, size) in files {
                if current_usage <= target {
                    break;
                }
                fs::remove_file(&path)
                    .context(format!("删除文件失败: {:?}", path))?;
                current_usage -= size;
                deleted_count += 1;
            }

            // 清理空目录
            Self::remove_empty_dirs(&date_dir)?;
        }

        Ok(deleted_count)
    }

    /// 递归收集目录下所有文件
    fn collect_files_recursive(dir: &Path, files: &mut Vec<(PathBuf, u64)>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                Self::collect_files_recursive(&path, files)?;
            } else {
                files.push((path, metadata.len()));
            }
        }
        Ok(())
    }

    /// 递归删除空目录
    fn remove_empty_dirs(dir: &Path) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                Self::remove_empty_dirs(&entry.path())?;
            }
        }
        // 如果目录为空则删除
        if fs::read_dir(dir)?.next().is_none() {
            fs::remove_dir(dir)?;
        }
        Ok(())
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
    fn test_get_current_usage_with_nested_files() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf(), 100);

        // 创建 shots/20231027/0_00-12_00/ 结构
        let period_dir = temp_dir.path().join("shots").join("20231027").join("0_00-12_00");
        fs::create_dir_all(&period_dir).unwrap();

        let file1 = period_dir.join("10-00-01_test.jpg");
        let mut f1 = fs::File::create(&file1).unwrap();
        f1.write_all(&vec![0u8; 1024]).unwrap();

        let file2 = period_dir.join("11-00-01_test.jpg");
        let mut f2 = fs::File::create(&file2).unwrap();
        f2.write_all(&vec![0u8; 2048]).unwrap();

        let usage = manager.get_current_usage().unwrap();
        assert_eq!(usage, 3072);
    }

    #[test]
    fn test_is_over_limit() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf(), 1); // 1MB

        let period_dir = temp_dir.path().join("shots").join("20231027").join("0_00-12_00");
        fs::create_dir_all(&period_dir).unwrap();

        // 小文件
        let file = period_dir.join("small.jpg");
        let mut f = fs::File::create(&file).unwrap();
        f.write_all(&vec![0u8; 1024]).unwrap();
        assert!(!manager.is_over_limit().unwrap());

        // 大文件超出限制
        let file2 = period_dir.join("large.jpg");
        let mut f2 = fs::File::create(&file2).unwrap();
        f2.write_all(&vec![0u8; 2 * 1024 * 1024]).unwrap();
        assert!(manager.is_over_limit().unwrap());
    }

    #[test]
    fn test_cleanup_old_screenshots() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path().to_path_buf(), 1); // 1MB

        // 创建多个日期目录的文件
        for day in 27..32 {
            let period_dir = temp_dir.path().join("shots").join(format!("202310{}", day)).join("0_00-12_00");
            fs::create_dir_all(&period_dir).unwrap();
            let path = period_dir.join(format!("10-00-01_{}.jpg", uuid::Uuid::new_v4()));
            let mut f = fs::File::create(&path).unwrap();
            f.write_all(&vec![0u8; 500 * 1024]).unwrap();
        }

        assert!(manager.is_over_limit().unwrap());

        let deleted = manager.cleanup_old_screenshots().unwrap();
        assert!(deleted > 0);

        let usage = manager.get_current_usage().unwrap();
        assert!(usage <= (manager.get_limit_bytes() as f64 * 0.8) as u64);
    }
}

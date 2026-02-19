/// 索引管理器 - 负责文件扫描、增量索引和存储
///
/// NOTE: Embedding 生成将在记忆系统重新设计时实现
/// 当前仅支持文本分块和存储（无向量索引）
///
/// 核心功能：
/// 1. 递归扫描Markdown文件
/// 2. 文件变更检测（基于哈希）
/// 3. 文本分块存储

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs;
use chrono::Utc;
use sha2::{Sha256, Digest};
use uuid::Uuid;

use crate::db::Database;
use super::chunker::{Chunker, ChunkConfig, TextChunk};

/// 索引管理器配置
#[derive(Debug, Clone)]
pub struct IndexConfig {
    /// 内存文件根目录
    pub memory_root: PathBuf,
    /// 分块配置
    pub chunk_config: ChunkConfig,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            memory_root: PathBuf::from("./memory"),
            chunk_config: ChunkConfig::default(),
        }
    }
}

/// 索引管理器
pub struct IndexManager {
    db: Arc<Database>,
    chunker: Chunker,
    config: IndexConfig,
}

/// 文件元数据
#[derive(Debug)]
struct FileMetadata {
    path: String,
    hash: String,
    mtime: i64,
    size: i64,
}

impl IndexManager {
    pub fn new(
        db: Arc<Database>,
        config: IndexConfig,
    ) -> Self {
        let chunker = Chunker::new(config.chunk_config.clone());
        Self {
            db,
            chunker,
            config,
        }
    }

    /// 执行增量同步
    pub async fn sync(&self) -> Result<SyncStats> {
        let mut stats = SyncStats::default();

        // 扫描所有Markdown文件
        let markdown_files = self.scan_markdown_files()?;
        stats.total_files = markdown_files.len();

        for file_path in markdown_files {
            match self.process_file(&file_path).await {
                Ok(file_stats) => {
                    stats.merge(file_stats);
                }
                Err(e) => {
                    log::error!("Failed to process file {}: {}", file_path.display(), e);
                    stats.failed_files += 1;
                }
            }
        }

        Ok(stats)
    }

    /// 处理单个文件
    async fn process_file(&self, file_path: &Path) -> Result<SyncStats> {
        let mut stats = SyncStats::default();

        // 读取文件内容
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        // 计算文件元数据
        let metadata = self.compute_file_metadata(file_path, &content)?;

        // 检查是否需要更新
        if !self.needs_update(&metadata)? {
            stats.skipped_files += 1;
            return Ok(stats);
        }

        // 删除旧的chunks
        self.delete_chunks_for_file(&metadata.path)?;

        // 分块
        let chunks = self.chunker.chunk_markdown(&content)?;
        if chunks.is_empty() {
            log::warn!("No chunks generated for file: {}", file_path.display());
            stats.indexed_files += 1;
            self.save_file_metadata(&metadata)?;
            return Ok(stats);
        }

        // 保存chunks（不含 embedding，待记忆系统重新设计时实现）
        self.save_chunks(&metadata.path, &chunks)?;

        // 更新文件元数据
        self.save_file_metadata(&metadata)?;

        stats.indexed_files += 1;
        stats.new_chunks += chunks.len();

        Ok(stats)
    }

    /// 扫描Markdown文件
    fn scan_markdown_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if !self.config.memory_root.exists() {
            return Ok(files);
        }

        self.scan_directory(&self.config.memory_root, &mut files)?;

        Ok(files)
    }

    /// 递归扫描目录
    fn scan_directory(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                files.push(path);
            }
        }

        Ok(())
    }

    /// 计算文件元数据
    fn compute_file_metadata(&self, file_path: &Path, content: &str) -> Result<FileMetadata> {
        let relative_path = file_path.strip_prefix(&self.config.memory_root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let metadata = fs::metadata(file_path)?;
        let mtime = metadata.modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let size = metadata.len() as i64;

        let hash = compute_file_hash(content);

        Ok(FileMetadata {
            path: relative_path,
            hash,
            mtime,
            size,
        })
    }

    /// 检查文件是否需要更新
    fn needs_update(&self, metadata: &FileMetadata) -> Result<bool> {
        self.db.with_connection(|conn| {
            let result: Option<String> = conn
                .prepare("SELECT hash FROM memory_files WHERE path = ?1")?
                .query_row([&metadata.path], |row| row.get(0))
                .ok();

            match result {
                Some(stored_hash) => Ok(stored_hash != metadata.hash),
                None => Ok(true), // 新文件
            }
        })
    }

    /// 保存文件元数据
    fn save_file_metadata(&self, metadata: &FileMetadata) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO memory_files
                 (path, source, hash, mtime, size)
                 VALUES (?1, 'activity', ?2, ?3, ?4)",
                rusqlite::params![
                    &metadata.path,
                    &metadata.hash,
                    metadata.mtime,
                    metadata.size,
                ],
            )?;
            Ok(())
        })
    }

    /// 删除文件的所有chunks
    fn delete_chunks_for_file(&self, file_path: &str) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "DELETE FROM memory_chunks WHERE file_path = ?1",
                [file_path],
            )?;
            Ok(())
        })
    }

    /// 保存chunks（不含 embedding）
    fn save_chunks(
        &self,
        file_path: &str,
        chunks: &[TextChunk],
    ) -> Result<()> {
        let now = Utc::now().timestamp();
        let empty_blob: Vec<u8> = Vec::new();

        self.db.with_connection(|conn| {
            for chunk in chunks {
                let id = Uuid::new_v4().to_string();

                conn.execute(
                    "INSERT INTO memory_chunks
                     (id, file_path, source, start_line, end_line, hash, model, text, embedding, updated_at)
                     VALUES (?1, ?2, 'activity', ?3, ?4, ?5, '', ?6, ?7, ?8)",
                    rusqlite::params![
                        id,
                        file_path,
                        chunk.start_line,
                        chunk.end_line,
                        &chunk.hash,
                        &chunk.text,
                        &empty_blob,
                        now,
                    ],
                )?;
            }
            Ok(())
        })
    }
}

/// 同步统计
#[derive(Debug, Default)]
pub struct SyncStats {
    pub total_files: usize,
    pub indexed_files: usize,
    pub skipped_files: usize,
    pub failed_files: usize,
    pub new_chunks: usize,
}

impl SyncStats {
    fn merge(&mut self, other: SyncStats) {
        self.indexed_files += other.indexed_files;
        self.skipped_files += other.skipped_files;
        self.failed_files += other.failed_files;
        self.new_chunks += other.new_chunks;
    }
}

/// 计算文件哈希
fn compute_file_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_file_hash() {
        let hash1 = compute_file_hash("test content");
        let hash2 = compute_file_hash("test content");
        let hash3 = compute_file_hash("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 16);
    }
}

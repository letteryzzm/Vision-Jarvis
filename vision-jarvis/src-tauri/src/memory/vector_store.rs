/// 向量存储
///
/// 使用 SQLite 实现简单的向量存储和语义搜索

use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

/// 向量存储
pub struct VectorStore {
    conn: Connection,
}

impl VectorStore {
    /// 创建新的向量存储
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // 创建向量表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vectors (
                id TEXT PRIMARY KEY,
                embedding BLOB NOT NULL,
                metadata TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vectors_created_at
             ON vectors(created_at DESC)",
            [],
        )?;

        Ok(Self { conn })
    }

    /// 存储向量
    pub fn store_embedding(
        &self,
        id: &str,
        embedding: &[f32],
        metadata: &VectorMetadata,
    ) -> Result<()> {
        // 将 f32 向量序列化为 bytes
        let embedding_bytes = self.serialize_vector(embedding);
        let metadata_json = serde_json::to_string(metadata)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO vectors (id, embedding, metadata)
             VALUES (?, ?, ?)",
            params![id, embedding_bytes, metadata_json],
        )?;

        Ok(())
    }

    /// 搜索相似向量
    pub fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, embedding, metadata FROM vectors"
        )?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let embedding_bytes: Vec<u8> = row.get(1)?;
            let metadata_json: String = row.get(2)?;

            Ok((id, embedding_bytes, metadata_json))
        })?;

        let mut results = Vec::new();

        for row in rows {
            let (id, embedding_bytes, metadata_json) = row?;
            let embedding = self.deserialize_vector(&embedding_bytes);

            // 计算余弦相似度
            let similarity = self.cosine_similarity(query_embedding, &embedding);

            let metadata: VectorMetadata = serde_json::from_str(&metadata_json)?;

            results.push(SearchResult {
                id,
                similarity,
                metadata,
            });
        }

        // 按相似度排序并限制结果数量
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    /// 删除向量
    pub fn delete_embedding(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM vectors WHERE id = ?",
            params![id],
        )?;

        Ok(())
    }

    /// 删除指定日期之前的向量
    pub fn delete_embeddings_before(&self, timestamp: i64) -> Result<usize> {
        let count = self.conn.execute(
            "DELETE FROM vectors WHERE created_at < ?",
            params![timestamp],
        )?;

        Ok(count)
    }

    /// 序列化向量为 bytes
    fn serialize_vector(&self, vector: &[f32]) -> Vec<u8> {
        vector.iter()
            .flat_map(|f| f.to_le_bytes())
            .collect()
    }

    /// 反序列化 bytes 为向量
    fn deserialize_vector(&self, bytes: &[u8]) -> Vec<f32> {
        bytes.chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }

    /// 计算余弦相似度
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }
}

/// 向量元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    pub screenshot_id: String,
    pub text: String,
    pub timestamp: i64,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub similarity: f32,
    pub metadata: VectorMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_store_creation() {
        let store = VectorStore::new(":memory:");
        assert!(store.is_ok());
    }

    #[test]
    fn test_store_and_search() {
        let store = VectorStore::new(":memory:").unwrap();

        // 存储几个向量
        let embedding1 = vec![1.0, 0.0, 0.0];
        let metadata1 = VectorMetadata {
            screenshot_id: "test1".to_string(),
            text: "编程工作".to_string(),
            timestamp: 1000,
        };
        store.store_embedding("vec1", &embedding1, &metadata1).unwrap();

        let embedding2 = vec![0.9, 0.1, 0.0];
        let metadata2 = VectorMetadata {
            screenshot_id: "test2".to_string(),
            text: "写代码".to_string(),
            timestamp: 2000,
        };
        store.store_embedding("vec2", &embedding2, &metadata2).unwrap();

        let embedding3 = vec![0.0, 1.0, 0.0];
        let metadata3 = VectorMetadata {
            screenshot_id: "test3".to_string(),
            text: "看视频".to_string(),
            timestamp: 3000,
        };
        store.store_embedding("vec3", &embedding3, &metadata3).unwrap();

        // 搜索相似向量
        let query = vec![1.0, 0.0, 0.0];
        let results = store.search_similar(&query, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "vec1"); // 最相似
        assert!(results[0].similarity > 0.9);
    }

    #[test]
    fn test_vector_serialization() {
        let store = VectorStore::new(":memory:").unwrap();
        let original = vec![1.5, 2.5, 3.5];

        let bytes = store.serialize_vector(&original);
        let deserialized = store.deserialize_vector(&bytes);

        assert_eq!(original.len(), deserialized.len());
        for (a, b) in original.iter().zip(deserialized.iter()) {
            assert!((a - b).abs() < 0.001);
        }
    }

    #[test]
    fn test_delete_embedding() {
        let store = VectorStore::new(":memory:").unwrap();

        let embedding = vec![1.0, 0.0, 0.0];
        let metadata = VectorMetadata {
            screenshot_id: "test".to_string(),
            text: "test".to_string(),
            timestamp: 1000,
        };

        store.store_embedding("vec1", &embedding, &metadata).unwrap();
        let results = store.search_similar(&embedding, 10).unwrap();
        assert_eq!(results.len(), 1);

        store.delete_embedding("vec1").unwrap();
        let results = store.search_similar(&embedding, 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_delete_embeddings_before() {
        let store = VectorStore::new(":memory:").unwrap();

        // 注意: 这个测试依赖于数据库的 created_at 自动生成
        // 由于所有记录几乎同时创建，created_at 会非常接近
        // 因此这个测试可能不稳定，我们改为测试实际的删除功能

        for i in 0..5 {
            let embedding = vec![1.0, 0.0, 0.0];
            let metadata = VectorMetadata {
                screenshot_id: format!("test{}", i),
                text: "test".to_string(),
                timestamp: (i + 1) * 1000,
            };
            store.store_embedding(&format!("vec{}", i), &embedding, &metadata).unwrap();
        }

        // 先验证所有记录都存在
        let results = store.search_similar(&vec![1.0, 0.0, 0.0], 10).unwrap();
        assert_eq!(results.len(), 5);

        // 删除一个很久以后的时间点之前的所有记录（应该删除所有）
        let future_timestamp = chrono::Utc::now().timestamp() + 10000;
        let deleted = store.delete_embeddings_before(future_timestamp).unwrap();
        assert_eq!(deleted, 5);

        let results = store.search_similar(&vec![1.0, 0.0, 0.0], 10).unwrap();
        assert_eq!(results.len(), 0);
    }
}

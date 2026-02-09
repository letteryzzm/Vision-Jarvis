/// 混合搜索引擎 - 结合向量搜索和关键词搜索
///
/// 搜索策略：
/// - Vector Search (70%权重): Cosine相似度
/// - Keyword Search (30%权重): SQL LIKE匹配
/// - 最终得分 = 0.7 * vector_score + 0.3 * keyword_score

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;

use crate::db::Database;
use crate::ai::embeddings::EmbeddingGenerator;

/// 搜索配置
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// 向量搜索权重
    pub vector_weight: f32,
    /// 关键词搜索权重
    pub keyword_weight: f32,
    /// 最大返回结果数
    pub max_results: usize,
    /// 最小相似度阈值
    pub min_similarity: f32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.7,
            keyword_weight: 0.3,
            max_results: 20,
            min_similarity: 0.3,
        }
    }
}

/// 混合搜索引擎
pub struct HybridSearch {
    db: Arc<Database>,
    embedder: EmbeddingGenerator,
    config: SearchConfig,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Chunk ID
    pub chunk_id: String,
    /// 文件路径
    pub file_path: String,
    /// 分块文本
    pub text: String,
    /// 起始行号
    pub start_line: i32,
    /// 结束行号
    pub end_line: i32,
    /// 综合得分
    pub score: f32,
    /// 向量得分
    pub vector_score: f32,
    /// 关键词得分
    pub keyword_score: f32,
    /// 关联的activity_id
    pub activity_id: Option<String>,
}

impl HybridSearch {
    pub fn new(
        db: Arc<Database>,
        embedder: EmbeddingGenerator,
        config: SearchConfig,
    ) -> Self {
        Self { db, embedder, config }
    }

    /// 执行混合搜索
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // 1. 向量搜索
        let query_embedding = self.embedder.generate_embedding(query).await?;
        let vector_results = self.vector_search(&query_embedding)?;

        // 2. 关键词搜索
        let keyword_results = self.keyword_search(query)?;

        // 3. 合并结果
        let merged = self.merge_results(vector_results, keyword_results);

        // 4. 按得分排序和过滤
        let mut filtered: Vec<_> = merged.into_iter()
            .filter(|r| r.score >= self.config.min_similarity)
            .collect();

        filtered.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        filtered.truncate(self.config.max_results);

        Ok(filtered)
    }

    /// 向量搜索（暴力cosine相似度）
    fn vector_search(&self, query_embedding: &[f32]) -> Result<HashMap<String, f32>> {
        let mut results = HashMap::new();

        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, embedding FROM memory_chunks"
            )?;

            let rows = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let embedding_blob: Vec<u8> = row.get(1)?;
                Ok((id, embedding_blob))
            })?;

            for row in rows {
                let (id, blob) = row?;

                // 反序列化embedding
                let chunk_embedding = deserialize_embedding(&blob)?;

                // 计算cosine相似度
                let similarity = cosine_similarity(query_embedding, &chunk_embedding);

                results.insert(id, similarity);
            }

            Ok(())
        })?;

        Ok(results)
    }

    /// 关键词搜索（SQL LIKE）
    fn keyword_search(&self, query: &str) -> Result<HashMap<String, f32>> {
        let mut results = HashMap::new();
        let pattern = format!("%{}%", query);

        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, text FROM memory_chunks WHERE text LIKE ?1"
            )?;

            let rows = stmt.query_map([&pattern], |row| {
                let id: String = row.get(0)?;
                let text: String = row.get(1)?;
                Ok((id, text))
            })?;

            for row in rows {
                let (id, text) = row?;

                // 简单的匹配度计算（出现次数）
                let score = calculate_keyword_score(&text, query);
                results.insert(id, score);
            }

            Ok(())
        })?;

        Ok(results)
    }

    /// 合并向量和关键词搜索结果
    fn merge_results(
        &self,
        vector_results: HashMap<String, f32>,
        keyword_results: HashMap<String, f32>,
    ) -> Vec<SearchResult> {
        let mut chunk_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        chunk_ids.extend(vector_results.keys().cloned());
        chunk_ids.extend(keyword_results.keys().cloned());

        let mut results = Vec::new();

        for chunk_id in chunk_ids {
            let vector_score = vector_results.get(&chunk_id).copied().unwrap_or(0.0);
            let keyword_score = keyword_results.get(&chunk_id).copied().unwrap_or(0.0);

            let combined_score =
                self.config.vector_weight * vector_score +
                self.config.keyword_weight * keyword_score;

            // 获取chunk详细信息
            if let Ok(chunk_info) = self.get_chunk_info(&chunk_id) {
                results.push(SearchResult {
                    chunk_id: chunk_id.clone(),
                    file_path: chunk_info.0,
                    text: chunk_info.1,
                    start_line: chunk_info.2,
                    end_line: chunk_info.3,
                    score: combined_score,
                    vector_score,
                    keyword_score,
                    activity_id: chunk_info.4,
                });
            }
        }

        results
    }

    /// 获取chunk详细信息
    fn get_chunk_info(&self, chunk_id: &str) -> Result<(String, String, i32, i32, Option<String>)> {
        self.db.with_connection(|conn| {
            conn.query_row(
                "SELECT file_path, text, start_line, end_line, activity_id
                 FROM memory_chunks
                 WHERE id = ?1",
                [chunk_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            ).map_err(Into::into)
        })
    }

    /// 按activity聚合搜索结果
    pub fn group_by_activity(&self, results: Vec<SearchResult>) -> HashMap<String, Vec<SearchResult>> {
        let mut grouped: HashMap<String, Vec<SearchResult>> = HashMap::new();

        for result in results {
            // 从file_path提取activity_id（假设路径格式: activities/YYYY-MM-DD/activity-XXX.md）
            let activity_key = result.activity_id.clone()
                .or_else(|| extract_activity_from_path(&result.file_path))
                .unwrap_or_else(|| "unknown".to_string());

            grouped.entry(activity_key)
                .or_insert_with(Vec::new)
                .push(result);
        }

        grouped
    }
}

/// 计算cosine相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// 计算关键词匹配得分
fn calculate_keyword_score(text: &str, query: &str) -> f32 {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    // 计算查询词在文本中出现的次数
    let count = text_lower.matches(&query_lower).count();

    // 归一化到0-1范围（使用对数缩放）
    if count == 0 {
        0.0
    } else {
        // 最大得分为1.0，每多出现一次增加对数值
        (1.0 + (count as f32).ln()) / (1.0 + 10.0_f32.ln())
    }
}

/// 从文件路径提取activity ID
fn extract_activity_from_path(path: &str) -> Option<String> {
    // 路径格式: activities/2024-01-15/activity-001.md
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 3 && parts[0] == "activities" {
        // 提取日期和编号: 2024-01-15/activity-001
        Some(format!("{}/{}", parts[1], parts[2].trim_end_matches(".md")))
    } else {
        None
    }
}

/// 反序列化BLOB为embedding
fn deserialize_embedding(blob: &[u8]) -> Result<Vec<f32>> {
    if blob.len() % 4 != 0 {
        anyhow::bail!("Invalid embedding blob length");
    }

    let embedding: Vec<f32> = blob
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .collect();

    Ok(embedding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let vec = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&vec, &vec);
        assert!((similarity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let similarity = cosine_similarity(&a, &b);
        assert!((similarity - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![-1.0, -2.0, -3.0];
        let similarity = cosine_similarity(&a, &b);
        assert!((similarity + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_different_length() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&a, &b);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_calculate_keyword_score() {
        let text = "This is a test text with test word appearing twice";

        let score1 = calculate_keyword_score(text, "test");
        assert!(score1 > 0.0);

        let score2 = calculate_keyword_score(text, "missing");
        assert_eq!(score2, 0.0);

        // 出现次数多的得分更高
        let text2 = "test test test test test";
        let score3 = calculate_keyword_score(text2, "test");
        assert!(score3 > score1);
    }

    #[test]
    fn test_calculate_keyword_score_case_insensitive() {
        let text = "Testing Test TEST";
        let score = calculate_keyword_score(text, "test");
        assert!(score > 0.0);
    }

    #[test]
    fn test_extract_activity_from_path() {
        let path1 = "activities/2024-01-15/activity-001.md";
        assert_eq!(
            extract_activity_from_path(path1),
            Some("2024-01-15/activity-001".to_string())
        );

        let path2 = "other/path/file.md";
        assert_eq!(extract_activity_from_path(path2), None);

        let path3 = "activities/2024-01-15";
        assert_eq!(extract_activity_from_path(path3), None);
    }

    #[test]
    fn test_deserialize_embedding() {
        // 创建测试BLOB: 3个float [1.0, 2.0, 3.0]
        let blob: Vec<u8> = vec![
            0, 0, 128, 63,  // 1.0
            0, 0, 0, 64,    // 2.0
            0, 0, 64, 64,   // 3.0
        ];

        let result = deserialize_embedding(&blob).unwrap();
        assert_eq!(result.len(), 3);
        assert!((result[0] - 1.0).abs() < 1e-6);
        assert!((result[1] - 2.0).abs() < 1e-6);
        assert!((result[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_search_config_default() {
        let config = SearchConfig::default();
        assert_eq!(config.vector_weight, 0.7);
        assert_eq!(config.keyword_weight, 0.3);
        assert_eq!(config.max_results, 20);
        assert_eq!(config.min_similarity, 0.3);
    }
}

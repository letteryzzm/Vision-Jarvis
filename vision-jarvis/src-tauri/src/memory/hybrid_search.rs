/// 混合搜索引擎 - 关键词搜索
///
/// NOTE: 向量搜索（Embedding）将在记忆系统重新设计时实现
/// 当前仅支持关键词搜索

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;

use crate::db::Database;

/// 搜索配置
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// 最大返回结果数
    pub max_results: usize,
    /// 最小相似度阈值
    pub min_similarity: f32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 20,
            min_similarity: 0.3,
        }
    }
}

/// 混合搜索引擎
pub struct HybridSearch {
    db: Arc<Database>,
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
    /// 关键词得分
    pub keyword_score: f32,
    /// 关联的activity_id
    pub activity_id: Option<String>,
}

impl HybridSearch {
    pub fn new(
        db: Arc<Database>,
        config: SearchConfig,
    ) -> Self {
        Self { db, config }
    }

    /// 执行搜索（当前仅关键词搜索）
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // 关键词搜索
        let keyword_results = self.keyword_search(query)?;

        // 按得分排序和过滤
        let mut filtered: Vec<_> = keyword_results.into_iter()
            .filter(|r| r.score >= self.config.min_similarity)
            .collect();

        filtered.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        filtered.truncate(self.config.max_results);

        Ok(filtered)
    }

    /// 关键词搜索（SQL LIKE）
    fn keyword_search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let pattern = format!("%{}%", query);

        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, file_path, text, start_line, end_line, activity_id
                 FROM memory_chunks WHERE text LIKE ?1"
            )?;

            let rows = stmt.query_map([&pattern], |row| {
                let id: String = row.get(0)?;
                let file_path: String = row.get(1)?;
                let text: String = row.get(2)?;
                let start_line: i32 = row.get(3)?;
                let end_line: i32 = row.get(4)?;
                let activity_id: Option<String> = row.get(5)?;
                Ok((id, file_path, text, start_line, end_line, activity_id))
            })?;

            for row in rows {
                let (id, file_path, text, start_line, end_line, activity_id) = row?;

                let score = calculate_keyword_score(&text, query);
                results.push(SearchResult {
                    chunk_id: id,
                    file_path,
                    text,
                    start_line,
                    end_line,
                    score,
                    keyword_score: score,
                    activity_id,
                });
            }

            Ok(())
        })?;

        Ok(results)
    }

    /// 按activity聚合搜索结果
    pub fn group_by_activity(&self, results: Vec<SearchResult>) -> HashMap<String, Vec<SearchResult>> {
        let mut grouped: HashMap<String, Vec<SearchResult>> = HashMap::new();

        for result in results {
            let activity_key = result.activity_id.clone()
                .or_else(|| extract_activity_from_path(&result.file_path))
                .unwrap_or_else(|| "unknown".to_string());

            grouped.entry(activity_key)
                .or_default()
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

    let count = text_lower.matches(&query_lower).count();

    if count == 0 {
        0.0
    } else {
        (1.0 + (count as f32).ln()) / (1.0 + 10.0_f32.ln())
    }
}

/// 从文件路径提取activity ID
fn extract_activity_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 3 && parts[0] == "activities" {
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
        assert_eq!(config.max_results, 20);
        assert_eq!(config.min_similarity, 0.3);
    }
}

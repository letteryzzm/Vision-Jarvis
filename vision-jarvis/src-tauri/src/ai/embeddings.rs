/// 向量嵌入生成
///
/// 使用 OpenAI text-embedding-3-small 生成文本向量

use super::*;

/// 嵌入生成器
pub struct EmbeddingGenerator {
    client: OpenAIClient,
}

impl EmbeddingGenerator {
    /// 创建新的生成器
    pub fn new(api_key: String) -> Result<Self> {
        let client = OpenAIClient::new(api_key)?;
        Ok(Self { client })
    }

    /// 生成文本嵌入向量
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
            input: text.to_string(),
        };

        let response = self.client.create_embedding(request).await?;

        let embedding = response.data.first()
            .context("响应中没有嵌入数据")?
            .embedding
            .clone();

        Ok(embedding)
    }

    /// 批量生成嵌入向量
    pub async fn generate_embeddings_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        for text in texts {
            let embedding = self.generate_embedding(&text).await?;
            embeddings.push(embedding);

            // 简单的速率限制：每次请求间隔 100ms
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(embeddings)
    }

    /// 计算余弦相似度
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let generator = EmbeddingGenerator::new("test-key".to_string());
        assert!(generator.is_ok());
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let similarity = EmbeddingGenerator::cosine_similarity(&a, &b);
        assert!((similarity - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        let similarity = EmbeddingGenerator::cosine_similarity(&c, &d);
        assert!(similarity.abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let similarity = EmbeddingGenerator::cosine_similarity(&a, &b);
        assert_eq!(similarity, 0.0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_generate_embedding_with_real_api() {
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let generator = EmbeddingGenerator::new(api_key).unwrap();
            let result = generator.generate_embedding("Test text").await;
            assert!(result.is_ok());
            let embedding = result.unwrap();
            assert_eq!(embedding.len(), 1536); // text-embedding-3-small 维度
        }
    }
}

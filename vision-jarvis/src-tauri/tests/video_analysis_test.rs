/// 视频分析集成测试
///
/// 运行: AIHUBMIX_KEY=your_key cargo test --test video_analysis_test -- --ignored --nocapture

#[cfg(test)]
mod tests {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as BASE64;
    use vision_jarvis_lib::ai::{AIProviderConfig, AIClient};

    fn create_client() -> AIClient {
        let config = AIProviderConfig::new(
            "aihubmix",
            "AIHubMix",
            "https://aihubmix.com",
            "sk-nCVTUgLBOA3XRzzMAfBc519b96894b089000DaBe93F012Ca",
            "gemini-2.5-flash-lite-preview-09-2025",
        );
        AIClient::new(config).unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn test_video_analysis() {
        let video_path = "/Users/lettery/Library/Application Support/stepfun-desktop/desktop-share/video/20260215/1771120852667-1771121737873-composite.mp4";

        let video_data = tokio::fs::read(video_path).await
            .expect("读取视频文件失败");
        println!("视频大小: {:.1}MB", video_data.len() as f64 / 1024.0 / 1024.0);

        let video_base64 = BASE64.encode(&video_data);
        println!("Base64大小: {:.1}MB", video_base64.len() as f64 / 1024.0 / 1024.0);

        let client = create_client();

        let prompt = r#"分析这段屏幕录制视频，严格按JSON格式返回：
{
  "application": "所有应用名称",
  "activity_type": "work|entertainment|communication|learning|other",
  "activity_description": "用户在做什么",
  "key_elements": ["关键元素"],
  "context_tags": ["标签"],
  "productivity_score": 5
}
只返回JSON。"#;

        println!("正在发送视频到AI分析...");
        let result = client.analyze_video(&video_base64, prompt).await;

        match result {
            Ok(response) => {
                println!("=== AI 分析结果 ===");
                println!("{}", response);
            }
            Err(e) => {
                panic!("视频分析失败: {}", e);
            }
        }
    }
}

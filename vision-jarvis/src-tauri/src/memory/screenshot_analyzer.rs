/// 录制分段AI分析器
///
/// 负责将屏幕录制视频发送给AI进行理解，提取结构化信息
/// 输出存入 screenshot_analyses 表，供后续活动分组和模式学习使用

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use log::{info, warn, error};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::ai::AIClient;
use crate::db::Database;
use crate::db::schema::ScreenshotAnalysis;

/// AI返回的分析结果（用于JSON解析）
/// 一次性提取所有下游组件需要的信息
#[derive(Debug, serde::Deserialize)]
struct AIAnalysisResult {
    application: String,
    activity_type: String,
    activity_description: String,
    #[serde(default)]
    key_elements: Vec<String>,
    ocr_text: Option<String>,
    #[serde(default)]
    context_tags: Vec<String>,
    #[serde(default = "default_productivity_score")]
    productivity_score: i32,
    #[serde(default = "default_activity_category")]
    activity_category: String,
    #[serde(default)]
    activity_summary: String,
    #[serde(default)]
    project_name: Option<String>,
    #[serde(default)]
    accomplishments: Vec<String>,
}

fn default_productivity_score() -> i32 {
    5
}

fn default_activity_category() -> String {
    "other".to_string()
}

/// 分析配置
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// 分析失败后的最大重试次数
    pub max_retries: u32,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            max_retries: 2,
        }
    }
}

/// 录制分段分析器
pub struct ScreenshotAnalyzer {
    ai_client: Arc<AIClient>,
    db: Arc<Database>,
    config: AnalyzerConfig,
}

impl ScreenshotAnalyzer {
    pub fn new(ai_client: Arc<AIClient>, db: Arc<Database>, config: AnalyzerConfig) -> Self {
        Self { ai_client, db, config }
    }

    /// 分析单个录制分段
    pub async fn analyze_recording(
        &self,
        recording_id: &str,
        video_path: &Path,
    ) -> Result<ScreenshotAnalysis> {
        info!("读取视频文件: {} (exists={})", video_path.display(), video_path.exists());
        let video_data = tokio::fs::read(video_path).await?;
        info!("视频文件大小: {} bytes, base64约: {} bytes", video_data.len(), video_data.len() * 4 / 3);
        let video_base64 = BASE64.encode(&video_data);

        let prompt = recording_understanding_prompt();
        let response = self.ai_client.analyze_video(&video_base64, &prompt).await
            .map_err(|e| anyhow::anyhow!("AI视频分析失败: {}", e))?;

        let ai_result = parse_ai_response(&response)?;
        let now = chrono::Utc::now().timestamp();

        let analysis = ScreenshotAnalysis {
            screenshot_id: recording_id.to_string(),
            application: ai_result.application,
            activity_type: ai_result.activity_type,
            activity_description: ai_result.activity_description,
            key_elements: ai_result.key_elements,
            ocr_text: ai_result.ocr_text,
            context_tags: ai_result.context_tags,
            productivity_score: ai_result.productivity_score.clamp(1, 10),
            analysis_json: response.clone(),
            analyzed_at: now,
            activity_category: ai_result.activity_category,
            activity_summary: ai_result.activity_summary,
            project_name: ai_result.project_name,
            accomplishments: ai_result.accomplishments,
        };

        self.save_analysis(&analysis)?;
        self.mark_recording_analyzed(recording_id)?;
        Ok(analysis)
    }

    /// 即时分析单条录制（由 channel 事件驱动，跳过 DB 查询）
    pub async fn analyze_single_direct(&self, id: &str, path: &std::path::Path) -> Result<()> {
        if !path.exists() {
            warn!("录制文件不存在，跳过: {}", path.display());
            self.mark_recording_analyzed(id)?;
            return Ok(());
        }
        let short_id = &id[..8.min(id.len())];
        info!("即时分析录制: {}...", short_id);
        for attempt in 0..=self.config.max_retries {
            match self.analyze_recording(id, path).await {
                Ok(_) => {
                    info!("即时分析完成: {}", short_id);
                    return Ok(());
                }
                Err(e) if attempt < self.config.max_retries => {
                    warn!("即时分析失败(重试{}/{}): {} - {}", attempt + 1, self.config.max_retries, short_id, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }
                Err(e) => {
                    error!("即时分析失败(已重试{}次): {} - {}", self.config.max_retries, short_id, e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// 标记录制为已分析
    fn mark_recording_analyzed(&self, recording_id: &str) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "UPDATE recordings SET analyzed = 1 WHERE id = ?1",
                [recording_id],
            )?;
            Ok(())
        })
    }

    /// 保存分析结果到数据库
    fn save_analysis(&self, analysis: &ScreenshotAnalysis) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO screenshot_analyses (
                    screenshot_id, application, activity_type, activity_description,
                    key_elements, ocr_text, context_tags, productivity_score,
                    analysis_json, analyzed_at,
                    activity_category, activity_summary, project_name, accomplishments
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                rusqlite::params![
                    &analysis.screenshot_id,
                    &analysis.application,
                    &analysis.activity_type,
                    &analysis.activity_description,
                    serde_json::to_string(&analysis.key_elements)?,
                    &analysis.ocr_text,
                    serde_json::to_string(&analysis.context_tags)?,
                    analysis.productivity_score,
                    &analysis.analysis_json,
                    analysis.analyzed_at,
                    &analysis.activity_category,
                    &analysis.activity_summary,
                    &analysis.project_name,
                    serde_json::to_string(&analysis.accomplishments)?,
                ],
            )?;
            Ok(())
        })
    }
}

/// 录制分段理解Prompt
fn recording_understanding_prompt() -> String {
    r#"分析这段屏幕录制视频，提取以下信息。严格按JSON格式返回，不要包含其他文字：

{
  "application": "主要使用的应用名称",
  "activity_type": "work|entertainment|communication|learning|other",
  "activity_description": "用户在这段时间内做了什么（一句话，要具体）",
  "activity_category": "work|entertainment|communication|other",
  "activity_summary": "这段时间的活动概述（供时间线展示）",
  "key_elements": ["关键元素1", "关键元素2"],
  "ocr_text": "屏幕上的重要文本（简要提取）",
  "context_tags": ["标签1", "标签2"],
  "productivity_score": 5,
  "project_name": "项目名称或null",
  "accomplishments": ["完成了XX", "修改了YY"]
}

要求：
1. application: 识别视频中主要使用的应用程序
2. activity_type: 只能是 work/entertainment/communication/learning/other 之一
3. activity_description: 综合整段视频描述用户活动（如"在VSCode中编写Rust代码并调试"）
4. activity_category: 只能是 work/entertainment/communication/other 之一
5. activity_summary: 简明概述这段时间的活动（供时间线展示）
6. key_elements: 提取窗口标题、文件名、网页标题等关键信息
7. ocr_text: 仅提取重要文本
8. context_tags: 2-5个描述当前上下文的标签
9. productivity_score: 1=纯娱乐 5=一般 10=深度工作
10. project_name: 如果能识别出用户在做什么项目则填写项目名（如"Vision-Jarvis"、"论文写作"），无法识别返回null
11. accomplishments: 这段时间的成果要点（1-3条），没有明显成果则返回空数组

只返回JSON，不要其他内容。"#.to_string()
}

/// 解析AI返回的JSON
fn parse_ai_response(response: &str) -> Result<AIAnalysisResult> {
    // 尝试直接解析
    if let Ok(result) = serde_json::from_str::<AIAnalysisResult>(response) {
        return Ok(result);
    }

    // 尝试提取JSON块（AI可能返回markdown代码块）
    let json_str = extract_json_from_response(response);
    serde_json::from_str::<AIAnalysisResult>(&json_str)
        .map_err(|e| anyhow::anyhow!("解析AI响应JSON失败: {} - 原始响应: {}", e, response))
}

/// 从AI响应中提取JSON（处理markdown代码块等情况）
fn extract_json_from_response(response: &str) -> String {
    // 尝试提取 ```json ... ``` 块
    if let Some(start) = response.find("```json") {
        let after_marker = &response[start + 7..];
        if let Some(end) = after_marker.find("```") {
            return after_marker[..end].trim().to_string();
        }
    }

    // 尝试提取 ``` ... ``` 块
    if let Some(start) = response.find("```") {
        let after_marker = &response[start + 3..];
        if let Some(end) = after_marker.find("```") {
            return after_marker[..end].trim().to_string();
        }
    }

    // 尝试提取第一个 { ... } 块
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            return response[start..=end].to_string();
        }
    }

    response.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clean_json() {
        let json = r#"{"application":"VSCode","activity_type":"work","activity_description":"编写Rust代码","key_elements":["main.rs"],"ocr_text":null,"context_tags":["coding","rust"],"productivity_score":8}"#;
        let result = parse_ai_response(json).unwrap();
        assert_eq!(result.application, "VSCode");
        assert_eq!(result.activity_type, "work");
        assert_eq!(result.productivity_score, 8);
    }

    #[test]
    fn test_parse_markdown_wrapped_json() {
        let response = r#"这是分析结果：

```json
{
  "application": "Chrome",
  "activity_type": "entertainment",
  "activity_description": "浏览B站视频",
  "key_elements": ["bilibili.com"],
  "ocr_text": null,
  "context_tags": ["video", "entertainment"],
  "productivity_score": 2
}
```"#;
        let result = parse_ai_response(response).unwrap();
        assert_eq!(result.application, "Chrome");
        assert_eq!(result.productivity_score, 2);
    }

    #[test]
    fn test_parse_with_extra_text() {
        let response = r#"根据录制分析：
{
  "application": "微信",
  "activity_type": "communication",
  "activity_description": "与同事聊天",
  "key_elements": ["群聊"],
  "context_tags": ["chat"],
  "productivity_score": 3
}
以上是分析结果。"#;
        let result = parse_ai_response(response).unwrap();
        assert_eq!(result.application, "微信");
        assert_eq!(result.activity_type, "communication");
    }

    #[test]
    fn test_default_productivity_score() {
        let json = r#"{"application":"Terminal","activity_type":"work","activity_description":"使用终端","key_elements":[],"context_tags":[]}"#;
        let result = parse_ai_response(json).unwrap();
        assert_eq!(result.productivity_score, 5);
    }

    #[test]
    fn test_extract_json_from_code_block() {
        let input = "```json\n{\"a\":1}\n```";
        assert_eq!(extract_json_from_response(input), "{\"a\":1}");
    }

    #[test]
    fn test_extract_json_from_braces() {
        let input = "some text {\"a\":1} more text";
        assert_eq!(extract_json_from_response(input), "{\"a\":1}");
    }

    #[test]
    fn test_recording_understanding_prompt_not_empty() {
        let prompt = recording_understanding_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("application"));
        assert!(prompt.contains("activity_type"));
    }
}

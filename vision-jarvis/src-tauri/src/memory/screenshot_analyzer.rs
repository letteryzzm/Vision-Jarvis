/// 截图AI分析器
///
/// 负责将截屏图片发送给AI进行理解，提取结构化信息
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

/// AI返回的截图理解结果（用于JSON解析）
#[derive(Debug, serde::Deserialize)]
struct AIScreenshotResult {
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
}

fn default_productivity_score() -> i32 {
    5
}

/// 分析配置
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// 每批分析的最大截图数
    pub batch_size: usize,
    /// 分析失败后的最大重试次数
    pub max_retries: u32,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            max_retries: 2,
        }
    }
}

/// 截图分析器
pub struct ScreenshotAnalyzer {
    ai_client: Arc<AIClient>,
    db: Arc<Database>,
    config: AnalyzerConfig,
}

impl ScreenshotAnalyzer {
    pub fn new(ai_client: Arc<AIClient>, db: Arc<Database>, config: AnalyzerConfig) -> Self {
        Self { ai_client, db, config }
    }

    /// 分析单张截图
    pub async fn analyze_screenshot(
        &self,
        screenshot_id: &str,
        image_path: &Path,
    ) -> Result<ScreenshotAnalysis> {
        // 1. 读取图片并转base64
        let image_data = tokio::fs::read(image_path).await?;
        let image_base64 = BASE64.encode(&image_data);

        // 2. 调用AI分析
        let prompt = screenshot_understanding_prompt();
        let response = self.ai_client.analyze_image(&image_base64, &prompt).await
            .map_err(|e| anyhow::anyhow!("AI分析失败: {}", e))?;

        // 3. 解析JSON结果
        let ai_result = parse_ai_response(&response)?;

        // 4. 构建分析结果
        let now = chrono::Utc::now().timestamp();
        let analysis = ScreenshotAnalysis {
            screenshot_id: screenshot_id.to_string(),
            application: ai_result.application,
            activity_type: ai_result.activity_type,
            activity_description: ai_result.activity_description,
            key_elements: ai_result.key_elements,
            ocr_text: ai_result.ocr_text,
            context_tags: ai_result.context_tags,
            productivity_score: ai_result.productivity_score.clamp(1, 10),
            analysis_json: response.clone(),
            analyzed_at: now,
        };

        // 5. 保存到数据库
        self.save_analysis(&analysis)?;

        // 6. 标记截图为已分析
        self.mark_screenshot_analyzed(screenshot_id)?;

        Ok(analysis)
    }

    /// 批量分析未处理的截图
    pub async fn analyze_pending(&self) -> Result<AnalysisBatchResult> {
        let pending = self.get_pending_screenshots()?;

        if pending.is_empty() {
            return Ok(AnalysisBatchResult::default());
        }

        info!("开始批量分析 {} 张截图", pending.len());

        let mut result = AnalysisBatchResult::default();
        result.total = pending.len();

        for screenshot in pending {
            let path = Path::new(&screenshot.path);

            if !path.exists() {
                warn!("截图文件不存在: {}", screenshot.path);
                result.skipped += 1;
                continue;
            }

            let mut retries = 0;
            loop {
                match self.analyze_screenshot(&screenshot.id, path).await {
                    Ok(_) => {
                        result.analyzed += 1;
                        break;
                    }
                    Err(e) => {
                        retries += 1;
                        if retries > self.config.max_retries {
                            error!("截图分析失败(已重试{}次): {} - {}", retries, screenshot.id, e);
                            result.failed += 1;
                            break;
                        }
                        warn!("截图分析失败(重试{}/{}): {} - {}", retries, self.config.max_retries, screenshot.id, e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }

        info!(
            "批量分析完成 - 总计: {}, 成功: {}, 跳过: {}, 失败: {}",
            result.total, result.analyzed, result.skipped, result.failed
        );

        Ok(result)
    }

    /// 获取待分析的截图
    fn get_pending_screenshots(&self) -> Result<Vec<PendingScreenshot>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT s.id, s.path
                 FROM screenshots s
                 LEFT JOIN screenshot_analyses sa ON s.id = sa.screenshot_id
                 WHERE sa.screenshot_id IS NULL
                   AND s.analyzed = 0
                 ORDER BY s.captured_at ASC
                 LIMIT ?1"
            )?;

            let screenshots = stmt
                .query_map([self.config.batch_size], |row| {
                    Ok(PendingScreenshot {
                        id: row.get(0)?,
                        path: row.get(1)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(screenshots)
        })
    }

    /// 保存分析结果到数据库
    fn save_analysis(&self, analysis: &ScreenshotAnalysis) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO screenshot_analyses (
                    screenshot_id, application, activity_type, activity_description,
                    key_elements, ocr_text, context_tags, productivity_score,
                    analysis_json, analyzed_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
                ],
            )?;
            Ok(())
        })
    }

    /// 标记截图为已分析
    fn mark_screenshot_analyzed(&self, screenshot_id: &str) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "UPDATE screenshots SET analyzed = 1, analyzed_at = strftime('%s', 'now') WHERE id = ?1",
                [screenshot_id],
            )?;
            Ok(())
        })
    }

    /// 同时更新旧的 analysis_result 字段（兼容V2活动分组器）
    pub fn sync_to_v2_format(&self, analysis: &ScreenshotAnalysis) -> Result<()> {
        let v2_json = serde_json::json!({
            "activity": analysis.activity_description,
            "application": analysis.application,
            "description": analysis.activity_description,
            "category": analysis.activity_type,
        });

        self.db.with_connection(|conn| {
            conn.execute(
                "UPDATE screenshots SET analysis_result = ?1 WHERE id = ?2",
                rusqlite::params![v2_json.to_string(), &analysis.screenshot_id],
            )?;
            Ok(())
        })
    }
}

/// 待分析截图
#[derive(Debug)]
struct PendingScreenshot {
    id: String,
    path: String,
}

/// 批量分析结果
#[derive(Debug, Default)]
pub struct AnalysisBatchResult {
    pub total: usize,
    pub analyzed: usize,
    pub skipped: usize,
    pub failed: usize,
}

/// 截图理解Prompt
fn screenshot_understanding_prompt() -> String {
    r#"分析这张屏幕截图，提取以下信息。严格按JSON格式返回，不要包含其他文字：

{
  "application": "应用名称（如VSCode、Chrome、微信等）",
  "activity_type": "work|entertainment|communication|learning|other",
  "activity_description": "用户正在做什么（一句话，要具体）",
  "key_elements": ["关键元素1", "关键元素2"],
  "ocr_text": "屏幕上的重要文本（如果有，简要提取）",
  "context_tags": ["标签1", "标签2"],
  "productivity_score": 5
}

要求：
1. application: 识别主要应用程序名称
2. activity_type: 只能是 work/entertainment/communication/learning/other 之一
3. activity_description: 必须具体（如"在VSCode中编写Rust代码"而非"使用电脑"）
4. key_elements: 提取窗口标题、文件名、网页标题等关键信息
5. ocr_text: 仅提取重要文本，不要全部OCR
6. context_tags: 2-5个描述当前上下文的标签
7. productivity_score: 1=纯娱乐 5=一般 10=深度工作

只返回JSON，不要其他内容。"#.to_string()
}

/// 解析AI返回的JSON
fn parse_ai_response(response: &str) -> Result<AIScreenshotResult> {
    // 尝试直接解析
    if let Ok(result) = serde_json::from_str::<AIScreenshotResult>(response) {
        return Ok(result);
    }

    // 尝试提取JSON块（AI可能返回markdown代码块）
    let json_str = extract_json_from_response(response);
    serde_json::from_str::<AIScreenshotResult>(&json_str)
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
        let response = r#"根据截图分析：
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
    fn test_screenshot_understanding_prompt_not_empty() {
        let prompt = screenshot_understanding_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("application"));
        assert!(prompt.contains("activity_type"));
    }
}

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
/// 一次性提取所有下游组件需要的信息
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
    // V5: 一次性分析扩展
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
            activity_category: ai_result.activity_category,
            activity_summary: ai_result.activity_summary,
            project_name: ai_result.project_name,
            accomplishments: ai_result.accomplishments,
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
        let mut result = AnalysisBatchResult { total: pending.len(), ..Default::default() };

        for screenshot in pending {
            match self.analyze_with_retry(&screenshot.id, &screenshot.path).await {
                Ok(_) => result.analyzed += 1,
                Err(None) => result.skipped += 1,
                Err(Some(_)) => result.failed += 1,
            }
        }

        info!("批量分析完成 - 总计: {}, 成功: {}, 跳过: {}, 失败: {}",
            result.total, result.analyzed, result.skipped, result.failed);
        Ok(result)
    }

    /// 分析单个录制分段
    pub async fn analyze_recording(
        &self,
        recording_id: &str,
        video_path: &Path,
    ) -> Result<ScreenshotAnalysis> {
        let video_data = tokio::fs::read(video_path).await?;
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

    /// 批量分析未处理的录制分段
    pub async fn analyze_pending_recordings(&self) -> Result<AnalysisBatchResult> {
        let pending = self.get_pending_recordings()?;

        if pending.is_empty() {
            return Ok(AnalysisBatchResult::default());
        }

        info!("开始批量分析 {} 个录制分段", pending.len());
        let mut result = AnalysisBatchResult { total: pending.len(), ..Default::default() };

        for rec in pending {
            match self.analyze_recording_with_retry(&rec.id, &rec.path).await {
                Ok(_) => result.analyzed += 1,
                Err(None) => result.skipped += 1,
                Err(Some(_)) => result.failed += 1,
            }
        }

        info!("录制分析完成 - 总计: {}, 成功: {}, 跳过: {}, 失败: {}",
            result.total, result.analyzed, result.skipped, result.failed);
        Ok(result)
    }

    /// 带重试的截图分析（Ok=成功, Err(None)=跳过, Err(Some)=失败）
    async fn analyze_with_retry(&self, id: &str, path: &str) -> std::result::Result<(), Option<anyhow::Error>> {
        let p = Path::new(path);
        if !p.exists() {
            warn!("截图文件不存在: {}", path);
            return Err(None);
        }
        for attempt in 0..=self.config.max_retries {
            match self.analyze_screenshot(id, p).await {
                Ok(_) => return Ok(()),
                Err(e) if attempt == self.config.max_retries => {
                    error!("截图分析失败(已重试{}次): {} - {}", attempt, id, e);
                    return Err(Some(e));
                }
                Err(e) => {
                    warn!("截图分析失败(重试{}/{}): {} - {}", attempt + 1, self.config.max_retries, id, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
        unreachable!()
    }

    /// 带重试的录制分析
    async fn analyze_recording_with_retry(&self, id: &str, path: &str) -> std::result::Result<(), Option<anyhow::Error>> {
        let p = Path::new(path);
        if !p.exists() {
            warn!("录制文件不存在: {}", path);
            return Err(None);
        }
        for attempt in 0..=self.config.max_retries {
            match self.analyze_recording(id, p).await {
                Ok(_) => return Ok(()),
                Err(e) if attempt == self.config.max_retries => {
                    error!("录制分析失败(已重试{}次): {} - {}", attempt, id, e);
                    return Err(Some(e));
                }
                Err(e) => {
                    warn!("录制分析失败(重试{}/{}): {} - {}", attempt + 1, self.config.max_retries, id, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
        unreachable!()
    }

    /// 获取待分析的录制分段
    fn get_pending_recordings(&self) -> Result<Vec<PendingScreenshot>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, path FROM recordings
                 WHERE analyzed = 0 AND end_time IS NOT NULL
                 ORDER BY start_time ASC
                 LIMIT ?1"
            )?;
            let recs = stmt
                .query_map([self.config.batch_size], |row| {
                    Ok(PendingScreenshot {
                        id: row.get(0)?,
                        path: row.get(1)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(recs)
        })
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
  "activity_category": "work|entertainment|communication|other",
  "activity_summary": "活动概述（供时间线展示，一句话）",
  "key_elements": ["关键元素1", "关键元素2"],
  "ocr_text": "屏幕上的重要文本（如果有，简要提取）",
  "context_tags": ["标签1", "标签2"],
  "productivity_score": 5,
  "project_name": "项目名称或null",
  "accomplishments": ["完成了XX"]
}

要求：
1. application: 识别主要应用程序名称
2. activity_type: 只能是 work/entertainment/communication/learning/other 之一
3. activity_description: 必须具体（如"在VSCode中编写Rust代码"而非"使用电脑"）
4. activity_category: 只能是 work/entertainment/communication/other 之一
5. activity_summary: 简明概述当前活动（供时间线展示）
6. key_elements: 提取窗口标题、文件名、网页标题等关键信息
7. ocr_text: 仅提取重要文本，不要全部OCR
8. context_tags: 2-5个描述当前上下文的标签
9. productivity_score: 1=纯娱乐 5=一般 10=深度工作
10. project_name: 如果能识别出用户在做什么项目则填写项目名，否则返回null
11. accomplishments: 从截图中提取的成果要点（0-3条），没有则返回空数组

只返回JSON，不要其他内容。"#.to_string()
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

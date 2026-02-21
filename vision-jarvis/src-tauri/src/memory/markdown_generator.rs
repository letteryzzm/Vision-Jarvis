/// Markdown生成器 - 将ActivitySession转换为可读的Markdown文件
///
/// 核心功能：
/// 1. YAML frontmatter序列化
/// 2. AI总结生成
/// 3. 录制分段时间线渲染
/// 4. 文件写入与目录管理

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::fs;
use tokio::sync::RwLock;

use crate::ai::AIClient;
use crate::db::schema::{ActivitySession, ScreenshotAnalysisSummary, ActivityCategory};

/// Markdown生成器配置
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// 存储根目录
    pub storage_root: PathBuf,
    /// 是否启用AI总结
    pub enable_ai_summary: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            storage_root: PathBuf::from("./memory"),
            enable_ai_summary: false,
        }
    }
}

/// Markdown生成器
pub struct MarkdownGenerator {
    config: GeneratorConfig,
    ai_client: Arc<RwLock<Option<Arc<AIClient>>>>,
}

/// YAML frontmatter结构
#[derive(Debug, Serialize, Deserialize)]
struct ActivityFrontmatter {
    id: String,
    title: String,
    start_time: String,
    end_time: String,
    duration_minutes: i64,
    application: String,
    category: String,
    tags: Vec<String>,
    screenshots: Vec<ScreenshotEntry>,
}

/// 录制条目(用于frontmatter)
#[derive(Debug, Serialize, Deserialize)]
struct ScreenshotEntry {
    id: String,
    timestamp: String,
    path: String,
    analysis: String,
}

impl MarkdownGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            config,
            ai_client: Arc::new(RwLock::new(None)),
        }
    }

    /// 动态注入 AI 客户端
    pub async fn set_ai_client(&self, client: Arc<AIClient>) {
        let mut guard = self.ai_client.write().await;
        *guard = Some(client);
    }

    /// 生成Markdown文件
    pub async fn generate(&self, activity: &ActivitySession) -> Result<PathBuf> {
        // 1. 生成frontmatter
        let frontmatter = self.build_frontmatter(activity);

        // 2. 生成AI总结(如果启用)
        let summary = if self.config.enable_ai_summary {
            let ai_guard = self.ai_client.read().await;
            if let Some(ref client) = *ai_guard {
                let prompt = self.build_summary_prompt(activity);
                match client.send_text(&prompt).await {
                    Ok(s) => s,
                    Err(e) => {
                        log::warn!("AI summary generation failed: {}, using template", e);
                        self.generate_template_summary(activity)
                    }
                }
            } else {
                self.generate_template_summary(activity)
            }
        } else {
            self.generate_template_summary(activity)
        };

        // 3. 生成录制时间线
        let timeline = self.build_recording_timeline(&activity.screenshot_analyses);

        // 4. 组装完整Markdown
        let content = self.assemble_markdown(&frontmatter, &summary, &timeline)?;

        // 5. 写入文件
        let file_path = self.write_file(&activity.markdown_path, &content)?;

        Ok(file_path)
    }

    /// 构建frontmatter
    fn build_frontmatter(&self, activity: &ActivitySession) -> ActivityFrontmatter {
        ActivityFrontmatter {
            id: activity.id.clone(),
            title: activity.title.clone(),
            start_time: format_timestamp_iso8601(activity.start_time),
            end_time: format_timestamp_iso8601(activity.end_time),
            duration_minutes: activity.duration_minutes,
            application: activity.application.clone(),
            category: format_category(&activity.category),
            tags: activity.tags.clone(),
            screenshots: activity.screenshot_analyses.iter().map(|s| {
                ScreenshotEntry {
                    id: s.id.clone(),
                    timestamp: format_timestamp_iso8601(s.timestamp),
                    path: s.path.clone(),
                    analysis: s.analysis.clone(),
                }
            }).collect(),
        }
    }

    /// 构建总结prompt
    fn build_summary_prompt(&self, activity: &ActivitySession) -> String {
        let screenshots_desc: Vec<String> = activity.screenshot_analyses.iter()
            .map(|s| format!("- {}: {}", format_timestamp_time(s.timestamp), s.analysis))
            .collect();

        format!(
            "活动信息：\n\
             标题: {}\n\
             应用: {}\n\
             时长: {}分钟\n\
             \n\
             截图分析：\n\
             {}\n\
             \n\
             请用2-3句话总结这次活动的主要内容和目的。",
            activity.title,
            activity.application,
            activity.duration_minutes,
            screenshots_desc.join("\n")
        )
    }

    /// 生成模板总结(fallback)
    fn generate_template_summary(&self, activity: &ActivitySession) -> String {
        format!(
            "在{}中花费了{}分钟。期间共{}个录制分段，主要活动包括：{}。",
            activity.application,
            activity.duration_minutes,
            activity.screenshot_ids.len(),
            activity.title
        )
    }

    /// 构建录制分段时间线
    fn build_recording_timeline(&self, recordings: &[ScreenshotAnalysisSummary]) -> String {
        if recordings.is_empty() {
            return String::from("无录制记录。");
        }

        let mut timeline = String::from("## 🎬 录制时间线\n\n");

        for recording in recordings {
            timeline.push_str(&format!(
                "### {}\n\n",
                format_timestamp_time(recording.timestamp)
            ));
            timeline.push_str(&format!("**分析**: {}\n\n", recording.analysis));
            timeline.push_str(&format!("**路径**: `{}`\n\n", recording.path));
            timeline.push_str("---\n\n");
        }

        timeline
    }

    /// 组装完整Markdown
    fn assemble_markdown(
        &self,
        frontmatter: &ActivityFrontmatter,
        summary: &str,
        timeline: &str,
    ) -> Result<String> {
        let yaml = serde_yaml::to_string(frontmatter)?;

        Ok(format!(
            "---\n{}\n---\n\n# {}\n\n## 📋 活动总结\n\n{}\n\n{}\n",
            yaml.trim(),
            frontmatter.title,
            summary,
            timeline
        ))
    }

    /// 写入文件
    fn write_file(&self, relative_path: &str, content: &str) -> Result<PathBuf> {
        let full_path = self.config.storage_root.join(relative_path);

        // 创建父目录
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 写入文件
        fs::write(&full_path, content)?;

        Ok(full_path)
    }
}

/// 格式化时间戳为ISO 8601
fn format_timestamp_iso8601(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| Utc::now())
        .to_rfc3339()
}

/// 格式化时间戳为HH:MM:SS
fn format_timestamp_time(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| Utc::now())
        .format("%H:%M:%S")
        .to_string()
}

/// 格式化活动分类
fn format_category(category: &ActivityCategory) -> String {
    match category {
        ActivityCategory::Work => "work".to_string(),
        ActivityCategory::Entertainment => "entertainment".to_string(),
        ActivityCategory::Communication => "communication".to_string(),
        ActivityCategory::Other => "other".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_activity() -> ActivitySession {
        ActivitySession {
            id: "activity-2024-01-15-001".to_string(),
            title: "在VSCode中编写Rust代码".to_string(),
            start_time: 1705300800, // 2024-01-15 10:00:00 UTC
            end_time: 1705304400,   // 2024-01-15 11:00:00 UTC
            duration_minutes: 60,
            application: "VSCode".to_string(),
            category: ActivityCategory::Work,
            screenshot_ids: vec!["s1".to_string(), "s2".to_string()],
            screenshot_analyses: vec![
                ScreenshotAnalysisSummary {
                    id: "s1".to_string(),
                    timestamp: 1705300800,
                    path: "screenshots/2024-01-15/s1.png".to_string(),
                    analysis: "编写Rust函数".to_string(),
                },
                ScreenshotAnalysisSummary {
                    id: "s2".to_string(),
                    timestamp: 1705302600,
                    path: "screenshots/2024-01-15/s2.png".to_string(),
                    analysis: "调试代码".to_string(),
                },
            ],
            tags: vec!["编程".to_string(), "Rust".to_string()],
            markdown_path: "activities/2024-01-15/activity-001.md".to_string(),
            summary: None,
            indexed: false,
            created_at: 1705304400,
        }
    }

    #[test]
    fn test_build_frontmatter() {
        let generator = MarkdownGenerator::new(GeneratorConfig::default());
        let activity = create_test_activity();

        let frontmatter = generator.build_frontmatter(&activity);

        assert_eq!(frontmatter.id, "activity-2024-01-15-001");
        assert_eq!(frontmatter.title, "在VSCode中编写Rust代码");
        assert_eq!(frontmatter.application, "VSCode");
        assert_eq!(frontmatter.category, "work");
        assert_eq!(frontmatter.tags.len(), 2);
        assert_eq!(frontmatter.screenshots.len(), 2);
    }

    #[test]
    fn test_generate_template_summary() {
        let generator = MarkdownGenerator::new(GeneratorConfig::default());
        let activity = create_test_activity();

        let summary = generator.generate_template_summary(&activity);

        assert!(summary.contains("VSCode"));
        assert!(summary.contains("60分钟"));
        assert!(summary.contains("2个录制分段"));
    }

    #[test]
    fn test_build_screenshot_timeline() {
        let generator = MarkdownGenerator::new(GeneratorConfig::default());
        let activity = create_test_activity();

        let timeline = generator.build_recording_timeline(&activity.screenshot_analyses);

        assert!(timeline.contains("## 🎬 录制时间线"));
        assert!(timeline.contains("编写Rust函数"));
        assert!(timeline.contains("调试代码"));
        assert!(timeline.contains("screenshots/2024-01-15/s1.png"));
    }

    #[tokio::test]
    async fn test_generate_markdown_without_ai() {
        let temp_dir = TempDir::new().unwrap();
        let config = GeneratorConfig {
            storage_root: temp_dir.path().to_path_buf(),
            enable_ai_summary: false,
        };

        let generator = MarkdownGenerator::new(config);
        let activity = create_test_activity();

        let result = generator.generate(&activity).await;
        assert!(result.is_ok());

        let file_path = result.unwrap();
        assert!(file_path.exists());

        // 读取文件验证内容
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---")); // frontmatter
        assert!(content.contains("id: activity-2024-01-15-001"));
        assert!(content.contains("# 在VSCode中编写Rust代码"));
        assert!(content.contains("## 📋 活动总结"));
        assert!(content.contains("## 🎬 录制时间线"));
    }

    #[test]
    fn test_write_file_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let config = GeneratorConfig {
            storage_root: temp_dir.path().to_path_buf(),
            enable_ai_summary: false,
        };

        let generator = MarkdownGenerator::new(config);
        let result = generator.write_file(
            "activities/2024-01-15/activity-001.md",
            "test content"
        );

        assert!(result.is_ok());
        let file_path = result.unwrap();
        assert!(file_path.exists());
        assert!(file_path.parent().unwrap().exists());
    }

    #[test]
    fn test_format_timestamp_iso8601() {
        let timestamp = 1705300800; // 2024-01-15 10:00:00 UTC
        let result = format_timestamp_iso8601(timestamp);
        // 验证格式是否符合RFC3339(ISO 8601)
        assert!(result.contains("2024-01-15"));
        assert!(result.contains('T'));
        assert!(result.contains('+') || result.contains('Z'));
    }

    #[test]
    fn test_format_timestamp_time() {
        let timestamp = 1705300800; // 2024-01-15 10:00:00 UTC
        let result = format_timestamp_time(timestamp);
        // 验证格式为HH:MM:SS
        assert_eq!(result.len(), 8);
        assert!(result.contains(':'));
        let parts: Vec<&str> = result.split(':').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_format_category() {
        assert_eq!(format_category(&ActivityCategory::Work), "work");
        assert_eq!(format_category(&ActivityCategory::Entertainment), "entertainment");
        assert_eq!(format_category(&ActivityCategory::Communication), "communication");
        assert_eq!(format_category(&ActivityCategory::Other), "other");
    }

    #[test]
    fn test_assemble_markdown_structure() {
        let generator = MarkdownGenerator::new(GeneratorConfig::default());
        let activity = create_test_activity();
        let frontmatter = generator.build_frontmatter(&activity);
        let summary = "测试总结";
        let timeline = "## 时间线\n测试时间线";

        let result = generator.assemble_markdown(&frontmatter, summary, timeline);
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.starts_with("---\n"));
        assert!(markdown.contains("---\n\n# 在VSCode中编写Rust代码"));
        assert!(markdown.contains("## 📋 活动总结"));
        assert!(markdown.contains("测试总结"));
        assert!(markdown.contains("测试时间线"));
    }
}

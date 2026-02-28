/// 总结生成器 - 生成日/周/月总结
///
/// 聚合活动数据，调用AI生成结构化总结
/// 输出存入 summaries 表和 Markdown 文件

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use log::{info, warn};
use tokio::sync::RwLock;

use crate::ai::AIClient;
use crate::db::Database;
use crate::db::schema::{ActivitySession, Summary, SummaryType};

/// 总结生成器配置
#[derive(Debug, Clone)]
pub struct SummaryConfig {
    /// 存储根目录
    pub storage_root: PathBuf,
    /// 是否启用AI总结
    pub enable_ai: bool,
}

impl Default for SummaryConfig {
    fn default() -> Self {
        Self {
            storage_root: PathBuf::from("./memory"),
            enable_ai: false,
        }
    }
}

/// 总结生成器
pub struct SummaryGenerator {
    ai_client: Arc<RwLock<Option<Arc<AIClient>>>>,
    db: Arc<Database>,
    config: SummaryConfig,
}

impl SummaryGenerator {
    pub fn new(
        ai_client: Option<Arc<AIClient>>,
        db: Arc<Database>,
        config: SummaryConfig,
    ) -> Self {
        Self {
            ai_client: Arc::new(RwLock::new(ai_client)),
            db,
            config,
        }
    }

    /// 动态注入 AI 客户端
    pub async fn set_ai_client(&self, client: Arc<AIClient>) {
        let mut guard = self.ai_client.write().await;
        *guard = Some(client);
    }

    /// 生成日总结
    pub async fn generate_daily(&self, date: &str) -> Result<Summary> {
        let activities = self.get_activities_for_date(date)?;

        if activities.is_empty() {
            return Err(anyhow::anyhow!("日期 {} 没有活动记录", date));
        }

        info!("为 {} 生成日总结，共 {} 个活动", date, activities.len());

        let content = if self.config.enable_ai {
            let ai_guard = self.ai_client.read().await;
            if let Some(ref client) = *ai_guard {
                match self.generate_ai_daily_summary(client, &activities).await {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("AI日总结生成失败: {}，使用模板", e);
                        self.generate_template_daily_summary(&activities, date)
                    }
                }
            } else {
                self.generate_template_daily_summary(&activities, date)
            }
        } else {
            self.generate_template_daily_summary(&activities, date)
        };

        let markdown_path = format!("long_term_memory/daily_summary/{}.md", date);
        let activity_ids: Vec<String> = activities.iter().map(|a| a.id.clone()).collect();

        // 提取关联的项目ID
        let project_ids = self.get_related_project_ids(&activity_ids)?;

        let summary = Summary {
            id: format!("summary-daily-{}", date),
            summary_type: SummaryType::Daily,
            date_start: date.to_string(),
            date_end: date.to_string(),
            content: content.clone(),
            activity_ids,
            project_ids: if project_ids.is_empty() { None } else { Some(project_ids) },
            markdown_path: markdown_path.clone(),
            created_at: Utc::now().timestamp(),
        };

        // 写入Markdown文件
        let full_content = self.format_daily_markdown(&summary, &activities);
        self.write_file(&markdown_path, &full_content)?;

        // 保存到数据库
        self.save_summary(&summary)?;

        Ok(summary)
    }

    /// AI生成日总结
    async fn generate_ai_daily_summary(
        &self,
        client: &AIClient,
        activities: &[ActivitySession],
    ) -> Result<String> {
        let activities_desc = activities.iter()
            .map(|a| format!(
                "- {} ({}-{}): {} ({}分钟, 效率:{})",
                a.application,
                format_time(a.start_time),
                format_time(a.end_time),
                a.title,
                a.duration_minutes,
                a.category.as_str(),
            ))
            .collect::<Vec<_>>()
            .join("\n");

        let total_minutes: i64 = activities.iter().map(|a| a.duration_minutes).sum();

        // V5: 从 screenshot_analyses 聚合 accomplishments
        let all_recording_ids: Vec<String> = activities.iter()
            .flat_map(|a| a.screenshot_ids.iter().cloned())
            .collect();
        let accomplishments = self.get_accomplishments_from_analyses(&all_recording_ids)?;
        let accomplishments_section = if accomplishments.is_empty() {
            String::new()
        } else {
            let items = accomplishments.iter()
                .map(|a| format!("- {}", a))
                .collect::<Vec<_>>()
                .join("\n");
            format!("\n\n## AI已提取的成果要点\n{}", items)
        };

        let prompt = format!(
            r#"基于今天的活动记录生成日总结。

## 今日活动（共{}个，总计{}分钟）
{}{}

请生成简洁的日总结，包含：
1. 时间分配概览（各类活动占比）
2. 主要完成事项（3-5条）
3. 效率评估
4. 明日建议

要求简洁专业，数据驱动。直接输出总结内容，不要包含标题。"#,
            activities.len(), total_minutes, activities_desc, accomplishments_section
        );

        let response = client.send_text(&prompt).await
            .map_err(|e| anyhow::anyhow!("AI调用失败: {}", e))?;

        Ok(response)
    }

    /// 模板日总结
    fn generate_template_daily_summary(&self, activities: &[ActivitySession], date: &str) -> String {
        let total_minutes: i64 = activities.iter().map(|a| a.duration_minutes).sum();

        // 按应用分组统计
        let mut app_time: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for a in activities {
            *app_time.entry(a.application.clone()).or_default() += a.duration_minutes;
        }

        let mut app_stats: Vec<_> = app_time.into_iter().collect();
        app_stats.sort_by(|a, b| b.1.cmp(&a.1));

        let app_summary = app_stats.iter()
            .map(|(app, mins)| format!("- {}: {}分钟", app, mins))
            .collect::<Vec<_>>()
            .join("\n");

        let activity_list = activities.iter()
            .map(|a| format!("- {}-{}: {} ({})", format_time(a.start_time), format_time(a.end_time), a.title, a.application))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "日期: {}\n总活动时间: {}分钟\n活动数: {}\n\n### 应用使用\n{}\n\n### 活动列表\n{}",
            date, total_minutes, activities.len(), app_summary, activity_list
        )
    }

    /// 格式化日总结Markdown
    fn format_daily_markdown(&self, summary: &Summary, activities: &[ActivitySession]) -> String {
        let frontmatter = format!(
            "---\nid: {}\ntype: daily\ndate: {}\nactivity_count: {}\ncreated_at: {}\n---",
            summary.id, summary.date_start, activities.len(), summary.created_at
        );

        format!(
            "{}\n\n# {} 日总结\n\n{}\n",
            frontmatter, summary.date_start, summary.content
        )
    }

    /// 获取指定日期的活动
    fn get_activities_for_date(&self, date: &str) -> Result<Vec<ActivitySession>> {
        let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
        let start_ts = parsed.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let end_ts = parsed.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp();

        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, start_time, end_time, duration_minutes,
                        application, category, screenshot_ids, tags,
                        markdown_path, summary, indexed, created_at
                 FROM activities
                 WHERE start_time >= ?1 AND start_time <= ?2
                 ORDER BY start_time ASC"
            )?;

            let activities = stmt.query_map(
                rusqlite::params![start_ts, end_ts],
                |row| {
                    let category_str: String = row.get(6)?;
                    let screenshot_ids_json: String = row.get(7)?;
                    let tags_json: String = row.get(8)?;

                    Ok(ActivitySession {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        start_time: row.get(2)?,
                        end_time: row.get(3)?,
                        duration_minutes: row.get(4)?,
                        application: row.get(5)?,
                        category: serde_json::from_str(&category_str).unwrap_or(crate::db::schema::ActivityCategory::Other),
                        screenshot_ids: serde_json::from_str(&screenshot_ids_json).unwrap_or_default(),
                        screenshot_analyses: Vec::new(),
                        tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                        markdown_path: row.get(9)?,
                        summary: row.get(10)?,
                        indexed: row.get::<_, i32>(11)? != 0,
                        created_at: row.get(12)?,
                    })
                },
            )?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(activities)
        })
    }

    /// 获取关联的项目ID
    fn get_related_project_ids(&self, _activity_ids: &[String]) -> Result<Vec<String>> {
        // TODO: 实现项目关联查询
        Ok(Vec::new())
    }

    /// 保存总结到数据库
    fn save_summary(&self, summary: &Summary) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO summaries (
                    id, summary_type, date_start, date_end, content,
                    activity_ids, project_ids, markdown_path, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    &summary.id,
                    summary.summary_type.as_str(),
                    &summary.date_start,
                    &summary.date_end,
                    &summary.content,
                    serde_json::to_string(&summary.activity_ids)?,
                    summary.project_ids.as_ref().map(|p| serde_json::to_string(p).ok()).flatten(),
                    &summary.markdown_path,
                    summary.created_at,
                ],
            )?;
            Ok(())
        })
    }

    /// V5: 从 screenshot_analyses 聚合 accomplishments
    fn get_accomplishments_from_analyses(&self, recording_ids: &[String]) -> Result<Vec<String>> {
        if recording_ids.is_empty() {
            return Ok(Vec::new());
        }

        self.db.with_connection(|conn| {
            let placeholders: Vec<String> = recording_ids.iter().enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect();
            let sql = format!(
                "SELECT accomplishments FROM screenshot_analyses \
                 WHERE screenshot_id IN ({}) AND accomplishments IS NOT NULL AND accomplishments != '[]'",
                placeholders.join(", ")
            );

            let mut stmt = conn.prepare(&sql)?;
            let params: Vec<&dyn rusqlite::types::ToSql> = recording_ids.iter()
                .map(|id| id as &dyn rusqlite::types::ToSql)
                .collect();

            let rows = stmt.query_map(params.as_slice(), |row| {
                row.get::<_, String>(0)
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            let mut all: Vec<String> = Vec::new();
            for json_str in &rows {
                if let Ok(items) = serde_json::from_str::<Vec<String>>(json_str) {
                    for item in items {
                        if !all.contains(&item) {
                            all.push(item);
                        }
                    }
                }
            }

            Ok(all)
        })
    }

    /// 写入文件
    fn write_file(&self, relative_path: &str, content: &str) -> Result<PathBuf> {
        let full_path = self.config.storage_root.join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, content)?;
        Ok(full_path)
    }
}

/// 活动分类的字符串表示（用于模板）
trait CategoryStr {
    fn as_str(&self) -> &str;
}

impl CategoryStr for crate::db::schema::ActivityCategory {
    fn as_str(&self) -> &str {
        match self {
            crate::db::schema::ActivityCategory::Work => "work",
            crate::db::schema::ActivityCategory::Entertainment => "entertainment",
            crate::db::schema::ActivityCategory::Communication => "communication",
            crate::db::schema::ActivityCategory::Other => "other",
        }
    }
}

/// 格式化时间戳为 HH:MM
fn format_time(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| Utc::now())
        .format("%H:%M")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::ActivityCategory;

    fn create_test_activity(id: &str, app: &str, start: i64, duration: i64) -> ActivitySession {
        ActivitySession {
            id: id.to_string(),
            title: format!("在{}中工作", app),
            start_time: start,
            end_time: start + duration * 60,
            duration_minutes: duration,
            application: app.to_string(),
            category: ActivityCategory::Work,
            screenshot_ids: vec![],
            screenshot_analyses: vec![],
            tags: vec![],
            markdown_path: String::new(),
            summary: None,
            indexed: false,
            created_at: start,
        }
    }

    #[test]
    fn test_template_daily_summary() {
        let db = Arc::new(Database::open_in_memory().unwrap());
        let gen = SummaryGenerator::new(None, db, SummaryConfig::default());

        let activities = vec![
            create_test_activity("a1", "VSCode", 1000, 60),
            create_test_activity("a2", "Chrome", 4600, 30),
        ];

        let summary = gen.generate_template_daily_summary(&activities, "2024-01-15");
        assert!(summary.contains("VSCode: 60分钟"));
        assert!(summary.contains("Chrome: 30分钟"));
        assert!(summary.contains("总活动时间: 90分钟"));
    }

    #[test]
    fn test_format_time() {
        let ts = 1705300800; // 2024-01-15 10:00:00 UTC
        let result = format_time(ts);
        assert_eq!(result.len(), 5);
        assert!(result.contains(':'));
    }
}

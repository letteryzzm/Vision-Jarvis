/// 项目提取器 - 从活动中自动识别和追踪项目
///
/// 核心逻辑：
/// 1. 优先从 screenshot_analyses 表读取 AI 已提取的 project_name
/// 2. 回退到规则提取（标题关键词、开发应用识别）
/// 3. 与现有项目进行相似度匹配
/// 4. 匹配成功则归入现有项目，否则创建新项目并生成Markdown文件

use anyhow::Result;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use log::{info, warn};
use rusqlite::OptionalExtension;
use uuid::Uuid;

use crate::db::Database;
use crate::db::schema::{ActivitySession, Project, ProjectStatus};

/// 项目提取器配置
#[derive(Debug, Clone)]
pub struct ProjectExtractorConfig {
    /// 存储根目录
    pub storage_root: PathBuf,
    /// 相似度阈值（0.0-1.0）
    pub similarity_threshold: f32,
}

impl Default for ProjectExtractorConfig {
    fn default() -> Self {
        Self {
            storage_root: PathBuf::from("./memory"),
            similarity_threshold: 0.6,
        }
    }
}

/// 项目提取器
pub struct ProjectExtractor {
    db: Arc<Database>,
    config: ProjectExtractorConfig,
}

impl ProjectExtractor {
    pub fn new(
        db: Arc<Database>,
        config: ProjectExtractorConfig,
    ) -> Self {
        Self { db, config }
    }

    /// 从活动中提取并匹配项目
    pub async fn extract_from_activity(&self, activity: &ActivitySession) -> Result<Option<String>> {
        // 1. 优先从 screenshot_analyses.project_name 读取（V5: AI一次性提取）
        let project_name = if let Some(name) = self.get_project_name_from_analyses(&activity.screenshot_ids)? {
            name
        } else if let Some(name) = self.rule_extract_project(activity) {
            name
        } else {
            return Ok(None);
        };

        // 2. 匹配现有项目
        let existing_projects = self.get_all_projects()?;
        let best_match = self.find_best_match(&project_name, &existing_projects);

        if let Some((project_id, similarity)) = best_match {
            if similarity >= self.config.similarity_threshold {
                // 归入现有项目
                self.add_activity_to_project(&project_id, activity)?;
                info!("活动 {} 归入项目 {} (相似度: {:.2})", activity.id, project_id, similarity);
                return Ok(Some(project_id));
            }
        }

        // 3. 创建新项目
        let project_id = self.create_project(&project_name, activity)?;
        info!("从活动 {} 创建新项目: {} ({})", activity.id, project_name, project_id);
        Ok(Some(project_id))
    }

    /// 批量处理未关联项目的活动
    pub async fn process_unlinked_activities(&self) -> Result<ProcessResult> {
        let activities = self.get_unlinked_activities()?;
        let mut result = ProcessResult::default();

        for activity in &activities {
            match self.extract_from_activity(activity).await {
                Ok(Some(_)) => result.linked += 1,
                Ok(None) => result.skipped += 1,
                Err(e) => {
                    warn!("处理活动 {} 失败: {}", activity.id, e);
                    result.failed += 1;
                }
            }
        }

        result.total = activities.len();
        Ok(result)
    }

    /// V5: 从 screenshot_analyses 表读取 AI 已提取的 project_name
    fn get_project_name_from_analyses(&self, recording_ids: &[String]) -> Result<Option<String>> {
        if recording_ids.is_empty() {
            return Ok(None);
        }

        self.db.with_connection(|conn| {
            let placeholders: Vec<String> = recording_ids.iter().enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect();
            let sql = format!(
                "SELECT project_name, COUNT(*) as cnt \
                 FROM screenshot_analyses \
                 WHERE screenshot_id IN ({}) AND project_name IS NOT NULL \
                 GROUP BY project_name \
                 ORDER BY cnt DESC \
                 LIMIT 1",
                placeholders.join(", ")
            );

            let mut stmt = conn.prepare(&sql)?;
            let params: Vec<&dyn rusqlite::types::ToSql> = recording_ids.iter()
                .map(|id| id as &dyn rusqlite::types::ToSql)
                .collect();

            let result = stmt.query_row(params.as_slice(), |row| {
                row.get::<_, String>(0)
            }).optional()?;

            Ok(result)
        })
    }

    /// 基于规则提取项目名称
    fn rule_extract_project(&self, activity: &ActivitySession) -> Option<String> {
        // 开发类活动 -> 以应用+上下文为项目
        let dev_apps = ["VSCode", "IntelliJ", "Xcode", "Android Studio", "Cursor", "Zed"];
        if dev_apps.iter().any(|app| activity.application.contains(app)) {
            // 从标题中提取项目关键词
            if let Some(project) = extract_project_keyword(&activity.title) {
                return Some(project);
            }
            // 从标签中查找
            for tag in &activity.tags {
                if !is_generic_tag(tag) {
                    return Some(tag.clone());
                }
            }
        }

        // 阅读/学习类
        let learning_keywords = ["阅读", "学习", "教程", "课程"];
        for keyword in learning_keywords {
            if activity.title.contains(keyword) {
                return Some(activity.title.clone());
            }
        }

        // 长时间活动（>30分钟）可能是项目
        if activity.duration_minutes >= 30 {
            if let Some(project) = extract_project_keyword(&activity.title) {
                return Some(project);
            }
        }

        None
    }

    /// 查找最佳匹配的现有项目
    fn find_best_match(&self, name: &str, projects: &[Project]) -> Option<(String, f32)> {
        let mut best: Option<(String, f32)> = None;

        for project in projects {
            let similarity = calculate_similarity(name, &project.title);
            if let Some((_, best_score)) = &best {
                if similarity > *best_score {
                    best = Some((project.id.clone(), similarity));
                }
            } else if similarity > 0.3 {
                best = Some((project.id.clone(), similarity));
            }
        }

        best
    }

    /// 获取所有活跃项目
    fn get_all_projects(&self) -> Result<Vec<Project>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, description, start_date, last_activity_date,
                        activity_count, tags, status, markdown_path, created_at
                 FROM projects
                 WHERE status = 'active'
                 ORDER BY last_activity_date DESC"
            )?;

            let projects = stmt.query_map([], |row| {
                let tags_json: String = row.get(6)?;
                let status_str: String = row.get(7)?;

                Ok(Project {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    start_date: row.get(3)?,
                    last_activity_date: row.get(4)?,
                    activity_count: row.get(5)?,
                    tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                    status: match status_str.as_str() {
                        "paused" => ProjectStatus::Paused,
                        "completed" => ProjectStatus::Completed,
                        _ => ProjectStatus::Active,
                    },
                    markdown_path: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(projects)
        })
    }

    /// 获取未关联项目的活动
    fn get_unlinked_activities(&self) -> Result<Vec<ActivitySession>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT a.id, a.title, a.start_time, a.end_time, a.duration_minutes,
                        a.application, a.category, a.screenshot_ids, a.tags,
                        a.markdown_path, a.summary, a.indexed, a.created_at
                 FROM activities a
                 WHERE a.project_id IS NULL
                 ORDER BY a.start_time DESC
                 LIMIT 50"
            )?;

            let activities = stmt.query_map([], |row| {
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
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(activities)
        })
    }

    /// 创建新项目
    fn create_project(&self, name: &str, activity: &ActivitySession) -> Result<String> {
        let now = Utc::now().timestamp();
        let project_id = format!("project-{}", Uuid::new_v4().to_string().split('-').next().unwrap());
        let slug = sanitize_filename(name);
        let markdown_path = format!("projects/{}.md", slug);

        let project = Project {
            id: project_id.clone(),
            title: name.to_string(),
            description: Some(format!("从活动\"{}\"中自动提取", activity.title)),
            start_date: activity.start_time,
            last_activity_date: activity.start_time,
            activity_count: 1,
            tags: activity.tags.clone(),
            status: ProjectStatus::Active,
            markdown_path: markdown_path.clone(),
            created_at: now,
        };

        // 保存到数据库
        self.db.with_connection(|conn| {
            conn.execute(
                "INSERT INTO projects (
                    id, title, description, start_date, last_activity_date,
                    activity_count, tags, status, markdown_path, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    &project.id,
                    &project.title,
                    &project.description,
                    project.start_date,
                    project.last_activity_date,
                    project.activity_count,
                    serde_json::to_string(&project.tags)?,
                    project.status.as_str(),
                    &project.markdown_path,
                    project.created_at,
                ],
            )?;
            conn.execute(
                "UPDATE activities SET project_id = ?1 WHERE id = ?2",
                rusqlite::params![&project.id, &activity.id],
            )?;
            Ok(())
        })?;

        // 生成Markdown
        let content = self.generate_project_markdown(&project, &[activity.clone()]);
        self.write_file(&markdown_path, &content)?;

        Ok(project_id)
    }

    /// 将活动添加到已有项目
    fn add_activity_to_project(&self, project_id: &str, activity: &ActivitySession) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "UPDATE projects
                 SET last_activity_date = MAX(last_activity_date, ?1),
                     activity_count = activity_count + 1,
                     updated_at = strftime('%s', 'now')
                 WHERE id = ?2",
                rusqlite::params![activity.start_time, project_id],
            )?;
            conn.execute(
                "UPDATE activities SET project_id = ?1 WHERE id = ?2",
                rusqlite::params![project_id, &activity.id],
            )?;
            Ok(())
        })
    }

    /// 生成项目Markdown
    fn generate_project_markdown(&self, project: &Project, activities: &[ActivitySession]) -> String {
        let frontmatter = format!(
            "---\nid: {}\ntitle: {}\nstatus: {}\nstart_date: {}\nlast_activity: {}\nactivity_count: {}\n---",
            project.id,
            project.title,
            project.status.as_str(),
            project.start_date,
            project.last_activity_date,
            project.activity_count,
        );

        let activity_list = activities.iter()
            .map(|a| format!("- {} ({}, {}分钟)", a.title, a.application, a.duration_minutes))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "{}\n\n# {}\n\n{}\n\n## 相关活动\n\n{}\n",
            frontmatter,
            project.title,
            project.description.as_deref().unwrap_or(""),
            activity_list
        )
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

/// 处理结果
#[derive(Debug, Default)]
pub struct ProcessResult {
    pub total: usize,
    pub linked: usize,
    pub skipped: usize,
    pub failed: usize,
}

/// 计算两个字符串的相似度（Jaccard + 子串匹配混合算法）
fn calculate_similarity(a: &str, b: &str) -> f32 {
    if a == b { return 1.0; }
    if a.is_empty() || b.is_empty() { return 0.0; }

    // 子串匹配
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
        return 0.85;
    }

    // 字符级Jaccard相似度
    let chars_a: std::collections::HashSet<char> = a.chars().collect();
    let chars_b: std::collections::HashSet<char> = b.chars().collect();

    let intersection = chars_a.intersection(&chars_b).count();
    let union = chars_a.union(&chars_b).count();

    if union == 0 { 0.0 } else { intersection as f32 / union as f32 }
}

/// 从标题中提取项目关键词
fn extract_project_keyword(title: &str) -> Option<String> {
    // 匹配"编写XX项目"、"开发XX"等模式
    let patterns = ["编写", "开发", "构建", "实现", "设计", "重构"];
    for pattern in patterns {
        if let Some(pos) = title.find(pattern) {
            let after = &title[pos..];
            if after.len() > pattern.len() {
                return Some(after.to_string());
            }
        }
    }

    // 匹配"XX项目"
    if title.contains("项目") {
        return Some(title.to_string());
    }

    None
}

/// 判断是否为通用标签（不适合作为项目名）
fn is_generic_tag(tag: &str) -> bool {
    let generic = ["编程", "工作", "学习", "浏览", "聊天", "开发",
                    "work", "coding", "browsing", "communication"];
    generic.iter().any(|g| tag == *g)
}

/// 清理文件名（移除不安全字符）
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c > '\u{007F}' { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_similarity_identical() {
        assert_eq!(calculate_similarity("hello", "hello"), 1.0);
    }

    #[test]
    fn test_calculate_similarity_substring() {
        let score = calculate_similarity("vision-jarvis项目", "vision-jarvis");
        assert!(score >= 0.8);
    }

    #[test]
    fn test_calculate_similarity_different() {
        let score = calculate_similarity("编程", "看电影");
        assert!(score < 0.5);
    }

    #[test]
    fn test_extract_project_keyword() {
        assert_eq!(
            extract_project_keyword("编写vision-jarvis项目"),
            Some("编写vision-jarvis项目".to_string())
        );
        assert!(extract_project_keyword("浏览网页").is_none());
    }

    #[test]
    fn test_is_generic_tag() {
        assert!(is_generic_tag("编程"));
        assert!(is_generic_tag("work"));
        assert!(!is_generic_tag("vision-jarvis"));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("编写vision-jarvis项目"), "编写vision-jarvis项目");
        assert_eq!(sanitize_filename("a/b\\c"), "a_b_c");
    }

    #[test]
    fn test_rule_extract_dev_activity() {
        let extractor = ProjectExtractor::new(
            Arc::new(Database::open_in_memory().unwrap()),
            ProjectExtractorConfig::default(),
        );

        let activity = ActivitySession {
            id: "a1".to_string(),
            title: "编写vision-jarvis项目".to_string(),
            start_time: 1000,
            end_time: 4600,
            duration_minutes: 60,
            application: "VSCode".to_string(),
            category: crate::db::schema::ActivityCategory::Work,
            screenshot_ids: vec![],
            screenshot_analyses: vec![],
            tags: vec!["rust".to_string()],
            markdown_path: String::new(),
            summary: None,
            indexed: false,
            created_at: 1000,
        };

        let result = extractor.rule_extract_project(&activity);
        assert!(result.is_some());
    }

    #[test]
    fn test_rule_extract_generic_activity() {
        let extractor = ProjectExtractor::new(
            Arc::new(Database::open_in_memory().unwrap()),
            ProjectExtractorConfig::default(),
        );

        let activity = ActivitySession {
            id: "a2".to_string(),
            title: "浏览网页".to_string(),
            start_time: 1000,
            end_time: 1600,
            duration_minutes: 10,
            application: "Chrome".to_string(),
            category: crate::db::schema::ActivityCategory::Entertainment,
            screenshot_ids: vec![],
            screenshot_analyses: vec![],
            tags: vec![],
            markdown_path: String::new(),
            summary: None,
            indexed: false,
            created_at: 1000,
        };

        let result = extractor.rule_extract_project(&activity);
        assert!(result.is_none());
    }
}

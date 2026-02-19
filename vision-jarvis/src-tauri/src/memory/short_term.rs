/// 短期记忆生成器
///
/// 聚合连续相似活动的截图，生成短期记忆

use anyhow::{Result, Context};
use crate::db::schema::{ShortTermMemory, Period};
use crate::db::Database;
use chrono::{DateTime, Utc, Timelike, NaiveDate};
use uuid::Uuid;

/// 短期记忆生成器
pub struct ShortTermMemoryGenerator {
    _db: Database,
}

impl ShortTermMemoryGenerator {
    /// 创建新的生成器
    pub fn new(db: Database) -> Self {
        Self { _db: db }
    }

    /// 生成指定日期的短期记忆
    pub fn generate_for_date(&self, date: NaiveDate) -> Result<Vec<ShortTermMemory>> {
        // 1. 获取该日期的所有已分析截图
        let screenshots = self.get_analyzed_screenshots_for_date(&date)?;

        if screenshots.is_empty() {
            return Ok(Vec::new());
        }

        // 2. 按活动和时间聚合截图
        let activity_groups = self.group_by_activity(screenshots)?;

        // 3. 为每个活动组生成短期记忆
        let mut memories = Vec::new();
        for group in activity_groups {
            let memory = self.create_memory_from_group(group, date)?;
            memories.push(memory);
        }

        Ok(memories)
    }

    /// 获取指定日期的已分析截图
    fn get_analyzed_screenshots_for_date(&self, _date: &NaiveDate) -> Result<Vec<ScreenshotInfo>> {
        // 这里简化实现，实际应该从数据库查询
        // TODO: 实现数据库查询逻辑
        Ok(Vec::new())
    }

    /// 按活动聚合截图
    fn group_by_activity(&self, screenshots: Vec<ScreenshotInfo>) -> Result<Vec<ActivityGroup>> {
        let mut groups = Vec::new();
        let mut current_group: Option<ActivityGroup> = None;

        for screenshot in screenshots {
            match &mut current_group {
                None => {
                    // 开始新组
                    current_group = Some(ActivityGroup {
                        activity: screenshot.activity.clone(),
                        screenshots: vec![screenshot],
                    });
                }
                Some(group) => {
                    // 判断是否应该合并到当前组
                    if self.should_merge(&group, &screenshot) {
                        group.screenshots.push(screenshot);
                    } else {
                        // 保存当前组，开始新组
                        groups.push(current_group.take().unwrap());
                        current_group = Some(ActivityGroup {
                            activity: screenshot.activity.clone(),
                            screenshots: vec![screenshot],
                        });
                    }
                }
            }
        }

        // 保存最后一个组
        if let Some(group) = current_group {
            groups.push(group);
        }

        Ok(groups)
    }

    /// 判断截图是否应该合并到当前活动组
    fn should_merge(&self, group: &ActivityGroup, screenshot: &ScreenshotInfo) -> bool {
        // 相同活动
        if group.activity != screenshot.activity {
            return false;
        }

        // 时间间隔小于5分钟
        if let Some(last_screenshot) = group.screenshots.last() {
            let time_diff = screenshot.timestamp - last_screenshot.timestamp;
            if time_diff > 300 {
                // 5分钟 = 300秒
                return false;
            }
        }

        true
    }

    /// 从活动组创建短期记忆
    fn create_memory_from_group(
        &self,
        group: ActivityGroup,
        date: NaiveDate,
    ) -> Result<ShortTermMemory> {
        let first = group.screenshots.first().context("空的活动组")?;
        let last = group.screenshots.last().context("空的活动组")?;

        // 计算时间范围
        let start_time = DateTime::from_timestamp(first.timestamp, 0)
            .context("无效的时间戳")?;
        let end_time = DateTime::from_timestamp(last.timestamp, 0)
            .context("无效的时间戳")?;

        let time_start = format!("{:02}:{:02}", start_time.hour(), start_time.minute());
        let time_end = format!("{:02}:{:02}", end_time.hour(), end_time.minute());

        // 判断时段
        let period = self.determine_period(start_time.hour());

        // 收集截图 ID
        let screenshot_ids: Vec<String> = group
            .screenshots
            .iter()
            .map(|s| s.id.clone())
            .collect();

        Ok(ShortTermMemory {
            id: Uuid::new_v4().to_string(),
            date: date.format("%Y-%m-%d").to_string(),
            time_start,
            time_end,
            period,
            activity: group.activity,
            summary: None, // AI 总结将在后续步骤生成
            screenshot_ids,
            created_at: Utc::now().timestamp(),
        })
    }

    /// 判断时段
    fn determine_period(&self, hour: u32) -> Period {
        match hour {
            0..=11 => Period::Morning,
            12..=17 => Period::Afternoon,
            _ => Period::Evening,
        }
    }
}

/// 截图信息（简化）
#[derive(Debug, Clone)]
struct ScreenshotInfo {
    id: String,
    timestamp: i64,
    activity: String,
}

/// 活动组
#[derive(Debug)]
struct ActivityGroup {
    activity: String,
    screenshots: Vec<ScreenshotInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_determine_period() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.initialize().unwrap();
        let generator = ShortTermMemoryGenerator::new(db);

        assert!(matches!(generator.determine_period(8), Period::Morning));
        assert!(matches!(generator.determine_period(14), Period::Afternoon));
        assert!(matches!(generator.determine_period(20), Period::Evening));
    }

    #[test]
    fn test_should_merge_same_activity() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.initialize().unwrap();
        let generator = ShortTermMemoryGenerator::new(db);

        let group = ActivityGroup {
            activity: "编程".to_string(),
            screenshots: vec![ScreenshotInfo {
                id: "1".to_string(),
                timestamp: 1000,
                activity: "编程".to_string(),
            }],
        };

        let screenshot = ScreenshotInfo {
            id: "2".to_string(),
            timestamp: 1200, // 200秒后
            activity: "编程".to_string(),
        };

        assert!(generator.should_merge(&group, &screenshot));
    }

    #[test]
    fn test_should_not_merge_different_activity() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.initialize().unwrap();
        let generator = ShortTermMemoryGenerator::new(db);

        let group = ActivityGroup {
            activity: "编程".to_string(),
            screenshots: vec![ScreenshotInfo {
                id: "1".to_string(),
                timestamp: 1000,
                activity: "编程".to_string(),
            }],
        };

        let screenshot = ScreenshotInfo {
            id: "2".to_string(),
            timestamp: 1100,
            activity: "浏览网页".to_string(), // 不同活动
        };

        assert!(!generator.should_merge(&group, &screenshot));
    }

    #[test]
    fn test_should_not_merge_long_gap() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.initialize().unwrap();
        let generator = ShortTermMemoryGenerator::new(db);

        let group = ActivityGroup {
            activity: "编程".to_string(),
            screenshots: vec![ScreenshotInfo {
                id: "1".to_string(),
                timestamp: 1000,
                activity: "编程".to_string(),
            }],
        };

        let screenshot = ScreenshotInfo {
            id: "2".to_string(),
            timestamp: 1400, // 400秒后，超过5分钟
            activity: "编程".to_string(),
        };

        assert!(!generator.should_merge(&group, &screenshot));
    }
}

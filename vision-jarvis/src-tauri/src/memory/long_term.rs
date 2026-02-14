/// 长期记忆生成器
///
/// 对日期范围内的短期记忆进行总结
/// NOTE: AI 总结功能将在记忆系统重新设计时实现

use anyhow::Result;
use crate::db::schema::{LongTermMemory, MainActivity};
use crate::db::Database;
use chrono::NaiveDate;
use uuid::Uuid;

/// 长期记忆生成器
pub struct LongTermMemoryGenerator {
    db: Database,
}

impl LongTermMemoryGenerator {
    /// 创建新的生成器
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 生成日期范围的长期记忆
    pub async fn generate_summary(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<LongTermMemory> {
        // 1. 获取日期范围内的所有短期记忆
        let short_term_memories = self.get_short_term_memories(start_date, end_date)?;

        if short_term_memories.is_empty() {
            anyhow::bail!("指定日期范围内没有短期记忆");
        }

        // 2. 提取主要活动
        let main_activities = self.extract_main_activities(&short_term_memories);

        // 3. 生成摘要（使用本地总结，AI 版本待记忆系统重新设计时实现）
        let summary = generate_default_summary(&short_term_memories);

        // 4. 创建长期记忆
        let memory = LongTermMemory {
            id: Uuid::new_v4().to_string(),
            date_start: start_date.format("%Y-%m-%d").to_string(),
            date_end: end_date.format("%Y-%m-%d").to_string(),
            summary,
            main_activities,
            created_at: chrono::Utc::now().timestamp(),
        };

        Ok(memory)
    }

    /// 获取日期范围内的短期记忆
    fn get_short_term_memories(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ShortTermMemorySummary>> {
        let start_str = start_date.format("%Y-%m-%d").to_string();
        let end_str = end_date.format("%Y-%m-%d").to_string();

        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT date, period, activity, time_start, time_end
                 FROM short_term_memories
                 WHERE date >= ?1 AND date <= ?2
                 ORDER BY date, time_start"
            )?;

            let memories = stmt.query_map([&start_str, &end_str], |row| {
                let time_start: String = row.get(3)?;
                let time_end: String = row.get(4)?;

                // 计算时长（分钟）
                let duration_minutes = calculate_duration(&time_start, &time_end);

                Ok(ShortTermMemorySummary {
                    date: row.get(0)?,
                    activity: row.get(2)?,
                    duration_minutes,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(memories)
        })
    }

    /// 提取主要活动
    fn extract_main_activities(
        &self,
        memories: &[ShortTermMemorySummary],
    ) -> Vec<MainActivity> {
        use std::collections::HashMap;

        // 统计每个活动的总时长
        let mut activity_durations: HashMap<String, i64> = HashMap::new();

        for memory in memories {
            let duration = memory.duration_minutes;
            *activity_durations.entry(memory.activity.clone()).or_insert(0) += duration;
        }

        // 按时长排序，取前5个
        let mut activities: Vec<_> = activity_durations.into_iter().collect();
        activities.sort_by(|a, b| b.1.cmp(&a.1));
        activities.truncate(5);

        // 转换为 MainActivity
        activities
            .into_iter()
            .map(|(activity, duration)| {
                let hours = duration / 60;
                let minutes = duration % 60;
                MainActivity {
                    date: String::new(),
                    activity,
                    duration: if hours > 0 {
                        format!("{}小时{}分钟", hours, minutes)
                    } else {
                        format!("{}分钟", minutes)
                    },
                }
            })
            .collect()
    }
}

/// 短期记忆摘要
#[derive(Debug, Clone)]
struct ShortTermMemorySummary {
    #[allow(dead_code)]
    date: String,
    activity: String,
    duration_minutes: i64,
}

/// 计算时长（分钟）
fn calculate_duration(time_start: &str, time_end: &str) -> i64 {
    let parse_time = |s: &str| -> Option<(u32, u32)> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 2 {
            let h = parts[0].parse().ok()?;
            let m = parts[1].parse().ok()?;
            Some((h, m))
        } else {
            None
        }
    };

    if let (Some((h1, m1)), Some((h2, m2))) = (parse_time(time_start), parse_time(time_end)) {
        let start_minutes = (h1 * 60 + m1) as i64;
        let end_minutes = (h2 * 60 + m2) as i64;
        (end_minutes - start_minutes).max(0)
    } else {
        0
    }
}

/// 生成默认摘要（本地统计版）
fn generate_default_summary(memories: &[ShortTermMemorySummary]) -> String {
    use std::collections::HashMap;

    let mut activity_durations: HashMap<String, i64> = HashMap::new();
    let mut total_duration = 0i64;

    for memory in memories {
        *activity_durations.entry(memory.activity.clone()).or_insert(0) += memory.duration_minutes;
        total_duration += memory.duration_minutes;
    }

    let mut activities: Vec<_> = activity_durations.into_iter().collect();
    activities.sort_by(|a, b| b.1.cmp(&a.1));

    let top_activities: Vec<String> = activities
        .iter()
        .take(3)
        .map(|(activity, duration)| {
            let hours = duration / 60;
            let minutes = duration % 60;
            if hours > 0 {
                format!("{}（{}小时{}分钟）", activity, hours, minutes)
            } else {
                format!("{}（{}分钟）", activity, minutes)
            }
        })
        .collect();

    let total_hours = total_duration / 60;
    let total_minutes = total_duration % 60;

    format!(
        "本周期共记录{}小时{}分钟的活动。主要活动包括：{}。",
        total_hours,
        total_minutes,
        top_activities.join("、")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_main_activities() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.initialize().unwrap();
        let generator = LongTermMemoryGenerator::new(db);

        let memories = vec![
            ShortTermMemorySummary {
                date: "2026-02-05".to_string(),
                activity: "编程".to_string(),
                duration_minutes: 120,
            },
            ShortTermMemorySummary {
                date: "2026-02-05".to_string(),
                activity: "编程".to_string(),
                duration_minutes: 60,
            },
            ShortTermMemorySummary {
                date: "2026-02-05".to_string(),
                activity: "浏览网页".to_string(),
                duration_minutes: 30,
            },
        ];

        let main_activities = generator.extract_main_activities(&memories);

        assert_eq!(main_activities.len(), 2);
        assert_eq!(main_activities[0].activity, "编程");
        assert_eq!(main_activities[0].duration, "3小时0分钟");
    }

    #[test]
    fn test_extract_main_activities_limit() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.initialize().unwrap();
        let generator = LongTermMemoryGenerator::new(db);

        let mut memories = Vec::new();
        for i in 0..10 {
            memories.push(ShortTermMemorySummary {
                date: "2026-02-05".to_string(),
                activity: format!("活动{}", i),
                duration_minutes: 60,
            });
        }

        let main_activities = generator.extract_main_activities(&memories);

        // 应该只返回前5个
        assert_eq!(main_activities.len(), 5);
    }
}

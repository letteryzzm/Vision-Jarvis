/// 习惯检测器 - 从活动历史中识别用户行为模式
///
/// 支持三种模式检测：
/// 1. 时间模式：固定时间的习惯（如"每天8:00打开微信"）
/// 2. 触发模式：特定事件触发的习惯（如"打开VSCode后必看GitHub"）
/// 3. 序列模式：固定顺序的活动序列（如"早晨例行：微信→邮件→日历"）

use anyhow::Result;
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use log::info;
use uuid::Uuid;

use crate::db::Database;
use crate::db::schema::{Habit, HabitPatternType};

/// 习惯检测器配置
#[derive(Debug, Clone)]
pub struct HabitDetectorConfig {
    /// 存储根目录
    pub storage_root: PathBuf,
    /// 分析天数（回溯多少天）
    pub lookback_days: i64,
    /// 最小出现次数（低于此数不算习惯）
    pub min_occurrences: usize,
    /// 最小置信度（低于此值不保存）
    pub min_confidence: f32,
}

impl Default for HabitDetectorConfig {
    fn default() -> Self {
        Self {
            storage_root: PathBuf::from("./memory"),
            lookback_days: 30,
            min_occurrences: 5,
            min_confidence: 0.5,
        }
    }
}

/// 习惯检测器
pub struct HabitDetector {
    db: Arc<Database>,
    config: HabitDetectorConfig,
}

/// 用于检测的活动记录（轻量版）
#[derive(Debug, Clone)]
struct ActivityRecord {
    application: String,
    activity_type: String,
    start_time: i64,
    end_time: i64,
    hour: u32,
    weekday: u32,
}

impl HabitDetector {
    pub fn new(db: Arc<Database>, config: HabitDetectorConfig) -> Self {
        Self { db, config }
    }

    /// 执行完整的习惯检测
    pub fn detect_all(&self) -> Result<DetectionResult> {
        let activities = self.get_recent_activities()?;

        if activities.is_empty() {
            return Ok(DetectionResult::default());
        }

        info!("分析 {} 条活动记录的行为模式", activities.len());

        let mut all_habits = Vec::new();

        // 检测三种模式
        let time_habits = self.detect_time_patterns(&activities)?;
        let trigger_habits = self.detect_trigger_patterns(&activities)?;
        let sequence_habits = self.detect_sequence_patterns(&activities)?;

        all_habits.extend(time_habits);
        all_habits.extend(trigger_habits);
        all_habits.extend(sequence_habits);

        // 保存/更新习惯
        let mut result = DetectionResult::default();
        let detected_ids: Vec<String> = all_habits.iter().map(|h| h.id.clone()).collect();

        for habit in &all_habits {
            match self.save_or_update_habit(habit) {
                Ok(is_new) => {
                    if is_new { result.new_habits += 1; } else { result.updated_habits += 1; }
                }
                Err(e) => {
                    log::warn!("保存习惯失败: {} - {}", habit.pattern_name, e);
                    result.failed += 1;
                }
            }
        }
        result.total_detected = all_habits.len();

        // V3: 衰减未被重新检测到的旧习惯
        let decay_result = self.decay_stale_habits(&detected_ids)?;
        result.decayed = decay_result.decayed;
        result.removed = decay_result.removed;

        Ok(result)
    }

    /// 检测时间模式
    fn detect_time_patterns(&self, activities: &[ActivityRecord]) -> Result<Vec<Habit>> {
        // 按 (application, hour) 分组
        let mut pattern_map: HashMap<(String, u32), Vec<i64>> = HashMap::new();

        for activity in activities {
            let key = (activity.application.clone(), activity.hour);
            pattern_map.entry(key).or_default().push(activity.start_time);
        }

        let mut habits = Vec::new();
        let now = Utc::now().timestamp();

        for ((app, hour), timestamps) in &pattern_map {
            if timestamps.len() < self.config.min_occurrences {
                continue;
            }

            let confidence = self.calculate_time_confidence(timestamps);
            if confidence < self.config.min_confidence {
                continue;
            }

            let slug = sanitize_slug(&format!("time-{}-{:02}", app, hour));
            habits.push(Habit {
                id: format!("habit-{}", slug),
                pattern_name: format!("每天 {:02}:00 使用 {}", hour, app),
                pattern_type: HabitPatternType::TimeBased,
                confidence,
                frequency: "daily".to_string(),
                trigger_conditions: None,
                typical_time: Some(format!("{:02}:00", hour)),
                last_occurrence: timestamps.iter().max().copied(),
                occurrence_count: timestamps.len() as i32,
                markdown_path: format!("habits/{}.md", slug),
                created_at: now,
                updated_at: now,
            });
        }

        Ok(habits)
    }

    /// 检测触发模式
    fn detect_trigger_patterns(&self, activities: &[ActivityRecord]) -> Result<Vec<Habit>> {
        // 构建转移矩阵：A -> B 的频率
        let mut transitions: HashMap<(String, String), usize> = HashMap::new();
        let mut from_counts: HashMap<String, usize> = HashMap::new();

        for window in activities.windows(2) {
            let from = &window[0];
            let to = &window[1];
            let time_gap = to.start_time - from.end_time;

            // 只统计5分钟内的转换
            if time_gap < 300 && time_gap >= 0 && from.application != to.application {
                *transitions.entry((from.application.clone(), to.application.clone())).or_default() += 1;
                *from_counts.entry(from.application.clone()).or_default() += 1;
            }
        }

        let mut habits = Vec::new();
        let now = Utc::now().timestamp();

        for ((from, to), count) in &transitions {
            if *count < self.config.min_occurrences {
                continue;
            }

            // 计算条件概率 P(to|from)
            let from_total = from_counts.get(from).copied().unwrap_or(1);
            let confidence = *count as f32 / from_total as f32;

            if confidence < self.config.min_confidence {
                continue;
            }

            let slug = sanitize_slug(&format!("trigger-{}-{}", from, to));
            let trigger_json = serde_json::json!({
                "from_app": from,
                "to_app": to,
                "transition_count": count,
            });

            habits.push(Habit {
                id: format!("habit-{}", slug),
                pattern_name: format!("使用{}后通常会使用{}", from, to),
                pattern_type: HabitPatternType::TriggerBased,
                confidence,
                frequency: "per-occurrence".to_string(),
                trigger_conditions: Some(trigger_json.to_string()),
                typical_time: None,
                last_occurrence: None,
                occurrence_count: *count as i32,
                markdown_path: format!("habits/{}.md", slug),
                created_at: now,
                updated_at: now,
            });
        }

        Ok(habits)
    }

    /// 检测序列模式
    fn detect_sequence_patterns(&self, activities: &[ActivityRecord]) -> Result<Vec<Habit>> {
        // 查找长度为3的应用序列
        let mut sequences: HashMap<(String, String, String), usize> = HashMap::new();

        for window in activities.windows(3) {
            let a = &window[0];
            let b = &window[1];
            let c = &window[2];

            // 三个活动在30分钟内完成
            let total_time = c.end_time - a.start_time;
            if total_time < 1800
                && a.application != b.application
                && b.application != c.application
            {
                let key = (a.application.clone(), b.application.clone(), c.application.clone());
                *sequences.entry(key).or_default() += 1;
            }
        }

        let mut habits = Vec::new();
        let now = Utc::now().timestamp();

        for ((a, b, c), count) in &sequences {
            if *count < self.config.min_occurrences {
                continue;
            }

            let confidence = (*count as f32 / self.config.lookback_days as f32).min(1.0);
            if confidence < self.config.min_confidence {
                continue;
            }

            let slug = sanitize_slug(&format!("seq-{}-{}-{}", a, b, c));
            let trigger_json = serde_json::json!({
                "sequence": [a, b, c],
                "count": count,
            });

            habits.push(Habit {
                id: format!("habit-{}", slug),
                pattern_name: format!("{}→{}→{}", a, b, c),
                pattern_type: HabitPatternType::SequenceBased,
                confidence,
                frequency: "daily".to_string(),
                trigger_conditions: Some(trigger_json.to_string()),
                typical_time: None,
                last_occurrence: None,
                occurrence_count: *count as i32,
                markdown_path: format!("habits/{}.md", slug),
                created_at: now,
                updated_at: now,
            });
        }

        Ok(habits)
    }

    /// 计算时间模式的置信度
    fn calculate_time_confidence(&self, timestamps: &[i64]) -> f32 {
        if timestamps.len() < 2 {
            return 0.0;
        }

        // 计算出现频率（出现天数/总天数）
        let frequency = timestamps.len() as f32 / self.config.lookback_days as f32;

        // 计算时间点的稳定性（标准差越小越稳定）
        let hours: Vec<f32> = timestamps.iter()
            .map(|ts| {
                DateTime::from_timestamp(*ts, 0)
                    .map(|dt| dt.hour() as f32 + dt.minute() as f32 / 60.0)
                    .unwrap_or(0.0)
            })
            .collect();

        let mean = hours.iter().sum::<f32>() / hours.len() as f32;
        let variance = hours.iter()
            .map(|h| (h - mean).powi(2))
            .sum::<f32>() / hours.len() as f32;
        let std_dev = variance.sqrt();

        // 标准差 < 1小时为高稳定性
        let stability = (1.0 - std_dev / 12.0).max(0.0);

        // 综合置信度 = 频率权重0.6 + 稳定性权重0.4
        (frequency * 0.6 + stability * 0.4).min(1.0)
    }

    /// 获取最近的活动记录
    fn get_recent_activities(&self) -> Result<Vec<ActivityRecord>> {
        let cutoff = Utc::now().timestamp() - self.config.lookback_days * 86400;

        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT application, category, start_time, end_time
                 FROM activities
                 WHERE start_time >= ?1
                 ORDER BY start_time ASC"
            )?;

            let records = stmt.query_map([cutoff], |row| {
                let application: String = row.get(0)?;
                let activity_type: String = row.get(1)?;
                let start_time: i64 = row.get(2)?;
                let end_time: i64 = row.get(3)?;

                let dt = DateTime::from_timestamp(start_time, 0)
                    .unwrap_or_else(|| Utc::now());

                Ok(ActivityRecord {
                    application,
                    activity_type,
                    start_time,
                    end_time,
                    hour: dt.hour(),
                    weekday: dt.weekday().num_days_from_monday(),
                })
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(records)
        })
    }

    /// 保存或更新习惯（返回是否为新习惯）
    fn save_or_update_habit(&self, habit: &Habit) -> Result<bool> {
        let existing = self.get_habit_by_id(&habit.id)?;
        let now = Utc::now().timestamp();

        if let Some(existing) = existing {
            // 更新现有习惯
            self.db.with_connection(|conn| {
                conn.execute(
                    "UPDATE habits SET
                        confidence = ?1, occurrence_count = ?2,
                        last_occurrence = ?3, updated_at = ?4
                     WHERE id = ?5",
                    rusqlite::params![
                        habit.confidence,
                        habit.occurrence_count,
                        habit.last_occurrence,
                        now,
                        &habit.id,
                    ],
                )?;
                Ok(false)
            })
        } else {
            // 创建新习惯
            self.db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO habits (
                        id, pattern_name, pattern_type, confidence, frequency,
                        trigger_conditions, typical_time, last_occurrence,
                        occurrence_count, markdown_path, created_at, updated_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    rusqlite::params![
                        &habit.id,
                        &habit.pattern_name,
                        habit.pattern_type.as_str(),
                        habit.confidence,
                        &habit.frequency,
                        &habit.trigger_conditions,
                        &habit.typical_time,
                        habit.last_occurrence,
                        habit.occurrence_count,
                        &habit.markdown_path,
                        now,
                        now,
                    ],
                )?;
                Ok(true)
            })?;

            // 生成Markdown
            let content = self.generate_habit_markdown(habit);
            self.write_file(&habit.markdown_path, &content)?;

            Ok(true)
        }
    }

    /// V3: 衰减未被重新检测到的旧习惯
    fn decay_stale_habits(&self, detected_ids: &[String]) -> Result<DecayResult> {
        let now = Utc::now().timestamp();
        let decay_threshold = self.config.lookback_days * 2 * 86400; // 2倍回溯期
        let remove_threshold = self.config.min_confidence * 0.3; // 低于30%最小置信度则删除
        let decay_factor = 0.7; // 每次衰减30%

        let all_habits = self.get_all_habits()?;
        let mut result = DecayResult::default();

        for habit in &all_habits {
            if detected_ids.contains(&habit.id) {
                continue; // 本轮检测到了，不衰减
            }

            let last_seen = habit.last_occurrence.unwrap_or(habit.updated_at);
            let age = now - last_seen;

            if age < decay_threshold {
                continue; // 还在合理范围内，不衰减
            }

            let new_confidence = habit.confidence * decay_factor;

            if new_confidence < remove_threshold {
                // 置信度过低，删除习惯
                self.delete_habit(&habit.id)?;
                result.removed += 1;
                info!("习惯已移除（置信度过低）: {} ({:.0}%)", habit.pattern_name, new_confidence * 100.0);
            } else {
                // 降低置信度
                self.db.with_connection(|conn| {
                    conn.execute(
                        "UPDATE habits SET confidence = ?1, updated_at = ?2 WHERE id = ?3",
                        rusqlite::params![new_confidence, now, &habit.id],
                    )?;
                    Ok(())
                })?;
                result.decayed += 1;
                info!("习惯置信度衰减: {} {:.0}% → {:.0}%",
                    habit.pattern_name, habit.confidence * 100.0, new_confidence * 100.0);
            }
        }

        Ok(result)
    }

    /// 获取所有习惯
    fn get_all_habits(&self) -> Result<Vec<Habit>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, pattern_name, pattern_type, confidence, frequency,
                        trigger_conditions, typical_time, last_occurrence,
                        occurrence_count, markdown_path, created_at, updated_at
                 FROM habits"
            )?;

            let habits = stmt.query_map([], |row| {
                let type_str: String = row.get(2)?;
                Ok(Habit {
                    id: row.get(0)?,
                    pattern_name: row.get(1)?,
                    pattern_type: match type_str.as_str() {
                        "trigger-based" => HabitPatternType::TriggerBased,
                        "sequence-based" => HabitPatternType::SequenceBased,
                        _ => HabitPatternType::TimeBased,
                    },
                    confidence: row.get(3)?,
                    frequency: row.get(4)?,
                    trigger_conditions: row.get(5)?,
                    typical_time: row.get(6)?,
                    last_occurrence: row.get(7)?,
                    occurrence_count: row.get(8)?,
                    markdown_path: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(habits)
        })
    }

    /// 删除习惯
    fn delete_habit(&self, id: &str) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute("DELETE FROM habits WHERE id = ?1", [id])?;
            Ok(())
        })
    }

    /// 获取已有习惯
    fn get_habit_by_id(&self, id: &str) -> Result<Option<Habit>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, pattern_name, pattern_type, confidence, frequency,
                        trigger_conditions, typical_time, last_occurrence,
                        occurrence_count, markdown_path, created_at, updated_at
                 FROM habits WHERE id = ?1"
            )?;

            let result = stmt.query_row([id], |row| {
                let type_str: String = row.get(2)?;
                Ok(Habit {
                    id: row.get(0)?,
                    pattern_name: row.get(1)?,
                    pattern_type: match type_str.as_str() {
                        "trigger-based" => HabitPatternType::TriggerBased,
                        "sequence-based" => HabitPatternType::SequenceBased,
                        _ => HabitPatternType::TimeBased,
                    },
                    confidence: row.get(3)?,
                    frequency: row.get(4)?,
                    trigger_conditions: row.get(5)?,
                    typical_time: row.get(6)?,
                    last_occurrence: row.get(7)?,
                    occurrence_count: row.get(8)?,
                    markdown_path: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            });

            match result {
                Ok(habit) => Ok(Some(habit)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        })
    }

    /// 生成习惯Markdown
    fn generate_habit_markdown(&self, habit: &Habit) -> String {
        let frontmatter = format!(
            "---\nid: {}\npattern_name: {}\npattern_type: {}\nconfidence: {:.2}\nfrequency: {}\n---",
            habit.id,
            habit.pattern_name,
            habit.pattern_type.as_str(),
            habit.confidence,
            habit.frequency,
        );

        let description = match habit.pattern_type {
            HabitPatternType::TimeBased => {
                format!(
                    "在每天 {} 时段，你通常会执行此操作。\n\n检测到 {} 次出现，置信度 {:.0}%。",
                    habit.typical_time.as_deref().unwrap_or("未知"),
                    habit.occurrence_count,
                    habit.confidence * 100.0,
                )
            }
            HabitPatternType::TriggerBased => {
                format!(
                    "当触发条件满足时，你通常会执行此操作。\n\n检测到 {} 次出现，置信度 {:.0}%。\n\n触发条件: {}",
                    habit.occurrence_count,
                    habit.confidence * 100.0,
                    habit.trigger_conditions.as_deref().unwrap_or("无"),
                )
            }
            HabitPatternType::SequenceBased => {
                format!(
                    "你倾向于按固定顺序使用这些应用。\n\n检测到 {} 次出现，置信度 {:.0}%。\n\n序列: {}",
                    habit.occurrence_count,
                    habit.confidence * 100.0,
                    habit.trigger_conditions.as_deref().unwrap_or("无"),
                )
            }
        };

        format!(
            "{}\n\n# {}\n\n## 模式描述\n\n{}\n",
            frontmatter, habit.pattern_name, description
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

/// 检测结果
#[derive(Debug, Default)]
pub struct DetectionResult {
    pub total_detected: usize,
    pub new_habits: usize,
    pub updated_habits: usize,
    pub failed: usize,
    /// V3: 置信度被衰减的习惯数
    pub decayed: usize,
    /// V3: 因置信度过低被移除的习惯数
    pub removed: usize,
}

/// 衰减结果
#[derive(Debug, Default)]
struct DecayResult {
    decayed: usize,
    removed: usize,
}

/// 清理slug（用于文件名和ID）
fn sanitize_slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c }
            else if c > '\u{007F}' { c }
            else { '-' }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_activity_record(app: &str, hour: u32, start: i64) -> ActivityRecord {
        ActivityRecord {
            application: app.to_string(),
            activity_type: "work".to_string(),
            start_time: start,
            end_time: start + 300,
            hour,
            weekday: 0,
        }
    }

    #[test]
    fn test_detect_time_patterns() {
        let db = Arc::new(Database::open_in_memory().unwrap());
        let config = HabitDetectorConfig {
            min_occurrences: 3,
            min_confidence: 0.1,
            lookback_days: 30,
            ..Default::default()
        };
        let detector = HabitDetector::new(db, config);

        // 创建10次在8点使用微信的活动
        let mut activities = Vec::new();
        for i in 0..10 {
            activities.push(create_activity_record("微信", 8, 1000 + i * 86400));
        }

        let habits = detector.detect_time_patterns(&activities).unwrap();
        assert!(!habits.is_empty());
        assert!(habits[0].pattern_name.contains("微信"));
        assert!(habits[0].pattern_name.contains("08:00"));
    }

    #[test]
    fn test_detect_trigger_patterns() {
        let db = Arc::new(Database::open_in_memory().unwrap());
        let config = HabitDetectorConfig {
            min_occurrences: 3,
            min_confidence: 0.1,
            lookback_days: 30,
            ..Default::default()
        };
        let detector = HabitDetector::new(db, config);

        // 创建VSCode -> Chrome的转换模式
        let mut activities = Vec::new();
        for i in 0..6 {
            let base = 1000 + i * 86400;
            activities.push(ActivityRecord {
                application: "VSCode".to_string(),
                activity_type: "work".to_string(),
                start_time: base,
                end_time: base + 100,
                hour: 10,
                weekday: 0,
            });
            activities.push(ActivityRecord {
                application: "Chrome".to_string(),
                activity_type: "work".to_string(),
                start_time: base + 120,
                end_time: base + 220,
                hour: 10,
                weekday: 0,
            });
        }

        let habits = detector.detect_trigger_patterns(&activities).unwrap();
        assert!(!habits.is_empty());
        assert!(habits[0].pattern_name.contains("VSCode"));
        assert!(habits[0].pattern_name.contains("Chrome"));
    }

    #[test]
    fn test_calculate_time_confidence() {
        let db = Arc::new(Database::open_in_memory().unwrap());
        let detector = HabitDetector::new(db, HabitDetectorConfig::default());

        // 稳定的时间戳（同一小时）
        let stable: Vec<i64> = (0..15).map(|i| {
            // 每天8:00左右
            1705300800 + i * 86400 + (i % 3) * 300  // 小幅波动
        }).collect();

        let confidence = detector.calculate_time_confidence(&stable);
        assert!(confidence > 0.3, "稳定模式的置信度应该较高: {}", confidence);
    }

    #[test]
    fn test_sanitize_slug() {
        assert_eq!(sanitize_slug("time-VSCode-08"), "time-VSCode-08");
        assert_eq!(sanitize_slug("trigger/a->b"), "trigger-a--b");
    }

    #[test]
    fn test_detect_empty_activities() {
        let db = Arc::new(Database::open_in_memory().unwrap());
        let detector = HabitDetector::new(db, HabitDetectorConfig::default());
        let result = detector.detect_all().unwrap();
        assert_eq!(result.total_detected, 0);
    }
}

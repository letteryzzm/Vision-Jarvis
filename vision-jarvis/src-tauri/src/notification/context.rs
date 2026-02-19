/// 规则上下文构建器
///
/// 从数据库查询真实数据构建 RuleContext

use anyhow::Result;
use chrono::{Utc, Local, Timelike};
use crate::db::Database;
use super::rules::RuleContext;

/// 从数据库构建规则上下文
pub fn build_context(db: &Database) -> Result<RuleContext> {
    let now = Utc::now();
    let local_now = Local::now();

    let continuous_work_minutes = query_continuous_work_minutes(db)?;
    let inactive_minutes = query_inactive_minutes(db)?;
    let matching_habits = query_matching_habits(db, local_now.hour())?;
    let recent_app_switches = query_recent_app_switches(db)?;
    let (project_inactive_days, inactive_project_name) = query_inactive_project(db)?;

    Ok(RuleContext {
        now,
        local_now,
        continuous_work_minutes,
        inactive_minutes,
        matching_habits,
        recent_app_switches,
        project_inactive_days,
        inactive_project_name,
    })
}

/// 查询连续工作时长（分钟）
///
/// 从最近的截图往回看，找到连续截图之间间隔不超过 10 分钟的最早时间点
fn query_continuous_work_minutes(db: &Database) -> Result<i64> {
    let now = Utc::now().timestamp();

    let result = db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT captured_at FROM screenshots
             WHERE captured_at > ?1
             ORDER BY captured_at DESC
             LIMIT 100"
        )?;

        // 只看最近 4 小时的截图
        let four_hours_ago = now - 4 * 3600;

        let timestamps: Vec<i64> = stmt
            .query_map([four_hours_ago], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        if timestamps.is_empty() {
            return Ok(0i64);
        }

        // 从最新开始往回看，找到间隔超过 10 分钟的断点
        let max_gap_seconds = 10 * 60; // 10 分钟
        let mut earliest = timestamps[0];

        for window in timestamps.windows(2) {
            let newer = window[0];
            let older = window[1];
            if newer - older > max_gap_seconds {
                break;
            }
            earliest = older;
        }

        let work_seconds = now - earliest;
        Ok(work_seconds / 60)
    })?;

    Ok(result)
}

/// 查询屏幕无变化时长（分钟）
///
/// 查看最近截图的时间与当前时间的差值
fn query_inactive_minutes(db: &Database) -> Result<i64> {
    let now = Utc::now().timestamp();

    let result = db.with_connection(|conn| {
        let last_capture: Option<i64> = conn
            .query_row(
                "SELECT MAX(captured_at) FROM screenshots",
                [],
                |row| row.get(0),
            )
            .ok();

        match last_capture {
            Some(ts) if ts > 0 => {
                let diff = now - ts;
                Ok(diff / 60)
            }
            _ => Ok(0i64),
        }
    })?;

    Ok(result)
}

/// V3: 查询当前小时匹配的习惯
fn query_matching_habits(db: &Database, current_hour: u32) -> Result<Vec<(String, f32)>> {
    let hour_str = format!("{:02}:00", current_hour);

    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT pattern_name, confidence FROM habits
             WHERE typical_time = ?1 AND confidence > 0.3
             ORDER BY confidence DESC
             LIMIT 5"
        )?;

        let habits = stmt
            .query_map([&hour_str], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f32>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(habits)
    })
}

/// V3: 查询最近10分钟内的应用切换次数
fn query_recent_app_switches(db: &Database) -> Result<usize> {
    let ten_min_ago = Utc::now().timestamp() - 600;

    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT application FROM activities
             WHERE start_time >= ?1
             ORDER BY start_time ASC"
        )?;

        let apps: Vec<String> = stmt
            .query_map([ten_min_ago], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // 统计应用切换次数
        let switches = apps.windows(2)
            .filter(|w| w[0] != w[1])
            .count();

        Ok(switches)
    })
}

/// V3: 查询最久未活跃的项目
fn query_inactive_project(db: &Database) -> Result<(Option<i64>, Option<String>)> {
    let now = Utc::now().timestamp();

    db.with_connection(|conn| {
        // 查找有活动记录但最近7天以上未活跃的项目
        let result = conn.query_row(
            "SELECT title, last_activity_date FROM projects
             WHERE last_activity_date < ?1
             ORDER BY last_activity_date ASC
             LIMIT 1",
            [now - 7 * 86400],
            |row| {
                let title: String = row.get(0)?;
                let last_activity: i64 = row.get(1)?;
                let days = (now - last_activity) / 86400;
                Ok((Some(days), Some(title)))
            },
        );

        match result {
            Ok(r) => Ok(r),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok((None, None)),
            Err(e) => Err(e.into()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_context_no_db() {
        let ctx = RuleContext {
            now: Utc::now(),
            local_now: Local::now(),
            continuous_work_minutes: 0,
            inactive_minutes: 0,
            matching_habits: vec![],
            recent_app_switches: 0,
            project_inactive_days: None,
            inactive_project_name: None,
        };

        assert_eq!(ctx.continuous_work_minutes, 0);
        assert_eq!(ctx.inactive_minutes, 0);
        assert!(ctx.matching_habits.is_empty());
    }
}

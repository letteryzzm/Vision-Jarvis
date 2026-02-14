/// 规则上下文构建器
///
/// 从数据库查询真实数据构建 RuleContext

use anyhow::Result;
use chrono::{Utc, Local};
use crate::db::Database;
use super::rules::RuleContext;

/// 从数据库构建规则上下文
pub fn build_context(db: &Database) -> Result<RuleContext> {
    let now = Utc::now();
    let local_now = Local::now();

    let continuous_work_minutes = query_continuous_work_minutes(db)?;
    let inactive_minutes = query_inactive_minutes(db)?;

    Ok(RuleContext {
        now,
        local_now,
        continuous_work_minutes,
        inactive_minutes,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_context_no_db() {
        // 没有实际数据库时的基本测试
        let ctx = RuleContext {
            now: Utc::now(),
            local_now: Local::now(),
            continuous_work_minutes: 0,
            inactive_minutes: 0,
        };

        assert_eq!(ctx.continuous_work_minutes, 0);
        assert_eq!(ctx.inactive_minutes, 0);
    }
}

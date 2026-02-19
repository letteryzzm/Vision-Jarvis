/// 记忆相关 Commands

use tauri::State;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use super::{ApiResponse, AppState};

/// 短期记忆信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortMemoryInfo {
    pub id: String,
    pub date: String,
    pub time_start: String,
    pub time_end: String,
    pub period: String,
    pub activity: String,
    pub summary: Option<String>,
}

/// 搜索记忆
#[tauri::command]
pub async fn search_memories(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<ApiResponse<Vec<ShortMemoryInfo>>, String> {
    let limit = limit.unwrap_or(10);

    // TODO: 实现向量搜索
    // 当前使用简单的关键词匹配

    // 清理用户输入，转义SQL通配符防止注入
    let sanitized_query = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");

    let result = state.db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, date, time_start, time_end, period, activity, summary
             FROM short_term_memories
             WHERE activity LIKE ?1 ESCAPE '\\' OR summary LIKE ?1 ESCAPE '\\'
             ORDER BY date DESC, time_start DESC
             LIMIT ?2"
        )?;

        let pattern = format!("%{}%", sanitized_query);
        let memories = stmt
            .query_map(rusqlite::params![pattern, limit as i64], |row| {
                Ok(ShortMemoryInfo {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    time_start: row.get(2)?,
                    time_end: row.get(3)?,
                    period: row.get(4)?,
                    activity: row.get(5)?,
                    summary: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(memories)
    });

    Ok(result.into())
}

/// 获取指定日期的记忆
#[tauri::command]
pub async fn get_memories_by_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<ApiResponse<Vec<ShortMemoryInfo>>, String> {
    // 验证日期格式
    let parsed_date = match NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Ok(ApiResponse::error(format!("日期格式错误，请使用YYYY-MM-DD格式: {}", e))),
    };

    let date_str = parsed_date.format("%Y-%m-%d").to_string();

    let result = state.db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, date, time_start, time_end, period, activity, summary
             FROM short_term_memories
             WHERE date = ?1
             ORDER BY time_start ASC"
        )?;

        let memories = stmt
            .query_map([&date_str], |row| {
                Ok(ShortMemoryInfo {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    time_start: row.get(2)?,
                    time_end: row.get(3)?,
                    period: row.get(4)?,
                    activity: row.get(5)?,
                    summary: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(memories)
    });

    Ok(result.into())
}

/// 生成指定日期的记忆
#[tauri::command]
pub async fn generate_memory(
    state: State<'_, AppState>,
    date: String,
) -> Result<ApiResponse<Vec<ShortMemoryInfo>>, String> {
    use crate::memory::short_term::ShortTermMemoryGenerator;

    let parsed_date = match NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Ok(ApiResponse::error(format!("日期格式错误: {}", e))),
    };

    let generator = ShortTermMemoryGenerator::new((*state.db).clone());

    let result = generator.generate_for_date(parsed_date);

    match result {
        Ok(memories) => {
            // 保存到数据库
            let mut errors = Vec::new();
            for memory in &memories {
                let save_result = state.db.with_connection(|conn| {
                    conn.execute(
                        "INSERT INTO short_term_memories (
                            id, date, time_start, time_end, period,
                            activity, summary, screenshot_ids, created_at
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        rusqlite::params![
                            &memory.id,
                            &memory.date,
                            &memory.time_start,
                            &memory.time_end,
                            serde_json::to_string(&memory.period).unwrap_or_default(),
                            &memory.activity,
                            &memory.summary,
                            serde_json::to_string(&memory.screenshot_ids).unwrap_or_default(),
                            memory.created_at,
                        ],
                    )?;
                    Ok(())
                });

                if let Err(e) = save_result {
                    log::error!("保存记忆失败: {}", e);
                    errors.push(format!("保存记忆 {} 失败: {}", memory.id, e));
                }
            }

            // 如果有错误，返回错误信息
            if !errors.is_empty() {
                return Ok(ApiResponse::error(errors.join("; ")));
            }

            // 转换为前端格式
            let info_list: Vec<ShortMemoryInfo> = memories
                .into_iter()
                .map(|m| ShortMemoryInfo {
                    id: m.id,
                    date: m.date,
                    time_start: m.time_start,
                    time_end: m.time_end,
                    period: format!("{:?}", m.period),
                    activity: m.activity,
                    summary: m.summary,
                })
                .collect();

            Ok(ApiResponse::success(info_list))
        }
        Err(e) => Ok(ApiResponse::error(format!("生成记忆失败: {}", e))),
    }
}


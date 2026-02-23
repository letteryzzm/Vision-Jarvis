/// 用户回归提醒生成器
///
/// 当用户从鼠标 idle 状态回归时，查询最近的截图分析记录，
/// 构建精简 prompt，调用 AI 生成个性化欢迎提醒文案。
/// Token 优化：仅传 3 个字段（~100 tokens input），避免传递完整 analysis_json。

use std::sync::Arc;
use log::{info, warn};
use serde_json;

use crate::db::Database;
use crate::ai::AIClient;
use crate::ai::provider::AIProviderConfig;

/// 从 screenshot_analyses 查到的精简上下文
struct AnalysisContext {
    activity_summary: String,
    activity_category: String,
    accomplishments: Vec<String>,
}

/// 用户回归提醒生成器
pub struct ReturnAdvisor {
    db: Arc<Database>,
    /// 当前活跃的 AI provider 配置（用于临时创建 AIClient）
    provider_config: Option<AIProviderConfig>,
}

impl ReturnAdvisor {
    pub fn new(db: Arc<Database>, provider_config: Option<AIProviderConfig>) -> Self {
        Self { db, provider_config }
    }

    /// 当用户从 idle 返回时调用，生成欢迎提醒文案
    ///
    /// 返回 None 表示：无 AI 配置、无分析记录、或 AI 调用失败
    pub async fn generate_return_hint(&self, idle_secs: u64) -> Option<String> {
        let provider_config = self.provider_config.as_ref()?;

        let ctx = self.fetch_latest_context()?;
        let idle_minutes = idle_secs / 60;

        let prompt = build_prompt(&ctx, idle_minutes);

        info!(
            "[ReturnAdvisor] Generating return hint (idle={}s, summary={})",
            idle_secs,
            ctx.activity_summary
        );

        let client = match AIClient::new(provider_config.clone()) {
            Ok(c) => c,
            Err(e) => {
                warn!("[ReturnAdvisor] Failed to create AI client: {}", e);
                return None;
            }
        };

        match client.send_text(&prompt).await {
            Ok(response) => {
                let hint = response.trim().to_string();
                info!("[ReturnAdvisor] Generated hint: {}", hint);
                Some(hint)
            }
            Err(e) => {
                warn!("[ReturnAdvisor] AI call failed: {}", e);
                None
            }
        }
    }

    /// 查询最近一条截图分析记录的精简字段
    fn fetch_latest_context(&self) -> Option<AnalysisContext> {
        self.db
            .with_connection(|conn| {
                let result = conn.query_row(
                    "SELECT activity_summary, activity_category, accomplishments
                     FROM screenshot_analyses
                     ORDER BY analyzed_at DESC
                     LIMIT 1",
                    [],
                    |row| {
                        let summary: String = row.get(0)?;
                        let category: String = row.get(1)?;
                        let accomplishments_json: String = row.get(2)?;
                        Ok((summary, category, accomplishments_json))
                    },
                );

                match result {
                    Ok((summary, category, acc_json)) => {
                        let accomplishments: Vec<String> =
                            serde_json::from_str(&acc_json).unwrap_or_default();
                        Ok(Some(AnalysisContext {
                            activity_summary: summary,
                            activity_category: category,
                            accomplishments,
                        }))
                    }
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(anyhow::anyhow!("DB query failed: {}", e)),
                }
            })
            .unwrap_or_else(|e| {
                warn!("[ReturnAdvisor] DB error: {}", e);
                None
            })
    }
}

/// 构建精简 prompt（约 80 tokens input）
fn build_prompt(ctx: &AnalysisContext, idle_minutes: u64) -> String {
    let accomplishments_str = if ctx.accomplishments.is_empty() {
        "暂无".to_string()
    } else {
        ctx.accomplishments.join("、")
    };

    let idle_desc = if idle_minutes == 0 {
        "不到1分钟".to_string()
    } else {
        format!("{}分钟", idle_minutes)
    };

    format!(
        "用户刚才在做：{}（{}）\n最近完成：{}\n他离开了{}后回来了。\n用一句温暖的话迎接他回来，提醒他可以从哪里继续。要简洁（30字以内）。",
        ctx.activity_summary,
        ctx.activity_category,
        accomplishments_str,
        idle_desc,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt_with_accomplishments() {
        let ctx = AnalysisContext {
            activity_summary: "调试 Rust 类型系统问题".to_string(),
            activity_category: "work".to_string(),
            accomplishments: vec!["修复了 xx bug".to_string()],
        };
        let prompt = build_prompt(&ctx, 5);
        assert!(prompt.contains("调试 Rust 类型系统问题"));
        assert!(prompt.contains("5分钟"));
        assert!(prompt.contains("修复了 xx bug"));
    }

    #[test]
    fn test_build_prompt_no_accomplishments() {
        let ctx = AnalysisContext {
            activity_summary: "浏览网页".to_string(),
            activity_category: "entertainment".to_string(),
            accomplishments: vec![],
        };
        let prompt = build_prompt(&ctx, 0);
        assert!(prompt.contains("暂无"));
        assert!(prompt.contains("不到1分钟"));
    }
}

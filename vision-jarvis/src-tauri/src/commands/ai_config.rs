/// AI 配置相关 Commands
///
/// 管理 AI 提供商配置（基于新的 provider 系统）

use super::ApiResponse;
use crate::ai::{AIProviderConfig, AIConfig, AIClient, ModelInfo, get_supported_models};
use crate::db::Database;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;
use log::info;

/// AI 配置摘要（前端展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfigSummary {
    /// 是否有可用的提供商
    pub has_provider: bool,
    /// 活动提供商名称
    pub active_provider_name: Option<String>,
    /// 活动提供商的模型
    pub active_model: Option<String>,
    /// 提供商数量
    pub provider_count: usize,
}

/// AI 配置状态
pub struct AIConfigState {
    config: Arc<Mutex<AIConfig>>,
    db: Option<Arc<Database>>,
}

impl AIConfigState {
    pub fn new(db: Arc<Database>) -> Self {
        let config = load_from_db(&db).unwrap_or_default();
        info!("AI config loaded: {} providers", config.providers.len());
        Self {
            config: Arc::new(Mutex::new(config)),
            db: Some(db),
        }
    }

    pub fn get(&self) -> AIConfig {
        self.config.lock().unwrap().clone()
    }

    pub fn update(&self, new_config: AIConfig) -> Result<(), String> {
        if let Some(ref db) = self.db {
            save_to_db(db, &new_config).map_err(|e| format!("保存配置失败: {}", e))?;
        }
        let mut config = self.config.lock().unwrap();
        *config = new_config;
        Ok(())
    }

    /// 获取活跃提供商的配置（用于创建AIClient）
    pub fn get_active_provider_config(&self) -> Option<crate::ai::AIProviderConfig> {
        let config = self.config.lock().unwrap();
        config.get_active_provider().cloned()
    }
}

impl Default for AIConfigState {
    fn default() -> Self {
        Self {
            config: Arc::new(Mutex::new(AIConfig::new())),
            db: None,
        }
    }
}

/// 从数据库加载 AI 配置
fn load_from_db(db: &Database) -> Result<AIConfig, anyhow::Error> {
    db.with_connection(|conn| {
        let json: Option<String> = conn
            .query_row(
                "SELECT value FROM ai_config WHERE key = 'ai_config'",
                [],
                |row| row.get(0),
            )
            .ok();
        match json {
            Some(j) => Ok(serde_json::from_str(&j)?),
            None => Ok(AIConfig::new()),
        }
    })
}

/// 保存 AI 配置到数据库
fn save_to_db(db: &Database, config: &AIConfig) -> Result<(), anyhow::Error> {
    let json = serde_json::to_string(config)?;
    db.with_connection(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO ai_config (key, value, updated_at) VALUES ('ai_config', ?1, strftime('%s', 'now'))",
            [&json],
        )?;
        Ok(())
    })
}

/// 获取 AI 配置摘要
#[tauri::command]
pub async fn get_ai_config_summary(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<AIConfigSummary>, String> {
    let config = state.get();
    let active = config.get_active_provider();

    let summary = AIConfigSummary {
        has_provider: active.is_some(),
        active_provider_name: active.map(|p| p.name.clone()),
        active_model: config.get_active_provider().map(|p| p.model.clone()),
        provider_count: config.providers.len(),
    };

    Ok(ApiResponse::success(summary))
}

/// 获取完整的 AI 配置
#[tauri::command]
pub async fn get_ai_config(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<AIConfig>, String> {
    let config = state.get();
    Ok(ApiResponse::success(config))
}

/// 更新 AI 提供商的 API 密钥
#[tauri::command]
pub async fn update_ai_api_key(
    state: State<'_, AIConfigState>,
    provider_id: String,
    api_key: String,
) -> Result<ApiResponse<bool>, String> {
    let mut config = state.get();

    if let Some(provider) = config.providers.iter_mut().find(|p| p.id == provider_id) {
        provider.api_key = api_key;
        match state.update(config) {
            Ok(_) => Ok(ApiResponse::success(true)),
            Err(e) => Ok(ApiResponse::error(format!("更新 API 密钥失败: {}", e))),
        }
    } else {
        Ok(ApiResponse::error(format!("未找到提供商: {}", provider_id)))
    }
}

/// 更新或添加 AI 提供商配置
#[tauri::command]
pub async fn update_ai_provider_config(
    state: State<'_, AIConfigState>,
    provider_config: AIProviderConfig,
) -> Result<ApiResponse<bool>, String> {
    let mut config = state.get();

    // 尝试更新，如果不存在则添加
    let result = if config.get_provider(&provider_config.id).is_some() {
        config.update_provider(provider_config)
    } else {
        config.add_provider(provider_config)
    };

    match result {
        Ok(_) => match state.update(config) {
            Ok(_) => Ok(ApiResponse::success(true)),
            Err(e) => Ok(ApiResponse::error(format!("保存配置失败: {}", e))),
        },
        Err(e) => Ok(ApiResponse::error(format!("配置失败: {}", e))),
    }
}

/// 设置活动的 AI 提供商（自动连接到管道）
#[tauri::command]
pub async fn set_active_ai_provider(
    state: State<'_, AIConfigState>,
    app_state: State<'_, super::AppState>,
    provider_id: String,
) -> Result<ApiResponse<bool>, String> {
    let mut config = state.get();

    match config.set_active_provider(&provider_id) {
        Ok(_) => match state.update(config) {
            Ok(_) => {
                // 自动连接到管道
                if let Some(provider) = state.get_active_provider_config() {
                    match AIClient::new(provider) {
                        Ok(client) => {
                            app_state.pipeline.connect_ai(client).await;
                        }
                        Err(e) => {
                            log::warn!("[AIConfig] 创建 AI 客户端失败，管道未连接: {}", e);
                        }
                    }
                }
                Ok(ApiResponse::success(true))
            },
            Err(e) => Ok(ApiResponse::error(format!("保存配置失败: {}", e))),
        },
        Err(e) => Ok(ApiResponse::error(format!("设��提供商失败: {}", e))),
    }
}

/// 测试 AI 提供商连接
#[tauri::command]
pub async fn test_ai_connection(
    state: State<'_, AIConfigState>,
    provider_id: String,
) -> Result<ApiResponse<String>, String> {
    let config = state.get();

    let provider = config.providers.iter().find(|p| p.id == provider_id);

    let Some(provider) = provider else {
        return Ok(ApiResponse::error(format!("未找到提供商: {}", provider_id)));
    };

    // 使用新的 AIClient 测试连接
    let client = match AIClient::new(provider.clone()) {
        Ok(c) => c,
        Err(e) => return Ok(ApiResponse::error(format!("创建客户端失败: {}", e))),
    };

    match client.test_connection().await {
        Ok(msg) => Ok(ApiResponse::success(msg)),
        Err(e) => Ok(ApiResponse::error(format!("连接测试失败: {}", e))),
    }
}

/// 获取可用的模型列表
#[tauri::command]
pub async fn get_available_ai_providers() -> Result<ApiResponse<Vec<ModelInfo>>, String> {
    let models = get_supported_models();
    Ok(ApiResponse::success(models))
}

/// 删除 AI 提供商
#[tauri::command]
pub async fn delete_ai_provider(
    state: State<'_, AIConfigState>,
    provider_id: String,
) -> Result<ApiResponse<bool>, String> {
    let mut config = state.get();

    match config.remove_provider(&provider_id) {
        Ok(_) => match state.update(config) {
            Ok(_) => Ok(ApiResponse::success(true)),
            Err(e) => Ok(ApiResponse::error(format!("保存配置失败: {}", e))),
        },
        Err(e) => Ok(ApiResponse::error(format!("删除提供商失败: {}", e))),
    }
}

/// 重置 AI 配置为默认值
#[tauri::command]
pub async fn reset_ai_config(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<bool>, String> {
    let default_config = AIConfig::new();
    match state.update(default_config) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("重置配置失败: {}", e))),
    }
}

/// 连接AI到记忆管道（启用截图分析）
#[tauri::command]
pub async fn connect_ai_to_pipeline(
    ai_state: State<'_, AIConfigState>,
    app_state: State<'_, super::AppState>,
) -> Result<ApiResponse<String>, String> {
    let provider = match ai_state.get_active_provider_config() {
        Some(p) => p,
        None => return Ok(ApiResponse::error("没有活跃的AI提供商，请先配置AI".to_string())),
    };

    let client = match AIClient::new(provider.clone()) {
        Ok(c) => c,
        Err(e) => return Ok(ApiResponse::error(format!("创建AI客户端失败: {}", e))),
    };

    app_state.pipeline.connect_ai(client).await;

    Ok(ApiResponse::success(format!(
        "AI已连接到管道 - 提供商: {}, 模型: {}",
        provider.name, provider.model
    )))
}

/// 获取管道状态
#[tauri::command]
pub async fn get_pipeline_status(
    app_state: State<'_, super::AppState>,
) -> Result<ApiResponse<PipelineStatus>, String> {
    let ai_connected = app_state.pipeline.is_ai_connected().await;

    Ok(ApiResponse::success(PipelineStatus {
        running: true,
        ai_connected,
    }))
}

/// 管道状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatus {
    pub running: bool,
    pub ai_connected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_state_creation() {
        let state = AIConfigState::default();
        let config = state.get();
        assert!(config.providers.is_empty());
        assert!(config.active_provider_id.is_none());
    }

    #[test]
    fn test_ai_config_state_update() {
        let state = AIConfigState::default();
        let mut new_config = AIConfig::new();
        let provider = AIProviderConfig::new(
            "test",
            "Test",
            "https://api.test.com",
            "test-key",
            "test-model",
        );
        new_config.add_provider(provider).unwrap();

        assert!(state.update(new_config).is_ok());
        assert_eq!(state.get().providers.len(), 1);
    }
}

/// AI 配置相关 Commands
///
/// 管理 AI 提供商配置

use super::ApiResponse;
use crate::ai::providers::{AIConfigCollection, AIConfigSummary, AIProvider, AIProviderConfig};
use std::sync::{Arc, Mutex};
use tauri::State;

/// AI 配置状态
pub struct AIConfigState {
    config: Arc<Mutex<AIConfigCollection>>,
}

impl AIConfigState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(AIConfigCollection::default())),
        }
    }

    pub fn with_config(config: AIConfigCollection) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
        }
    }

    pub fn get(&self) -> AIConfigCollection {
        self.config.lock().unwrap().clone()
    }

    pub fn update(&self, new_config: AIConfigCollection) -> Result<(), String> {
        new_config.validate().map_err(|e| e.to_string())?;
        let mut config = self.config.lock().unwrap();
        *config = new_config;
        Ok(())
    }
}

impl Default for AIConfigState {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取 AI 配置摘要
#[tauri::command]
pub async fn get_ai_config_summary(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<AIConfigSummary>, String> {
    let config = state.get();
    let summary = AIConfigSummary::from(&config);
    Ok(ApiResponse::success(summary))
}

/// 获取完整的 AI 配置
#[tauri::command]
pub async fn get_ai_config(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<AIConfigCollection>, String> {
    let config = state.get();
    Ok(ApiResponse::success(config))
}

/// 更新 AI 提供商的 API 密钥
#[tauri::command]
pub async fn update_ai_api_key(
    state: State<'_, AIConfigState>,
    provider: AIProvider,
    api_key: Option<String>,
) -> Result<ApiResponse<bool>, String> {
    let mut config = state.get();
    config.update_api_key(&provider, api_key);

    match state.update(config) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("更新 API 密钥失败: {}", e))),
    }
}

/// 更新 AI 提供商配置
#[tauri::command]
pub async fn update_ai_provider_config(
    state: State<'_, AIConfigState>,
    provider: AIProvider,
    config_update: AIProviderConfig,
) -> Result<ApiResponse<bool>, String> {
    // 验证provider匹配
    if config_update.provider != provider {
        return Ok(ApiResponse::error(
            format!(
                "Provider不匹配: 期望 {:?}，实际配置为 {:?}",
                provider, config_update.provider
            )
        ));
    }

    let mut config = state.get();

    // 更新指定提供商的配置
    let provider_config = config.get_config_mut(&provider);
    *provider_config = config_update;

    match state.update(config) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("更新配置失败: {}", e))),
    }
}

/// 设置活动的 AI 提供商
#[tauri::command]
pub async fn set_active_ai_provider(
    state: State<'_, AIConfigState>,
    provider: AIProvider,
) -> Result<ApiResponse<bool>, String> {
    let mut config = state.get();

    match config.set_active_provider(provider) {
        Ok(_) => match state.update(config) {
            Ok(_) => Ok(ApiResponse::success(true)),
            Err(e) => Ok(ApiResponse::error(format!("保存配置失败: {}", e))),
        },
        Err(e) => Ok(ApiResponse::error(format!("设置提供商失败: {}", e))),
    }
}

/// 测试 AI 提供商连接
#[tauri::command]
pub async fn test_ai_connection(
    state: State<'_, AIConfigState>,
    provider: AIProvider,
) -> Result<ApiResponse<String>, String> {
    let config = state.get();
    let provider_config = config.get_config(&provider);

    // 验证配置
    if let Err(e) = provider_config.validate() {
        return Ok(ApiResponse::error(format!("配置无效: {}", e)));
    }

    // 根据提供商类型测试连接
    match provider {
        AIProvider::OpenAI => test_openai_connection(provider_config).await,
        AIProvider::Anthropic => test_anthropic_connection(provider_config).await,
        AIProvider::Google => test_google_connection(provider_config).await,
        AIProvider::Local => test_local_connection(provider_config).await,
        AIProvider::Custom => test_custom_connection(provider_config).await,
    }
}

/// 获取可用的提供商列表
#[tauri::command]
pub async fn get_available_ai_providers(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<Vec<AIProvider>>, String> {
    let config = state.get();
    let providers = config.get_available_providers();
    Ok(ApiResponse::success(providers))
}

/// 重置 AI 配置为默认值
#[tauri::command]
pub async fn reset_ai_config(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<bool>, String> {
    let default_config = AIConfigCollection::default();
    match state.update(default_config) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("重置配置失败: {}", e))),
    }
}

// 测试连接的内部函数

async fn test_openai_connection(config: &AIProviderConfig) -> Result<ApiResponse<String>, String> {
    let api_key = config.api_key.as_ref().ok_or("缺少 API 密钥")?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/models", config.base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            Ok(ApiResponse::success("OpenAI 连接成功".to_string()))
        }
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Ok(ApiResponse::error(format!(
                "OpenAI 连接失败: {} - {}",
                status, text
            )))
        }
        Err(e) => Ok(ApiResponse::error(format!("网络错误: {}", e))),
    }
}

async fn test_anthropic_connection(
    config: &AIProviderConfig,
) -> Result<ApiResponse<String>, String> {
    let api_key = config.api_key.as_ref().ok_or("缺少 API 密钥")?;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/messages", config.base_url))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(r#"{"model":"claude-3-haiku-20240307","max_tokens":1,"messages":[{"role":"user","content":"Hi"}]}"#)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 400 => {
            // 400 可能是请求格式问题，但说明 API 密钥有效
            Ok(ApiResponse::success("Anthropic 连接成功".to_string()))
        }
        Ok(resp) if resp.status().as_u16() == 401 => {
            Ok(ApiResponse::error("API 密钥无效".to_string()))
        }
        Ok(resp) => {
            let status = resp.status();
            Ok(ApiResponse::error(format!("Anthropic 连接失败: {}", status)))
        }
        Err(e) => Ok(ApiResponse::error(format!("网络错误: {}", e))),
    }
}

async fn test_google_connection(config: &AIProviderConfig) -> Result<ApiResponse<String>, String> {
    let api_key = config.api_key.as_ref().ok_or("缺少 API 密钥")?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/models?key={}", config.base_url, api_key))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            Ok(ApiResponse::success("Google AI 连接成功".to_string()))
        }
        Ok(resp) => {
            let status = resp.status();
            Ok(ApiResponse::error(format!("Google AI 连接失败: {}", status)))
        }
        Err(e) => Ok(ApiResponse::error(format!("网络错误: {}", e))),
    }
}

async fn test_local_connection(config: &AIProviderConfig) -> Result<ApiResponse<String>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/tags", config.base_url.replace("/api", "")))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            Ok(ApiResponse::success("本地模型连接成功".to_string()))
        }
        Ok(resp) => {
            let status = resp.status();
            Ok(ApiResponse::error(format!("本地模型连接失败: {}", status)))
        }
        Err(e) => Ok(ApiResponse::error(format!(
            "无法连接到本地模型服务: {}",
            e
        ))),
    }
}

async fn test_custom_connection(config: &AIProviderConfig) -> Result<ApiResponse<String>, String> {
    let client = reqwest::Client::new();

    let mut request = client.get(&config.base_url);

    if let Some(ref api_key) = config.api_key {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request.send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            Ok(ApiResponse::success("自定义提供商连接成功".to_string()))
        }
        Ok(resp) => {
            let status = resp.status();
            Ok(ApiResponse::error(format!("连接失败: {}", status)))
        }
        Err(e) => Ok(ApiResponse::error(format!("网络错误: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_state_creation() {
        let state = AIConfigState::new();
        let config = state.get();
        assert_eq!(config.active_provider, AIProvider::OpenAI);
    }

    #[test]
    fn test_ai_config_state_update() {
        let state = AIConfigState::new();
        let mut new_config = AIConfigCollection::default();
        new_config.update_api_key(&AIProvider::OpenAI, Some("test-key".to_string()));

        assert!(state.update(new_config).is_ok());
        assert!(state.get().openai.has_api_key());
    }
}

/// AI 集成模块
///
/// 提供 AI 提供商管理、API 客户端、Prompt 模板和图像分析功能

// 新的模块
pub mod provider;
pub mod client;
pub mod prompt;

// 重新导出常用类型
pub use provider::{AIProviderConfig, AIConfig, ModelInfo, get_supported_models};
pub use client::{AIClient, AIMessage, AIContent};
pub use prompt::{
    PromptTemplate, PromptBuilder,
    screenshot_analysis_prompt,
    activity_summary_prompt,
    work_mode_detection_prompt,
    app_usage_analysis_prompt,
};

pub mod provider;
pub mod client;
pub mod prompt;
pub mod traits;
pub mod providers;
pub mod factory;
pub mod frame_extractor;

pub use provider::{AIProviderConfig, AIConfig, ModelInfo, ProviderType, get_supported_models};
pub use client::AIClient;
pub use traits::AIProvider;
pub use prompt::{
    PromptTemplate, PromptBuilder,
    screenshot_analysis_prompt,
    activity_summary_prompt,
    work_mode_detection_prompt,
    app_usage_analysis_prompt,
};

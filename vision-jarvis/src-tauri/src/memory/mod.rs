/// 记忆管理模块
///
/// 负责屏幕录制分析、活动分组、项目追踪、习惯检测和总结生成
/// V5: 一次性AI分析架构 - 录制分段由AI一次性提取所有信息

pub mod short_term;
pub mod activity_grouper;
pub mod markdown_generator;
pub mod chunker;
pub mod index_manager;
pub mod pipeline;
pub mod screenshot_analyzer;
pub mod summary_generator;
pub mod project_extractor;
pub mod habit_detector;

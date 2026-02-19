/// 记忆管理模块
///
/// 负责短期记忆、长期记忆的生成和管理
/// V2: 事项驱动记忆系统
/// V3: 主动式AI记忆系统

pub mod vector_store;
pub mod short_term;
pub mod long_term;
pub mod scheduler;
pub mod activity_grouper;
pub mod markdown_generator;
pub mod chunker;
pub mod index_manager;
pub mod hybrid_search;
pub mod pipeline;

// V3: 主动式AI记忆系统
pub mod screenshot_analyzer;
pub mod summary_generator;
pub mod project_extractor;
pub mod habit_detector;

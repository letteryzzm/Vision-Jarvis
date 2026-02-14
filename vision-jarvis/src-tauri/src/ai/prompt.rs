/// AI Prompt 模板系统
///
/// 提供预定义的 Prompt 模板用于不同的分析场景

use std::collections::HashMap;

/// Prompt 模板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptTemplate {
    /// 屏幕截图分析
    ScreenshotAnalysis,

    /// 活动总结
    ActivitySummary,

    /// 工作模式识别
    WorkModeDetection,

    /// 应用使用分析
    AppUsageAnalysis,

    /// 自定义模板
    Custom,
}

/// Prompt 构建器
pub struct PromptBuilder {
    template: PromptTemplate,
    variables: HashMap<String, String>,
}

impl PromptBuilder {
    /// 创建新的 Prompt 构建器
    pub fn new(template: PromptTemplate) -> Self {
        Self {
            template,
            variables: HashMap::new(),
        }
    }

    /// 设置变量
    pub fn set_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// 构建最终的 Prompt
    pub fn build(&self) -> String {
        let base_template = self.get_base_template();
        self.replace_variables(base_template)
    }

    /// 获取基础模板
    fn get_base_template(&self) -> &str {
        match self.template {
            PromptTemplate::ScreenshotAnalysis => SCREENSHOT_ANALYSIS_TEMPLATE,
            PromptTemplate::ActivitySummary => ACTIVITY_SUMMARY_TEMPLATE,
            PromptTemplate::WorkModeDetection => WORK_MODE_DETECTION_TEMPLATE,
            PromptTemplate::AppUsageAnalysis => APP_USAGE_ANALYSIS_TEMPLATE,
            PromptTemplate::Custom => self.variables.get("template").map(|s| s.as_str()).unwrap_or(""),
        }
    }

    /// 替换模板中的变量
    fn replace_variables(&self, template: &str) -> String {
        let mut result = template.to_string();

        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }
}

/// 屏幕截图分析模板
const SCREENSHOT_ANALYSIS_TEMPLATE: &str = r#"请分析这张屏幕截图，提供以下信息：

1. **当前活动**: 用户正在做什么？
2. **应用程序**: 识别出的应用程序名称
3. **内容类型**: 工作、学习、娱乐、社交等
4. **关键信息**: 截图中的重要文本或内容摘要
5. **时间估计**: 这个活动可能持续多久

请用简洁的中文回答，格式化为 JSON：
```json
{
  "activity": "活动描述",
  "applications": ["应用1", "应用2"],
  "content_type": "类型",
  "key_info": "关键信息摘要",
  "estimated_duration": "时间估计"
}
```"#;

/// 活动总结模板
const ACTIVITY_SUMMARY_TEMPLATE: &str = r#"基于以下屏幕截图序列，总结用户在 {{time_range}} 期间的活动：

请提供：
1. **主要活动**: 用户主要在做什么
2. **时间分配**: 各类活动的时间占比
3. **效率评估**: 工作效率如何
4. **建议**: 改进建议

请用简洁的中文回答。"#;

/// 工作模式识别模板
const WORK_MODE_DETECTION_TEMPLATE: &str = r#"分析这张截图，判断用户当前的工作模式：

可能的模式：
- **深度工作**: 专注于单一任务，无干扰
- **浅层工作**: 多任务切换，频繁中断
- **学习模式**: 阅读、观看教程、做笔记
- **休息模式**: 娱乐、社交、放松
- **会议模式**: 视频会议、协作工具

请判断当前模式并给出置信度（0-100%）。

格式：
```json
{
  "mode": "模式名称",
  "confidence": 85,
  "reasoning": "判断理由"
}
```"#;

/// 应用使用分析模板
const APP_USAGE_ANALYSIS_TEMPLATE: &str = r#"分析截图中的应用程序使用情况：

请识别：
1. **应用名称**: 当前使用的应用
2. **应用类别**: 开发工具、浏览器、办公软件、娱乐等
3. **使用目的**: 用户使用该应用的目的
4. **生产力评分**: 1-10 分，评估对生产力的贡献

格式：
```json
{
  "app_name": "应用名称",
  "category": "类别",
  "purpose": "使用目的",
  "productivity_score": 8
}
```"#;

/// 快捷方法：创建屏幕截图分析 Prompt
pub fn screenshot_analysis_prompt() -> String {
    PromptBuilder::new(PromptTemplate::ScreenshotAnalysis).build()
}

/// 快捷方法：创建活动总结 Prompt
pub fn activity_summary_prompt(time_range: &str) -> String {
    PromptBuilder::new(PromptTemplate::ActivitySummary)
        .set_variable("time_range", time_range)
        .build()
}

/// 快捷方法：创建工作模式识别 Prompt
pub fn work_mode_detection_prompt() -> String {
    PromptBuilder::new(PromptTemplate::WorkModeDetection).build()
}

/// 快捷方法：创建应用使用分析 Prompt
pub fn app_usage_analysis_prompt() -> String {
    PromptBuilder::new(PromptTemplate::AppUsageAnalysis).build()
}

/// 快捷方法：创建自定义 Prompt
pub fn custom_prompt(template: &str, variables: HashMap<String, String>) -> String {
    let mut builder = PromptBuilder::new(PromptTemplate::Custom)
        .set_variable("template", template);

    for (key, value) in variables {
        builder = builder.set_variable(key, value);
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_analysis_prompt() {
        let prompt = screenshot_analysis_prompt();
        assert!(prompt.contains("屏幕截图"));
        assert!(prompt.contains("当前活动"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_activity_summary_prompt() {
        let prompt = activity_summary_prompt("过去1小时");
        assert!(prompt.contains("过去1小时"));
        assert!(prompt.contains("主要活动"));
    }

    #[test]
    fn test_work_mode_detection_prompt() {
        let prompt = work_mode_detection_prompt();
        assert!(prompt.contains("工作模式"));
        assert!(prompt.contains("深度工作"));
        assert!(prompt.contains("confidence"));
    }

    #[test]
    fn test_app_usage_analysis_prompt() {
        let prompt = app_usage_analysis_prompt();
        assert!(prompt.contains("应用程序"));
        assert!(prompt.contains("生产力评分"));
    }

    #[test]
    fn test_custom_prompt() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "张三".to_string());
        vars.insert("task".to_string(), "编程".to_string());

        let prompt = custom_prompt("你好 {{name}}，你正在 {{task}}", vars);
        assert_eq!(prompt, "你好 张三，你正在 编程");
    }

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new(PromptTemplate::ActivitySummary)
            .set_variable("time_range", "今天")
            .set_variable("user", "测试用户")
            .build();

        assert!(prompt.contains("今天"));
    }

    #[test]
    fn test_variable_replacement() {
        let builder = PromptBuilder::new(PromptTemplate::Custom)
            .set_variable("template", "Hello {{name}}, you are {{age}} years old")
            .set_variable("name", "Alice")
            .set_variable("age", "25");

        let result = builder.build();
        assert_eq!(result, "Hello Alice, you are 25 years old");
    }

    #[test]
    fn test_missing_variable() {
        let builder = PromptBuilder::new(PromptTemplate::Custom)
            .set_variable("template", "Hello {{name}}, you are {{age}} years old")
            .set_variable("name", "Alice");

        let result = builder.build();
        // 未替换的变量保持原样
        assert!(result.contains("{{age}}"));
    }
}

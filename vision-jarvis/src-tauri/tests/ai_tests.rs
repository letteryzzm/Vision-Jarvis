#[cfg(test)]
mod ai_module_tests {
    use vision_jarvis_lib::ai::{
        AIProviderConfig, AIConfig, get_supported_models,
        screenshot_analysis_prompt, activity_summary_prompt,
    };

    #[test]
    fn test_provider_config() {
        let config = AIProviderConfig::new(
            "aihubmix",
            "AIHubMix",
            "https://api.aihubmix.com",
            "test-key",
            "claude-opus-4-6",
        );

        assert_eq!(config.id, "aihubmix");
        assert_eq!(config.model, "claude-opus-4-6");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ai_config() {
        let mut config = AIConfig::new();
        let provider = AIProviderConfig::new(
            "test",
            "Test",
            "https://api.test.com",
            "key",
            "model",
        );

        assert!(config.add_provider(provider).is_ok());
        assert_eq!(config.providers.len(), 1);
    }

    #[test]
    fn test_supported_models() {
        let models = get_supported_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "claude-opus-4-6"));
        assert!(models.iter().any(|m| m.id == "glm-5"));
    }

    #[test]
    fn test_prompt_templates() {
        let prompt = screenshot_analysis_prompt();
        assert!(prompt.contains("屏幕截图"));

        let summary = activity_summary_prompt("1小时");
        assert!(summary.contains("1小时"));
    }
}

//! Unit tests for post-processing pipeline

#[cfg(test)]
mod tests {
    use super::super::super::super::post_processing::{PostProcessingConfig, PostProcessor};
    use super::super::super::super::ollama::ProcessingMode;

    #[test]
    fn test_default_config() {
        let config = PostProcessingConfig::default();
        assert!(!config.enable_custom_words);
        assert!(!config.enable_ollama);
        assert!(config.enable_formatting);
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.ollama_model, "llama3.2");
        assert_eq!(config.ollama_mode, ProcessingMode::Disabled);
    }

    #[test]
    fn test_config_creation() {
        let config = PostProcessingConfig {
            enable_custom_words: true,
            custom_words: vec!["test".to_string()],
            custom_words_threshold: 0.3,
            enable_ollama: false,
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "mistral".to_string(),
            ollama_mode: ProcessingMode::Punctuation,
            enable_formatting: true,
        };

        assert!(config.enable_custom_words);
        assert_eq!(config.custom_words.len(), 1);
        assert_eq!(config.ollama_model, "mistral");
    }

    #[test]
    fn test_processor_creation() {
        let processor = PostProcessor::default();
        // Should create without errors
        assert!(true);
    }

    #[test]
    fn test_processor_with_config() {
        let config = PostProcessingConfig::default();
        let processor = PostProcessor::new(config);
        // Should create without errors
        assert!(true);
    }

    #[tokio::test]
    async fn test_format_output_only() {
        let mut config = PostProcessingConfig::default();
        config.enable_formatting = true;
        config.enable_custom_words = false;
        config.enable_ollama = false;

        let processor = PostProcessor::new(config);
        let result = processor
            .process("  hello   world  ".to_string())
            .await
            .unwrap();

        assert_eq!(result, "hello world");
    }

    #[tokio::test]
    async fn test_formatting_with_newlines() {
        let config = PostProcessingConfig::default();
        let processor = PostProcessor::new(config);

        let result = processor
            .process("hello\n\nworld\n".to_string())
            .await
            .unwrap();

        assert_eq!(result, "hello world");
    }

    #[tokio::test]
    async fn test_formatting_with_tabs() {
        let config = PostProcessingConfig::default();
        let processor = PostProcessor::new(config);

        let result = processor
            .process("hello\t\tworld".to_string())
            .await
            .unwrap();

        assert_eq!(result, "hello world");
    }

    #[tokio::test]
    async fn test_empty_input() {
        let config = PostProcessingConfig::default();
        let processor = PostProcessor::new(config);

        let result = processor.process("".to_string()).await.unwrap();
        assert_eq!(result, "");
    }

    #[tokio::test]
    async fn test_whitespace_only() {
        let config = PostProcessingConfig::default();
        let processor = PostProcessor::new(config);

        let result = processor.process("   \n\t  ".to_string()).await.unwrap();
        assert_eq!(result, "");
    }

    #[tokio::test]
    async fn test_disabled_formatting() {
        let mut config = PostProcessingConfig::default();
        config.enable_formatting = false;

        let processor = PostProcessor::new(config);
        let result = processor
            .process("  hello   world  ".to_string())
            .await
            .unwrap();

        // Should preserve whitespace
        assert_eq!(result, "  hello   world  ");
    }

    #[test]
    fn test_config_serialization() {
        let config = PostProcessingConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PostProcessingConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.enable_ollama, config.enable_ollama);
        assert_eq!(deserialized.ollama_url, config.ollama_url);
    }

    #[tokio::test]
    async fn test_ollama_disabled_by_default() {
        let processor = PostProcessor::default();
        let available = processor.is_ollama_available().await;
        assert!(!available); // Should be false when disabled
    }

    #[tokio::test]
    async fn test_list_models_when_disabled() {
        let processor = PostProcessor::default();
        let result = processor.list_ollama_models().await;
        assert!(result.is_err()); // Should error when Ollama is disabled
    }

    #[test]
    fn test_update_config() {
        let mut processor = PostProcessor::default();
        let mut new_config = PostProcessingConfig::default();
        new_config.enable_formatting = false;
        new_config.ollama_model = "custom-model".to_string();

        processor.update_config(new_config);
        // Should complete without error
        assert!(true);
    }
}

//! Unit tests for Ollama integration

#[cfg(test)]
mod tests {
    use super::super::super::super::ollama::{OllamaClient, OllamaRequest, ProcessingMode};

    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient::new("http://localhost:11434");
        assert!(client.is_ok());
    }

    #[test]
    fn test_ollama_client_default() {
        let client = OllamaClient::default();
        assert!(client.is_ok());
    }

    #[test]
    fn test_ollama_request_builder() {
        let request = OllamaRequest::new("llama3.2")
            .with_system("Test system")
            .with_text("Test text")
            .with_temperature(0.5);

        assert_eq!(request.model, "llama3.2");
        assert_eq!(request.system, Some("Test system".to_string()));
        assert_eq!(request.prompt, "Test text");
        assert_eq!(request.temperature, Some(0.5));
        assert_eq!(request.stream, false);
    }

    #[test]
    fn test_ollama_request_prompt_override() {
        let request = OllamaRequest::new("test")
            .with_text("Initial")
            .with_prompt("Override");

        assert_eq!(request.prompt, "Override");
    }

    #[test]
    fn test_temperature_clamping() {
        let request = OllamaRequest::new("test").with_temperature(5.0);
        assert_eq!(request.temperature, Some(2.0)); // Should clamp to 2.0

        let request = OllamaRequest::new("test").with_temperature(-1.0);
        assert_eq!(request.temperature, Some(0.0)); // Should clamp to 0.0

        let request = OllamaRequest::new("test").with_temperature(1.5);
        assert_eq!(request.temperature, Some(1.5)); // Should stay as is
    }

    #[test]
    fn test_processing_mode_equality() {
        assert_eq!(ProcessingMode::Punctuation, ProcessingMode::Punctuation);
        assert_eq!(ProcessingMode::Disabled, ProcessingMode::Disabled);
        assert_eq!(ProcessingMode::Summarize, ProcessingMode::Summarize);
        assert_eq!(
            ProcessingMode::CommandExtraction,
            ProcessingMode::CommandExtraction
        );

        assert_ne!(ProcessingMode::Punctuation, ProcessingMode::Summarize);
        assert_ne!(ProcessingMode::Disabled, ProcessingMode::Punctuation);
    }

    #[test]
    fn test_custom_processing_mode() {
        let mode1 = ProcessingMode::Custom {
            prompt: "Test".to_string(),
        };
        let mode2 = ProcessingMode::Custom {
            prompt: "Test".to_string(),
        };
        let mode3 = ProcessingMode::Custom {
            prompt: "Different".to_string(),
        };

        assert_eq!(mode1, mode2);
        assert_ne!(mode1, mode3);
    }

    #[test]
    fn test_processing_mode_serialization() {
        let mode = ProcessingMode::Punctuation;
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: ProcessingMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ProcessingMode::Punctuation);
    }

    #[test]
    fn test_streaming_flag() {
        let request = OllamaRequest::new("test").with_streaming(true);
        assert_eq!(request.stream, true);

        let request = OllamaRequest::new("test").with_streaming(false);
        assert_eq!(request.stream, false);
    }

    // Integration tests (require Ollama to be running)
    #[tokio::test]
    #[ignore] // Only run when Ollama is available
    async fn test_ollama_availability() {
        let client = OllamaClient::default().unwrap();
        // This test requires Ollama to be running
        let available = client.is_available().await;
        // We just check that the method doesn't panic
        println!("Ollama available: {}", available);
    }

    #[tokio::test]
    #[ignore] // Only run when Ollama is available
    async fn test_list_models() {
        let client = OllamaClient::default().unwrap();
        if client.is_available().await {
            let models = client.list_models().await;
            assert!(models.is_ok() || models.is_err()); // Just verify it doesn't panic
        }
    }

    #[tokio::test]
    #[ignore] // Only run when Ollama is available
    async fn test_process_text() {
        let client = OllamaClient::default().unwrap();
        if client.is_available().await {
            let request = OllamaRequest::new("llama3.2")
                .with_system("Add punctuation")
                .with_text("hello world")
                .with_temperature(0.1);

            let result = client.process(request).await;
            // Result could be Ok or Err depending on whether model exists
            println!("Process result: {:?}", result);
        }
    }
}

//! Post-processing pipeline for transcriptions
//!
//! Provides a flexible, extensible pipeline for processing transcription text
//! through multiple stages.
//!
//! # Architecture
//!
//! ```text
//! Raw Transcription
//!   ↓
//! Custom Words → Ollama Enhancement → Output Formatting → Final Text
//! ```
//!
//! Each stage is optional and configurable.

use anyhow::Result;
use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::ollama::{OllamaClient, ProcessingMode};

/// Configuration for post-processing pipeline
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostProcessingConfig {
    /// Enable custom word corrections
    pub enable_custom_words: bool,
    /// Custom words to apply
    pub custom_words: Vec<String>,
    /// Custom word matching threshold (0.0 = exact, 1.0 = any)
    pub custom_words_threshold: f64,

    /// Enable Ollama processing
    pub enable_ollama: bool,
    /// Ollama server URL
    pub ollama_url: String,
    /// Ollama model to use
    pub ollama_model: String,
    /// Ollama processing mode
    pub ollama_mode: ProcessingMode,

    /// Enable output formatting (trim, normalize whitespace)
    pub enable_formatting: bool,
}

impl Default for PostProcessingConfig {
    fn default() -> Self {
        Self {
            enable_custom_words: false,
            custom_words: Vec::new(),
            custom_words_threshold: 0.5,
            enable_ollama: false,
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3.2".to_string(),
            ollama_mode: ProcessingMode::Disabled,
            enable_formatting: true,
        }
    }
}

/// Post-processing pipeline
///
/// Processes transcription text through multiple configurable stages.
pub struct PostProcessor {
    config: PostProcessingConfig,
    ollama_client: Option<OllamaClient>,
}

impl PostProcessor {
    /// Creates a new post-processor with configuration
    pub fn new(config: PostProcessingConfig) -> Self {
        let ollama_client = if config.enable_ollama {
            match OllamaClient::new(&config.ollama_url) {
                Ok(client) => {
                    info!("Ollama client initialized: {}", config.ollama_url);
                    Some(client)
                }
                Err(e) => {
                    log::warn!("Failed to create Ollama client: {}. Ollama processing will be disabled.", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            config,
            ollama_client,
        }
    }

    /// Creates a post-processor with default settings
    pub fn default() -> Self {
        Self::new(PostProcessingConfig::default())
    }

    /// Updates configuration
    pub fn update_config(&mut self, config: PostProcessingConfig) {
        let needs_ollama_reinit = self.config.ollama_url != config.ollama_url;

        self.config = config;

        if needs_ollama_reinit && self.config.enable_ollama {
            match OllamaClient::new(&self.config.ollama_url) {
                Ok(client) => {
                    info!("Ollama client reinitialized: {}", self.config.ollama_url);
                    self.ollama_client = Some(client);
                }
                Err(e) => {
                    log::error!("Failed to reinitialize Ollama client: {}", e);
                    self.ollama_client = None;
                }
            }
        }
    }

    /// Processes text through the complete pipeline
    ///
    /// # Arguments
    ///
    /// * `text` - Raw transcription text
    ///
    /// # Returns
    ///
    /// Processed text after all enabled stages
    pub async fn process(&self, text: String) -> Result<String> {
        let mut result = text;

        debug!("Starting post-processing pipeline");
        debug!("Input length: {} chars", result.len());

        // Stage 1: Custom word corrections
        if self.config.enable_custom_words && !self.config.custom_words.is_empty() {
            result = self.apply_custom_words(&result);
            debug!("After custom words: {} chars", result.len());
        }

        // Stage 2: Ollama processing
        if self.config.enable_ollama && self.config.ollama_mode != ProcessingMode::Disabled {
            if let Some(ref client) = self.ollama_client {
                match self.apply_ollama(client, &result).await {
                    Ok(processed) => {
                        result = processed;
                        debug!("After Ollama: {} chars", result.len());
                    }
                    Err(e) => {
                        log::warn!("Ollama processing failed: {}. Using original text.", e);
                    }
                }
            } else {
                log::warn!("Ollama is enabled but client is not available");
            }
        }

        // Stage 3: Output formatting
        if self.config.enable_formatting {
            result = self.format_output(&result);
            debug!("After formatting: {} chars", result.len());
        }

        info!("Post-processing complete: {} → {} chars", text.len(), result.len());
        Ok(result)
    }

    /// Stage 1: Apply custom word corrections
    fn apply_custom_words(&self, text: &str) -> String {
        use crate::audio_toolkit::apply_custom_words;

        let corrected = apply_custom_words(
            text,
            &self.config.custom_words,
            self.config.custom_words_threshold,
        );

        debug!("Custom words applied: {} corrections",
            if corrected != text { "some" } else { "no" });

        corrected
    }

    /// Stage 2: Apply Ollama processing
    async fn apply_ollama(&self, client: &OllamaClient, text: &str) -> Result<String> {
        // Check if Ollama is available before processing
        if !client.is_available().await {
            return Err(anyhow::anyhow!("Ollama is not available"));
        }

        let result = client
            .process_with_mode(text, self.config.ollama_mode.clone(), &self.config.ollama_model)
            .await?;

        Ok(result)
    }

    /// Stage 3: Format output
    fn format_output(&self, text: &str) -> String {
        // Normalize whitespace
        let result = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        // Trim
        result.trim().to_string()
    }

    /// Checks if Ollama is available
    pub async fn is_ollama_available(&self) -> bool {
        if let Some(ref client) = self.ollama_client {
            client.is_available().await
        } else {
            false
        }
    }

    /// Lists available Ollama models
    pub async fn list_ollama_models(&self) -> Result<Vec<String>> {
        if let Some(ref client) = self.ollama_client {
            client.list_models().await
        } else {
            Err(anyhow::anyhow!("Ollama client not initialized"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PostProcessingConfig::default();
        assert!(!config.enable_custom_words);
        assert!(!config.enable_ollama);
        assert!(config.enable_formatting);
    }

    #[test]
    fn test_format_output() {
        let processor = PostProcessor::default();
        let result = processor.format_output("  hello   world  ");
        assert_eq!(result, "hello world");
    }

    #[tokio::test]
    async fn test_process_with_formatting_only() {
        let mut config = PostProcessingConfig::default();
        config.enable_formatting = true;

        let processor = PostProcessor::new(config);
        let result = processor.process("  test  text  ".to_string()).await.unwrap();

        assert_eq!(result, "test text");
    }

    #[test]
    fn test_custom_words_disabled_by_default() {
        let processor = PostProcessor::default();
        assert!(!processor.config.enable_custom_words);
    }

    #[test]
    fn test_ollama_disabled_by_default() {
        let processor = PostProcessor::default();
        assert!(!processor.config.enable_ollama);
        assert!(processor.ollama_client.is_none());
    }
}

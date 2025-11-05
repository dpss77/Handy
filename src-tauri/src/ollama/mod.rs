//! Ollama integration module
//!
//! Provides local LLM post-processing of transcriptions via Ollama's HTTP API.
//!
//! # Features
//!
//! - **Zero Model Duplication**: Uses Ollama's existing models via HTTP API
//! - **Privacy First**: All processing happens locally (localhost:11434)
//! - **Optional**: Gracefully degrades if Ollama is not available
//! - **Flexible**: Supports any Ollama model and custom prompts
//!
//! # Use Cases
//!
//! 1. **Punctuation & Capitalization** - Auto-format raw transcriptions
//! 2. **Summarization** - Condense long transcriptions
//! 3. **Command Extraction** - Parse voice commands from natural language
//! 4. **Translation Enhancement** - Improve multi-language output
//! 5. **Custom Processing** - User-defined prompts for domain-specific needs
//!
//! # Architecture
//!
//! ```text
//! Audio → Whisper/Parakeet → [Optional] Ollama Enhancement → Output
//! ```
//!
//! # Example
//!
//! ```no_run
//! use handy_app_lib::ollama::{OllamaClient, OllamaRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OllamaClient::new("http://localhost:11434")?;
//!
//! // Check if Ollama is available
//! if client.is_available().await {
//!     let request = OllamaRequest::new("llama3.2")
//!         .with_prompt("Add proper punctuation and capitalization")
//!         .with_text("hello world this is a test");
//!
//!     let enhanced = client.process(request).await?;
//!     println!("Enhanced: {}", enhanced);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Ollama HTTP client for local LLM processing
///
/// Communicates with a locally-running Ollama instance via HTTP API.
#[derive(Clone, Debug)]
pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
    timeout: Duration,
}

/// Request to Ollama for text processing
#[derive(Clone, Debug, Serialize)]
pub struct OllamaRequest {
    /// Model name (e.g., "llama3.2", "mistral", "gemma2")
    pub model: String,
    /// System prompt describing the task
    pub system: Option<String>,
    /// The text to process (transcription)
    pub prompt: String,
    /// Temperature (0.0 = deterministic, 1.0 = creative)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Whether to stream the response
    pub stream: bool,
}

/// Response from Ollama
#[derive(Clone, Debug, Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
}

/// Ollama processing mode
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessingMode {
    /// Add punctuation and capitalization
    Punctuation,
    /// Summarize the text
    Summarize,
    /// Extract commands
    CommandExtraction,
    /// Custom user-defined prompt
    Custom { prompt: String },
    /// No processing
    Disabled,
}

impl OllamaClient {
    /// Creates a new Ollama client
    ///
    /// # Arguments
    ///
    /// * `base_url` - Ollama API endpoint (typically "http://localhost:11434")
    ///
    /// # Example
    ///
    /// ```
    /// # use handy_app_lib::ollama::OllamaClient;
    /// let client = OllamaClient::new("http://localhost:11434")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(base_url: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
            timeout: Duration::from_secs(30),
        })
    }

    /// Creates client with default settings (localhost:11434)
    pub fn default() -> Result<Self> {
        Self::new("http://localhost:11434")
    }

    /// Sets request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Checks if Ollama is available
    ///
    /// Makes a lightweight request to verify Ollama is running.
    ///
    /// # Returns
    ///
    /// `true` if Ollama is accessible, `false` otherwise
    pub async fn is_available(&self) -> bool {
        match self.client.get(&self.base_url).send().await {
            Ok(response) => {
                let available = response.status().is_success();
                if available {
                    debug!("Ollama is available at {}", self.base_url);
                } else {
                    warn!("Ollama responded with status: {}", response.status());
                }
                available
            }
            Err(e) => {
                debug!("Ollama not available: {}", e);
                false
            }
        }
    }

    /// Lists available models
    ///
    /// # Returns
    ///
    /// Vector of model names available in Ollama
    pub async fn list_models(&self) -> Result<Vec<String>> {
        #[derive(Deserialize)]
        struct ModelList {
            models: Vec<Model>,
        }

        #[derive(Deserialize)]
        struct Model {
            name: String,
        }

        let url = format!("{}/api/tags", self.base_url);
        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to list models: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Ollama returned error: {}", response.status()));
        }

        let model_list: ModelList = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse model list: {}", e))?;

        Ok(model_list.models.into_iter().map(|m| m.name).collect())
    }

    /// Processes text using Ollama
    ///
    /// # Arguments
    ///
    /// * `request` - Processing request with model and prompt
    ///
    /// # Returns
    ///
    /// Processed text from Ollama
    ///
    /// # Errors
    ///
    /// Returns error if Ollama is unavailable or processing fails
    pub async fn process(&self, request: OllamaRequest) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        debug!(
            "Processing with Ollama model '{}': {} chars",
            request.model,
            request.prompt.len()
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| anyhow!("Ollama request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Ollama returned error {}: {}",
                status,
                error_text
            ));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

        info!(
            "Ollama processing complete: {} → {} chars",
            request.prompt.len(),
            ollama_response.response.len()
        );

        Ok(ollama_response.response.trim().to_string())
    }

    /// Processes text with a predefined mode
    ///
    /// # Arguments
    ///
    /// * `text` - Input text to process
    /// * `mode` - Processing mode
    /// * `model` - Ollama model name
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use handy_app_lib::ollama::{OllamaClient, ProcessingMode};
    /// # async fn example() -> Result<(), anyhow::Error> {
    /// let client = OllamaClient::default()?;
    /// let result = client.process_with_mode(
    ///     "hello world",
    ///     ProcessingMode::Punctuation,
    ///     "llama3.2"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn process_with_mode(
        &self,
        text: &str,
        mode: ProcessingMode,
        model: &str,
    ) -> Result<String> {
        let request = match mode {
            ProcessingMode::Disabled => return Ok(text.to_string()),
            ProcessingMode::Punctuation => OllamaRequest::new(model)
                .with_system("You are a text formatter. Add proper punctuation and capitalization to the following text. Only return the formatted text, nothing else.")
                .with_text(text)
                .with_temperature(0.1), // Low temperature for consistency
            ProcessingMode::Summarize => OllamaRequest::new(model)
                .with_system("You are a summarization expert. Provide a concise summary of the following text. Only return the summary, nothing else.")
                .with_text(text)
                .with_temperature(0.3),
            ProcessingMode::CommandExtraction => OllamaRequest::new(model)
                .with_system("You are a command parser. Extract any commands or action items from the following text. List them clearly. If no commands are present, return 'No commands found'.")
                .with_text(text)
                .with_temperature(0.2),
            ProcessingMode::Custom { prompt } => OllamaRequest::new(model)
                .with_system(&prompt)
                .with_text(text)
                .with_temperature(0.3),
        };

        self.process(request).await
    }
}

impl OllamaRequest {
    /// Creates a new request
    ///
    /// # Arguments
    ///
    /// * `model` - Ollama model name (e.g., "llama3.2")
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
            system: None,
            prompt: String::new(),
            temperature: None,
            stream: false,
        }
    }

    /// Sets the system prompt
    pub fn with_system(mut self, system: &str) -> Self {
        self.system = Some(system.to_string());
        self
    }

    /// Sets the text to process
    pub fn with_text(mut self, text: &str) -> Self {
        self.prompt = text.to_string();
        self
    }

    /// Sets a custom prompt (overrides with_text)
    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.prompt = prompt.to_string();
        self
    }

    /// Sets the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 2.0));
        self
    }

    /// Enables streaming (not yet implemented)
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.stream = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_temperature_clamping() {
        let request = OllamaRequest::new("test").with_temperature(5.0);
        assert_eq!(request.temperature, Some(2.0)); // Should clamp to 2.0

        let request = OllamaRequest::new("test").with_temperature(-1.0);
        assert_eq!(request.temperature, Some(0.0)); // Should clamp to 0.0
    }

    #[test]
    fn test_processing_mode_equality() {
        assert_eq!(ProcessingMode::Punctuation, ProcessingMode::Punctuation);
        assert_eq!(ProcessingMode::Disabled, ProcessingMode::Disabled);
        assert_ne!(ProcessingMode::Punctuation, ProcessingMode::Summarize);
    }
}

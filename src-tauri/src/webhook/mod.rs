//! Webhook system for external integrations
//!
//! Allows Handy to send transcription events to external services via HTTP webhooks.

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Webhook event types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    /// Recording started
    RecordingStarted,
    /// Recording stopped
    RecordingStopped,
    /// Transcription completed
    TranscriptionComplete,
    /// Transcription error
    TranscriptionError,
}

/// Webhook authentication method
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebhookAuth {
    /// No authentication
    None,
    /// Bearer token authentication
    Bearer { token: String },
    /// API key in header
    ApiKey { header: String, value: String },
    /// Basic authentication
    Basic { username: String, password: String },
}

/// Webhook configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Unique identifier for this webhook
    pub id: String,
    /// Display name
    pub name: String,
    /// Webhook URL
    pub url: String,
    /// Events to trigger on
    pub events: Vec<WebhookEventType>,
    /// Authentication method
    #[serde(default)]
    pub auth: WebhookAuth,
    /// Whether webhook is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Retry attempts on failure
    #[serde(default = "default_retries")]
    pub max_retries: u32,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_true() -> bool {
    true
}

fn default_retries() -> u32 {
    3
}

fn default_timeout() -> u64 {
    10
}

impl Default for WebhookAuth {
    fn default() -> Self {
        WebhookAuth::None
    }
}

/// Webhook payload
#[derive(Clone, Debug, Serialize)]
pub struct WebhookPayload {
    /// Event type
    pub event: WebhookEventType,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Event data
    #[serde(flatten)]
    pub data: WebhookData,
}

/// Webhook event data
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebhookData {
    /// Recording started event
    RecordingStarted {},
    /// Recording stopped event
    RecordingStopped {
        duration_secs: f64,
    },
    /// Transcription complete event
    TranscriptionComplete {
        text: String,
        duration_secs: f64,
        language: String,
        model: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        confidence: Option<f32>,
    },
    /// Transcription error event
    TranscriptionError {
        error: String,
    },
}

/// Webhook client
pub struct WebhookClient {
    client: reqwest::Client,
}

impl WebhookClient {
    /// Creates a new webhook client
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Sends a webhook
    ///
    /// # Arguments
    ///
    /// * `config` - Webhook configuration
    /// * `payload` - Webhook payload
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, error otherwise
    pub async fn send(&self, config: &WebhookConfig, payload: WebhookPayload) -> Result<()> {
        if !config.enabled {
            debug!("Webhook {} disabled, skipping", config.name);
            return Ok(());
        }

        if !config.events.contains(&payload.event) {
            debug!(
                "Webhook {} not configured for {:?}, skipping",
                config.name, payload.event
            );
            return Ok(());
        }

        info!(
            "Sending webhook {} for event {:?}",
            config.name, payload.event
        );

        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            if attempt > 0 {
                let backoff = Duration::from_millis(100 * (1 << attempt));
                debug!("Retry attempt {} after {:?}", attempt, backoff);
                tokio::time::sleep(backoff).await;
            }

            match self.send_once(config, &payload).await {
                Ok(()) => {
                    info!("Webhook {} sent successfully", config.name);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Webhook {} attempt {} failed: {}", config.name, attempt, e);
                    last_error = Some(e);
                }
            }
        }

        if let Some(e) = last_error {
            error!(
                "Webhook {} failed after {} attempts",
                config.name,
                config.max_retries + 1
            );
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Sends webhook once (no retries)
    async fn send_once(&self, config: &WebhookConfig, payload: &WebhookPayload) -> Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Add authentication
        match &config.auth {
            WebhookAuth::None => {}
            WebhookAuth::Bearer { token } => {
                let value = format!("Bearer {}", token);
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&value)
                        .context("Invalid bearer token")?,
                );
            }
            WebhookAuth::ApiKey { header, value } => {
                headers.insert(
                    reqwest::header::HeaderName::from_bytes(header.as_bytes())
                        .context("Invalid header name")?,
                    HeaderValue::from_str(value).context("Invalid header value")?,
                );
            }
            WebhookAuth::Basic { username, password } => {
                let credentials = base64::encode(format!("{}:{}", username, password));
                let value = format!("Basic {}", credentials);
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&value).context("Invalid basic auth")?,
                );
            }
        }

        let body = serde_json::to_string(&payload).context("Failed to serialize payload")?;

        let response = self
            .client
            .post(&config.url)
            .headers(headers)
            .body(body)
            .timeout(Duration::from_secs(config.timeout_secs))
            .send()
            .await
            .context("Failed to send webhook")?;

        if response.status().is_success() {
            debug!("Webhook received success response: {}", response.status());
            Ok(())
        } else {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "No body".to_string());
            Err(anyhow::anyhow!(
                "Webhook failed with status {}: {}",
                status,
                body
            ))
        }
    }
}

impl Default for WebhookClient {
    fn default() -> Self {
        Self::new().expect("Failed to create webhook client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_config_serialization() {
        let config = WebhookConfig {
            id: "test-1".to_string(),
            name: "Test Webhook".to_string(),
            url: "https://example.com/webhook".to_string(),
            events: vec![
                WebhookEventType::TranscriptionComplete,
                WebhookEventType::RecordingStarted,
            ],
            auth: WebhookAuth::Bearer {
                token: "secret".to_string(),
            },
            enabled: true,
            max_retries: 3,
            timeout_secs: 10,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: WebhookConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.id, deserialized.id);
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.url, deserialized.url);
        assert_eq!(config.events, deserialized.events);
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            event: WebhookEventType::TranscriptionComplete,
            timestamp: "2025-11-05T10:00:00Z".to_string(),
            data: WebhookData::TranscriptionComplete {
                text: "Hello world".to_string(),
                duration_secs: 1.5,
                language: "en".to_string(),
                model: "whisper-small".to_string(),
                confidence: Some(0.95),
            },
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"event\":\"transcription_complete\""));
        assert!(json.contains("\"text\":\"Hello world\""));
        assert!(json.contains("\"confidence\":0.95"));
    }

    #[test]
    fn test_webhook_auth_variants() {
        let none = WebhookAuth::None;
        let bearer = WebhookAuth::Bearer {
            token: "token".to_string(),
        };
        let api_key = WebhookAuth::ApiKey {
            header: "X-API-Key".to_string(),
            value: "key".to_string(),
        };
        let basic = WebhookAuth::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        let none_json = serde_json::to_string(&none).unwrap();
        let bearer_json = serde_json::to_string(&bearer).unwrap();
        let api_key_json = serde_json::to_string(&api_key).unwrap();
        let basic_json = serde_json::to_string(&basic).unwrap();

        assert!(none_json.contains("\"type\":\"none\""));
        assert!(bearer_json.contains("\"type\":\"bearer\""));
        assert!(api_key_json.contains("\"type\":\"api_key\""));
        assert!(basic_json.contains("\"type\":\"basic\""));
    }

    #[test]
    fn test_webhook_event_filtering() {
        let config = WebhookConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            url: "https://example.com".to_string(),
            events: vec![WebhookEventType::TranscriptionComplete],
            auth: WebhookAuth::None,
            enabled: true,
            max_retries: 0,
            timeout_secs: 10,
        };

        assert!(config
            .events
            .contains(&WebhookEventType::TranscriptionComplete));
        assert!(!config.events.contains(&WebhookEventType::RecordingStarted));
    }

    #[test]
    fn test_default_values() {
        let config = WebhookConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            url: "https://example.com".to_string(),
            events: vec![],
            auth: WebhookAuth::default(),
            enabled: default_true(),
            max_retries: default_retries(),
            timeout_secs: default_timeout(),
        };

        assert_eq!(config.enabled, true);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_secs, 10);
    }
}

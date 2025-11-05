//! Tauri commands for webhook functionality

use crate::webhook::{WebhookAuth, WebhookClient, WebhookConfig, WebhookEventType};
use serde_json;

#[tauri::command]
pub async fn test_webhook(
    url: String,
    auth_type: String,
    auth_value: Option<String>,
) -> Result<bool, String> {
    // Parse auth
    let auth = match auth_type.as_str() {
        "none" => WebhookAuth::None,
        "bearer" => WebhookAuth::Bearer {
            token: auth_value.ok_or("Bearer token required")?,
        },
        "api_key" => {
            let value = auth_value.ok_or("API key value required")?;
            WebhookAuth::ApiKey {
                header: "X-API-Key".to_string(),
                value,
            }
        }
        "basic" => {
            let credentials = auth_value.ok_or("Basic auth credentials required")?;
            let parts: Vec<&str> = credentials.split(':').collect();
            if parts.len() != 2 {
                return Err("Basic auth format: username:password".to_string());
            }
            WebhookAuth::Basic {
                username: parts[0].to_string(),
                password: parts[1].to_string(),
            }
        }
        _ => return Err(format!("Unknown auth type: {}", auth_type)),
    };

    // Create test config
    let config = WebhookConfig {
        id: "test".to_string(),
        name: "Test".to_string(),
        url,
        events: vec![WebhookEventType::TranscriptionComplete],
        auth,
        enabled: true,
        max_retries: 0,
        timeout_secs: 5,
    };

    // Create test payload
    let payload = crate::webhook::WebhookPayload {
        event: WebhookEventType::TranscriptionComplete,
        timestamp: chrono::Utc::now().to_rfc3339(),
        data: crate::webhook::WebhookData::TranscriptionComplete {
            text: "Test transcription".to_string(),
            duration_secs: 1.0,
            language: "en".to_string(),
            model: "test".to_string(),
            confidence: None,
        },
    };

    // Try to send
    let client = WebhookClient::new().map_err(|e| e.to_string())?;
    match client.send(&config, payload).await {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn validate_webhook_url(url: String) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

#[tauri::command]
pub fn get_webhook_event_types() -> Vec<String> {
    vec![
        "recording_started".to_string(),
        "recording_stopped".to_string(),
        "transcription_complete".to_string(),
        "transcription_error".to_string(),
    ]
}

#[tauri::command]
pub fn get_webhook_auth_types() -> Vec<String> {
    vec![
        "none".to_string(),
        "bearer".to_string(),
        "api_key".to_string(),
        "basic".to_string(),
    ]
}

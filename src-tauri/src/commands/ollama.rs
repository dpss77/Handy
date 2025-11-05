//! Tauri commands for Ollama integration

use crate::ollama::{OllamaClient, ProcessingMode};
use tauri::State;

#[tauri::command]
pub async fn check_ollama_available(url: Option<String>) -> Result<bool, String> {
    let client = if let Some(url) = url {
        OllamaClient::new(&url).map_err(|e| e.to_string())?
    } else {
        OllamaClient::default().map_err(|e| e.to_string())?
    };

    Ok(client.is_available().await)
}

#[tauri::command]
pub async fn list_ollama_models(url: Option<String>) -> Result<Vec<String>, String> {
    let client = if let Some(url) = url {
        OllamaClient::new(&url).map_err(|e| e.to_string())?
    } else {
        OllamaClient::default().map_err(|e| e.to_string())?
    };

    client.list_models().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_ollama_processing(
    text: String,
    model: String,
    url: Option<String>,
) -> Result<String, String> {
    let client = if let Some(url) = url {
        OllamaClient::new(&url).map_err(|e| e.to_string())?
    } else {
        OllamaClient::default().map_err(|e| e.to_string())?
    };

    client
        .process_with_mode(&text, ProcessingMode::Punctuation, &model)
        .await
        .map_err(|e| e.to_string())
}

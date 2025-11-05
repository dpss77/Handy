//! Tauri commands for export functionality

use crate::export::{ExportFormat, TranscriptionExport, TranscriptionSegment};
use chrono::{DateTime, Utc};
use tauri::AppHandle;

#[tauri::command]
pub fn export_transcription(
    text: String,
    language: String,
    model: String,
    timestamp: String,
    duration_secs: Option<f64>,
    confidence: Option<f32>,
    segments: Option<Vec<TranscriptionSegment>>,
    format: String,
) -> Result<String, String> {
    // Parse timestamp
    let timestamp: DateTime<Utc> = timestamp
        .parse()
        .map_err(|e| format!("Invalid timestamp: {}", e))?;

    // Parse format
    let format: ExportFormat = serde_json::from_value(serde_json::json!(format))
        .map_err(|e| format!("Invalid format: {}", e))?;

    // Create export
    let mut export = TranscriptionExport {
        text,
        language,
        model,
        timestamp,
        duration_secs,
        confidence,
        segments,
    };

    // Export to string
    export
        .export(format)
        .map_err(|e| format!("Export failed: {}", e))
}

#[tauri::command]
pub async fn export_transcription_to_file(
    app: AppHandle,
    text: String,
    language: String,
    model: String,
    timestamp: String,
    duration_secs: Option<f64>,
    confidence: Option<f32>,
    segments: Option<Vec<TranscriptionSegment>>,
    format: String,
    file_path: String,
) -> Result<(), String> {
    // Parse timestamp
    let timestamp: DateTime<Utc> = timestamp
        .parse()
        .map_err(|e| format!("Invalid timestamp: {}", e))?;

    // Parse format
    let format: ExportFormat = serde_json::from_value(serde_json::json!(format))
        .map_err(|e| format!("Invalid format: {}", e))?;

    // Create export
    let export = TranscriptionExport {
        text,
        language,
        model,
        timestamp,
        duration_secs,
        confidence,
        segments,
    };

    // Export to file
    export
        .export_to_file(&file_path, format)
        .map_err(|e| format!("Export to file failed: {}", e))
}

#[tauri::command]
pub fn get_export_formats() -> Vec<String> {
    vec![
        "text".to_string(),
        "markdown".to_string(),
        "json".to_string(),
        "srt".to_string(),
        "vtt".to_string(),
    ]
}

//! Export functionality for transcriptions
//!
//! Supports multiple export formats: plain text, markdown, JSON, SRT, and VTT.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

/// Export format
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// Plain text (.txt)
    Text,
    /// Markdown (.md)
    Markdown,
    /// JSON structured data (.json)
    Json,
    /// SRT subtitles (.srt)
    Srt,
    /// WebVTT captions (.vtt)
    Vtt,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportFormat::Text => write!(f, "txt"),
            ExportFormat::Markdown => write!(f, "md"),
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Srt => write!(f, "srt"),
            ExportFormat::Vtt => write!(f, "vtt"),
        }
    }
}

/// Transcription data for export
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptionExport {
    /// Transcribed text
    pub text: String,
    /// Language code (e.g., "en", "es")
    pub language: String,
    /// Model used for transcription
    pub model: String,
    /// Timestamp when transcription was created
    pub timestamp: DateTime<Utc>,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f64>,
    /// Confidence score (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    /// Segments with timestamps (for SRT/VTT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<TranscriptionSegment>>,
}

/// A timestamped segment of transcription
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    /// Start time in seconds
    pub start_secs: f64,
    /// End time in seconds
    pub end_secs: f64,
    /// Segment text
    pub text: String,
}

impl TranscriptionExport {
    /// Creates a new transcription export
    pub fn new(text: String, language: String, model: String) -> Self {
        Self {
            text,
            language,
            model,
            timestamp: Utc::now(),
            duration_secs: None,
            confidence: None,
            segments: None,
        }
    }

    /// Sets duration
    pub fn with_duration(mut self, duration_secs: f64) -> Self {
        self.duration_secs = Some(duration_secs);
        self
    }

    /// Sets confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Sets segments
    pub fn with_segments(mut self, segments: Vec<TranscriptionSegment>) -> Self {
        self.segments = Some(segments);
        self
    }

    /// Exports to the specified format
    pub fn export(&self, format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::Text => self.export_text(),
            ExportFormat::Markdown => self.export_markdown(),
            ExportFormat::Json => self.export_json(),
            ExportFormat::Srt => self.export_srt(),
            ExportFormat::Vtt => self.export_vtt(),
        }
    }

    /// Exports to a file
    pub fn export_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        format: ExportFormat,
    ) -> Result<()> {
        let content = self.export(format)?;
        fs::write(path, content).context("Failed to write export file")?;
        Ok(())
    }

    /// Exports as plain text
    fn export_text(&self) -> Result<String> {
        Ok(self.text.clone())
    }

    /// Exports as markdown
    fn export_markdown(&self) -> Result<String> {
        let mut content = String::new();

        // Title
        content.push_str("# Transcription\n\n");

        // Metadata
        content.push_str("## Metadata\n\n");
        content.push_str(&format!("- **Date**: {}\n", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        content.push_str(&format!("- **Language**: {}\n", self.language));
        content.push_str(&format!("- **Model**: {}\n", self.model));

        if let Some(duration) = self.duration_secs {
            content.push_str(&format!("- **Duration**: {:.2}s\n", duration));
        }

        if let Some(confidence) = self.confidence {
            content.push_str(&format!("- **Confidence**: {:.1}%\n", confidence * 100.0));
        }

        // Content
        content.push_str("\n## Content\n\n");
        content.push_str(&self.text);
        content.push('\n');

        Ok(content)
    }

    /// Exports as JSON
    fn export_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize to JSON")
    }

    /// Exports as SRT subtitles
    fn export_srt(&self) -> Result<String> {
        let segments = self.segments.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Cannot export to SRT without segment timestamps")
        })?;

        let mut content = String::new();

        for (index, segment) in segments.iter().enumerate() {
            // Sequence number
            content.push_str(&format!("{}\n", index + 1));

            // Timestamp range (SRT format: HH:MM:SS,mmm --> HH:MM:SS,mmm)
            let start = format_srt_timestamp(segment.start_secs);
            let end = format_srt_timestamp(segment.end_secs);
            content.push_str(&format!("{} --> {}\n", start, end));

            // Text content
            content.push_str(&segment.text);
            content.push_str("\n\n");
        }

        Ok(content)
    }

    /// Exports as WebVTT captions
    fn export_vtt(&self) -> Result<String> {
        let segments = self.segments.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Cannot export to VTT without segment timestamps")
        })?;

        let mut content = String::new();

        // VTT header
        content.push_str("WEBVTT\n\n");

        for segment in segments {
            // Timestamp range (VTT format: HH:MM:SS.mmm --> HH:MM:SS.mmm)
            let start = format_vtt_timestamp(segment.start_secs);
            let end = format_vtt_timestamp(segment.end_secs);
            content.push_str(&format!("{} --> {}\n", start, end));

            // Text content
            content.push_str(&segment.text);
            content.push_str("\n\n");
        }

        Ok(content)
    }
}

/// Formats seconds as SRT timestamp (HH:MM:SS,mmm)
fn format_srt_timestamp(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    let millis = ((seconds % 1.0) * 1000.0).floor() as u32;

    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

/// Formats seconds as VTT timestamp (HH:MM:SS.mmm)
fn format_vtt_timestamp(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    let millis = ((seconds % 1.0) * 1000.0).floor() as u32;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_export() -> TranscriptionExport {
        TranscriptionExport::new(
            "Hello world, this is a test.".to_string(),
            "en".to_string(),
            "whisper-small".to_string(),
        )
        .with_duration(2.5)
        .with_confidence(0.95)
    }

    fn create_test_export_with_segments() -> TranscriptionExport {
        create_test_export().with_segments(vec![
            TranscriptionSegment {
                start_secs: 0.0,
                end_secs: 1.5,
                text: "Hello world".to_string(),
            },
            TranscriptionSegment {
                start_secs: 1.5,
                end_secs: 2.5,
                text: "this is a test.".to_string(),
            },
        ])
    }

    #[test]
    fn test_export_text() {
        let export = create_test_export();
        let result = export.export(ExportFormat::Text).unwrap();
        assert_eq!(result, "Hello world, this is a test.");
    }

    #[test]
    fn test_export_markdown() {
        let export = create_test_export();
        let result = export.export(ExportFormat::Markdown).unwrap();

        assert!(result.contains("# Transcription"));
        assert!(result.contains("## Metadata"));
        assert!(result.contains("Language**: en"));
        assert!(result.contains("Model**: whisper-small"));
        assert!(result.contains("Duration**: 2.50s"));
        assert!(result.contains("Confidence**: 95.0%"));
        assert!(result.contains("Hello world, this is a test."));
    }

    #[test]
    fn test_export_json() {
        let export = create_test_export();
        let result = export.export(ExportFormat::Json).unwrap();

        // Parse back to verify structure
        let parsed: TranscriptionExport = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.text, "Hello world, this is a test.");
        assert_eq!(parsed.language, "en");
        assert_eq!(parsed.model, "whisper-small");
        assert_eq!(parsed.duration_secs, Some(2.5));
        assert_eq!(parsed.confidence, Some(0.95));
    }

    #[test]
    fn test_export_srt() {
        let export = create_test_export_with_segments();
        let result = export.export(ExportFormat::Srt).unwrap();

        assert!(result.contains("1\n"));
        assert!(result.contains("00:00:00,000 --> 00:00:01,500"));
        assert!(result.contains("Hello world"));
        assert!(result.contains("2\n"));
        assert!(result.contains("00:00:01,500 --> 00:00:02,500"));
        assert!(result.contains("this is a test."));
    }

    #[test]
    fn test_export_vtt() {
        let export = create_test_export_with_segments();
        let result = export.export(ExportFormat::Vtt).unwrap();

        assert!(result.starts_with("WEBVTT\n\n"));
        assert!(result.contains("00:00:00.000 --> 00:00:01.500"));
        assert!(result.contains("Hello world"));
        assert!(result.contains("00:00:01.500 --> 00:00:02.500"));
        assert!(result.contains("this is a test."));
    }

    #[test]
    fn test_export_srt_without_segments_fails() {
        let export = create_test_export();
        let result = export.export(ExportFormat::Srt);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("without segment timestamps"));
    }

    #[test]
    fn test_format_srt_timestamp() {
        assert_eq!(format_srt_timestamp(0.0), "00:00:00,000");
        assert_eq!(format_srt_timestamp(1.5), "00:00:01,500");
        assert_eq!(format_srt_timestamp(65.123), "00:01:05,123");
        assert_eq!(format_srt_timestamp(3661.789), "01:01:01,789");
    }

    #[test]
    fn test_format_vtt_timestamp() {
        assert_eq!(format_vtt_timestamp(0.0), "00:00:00.000");
        assert_eq!(format_vtt_timestamp(1.5), "00:00:01.500");
        assert_eq!(format_vtt_timestamp(65.123), "00:01:05.123");
        assert_eq!(format_vtt_timestamp(3661.789), "01:01:01.789");
    }

    #[test]
    fn test_export_format_display() {
        assert_eq!(ExportFormat::Text.to_string(), "txt");
        assert_eq!(ExportFormat::Markdown.to_string(), "md");
        assert_eq!(ExportFormat::Json.to_string(), "json");
        assert_eq!(ExportFormat::Srt.to_string(), "srt");
        assert_eq!(ExportFormat::Vtt.to_string(), "vtt");
    }
}

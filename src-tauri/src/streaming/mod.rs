//! Real-time streaming transcription module
//!
//! Provides chunked audio processing with partial results for improved UX.
//!
//! # Architecture
//!
//! Instead of waiting for complete audio, streams partial transcription results:
//!
//! ```text
//! Audio Stream → Chunker → Whisper (chunked) → Partial Results → UI
//! ```
//!
//! # Benefits
//!
//! - **Lower latency**: See results as you speak
//! - **Better UX**: Visual feedback during long recordings
//! - **Cancellation**: Stop mid-transcription if needed

use anyhow::Result;
use log::{debug, info};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

/// Partial transcription result
#[derive(Clone, Debug, Serialize)]
pub struct PartialTranscription {
    /// Chunk index (0-based)
    pub chunk_index: usize,
    /// Total number of chunks (if known)
    pub total_chunks: Option<usize>,
    /// Partial text from this chunk
    pub text: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Whether this is the final chunk
    pub is_final: bool,
}

/// Configuration for streaming transcription
#[derive(Clone, Debug)]
pub struct StreamingConfig {
    /// Size of each audio chunk in samples
    pub chunk_size: usize,
    /// Overlap between chunks (in samples) for context
    pub overlap_size: usize,
    /// Minimum confidence to emit partial results
    pub min_confidence: f32,
    /// Enable partial result emission
    pub enable_partial_results: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 16000 * 3, // 3 seconds at 16kHz
            overlap_size: 16000,   // 1 second overlap
            min_confidence: 0.5,
            enable_partial_results: true,
        }
    }
}

/// Streaming transcription processor
///
/// Processes audio in chunks and emits partial results.
pub struct StreamingTranscriber {
    config: StreamingConfig,
    app_handle: AppHandle,
    buffer: Arc<Mutex<Vec<f32>>>,
    chunk_count: Arc<Mutex<usize>>,
}

impl StreamingTranscriber {
    /// Creates a new streaming transcriber
    pub fn new(config: StreamingConfig, app_handle: AppHandle) -> Self {
        Self {
            config,
            app_handle,
            buffer: Arc::new(Mutex::new(Vec::new())),
            chunk_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates with default configuration
    pub fn default(app_handle: AppHandle) -> Self {
        Self::new(StreamingConfig::default(), app_handle)
    }

    /// Adds audio samples to the buffer
    ///
    /// Automatically processes chunks when buffer is large enough.
    pub fn push_audio(&self, samples: &[f32]) -> Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(samples);

        debug!("Buffer size: {} samples", buffer.len());

        // Check if we have enough for a chunk
        if buffer.len() >= self.config.chunk_size {
            self.process_next_chunk(&mut buffer)?;
        }

        Ok(())
    }

    /// Processes the next chunk from the buffer
    fn process_next_chunk(&self, buffer: &mut Vec<f32>) -> Result<()> {
        if buffer.len() < self.config.chunk_size {
            return Ok(());
        }

        // Extract chunk
        let chunk: Vec<f32> = buffer.drain(..self.config.chunk_size).collect();
        let chunk_index = *self.chunk_count.lock().unwrap();
        *self.chunk_count.lock().unwrap() += 1;

        // Keep overlap in buffer
        let overlap_start = chunk.len().saturating_sub(self.config.overlap_size);
        buffer.splice(0..0, chunk[overlap_start..].iter().copied());

        debug!(
            "Processing chunk {}: {} samples",
            chunk_index,
            chunk.len()
        );

        // In real implementation, this would call the transcription engine
        // For now, simulate processing
        let partial_result = self.simulate_transcription(&chunk, chunk_index)?;

        // Emit partial result if enabled
        if self.config.enable_partial_results {
            self.emit_partial_result(partial_result)?;
        }

        Ok(())
    }

    /// Simulates transcription (placeholder for actual implementation)
    fn simulate_transcription(
        &self,
        _chunk: &[f32],
        chunk_index: usize,
    ) -> Result<PartialTranscription> {
        Ok(PartialTranscription {
            chunk_index,
            total_chunks: None,
            text: format!("Chunk {} text", chunk_index),
            confidence: 0.85,
            is_final: false,
        })
    }

    /// Emits a partial result to the frontend
    fn emit_partial_result(&self, result: PartialTranscription) -> Result<()> {
        if result.confidence >= self.config.min_confidence {
            info!(
                "Emitting partial result: chunk {} ({} chars)",
                result.chunk_index,
                result.text.len()
            );

            self.app_handle
                .emit("partial-transcription", result)
                .map_err(|e| anyhow::anyhow!("Failed to emit partial result: {}", e))?;
        } else {
            debug!(
                "Skipping low confidence result: {}",
                result.confidence
            );
        }

        Ok(())
    }

    /// Flushes remaining buffer and emits final result
    pub fn finalize(&self) -> Result<()> {
        let mut buffer = self.buffer.lock().unwrap();

        if !buffer.is_empty() {
            debug!("Finalizing with {} remaining samples", buffer.len());

            let chunk_index = *self.chunk_count.lock().unwrap();
            let final_chunk = buffer.clone();

            let mut final_result = self.simulate_transcription(&final_chunk, chunk_index)?;
            final_result.is_final = true;
            final_result.total_chunks = Some(chunk_index + 1);

            self.emit_partial_result(final_result)?;
        }

        // Reset state
        buffer.clear();
        *self.chunk_count.lock().unwrap() = 0;

        Ok(())
    }

    /// Cancels ongoing streaming and clears buffer
    pub fn cancel(&self) {
        debug!("Cancelling streaming transcription");
        self.buffer.lock().unwrap().clear();
        *self.chunk_count.lock().unwrap() = 0;
    }

    /// Gets current buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    /// Gets number of processed chunks
    pub fn chunk_count(&self) -> usize {
        *self.chunk_count.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StreamingConfig::default();
        assert_eq!(config.chunk_size, 16000 * 3);
        assert_eq!(config.overlap_size, 16000);
        assert_eq!(config.min_confidence, 0.5);
        assert!(config.enable_partial_results);
    }

    #[test]
    fn test_config_creation() {
        let config = StreamingConfig {
            chunk_size: 8000,
            overlap_size: 2000,
            min_confidence: 0.7,
            enable_partial_results: false,
        };

        assert_eq!(config.chunk_size, 8000);
        assert_eq!(config.overlap_size, 2000);
        assert_eq!(config.min_confidence, 0.7);
        assert!(!config.enable_partial_results);
    }

    #[test]
    fn test_partial_transcription_serialization() {
        let partial = PartialTranscription {
            chunk_index: 0,
            total_chunks: Some(5),
            text: "Test".to_string(),
            confidence: 0.9,
            is_final: false,
        };

        let json = serde_json::to_string(&partial).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("0.9"));
    }

    // Note: Full tests require Tauri app context
    // These are basic structure tests
}

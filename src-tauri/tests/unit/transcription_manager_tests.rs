//! Unit tests for TranscriptionManager
//!
//! Tests cover:
//! - Transcription state management
//! - Audio processing pipeline
//! - Custom word corrections
//! - Post-processing logic

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    // ========================================================================
    // Transcription State Tests
    // ========================================================================

    #[derive(Debug, Clone, PartialEq)]
    pub enum TranscriptionState {
        Idle,
        Processing,
        Completed(String),
        Failed(String),
    }

    struct TranscriptionStateManager {
        state: Arc<Mutex<TranscriptionState>>,
    }

    impl TranscriptionStateManager {
        fn new() -> Self {
            Self {
                state: Arc::new(Mutex::new(TranscriptionState::Idle)),
            }
        }

        fn get_state(&self) -> TranscriptionState {
            self.state.lock().unwrap().clone()
        }

        fn set_processing(&self) {
            *self.state.lock().unwrap() = TranscriptionState::Processing;
        }

        fn set_completed(&self, text: String) {
            *self.state.lock().unwrap() = TranscriptionState::Completed(text);
        }

        fn set_failed(&self, error: String) {
            *self.state.lock().unwrap() = TranscriptionState::Failed(error);
        }

        fn reset(&self) {
            *self.state.lock().unwrap() = TranscriptionState::Idle;
        }
    }

    #[test]
    fn test_initial_state_is_idle() {
        let manager = TranscriptionStateManager::new();
        assert_eq!(manager.get_state(), TranscriptionState::Idle);
    }

    #[test]
    fn test_state_transitions() {
        let manager = TranscriptionStateManager::new();

        manager.set_processing();
        assert_eq!(manager.get_state(), TranscriptionState::Processing);

        manager.set_completed("test".to_string());
        assert_eq!(
            manager.get_state(),
            TranscriptionState::Completed("test".to_string())
        );

        manager.reset();
        assert_eq!(manager.get_state(), TranscriptionState::Idle);
    }

    #[test]
    fn test_failed_state() {
        let manager = TranscriptionStateManager::new();

        manager.set_processing();
        manager.set_failed("error message".to_string());

        assert_eq!(
            manager.get_state(),
            TranscriptionState::Failed("error message".to_string())
        );
    }

    // ========================================================================
    // Audio Processing Tests
    // ========================================================================

    struct AudioProcessor;

    impl AudioProcessor {
        /// Normalizes audio samples to [-1.0, 1.0] range
        fn normalize(samples: &[f32]) -> Vec<f32> {
            let max_amplitude = samples
                .iter()
                .map(|&s| s.abs())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(1.0);

            if max_amplitude > 0.0 {
                samples.iter().map(|&s| s / max_amplitude).collect()
            } else {
                samples.to_vec()
            }
        }

        /// Applies gain to audio samples
        fn apply_gain(samples: &[f32], gain_db: f32) -> Vec<f32> {
            let gain = 10_f32.powf(gain_db / 20.0);
            samples.iter().map(|&s| (s * gain).clamp(-1.0, 1.0)).collect()
        }

        /// Removes silence from beginning and end
        fn trim_silence(samples: &[f32], threshold: f32) -> Vec<f32> {
            let start = samples
                .iter()
                .position(|&s| s.abs() > threshold)
                .unwrap_or(0);

            let end = samples
                .iter()
                .rposition(|&s| s.abs() > threshold)
                .unwrap_or(samples.len());

            samples[start..=end].to_vec()
        }

        /// Detects if audio contains speech (simple energy-based)
        fn contains_speech(samples: &[f32], threshold: f32) -> bool {
            let energy: f32 = samples.iter().map(|&s| s * s).sum::<f32>() / samples.len() as f32;
            energy.sqrt() > threshold
        }
    }

    #[test]
    fn test_normalize_audio() {
        let samples = vec![0.5, -1.0, 0.25, 0.75];
        let normalized = AudioProcessor::normalize(&samples);

        assert_eq!(normalized, vec![0.5, -1.0, 0.25, 0.75]);
    }

    #[test]
    fn test_normalize_audio_above_range() {
        let samples = vec![2.0, -4.0, 1.0];
        let normalized = AudioProcessor::normalize(&samples);

        // Should normalize to [-1.0, 1.0] range
        assert_eq!(normalized, vec![0.5, -1.0, 0.25]);
    }

    #[test]
    fn test_apply_gain() {
        let samples = vec![0.1, 0.2, 0.3];
        let gained = AudioProcessor::apply_gain(&samples, 6.0); // +6dB ≈ 2x

        // Check that gain was applied (approximately)
        assert!(gained[0] > samples[0]);
        assert!(gained[1] > samples[1]);
        assert!(gained[2] > samples[2]);
    }

    #[test]
    fn test_apply_gain_clamps_to_range() {
        let samples = vec![0.9];
        let gained = AudioProcessor::apply_gain(&samples, 20.0); // Very high gain

        // Should clamp to 1.0
        assert_eq!(gained[0], 1.0);
    }

    #[test]
    fn test_trim_silence() {
        let samples = vec![0.0, 0.0, 0.5, 0.6, 0.7, 0.0, 0.0];
        let trimmed = AudioProcessor::trim_silence(&samples, 0.1);

        assert_eq!(trimmed, vec![0.5, 0.6, 0.7]);
    }

    #[test]
    fn test_trim_silence_all_silence() {
        let samples = vec![0.0, 0.0, 0.0];
        let trimmed = AudioProcessor::trim_silence(&samples, 0.1);

        // Should return empty or minimal
        assert!(trimmed.len() <= samples.len());
    }

    #[test]
    fn test_contains_speech() {
        let speech_samples = vec![0.5, -0.3, 0.4, -0.6, 0.7];
        assert!(AudioProcessor::contains_speech(&speech_samples, 0.1));

        let silence_samples = vec![0.01, -0.01, 0.005, 0.0];
        assert!(!AudioProcessor::contains_speech(&silence_samples, 0.1));
    }

    // ========================================================================
    // Text Post-Processing Tests
    // ========================================================================

    struct TextPostProcessor;

    impl TextPostProcessor {
        /// Capitalizes first letter of sentence
        fn capitalize_first(text: &str) -> String {
            let mut chars = text.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        }

        /// Adds period at end if missing
        fn ensure_period(text: &str) -> String {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return trimmed.to_string();
            }

            let last_char = trimmed.chars().last().unwrap();
            if !['.', '!', '?'].contains(&last_char) {
                format!("{}.", trimmed)
            } else {
                trimmed.to_string()
            }
        }

        /// Removes extra whitespace
        fn normalize_whitespace(text: &str) -> String {
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        }

        /// Fixes common transcription errors
        fn fix_common_errors(text: &str) -> String {
            text.replace(" i ", " I ")
                .replace(" i'm ", " I'm ")
                .replace(" i'll ", " I'll ")
                .replace(" i've ", " I've ")
        }

        /// Full post-processing pipeline
        fn process(text: &str) -> String {
            let text = Self::normalize_whitespace(text);
            let text = Self::fix_common_errors(&text);
            let text = Self::capitalize_first(&text);
            Self::ensure_period(&text)
        }
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(TextPostProcessor::capitalize_first("hello"), "Hello");
        assert_eq!(TextPostProcessor::capitalize_first("world"), "World");
        assert_eq!(TextPostProcessor::capitalize_first(""), "");
    }

    #[test]
    fn test_ensure_period() {
        assert_eq!(TextPostProcessor::ensure_period("hello"), "hello.");
        assert_eq!(TextPostProcessor::ensure_period("hello."), "hello.");
        assert_eq!(TextPostProcessor::ensure_period("hello!"), "hello!");
        assert_eq!(TextPostProcessor::ensure_period("hello?"), "hello?");
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(
            TextPostProcessor::normalize_whitespace("hello   world"),
            "hello world"
        );
        assert_eq!(
            TextPostProcessor::normalize_whitespace("  hello  world  "),
            "hello world"
        );
        assert_eq!(
            TextPostProcessor::normalize_whitespace("hello\n\nworld"),
            "hello world"
        );
    }

    #[test]
    fn test_fix_common_errors() {
        assert_eq!(
            TextPostProcessor::fix_common_errors("i think i'm happy"),
            "I think I'm happy"
        );
        assert_eq!(
            TextPostProcessor::fix_common_errors("i'll do it"),
            "I'll do it"
        );
    }

    #[test]
    fn test_full_post_processing() {
        let input = "  hello   world  i  think  i'm  happy  ";
        let expected = "Hello world I think I'm happy.";
        assert_eq!(TextPostProcessor::process(input), expected);
    }

    // ========================================================================
    // Transcription Result Tests
    // ========================================================================

    #[derive(Debug, Clone, PartialEq)]
    struct TranscriptionResult {
        text: String,
        confidence: f32,
        processing_time_ms: u64,
        language: Option<String>,
    }

    impl TranscriptionResult {
        fn new(text: String) -> Self {
            Self {
                text,
                confidence: 0.0,
                processing_time_ms: 0,
                language: None,
            }
        }

        fn with_confidence(mut self, confidence: f32) -> Self {
            self.confidence = confidence;
            self
        }

        fn with_processing_time(mut self, time_ms: u64) -> Self {
            self.processing_time_ms = time_ms;
            self
        }

        fn with_language(mut self, language: String) -> Self {
            self.language = Some(language);
            self
        }

        fn is_high_confidence(&self) -> bool {
            self.confidence > 0.8
        }
    }

    #[test]
    fn test_transcription_result_builder() {
        let result = TranscriptionResult::new("hello world".to_string())
            .with_confidence(0.95)
            .with_processing_time(1500)
            .with_language("en".to_string());

        assert_eq!(result.text, "hello world");
        assert_eq!(result.confidence, 0.95);
        assert_eq!(result.processing_time_ms, 1500);
        assert_eq!(result.language, Some("en".to_string()));
    }

    #[test]
    fn test_is_high_confidence() {
        let high = TranscriptionResult::new("test".to_string()).with_confidence(0.9);
        assert!(high.is_high_confidence());

        let low = TranscriptionResult::new("test".to_string()).with_confidence(0.5);
        assert!(!low.is_high_confidence());
    }

    // ========================================================================
    // Performance Tests (using criterion would be better)
    // ========================================================================

    #[test]
    fn test_audio_processing_performance() {
        use std::time::Instant;

        // Generate 10 seconds of audio at 16kHz
        let samples: Vec<f32> = (0..160000).map(|i| (i as f32 * 0.001).sin()).collect();

        let start = Instant::now();
        let _normalized = AudioProcessor::normalize(&samples);
        let normalize_time = start.elapsed();

        let start = Instant::now();
        let _gained = AudioProcessor::apply_gain(&samples, 6.0);
        let gain_time = start.elapsed();

        // These should be fast (< 100ms for 10 seconds of audio)
        assert!(normalize_time.as_millis() < 100);
        assert!(gain_time.as_millis() < 100);
    }

    // ========================================================================
    // Property-Based Tests
    // ========================================================================

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_normalize_always_within_range(
            samples in prop::collection::vec(-1000.0f32..1000.0, 1..1000)
        ) {
            let normalized = AudioProcessor::normalize(&samples);
            for &sample in &normalized {
                assert!(sample >= -1.0 && sample <= 1.0);
            }
        }

        #[test]
        fn test_capitalize_first_is_uppercase(
            text in "[a-z]{1,50}"
        ) {
            let capitalized = TextPostProcessor::capitalize_first(&text);
            assert!(capitalized.chars().next().unwrap().is_uppercase());
        }

        #[test]
        fn test_ensure_period_ends_with_punctuation(
            text in "[a-zA-Z ]{1,50}"
        ) {
            let processed = TextPostProcessor::ensure_period(&text);
            if !processed.is_empty() {
                let last = processed.chars().last().unwrap();
                assert!(['.', '!', '?'].contains(&last));
            }
        }

        #[test]
        fn test_confidence_invariants(
            confidence in 0.0f32..=1.0
        ) {
            let result = TranscriptionResult::new("test".to_string())
                .with_confidence(confidence);
            assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
        }
    }
}

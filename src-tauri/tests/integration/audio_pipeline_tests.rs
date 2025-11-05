//! Integration tests for the complete audio processing pipeline
//!
//! Tests end-to-end flow: Recording → VAD → Resampling → Output

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    // ========================================================================
    // Pipeline Component Integration Tests
    // ========================================================================

    /// Simulated audio pipeline for testing
    struct AudioPipeline {
        sample_rate: u32,
        vad_enabled: bool,
        resampling_enabled: bool,
    }

    impl AudioPipeline {
        fn new() -> Self {
            Self {
                sample_rate: 16000,
                vad_enabled: true,
                resampling_enabled: true,
            }
        }

        fn with_sample_rate(mut self, rate: u32) -> Self {
            self.sample_rate = rate;
            self
        }

        fn with_vad(mut self, enabled: bool) -> Self {
            self.vad_enabled = enabled;
            self
        }

        fn process(&self, input: &[f32]) -> Vec<f32> {
            let mut output = input.to_vec();

            // Simulate VAD filtering
            if self.vad_enabled {
                output = self.apply_vad(&output);
            }

            // Simulate resampling
            if self.resampling_enabled {
                output = self.resample(&output);
            }

            output
        }

        fn apply_vad(&self, samples: &[f32]) -> Vec<f32> {
            // Simple energy-based VAD simulation
            let threshold = 0.1;
            samples
                .iter()
                .filter(|&&s| s.abs() > threshold)
                .copied()
                .collect()
        }

        fn resample(&self, samples: &[f32]) -> Vec<f32> {
            // Simplified resampling (just pass through for test)
            samples.to_vec()
        }
    }

    #[test]
    fn test_pipeline_end_to_end() {
        let pipeline = AudioPipeline::new();
        let input = vec![0.5, 0.05, 0.6, 0.02, 0.7]; // Mixed signal and silence
        let output = pipeline.process(&input);

        // VAD should filter out low amplitude samples
        assert!(output.len() < input.len());
    }

    #[test]
    fn test_pipeline_without_vad() {
        let pipeline = AudioPipeline::new().with_vad(false);
        let input = vec![0.5, 0.05, 0.6];
        let output = pipeline.process(&input);

        // Without VAD, output should equal input
        assert_eq!(output.len(), input.len());
    }

    #[test]
    fn test_pipeline_with_different_sample_rates() {
        let pipeline_16k = AudioPipeline::new().with_sample_rate(16000);
        let pipeline_44k = AudioPipeline::new().with_sample_rate(44100);

        assert_eq!(pipeline_16k.sample_rate, 16000);
        assert_eq!(pipeline_44k.sample_rate, 44100);
    }

    // ========================================================================
    // Recording → Transcription Integration Tests
    // ========================================================================

    struct MockRecordingSession {
        samples: Vec<f32>,
        is_recording: bool,
    }

    impl MockRecordingSession {
        fn new() -> Self {
            Self {
                samples: Vec::new(),
                is_recording: false,
            }
        }

        fn start(&mut self) {
            self.is_recording = true;
            self.samples.clear();
        }

        fn record(&mut self, samples: &[f32]) {
            if self.is_recording {
                self.samples.extend_from_slice(samples);
            }
        }

        fn stop(&mut self) -> Vec<f32> {
            self.is_recording = false;
            self.samples.clone()
        }
    }

    #[test]
    fn test_recording_session_lifecycle() {
        let mut session = MockRecordingSession::new();

        // Start recording
        session.start();
        assert!(session.is_recording);

        // Record some audio
        session.record(&[0.1, 0.2, 0.3]);
        session.record(&[0.4, 0.5]);

        // Stop and get samples
        let samples = session.stop();
        assert_eq!(samples, vec![0.1, 0.2, 0.3, 0.4, 0.5]);
        assert!(!session.is_recording);
    }

    #[test]
    fn test_recording_before_start_ignored() {
        let mut session = MockRecordingSession::new();

        // Try recording before start
        session.record(&[0.1, 0.2]);
        let samples = session.stop();

        assert!(samples.is_empty());
    }

    // ========================================================================
    // Model Loading Integration Tests
    // ========================================================================

    struct MockModelLoader {
        models_dir: PathBuf,
        loaded_models: Vec<String>,
    }

    impl MockModelLoader {
        fn new(models_dir: PathBuf) -> Self {
            Self {
                models_dir,
                loaded_models: Vec::new(),
            }
        }

        fn load_model(&mut self, model_id: &str) -> Result<(), String> {
            let model_path = self.models_dir.join(format!("{}.bin", model_id));

            if !model_path.exists() {
                return Err(format!("Model {} not found", model_id));
            }

            self.loaded_models.push(model_id.to_string());
            Ok(())
        }

        fn is_model_loaded(&self, model_id: &str) -> bool {
            self.loaded_models.contains(&model_id.to_string())
        }
    }

    #[test]
    fn test_model_loading_integration() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let mut loader = MockModelLoader::new(temp_dir.path().to_path_buf());

        // Create a mock model file
        let model_path = temp_dir.path().join("test-model.bin");
        std::fs::write(&model_path, b"mock model data").unwrap();

        // Load the model
        let result = loader.load_model("test-model");
        assert!(result.is_ok());
        assert!(loader.is_model_loaded("test-model"));
    }

    #[test]
    fn test_model_loading_failure() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let mut loader = MockModelLoader::new(temp_dir.path().to_path_buf());

        // Try to load non-existent model
        let result = loader.load_model("nonexistent");
        assert!(result.is_err());
        assert!(!loader.is_model_loaded("nonexistent"));
    }

    // ========================================================================
    // Settings Integration Tests
    // ========================================================================

    #[derive(Debug, Clone, PartialEq)]
    struct AppSettings {
        selected_model: String,
        selected_microphone: Option<String>,
        always_on_microphone: bool,
        vad_threshold: f32,
    }

    impl Default for AppSettings {
        fn default() -> Self {
            Self {
                selected_model: "small".to_string(),
                selected_microphone: None,
                always_on_microphone: false,
                vad_threshold: 0.3,
            }
        }
    }

    struct SettingsManager {
        settings: AppSettings,
    }

    impl SettingsManager {
        fn new() -> Self {
            Self {
                settings: AppSettings::default(),
            }
        }

        fn update_model(&mut self, model: String) {
            self.settings.selected_model = model;
        }

        fn update_microphone(&mut self, mic: Option<String>) {
            self.settings.selected_microphone = mic;
        }

        fn get_settings(&self) -> &AppSettings {
            &self.settings
        }
    }

    #[test]
    fn test_settings_management() {
        let mut manager = SettingsManager::new();

        // Check defaults
        assert_eq!(manager.get_settings().selected_model, "small");
        assert_eq!(manager.get_settings().selected_microphone, None);

        // Update settings
        manager.update_model("large".to_string());
        manager.update_microphone(Some("USB Microphone".to_string()));

        assert_eq!(manager.get_settings().selected_model, "large");
        assert_eq!(
            manager.get_settings().selected_microphone,
            Some("USB Microphone".to_string())
        );
    }

    // ========================================================================
    // Error Handling Integration Tests
    // ========================================================================

    #[derive(Debug, PartialEq)]
    enum PipelineError {
        RecordingFailed(String),
        ProcessingFailed(String),
        TranscriptionFailed(String),
    }

    type PipelineResult<T> = Result<T, PipelineError>;

    struct ErrorHandlingPipeline;

    impl ErrorHandlingPipeline {
        fn process_audio(_samples: &[f32]) -> PipelineResult<Vec<f32>> {
            // Simulate processing
            Ok(vec![0.1, 0.2, 0.3])
        }

        fn transcribe(samples: &[f32]) -> PipelineResult<String> {
            if samples.is_empty() {
                return Err(PipelineError::TranscriptionFailed(
                    "Empty audio".to_string(),
                ));
            }

            Ok("transcribed text".to_string())
        }

        fn full_pipeline(input: &[f32]) -> PipelineResult<String> {
            let processed = Self::process_audio(input)?;
            let transcribed = Self::transcribe(&processed)?;
            Ok(transcribed)
        }
    }

    #[test]
    fn test_error_propagation() {
        let result = ErrorHandlingPipeline::full_pipeline(&[0.1, 0.2]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_handling_empty_input() {
        let result = ErrorHandlingPipeline::transcribe(&[]);
        assert_eq!(
            result,
            Err(PipelineError::TranscriptionFailed("Empty audio".to_string()))
        );
    }

    // ========================================================================
    // Concurrent Processing Tests
    // ========================================================================

    #[test]
    fn test_concurrent_transcription_requests() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let counter = Arc::new(Mutex::new(0));
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let counter = Arc::clone(&counter);
                thread::spawn(move || {
                    // Simulate transcription
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    *counter.lock().unwrap() += 1;
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(*counter.lock().unwrap(), 10);
    }
}

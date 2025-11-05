//! Test utilities for Handy test suite
//!
//! Provides common test helpers, mocks, and fixtures

use std::path::PathBuf;
use tempfile::TempDir;

/// Creates a temporary directory for test data
///
/// # Examples
/// ```
/// let test_dir = create_test_dir();
/// ```
pub fn create_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Creates a test configuration directory with sample files
pub fn create_test_config_dir() -> (TempDir, PathBuf) {
    let temp_dir = create_test_dir();
    let config_path = temp_dir.path().join("config");
    std::fs::create_dir(&config_path).expect("Failed to create config dir");
    (temp_dir, config_path)
}

/// Mock audio samples for testing
pub mod mock_audio {
    /// Generates a sine wave audio sample
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    /// * `duration_secs` - Duration in seconds
    /// * `frequency` - Frequency in Hz
    pub fn generate_sine_wave(sample_rate: u32, duration_secs: f32, frequency: f32) -> Vec<f32> {
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect()
    }

    /// Generates silence (zeros) for testing
    pub fn generate_silence(sample_rate: u32, duration_secs: f32) -> Vec<f32> {
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        vec![0.0; num_samples]
    }

    /// Generates white noise for testing
    pub fn generate_noise(sample_rate: u32, duration_secs: f32) -> Vec<f32> {
        use rand::Rng;
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let mut rng = rand::thread_rng();
        (0..num_samples)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect()
    }
}

/// Mock models for testing
pub mod mock_models {
    use std::path::PathBuf;

    /// Creates a mock model file structure
    pub fn create_mock_model(base_path: &PathBuf, model_name: &str) -> PathBuf {
        let model_dir = base_path.join(model_name);
        std::fs::create_dir_all(&model_dir).expect("Failed to create mock model dir");

        // Create a dummy model file
        let model_file = model_dir.join("model.bin");
        std::fs::write(&model_file, b"mock model data").expect("Failed to write mock model");

        model_file
    }
}

/// Test assertions for audio data
pub mod audio_assertions {
    /// Asserts that audio samples are within valid range [-1.0, 1.0]
    pub fn assert_samples_in_range(samples: &[f32]) {
        for (i, &sample) in samples.iter().enumerate() {
            assert!(
                sample >= -1.0 && sample <= 1.0,
                "Sample at index {} out of range: {}",
                i,
                sample
            );
        }
    }

    /// Asserts that audio samples contain non-silence
    pub fn assert_contains_audio(samples: &[f32], threshold: f32) {
        let max_amplitude = samples
            .iter()
            .map(|&s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        assert!(
            max_amplitude > threshold,
            "Audio appears to be silence. Max amplitude: {}",
            max_amplitude
        );
    }

    /// Asserts that audio is approximately silence
    pub fn assert_is_silence(samples: &[f32], threshold: f32) {
        let max_amplitude = samples
            .iter()
            .map(|&s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        assert!(
            max_amplitude < threshold,
            "Audio is not silence. Max amplitude: {}",
            max_amplitude
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_dir() {
        let temp_dir = create_test_dir();
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_generate_sine_wave() {
        let samples = mock_audio::generate_sine_wave(44100, 1.0, 440.0);
        assert_eq!(samples.len(), 44100);
        audio_assertions::assert_samples_in_range(&samples);
        audio_assertions::assert_contains_audio(&samples, 0.1);
    }

    #[test]
    fn test_generate_silence() {
        let samples = mock_audio::generate_silence(44100, 1.0);
        assert_eq!(samples.len(), 44100);
        audio_assertions::assert_is_silence(&samples, 0.01);
    }
}

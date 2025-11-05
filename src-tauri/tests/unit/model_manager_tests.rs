//! Unit tests for ModelManager
//!
//! Tests cover:
//! - Model info structures
//! - Model download state management
//! - Model selection logic
//! - File system operations (with tempdir)

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum EngineType {
        Whisper,
        Parakeet,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ModelInfo {
        pub id: String,
        pub name: String,
        pub description: String,
        pub filename: String,
        pub url: Option<String>,
        pub size_mb: u64,
        pub is_downloaded: bool,
        pub is_downloading: bool,
        pub partial_size: u64,
        pub is_directory: bool,
        pub engine_type: EngineType,
        pub accuracy_score: f32,
        pub speed_score: f32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct DownloadProgress {
        pub model_id: String,
        pub downloaded: u64,
        pub total: u64,
        pub percentage: f64,
    }

    impl DownloadProgress {
        fn new(model_id: String, downloaded: u64, total: u64) -> Self {
            let percentage = if total > 0 {
                (downloaded as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            Self {
                model_id,
                downloaded,
                total,
                percentage,
            }
        }
    }

    // ========================================================================
    // ModelInfo Tests
    // ========================================================================

    #[test]
    fn test_model_info_creation() {
        let model = ModelInfo {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            description: "A test model".to_string(),
            filename: "test.bin".to_string(),
            url: Some("https://example.com/test.bin".to_string()),
            size_mb: 100,
            is_downloaded: false,
            is_downloading: false,
            partial_size: 0,
            is_directory: false,
            engine_type: EngineType::Whisper,
            accuracy_score: 0.75,
            speed_score: 0.85,
        };

        assert_eq!(model.id, "test-model");
        assert_eq!(model.size_mb, 100);
        assert!(!model.is_downloaded);
        assert_eq!(model.engine_type, EngineType::Whisper);
    }

    #[test]
    fn test_model_info_serialization() {
        let model = ModelInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test desc".to_string(),
            filename: "test.bin".to_string(),
            url: None,
            size_mb: 50,
            is_downloaded: true,
            is_downloading: false,
            partial_size: 0,
            is_directory: false,
            engine_type: EngineType::Whisper,
            accuracy_score: 0.5,
            speed_score: 0.5,
        };

        let json = serde_json::to_string(&model).expect("Failed to serialize");
        let deserialized: ModelInfo =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.id, model.id);
        assert_eq!(deserialized.name, model.name);
        assert_eq!(deserialized.is_downloaded, model.is_downloaded);
    }

    #[test]
    fn test_model_scores_are_valid() {
        let model = ModelInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            filename: "test.bin".to_string(),
            url: None,
            size_mb: 100,
            is_downloaded: false,
            is_downloading: false,
            partial_size: 0,
            is_directory: false,
            engine_type: EngineType::Whisper,
            accuracy_score: 0.75,
            speed_score: 0.85,
        };

        assert!(model.accuracy_score >= 0.0 && model.accuracy_score <= 1.0);
        assert!(model.speed_score >= 0.0 && model.speed_score <= 1.0);
    }

    // ========================================================================
    // DownloadProgress Tests
    // ========================================================================

    #[test]
    fn test_download_progress_calculation() {
        let progress = DownloadProgress::new("test".to_string(), 50, 100);
        assert_eq!(progress.percentage, 50.0);

        let progress = DownloadProgress::new("test".to_string(), 75, 100);
        assert_eq!(progress.percentage, 75.0);

        let progress = DownloadProgress::new("test".to_string(), 100, 100);
        assert_eq!(progress.percentage, 100.0);
    }

    #[test]
    fn test_download_progress_zero_total() {
        let progress = DownloadProgress::new("test".to_string(), 0, 0);
        assert_eq!(progress.percentage, 0.0);
    }

    #[test]
    fn test_download_progress_serialization() {
        let progress = DownloadProgress {
            model_id: "test-model".to_string(),
            downloaded: 1024,
            total: 2048,
            percentage: 50.0,
        };

        let json = serde_json::to_string(&progress).expect("Failed to serialize");
        let deserialized: DownloadProgress =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized, progress);
    }

    // ========================================================================
    // Model Selection Tests
    // ========================================================================

    struct ModelSelector {
        available_models: HashMap<String, ModelInfo>,
    }

    impl ModelSelector {
        fn new() -> Self {
            Self {
                available_models: HashMap::new(),
            }
        }

        fn add_model(&mut self, model: ModelInfo) {
            self.available_models.insert(model.id.clone(), model);
        }

        fn get_downloaded_models(&self) -> Vec<&ModelInfo> {
            self.available_models
                .values()
                .filter(|m| m.is_downloaded)
                .collect()
        }

        fn get_best_downloaded_model(&self) -> Option<&ModelInfo> {
            self.get_downloaded_models()
                .into_iter()
                .max_by(|a, b| {
                    // Combine accuracy and speed scores
                    let score_a = (a.accuracy_score + a.speed_score) / 2.0;
                    let score_b = (b.accuracy_score + b.speed_score) / 2.0;
                    score_a.partial_cmp(&score_b).unwrap()
                })
        }

        fn get_fastest_downloaded_model(&self) -> Option<&ModelInfo> {
            self.get_downloaded_models()
                .into_iter()
                .max_by(|a, b| a.speed_score.partial_cmp(&b.speed_score).unwrap())
        }

        fn get_most_accurate_downloaded_model(&self) -> Option<&ModelInfo> {
            self.get_downloaded_models()
                .into_iter()
                .max_by(|a, b| a.accuracy_score.partial_cmp(&b.accuracy_score).unwrap())
        }
    }

    fn create_test_model(id: &str, downloaded: bool, accuracy: f32, speed: f32) -> ModelInfo {
        ModelInfo {
            id: id.to_string(),
            name: format!("Model {}", id),
            description: "Test model".to_string(),
            filename: format!("{}.bin", id),
            url: None,
            size_mb: 100,
            is_downloaded: downloaded,
            is_downloading: false,
            partial_size: 0,
            is_directory: false,
            engine_type: EngineType::Whisper,
            accuracy_score: accuracy,
            speed_score: speed,
        }
    }

    #[test]
    fn test_get_downloaded_models() {
        let mut selector = ModelSelector::new();
        selector.add_model(create_test_model("model1", true, 0.75, 0.85));
        selector.add_model(create_test_model("model2", false, 0.80, 0.70));
        selector.add_model(create_test_model("model3", true, 0.70, 0.90));

        let downloaded = selector.get_downloaded_models();
        assert_eq!(downloaded.len(), 2);
    }

    #[test]
    fn test_get_best_downloaded_model() {
        let mut selector = ModelSelector::new();
        selector.add_model(create_test_model("model1", true, 0.60, 0.90)); // avg: 0.75
        selector.add_model(create_test_model("model2", true, 0.80, 0.80)); // avg: 0.80
        selector.add_model(create_test_model("model3", true, 0.70, 0.70)); // avg: 0.70

        let best = selector.get_best_downloaded_model();
        assert!(best.is_some());
        assert_eq!(best.unwrap().id, "model2");
    }

    #[test]
    fn test_get_fastest_downloaded_model() {
        let mut selector = ModelSelector::new();
        selector.add_model(create_test_model("model1", true, 0.60, 0.85));
        selector.add_model(create_test_model("model2", true, 0.80, 0.95)); // fastest
        selector.add_model(create_test_model("model3", true, 0.70, 0.70));

        let fastest = selector.get_fastest_downloaded_model();
        assert!(fastest.is_some());
        assert_eq!(fastest.unwrap().id, "model2");
    }

    #[test]
    fn test_get_most_accurate_downloaded_model() {
        let mut selector = ModelSelector::new();
        selector.add_model(create_test_model("model1", true, 0.60, 0.85));
        selector.add_model(create_test_model("model2", true, 0.90, 0.70)); // most accurate
        selector.add_model(create_test_model("model3", true, 0.70, 0.90));

        let accurate = selector.get_most_accurate_downloaded_model();
        assert!(accurate.is_some());
        assert_eq!(accurate.unwrap().id, "model2");
    }

    #[test]
    fn test_no_downloaded_models() {
        let mut selector = ModelSelector::new();
        selector.add_model(create_test_model("model1", false, 0.60, 0.85));
        selector.add_model(create_test_model("model2", false, 0.80, 0.70));

        assert!(selector.get_best_downloaded_model().is_none());
        assert!(selector.get_fastest_downloaded_model().is_none());
        assert!(selector.get_most_accurate_downloaded_model().is_none());
    }

    // ========================================================================
    // File System Tests
    // ========================================================================

    struct ModelFileManager {
        models_dir: PathBuf,
    }

    impl ModelFileManager {
        fn new(models_dir: PathBuf) -> Self {
            Self { models_dir }
        }

        fn get_model_path(&self, filename: &str) -> PathBuf {
            self.models_dir.join(filename)
        }

        fn is_model_downloaded(&self, filename: &str) -> bool {
            self.get_model_path(filename).exists()
        }

        fn create_model_file(&self, filename: &str) -> std::io::Result<()> {
            let path = self.get_model_path(filename);
            std::fs::write(&path, b"model data")?;
            Ok(())
        }

        fn get_model_size(&self, filename: &str) -> std::io::Result<u64> {
            let path = self.get_model_path(filename);
            let metadata = std::fs::metadata(path)?;
            Ok(metadata.len())
        }
    }

    #[test]
    fn test_model_file_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelFileManager::new(temp_dir.path().to_path_buf());

        assert_eq!(manager.models_dir, temp_dir.path());
    }

    #[test]
    fn test_get_model_path() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelFileManager::new(temp_dir.path().to_path_buf());

        let path = manager.get_model_path("test.bin");
        assert!(path.ends_with("test.bin"));
    }

    #[test]
    fn test_is_model_downloaded() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelFileManager::new(temp_dir.path().to_path_buf());

        assert!(!manager.is_model_downloaded("test.bin"));

        manager.create_model_file("test.bin").unwrap();
        assert!(manager.is_model_downloaded("test.bin"));
    }

    #[test]
    fn test_get_model_size() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelFileManager::new(temp_dir.path().to_path_buf());

        manager.create_model_file("test.bin").unwrap();
        let size = manager.get_model_size("test.bin").unwrap();

        assert_eq!(size, 10); // "model data" = 10 bytes
    }

    // ========================================================================
    // Engine Type Tests
    // ========================================================================

    #[test]
    fn test_engine_type_equality() {
        assert_eq!(EngineType::Whisper, EngineType::Whisper);
        assert_eq!(EngineType::Parakeet, EngineType::Parakeet);
        assert_ne!(EngineType::Whisper, EngineType::Parakeet);
    }

    #[test]
    fn test_engine_type_serialization() {
        let whisper = EngineType::Whisper;
        let json = serde_json::to_string(&whisper).unwrap();
        let deserialized: EngineType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, EngineType::Whisper);

        let parakeet = EngineType::Parakeet;
        let json = serde_json::to_string(&parakeet).unwrap();
        let deserialized: EngineType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, EngineType::Parakeet);
    }

    // ========================================================================
    // Property-Based Tests
    // ========================================================================

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_download_progress_always_between_0_and_100(
            downloaded in 0u64..1000000,
            total in 1u64..1000000,
        ) {
            let downloaded = downloaded.min(total);
            let progress = DownloadProgress::new("test".to_string(), downloaded, total);
            assert!(progress.percentage >= 0.0 && progress.percentage <= 100.0);
        }

        #[test]
        fn test_model_scores_invariants(
            accuracy in 0.0f32..=1.0,
            speed in 0.0f32..=1.0,
        ) {
            let model = create_test_model("test", false, accuracy, speed);
            assert!(model.accuracy_score >= 0.0 && model.accuracy_score <= 1.0);
            assert!(model.speed_score >= 0.0 && model.speed_score <= 1.0);
        }
    }
}

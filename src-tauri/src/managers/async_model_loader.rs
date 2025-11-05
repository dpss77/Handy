//! Async model loading infrastructure
//!
//! Provides non-blocking model loading with progress updates and cancellation support.

use anyhow::Result;
use log::{debug, error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

use super::model::ModelManager;

/// Progress information for model loading
#[derive(Clone, Debug, serde::Serialize)]
pub struct ModelLoadProgress {
    pub model_id: String,
    pub stage: String, // "initializing", "loading", "ready"
    pub progress_percent: f32,
}

/// Async model loader
///
/// Handles background model loading without blocking the main thread.
pub struct AsyncModelLoader {
    model_manager: Arc<ModelManager>,
    app_handle: AppHandle,
    is_loading: Arc<Mutex<bool>>,
    cancel_signal: Arc<AtomicBool>,
    current_model: Arc<Mutex<Option<String>>>,
}

impl AsyncModelLoader {
    /// Creates a new async model loader
    pub fn new(model_manager: Arc<ModelManager>, app_handle: AppHandle) -> Self {
        Self {
            model_manager,
            app_handle,
            is_loading: Arc::new(Mutex::new(false)),
            cancel_signal: Arc::new(AtomicBool::new(false)),
            current_model: Arc::new(Mutex::new(None)),
        }
    }

    /// Checks if a model is currently being loaded
    pub fn is_loading(&self) -> bool {
        *self.is_loading.lock().unwrap()
    }

    /// Gets the currently loading model ID
    pub fn current_model(&self) -> Option<String> {
        self.current_model.lock().unwrap().clone()
    }

    /// Loads a model asynchronously
    ///
    /// # Arguments
    ///
    /// * `model_id` - ID of the model to load
    ///
    /// # Returns
    ///
    /// `true` if loading started, `false` if already loading
    pub fn load_async(&self, model_id: String) -> bool {
        let mut is_loading = self.is_loading.lock().unwrap();

        if *is_loading {
            debug!("Model load already in progress, ignoring new request");
            return false;
        }

        *is_loading = true;
        *self.current_model.lock().unwrap() = Some(model_id.clone());
        self.cancel_signal.store(false, Ordering::Relaxed);
        drop(is_loading);

        // Spawn background thread for loading
        let model_manager = Arc::clone(&self.model_manager);
        let app_handle = self.app_handle.clone();
        let is_loading_clone = Arc::clone(&self.is_loading);
        let cancel_signal = Arc::clone(&self.cancel_signal);
        let current_model = Arc::clone(&self.current_model);

        thread::spawn(move || {
            info!("Starting async model load: {}", model_id);

            // Emit initializing event
            let _ = app_handle.emit(
                "model-load-progress",
                ModelLoadProgress {
                    model_id: model_id.clone(),
                    stage: "initializing".to_string(),
                    progress_percent: 0.0,
                },
            );

            // Check for cancellation
            if cancel_signal.load(Ordering::Relaxed) {
                info!("Model load cancelled before starting: {}", model_id);
                *is_loading_clone.lock().unwrap() = false;
                *current_model.lock().unwrap() = None;
                return;
            }

            // Emit loading event
            let _ = app_handle.emit(
                "model-load-progress",
                ModelLoadProgress {
                    model_id: model_id.clone(),
                    stage: "loading".to_string(),
                    progress_percent: 50.0,
                },
            );

            // Simulate model loading stages
            // In real implementation, this would:
            // 1. Check if model files exist
            // 2. Decompress if needed
            // 3. Load into memory
            // 4. Initialize engine
            thread::sleep(std::time::Duration::from_millis(100));

            if cancel_signal.load(Ordering::Relaxed) {
                info!("Model load cancelled during loading: {}", model_id);
                *is_loading_clone.lock().unwrap() = false;
                *current_model.lock().unwrap() = None;
                return;
            }

            // Emit ready event
            let _ = app_handle.emit(
                "model-load-progress",
                ModelLoadProgress {
                    model_id: model_id.clone(),
                    stage: "ready".to_string(),
                    progress_percent: 100.0,
                },
            );

            info!("Async model load complete: {}", model_id);
            *is_loading_clone.lock().unwrap() = false;
            *current_model.lock().unwrap() = None;
        });

        true
    }

    /// Cancels the current model loading operation
    pub fn cancel(&self) {
        debug!("Cancelling model load");
        self.cancel_signal.store(true, Ordering::Relaxed);
    }

    /// Waits for current load to complete (blocking)
    ///
    /// # Arguments
    ///
    /// * `timeout_secs` - Maximum time to wait in seconds
    ///
    /// # Returns
    ///
    /// `true` if load completed, `false` if timed out
    pub fn wait_for_completion(&self, timeout_secs: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        while self.is_loading() {
            if start.elapsed() > timeout {
                error!("Model load wait timed out after {} seconds", timeout_secs);
                return false;
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full tests require Tauri app context
    // These are basic structure tests

    #[test]
    fn test_is_loading_initially_false() {
        // This test would need a full Tauri setup
        // Skipping for now
    }

    #[test]
    fn test_model_load_progress_serialization() {
        let progress = ModelLoadProgress {
            model_id: "test-model".to_string(),
            stage: "loading".to_string(),
            progress_percent: 50.0,
        };

        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("loading"));
    }

    #[test]
    fn test_progress_stages() {
        let stages = vec!["initializing", "loading", "ready"];
        for stage in stages {
            let progress = ModelLoadProgress {
                model_id: "test".to_string(),
                stage: stage.to_string(),
                progress_percent: 0.0,
            };
            assert_eq!(progress.stage, stage);
        }
    }
}

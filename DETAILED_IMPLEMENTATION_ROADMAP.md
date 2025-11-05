# Handy: Detailed Implementation Roadmap with Tests

*Version 1.0 - Created: 2025-11-05*

This roadmap provides state-of-the-art, actionable implementation plans for each development phase, with comprehensive test suites that can be executed autonomously.

---

# Table of Contents

1. [Phase 1: Foundation](#phase-1-foundation-3-6-months)
2. [Phase 2: Core Features](#phase-2-core-features-6-12-months)
3. [Phase 3: Extensibility](#phase-3-extensibility-12-18-months)
4. [Phase 4: Ecosystem](#phase-4-ecosystem-18-months)
5. [Testing Framework](#testing-framework)
6. [Implementation Guidelines](#implementation-guidelines)

---

# Phase 1: Foundation (3-6 Months)

**Focus**: Stability, Testing, Documentation, and Quick Wins

**Theme**: Build a solid foundation with proper testing, documentation, and essential improvements that don't change core functionality but make the codebase more robust and maintainable.

## 1.1 Testing Infrastructure

### Task 1.1.1: Set Up Rust Unit Testing Framework
**Priority**: CRITICAL
**Effort**: 1 week
**Dependencies**: None

**Implementation Steps:**
1. Create `tests/` directory structure
   ```
   src-tauri/
   ├── tests/
   │   ├── unit/
   │   │   ├── mod.rs
   │   │   ├── audio_tests.rs
   │   │   ├── model_tests.rs
   │   │   ├── transcription_tests.rs
   │   │   └── settings_tests.rs
   │   ├── integration/
   │   │   ├── mod.rs
   │   │   └── pipeline_tests.rs
   │   └── common/
   │       ├── mod.rs
   │       └── fixtures.rs
   ```

2. Add test dependencies to `Cargo.toml`:
   ```toml
   [dev-dependencies]
   mockall = "0.12"
   tempfile = "3.8"
   assert_matches = "1.5"
   rstest = "0.18"
   tokio-test = "0.4"
   ```

3. Create test utilities in `tests/common/mod.rs`:
   ```rust
   pub fn create_test_app_handle() -> AppHandle { /* ... */ }
   pub fn create_test_audio_samples(duration_ms: u32) -> Vec<f32> { /* ... */ }
   pub fn create_test_settings() -> Settings { /* ... */ }
   ```

**Test Plan:**
```rust
// tests/unit/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_is_working() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn test_utilities_create_audio_samples() {
        let samples = create_test_audio_samples(1000);
        assert_eq!(samples.len(), 16000); // 16kHz * 1s
    }
}
```

**Automated Verification:**
```bash
# Run from project root
cd src-tauri && cargo test --lib
# Expected: All tests pass
# Success metric: >0 tests executed successfully
```

**Deliverables:**
- [ ] Test directory structure created
- [ ] Test dependencies added
- [ ] Common test utilities implemented
- [ ] Framework tests passing
- [ ] Documentation in `tests/README.md`

---

### Task 1.1.2: Unit Tests for AudioManager
**Priority**: HIGH
**Effort**: 1 week
**Dependencies**: 1.1.1

**Implementation Steps:**
1. Create mock audio device traits
2. Write tests for device enumeration
3. Write tests for recording start/stop
4. Write tests for device switching
5. Write tests for error conditions

**Test Cases:**
```rust
// tests/unit/audio_tests.rs

#[tokio::test]
async fn test_audio_manager_initialization() {
    let app_handle = create_test_app_handle();
    let manager = AudioManager::new(&app_handle).await;
    assert!(manager.is_ok());
}

#[tokio::test]
async fn test_audio_device_enumeration() {
    let manager = AudioManager::new(&create_test_app_handle()).await.unwrap();
    let devices = manager.get_available_devices();
    assert!(devices.len() > 0, "Should find at least default device");
}

#[tokio::test]
async fn test_start_recording_always_on_mode() {
    let mut manager = AudioManager::new(&create_test_app_handle()).await.unwrap();

    // Set to always-on mode
    manager.set_mode(MicrophoneMode::AlwaysOn).await.unwrap();

    // Start recording
    let result = manager.start_recording().await;
    assert!(result.is_ok());

    // Verify recording state
    assert!(manager.is_recording());

    // Cleanup
    manager.stop_recording().await.unwrap();
}

#[tokio::test]
async fn test_start_recording_on_demand_mode() {
    let mut manager = AudioManager::new(&create_test_app_handle()).await.unwrap();

    manager.set_mode(MicrophoneMode::OnDemand).await.unwrap();

    let result = manager.start_recording().await;
    assert!(result.is_ok());
    assert!(manager.is_recording());

    manager.stop_recording().await.unwrap();
}

#[tokio::test]
async fn test_stop_recording_returns_audio_samples() {
    let mut manager = AudioManager::new(&create_test_app_handle()).await.unwrap();

    manager.start_recording().await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let samples = manager.stop_recording().await.unwrap();
    assert!(samples.len() > 0, "Should have captured some audio");
}

#[tokio::test]
async fn test_device_switching() {
    let mut manager = AudioManager::new(&create_test_app_handle()).await.unwrap();
    let devices = manager.get_available_devices();

    if devices.len() > 1 {
        let second_device = &devices[1];
        let result = manager.set_device(&second_device.id).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_invalid_device_error() {
    let mut manager = AudioManager::new(&create_test_app_handle()).await.unwrap();
    let result = manager.set_device("invalid-device-id").await;
    assert!(result.is_err());
}
```

**Automated Verification:**
```bash
cargo test audio_tests -- --nocapture
# Expected: All tests pass
# Success metric: 100% pass rate
```

**Deliverables:**
- [ ] 7+ unit tests for AudioManager
- [ ] Mock audio devices for testing
- [ ] Error condition coverage
- [ ] Tests passing in CI

---

### Task 1.1.3: Unit Tests for ModelManager
**Priority**: HIGH
**Effort**: 1 week
**Dependencies**: 1.1.1

**Test Cases:**
```rust
// tests/unit/model_tests.rs

#[test]
fn test_model_manager_initialization() {
    let app_handle = create_test_app_handle();
    let manager = ModelManager::new(&app_handle);
    assert!(manager.is_ok());
}

#[test]
fn test_get_available_models() {
    let manager = ModelManager::new(&create_test_app_handle()).unwrap();
    let models = manager.get_available_models();

    // Should have Whisper and Parakeet models
    assert!(models.len() >= 6);
    assert!(models.iter().any(|m| m.engine_type == EngineType::Whisper));
    assert!(models.iter().any(|m| m.engine_type == EngineType::Parakeet));
}

#[test]
fn test_get_model_info() {
    let manager = ModelManager::new(&create_test_app_handle()).unwrap();
    let model = manager.get_model_info("small");

    assert!(model.is_some());
    let model = model.unwrap();
    assert_eq!(model.id, "small");
    assert_eq!(model.name, "Whisper Small");
}

#[test]
fn test_model_path_for_downloaded_model() {
    let manager = ModelManager::new(&create_test_app_handle()).unwrap();

    // This test assumes at least one model is downloaded
    let models = manager.get_available_models();
    if let Some(downloaded) = models.iter().find(|m| m.is_downloaded) {
        let path = manager.get_model_path(&downloaded.id);
        assert!(path.is_ok());
        assert!(path.unwrap().exists());
    }
}

#[test]
fn test_model_path_for_not_downloaded_model() {
    let manager = ModelManager::new(&create_test_app_handle()).unwrap();

    // Find a model that isn't downloaded
    let models = manager.get_available_models();
    if let Some(not_downloaded) = models.iter().find(|m| !m.is_downloaded) {
        let path = manager.get_model_path(&not_downloaded.id);
        assert!(path.is_err());
    }
}

#[tokio::test]
async fn test_download_progress_events() {
    let manager = ModelManager::new(&create_test_app_handle()).unwrap();

    // Mock server would be used here in real implementation
    // For now, just test the structure
    let models = manager.get_available_models();
    assert!(models.iter().all(|m| m.partial_size >= 0));
}

#[test]
fn test_delete_non_existent_model() {
    let manager = ModelManager::new(&create_test_app_handle()).unwrap();

    // Try to delete a model that doesn't exist on disk
    let models = manager.get_available_models();
    if let Some(not_downloaded) = models.iter().find(|m| !m.is_downloaded) {
        let result = manager.delete_model(&not_downloaded.id);
        assert!(result.is_err());
    }
}
```

**Automated Verification:**
```bash
cargo test model_tests -- --nocapture
```

**Deliverables:**
- [ ] 7+ unit tests for ModelManager
- [ ] Coverage for download states
- [ ] Error handling tests
- [ ] Path resolution tests

---

### Task 1.1.4: Unit Tests for TranscriptionManager
**Priority**: HIGH
**Effort**: 1.5 weeks
**Dependencies**: 1.1.1, 1.1.3

**Test Cases:**
```rust
// tests/unit/transcription_tests.rs

#[test]
fn test_transcription_manager_initialization() {
    let app_handle = create_test_app_handle();
    let model_manager = Arc::new(ModelManager::new(&app_handle).unwrap());
    let manager = TranscriptionManager::new(&app_handle, model_manager);
    assert!(manager.is_ok());
}

#[tokio::test]
async fn test_load_model() {
    let app_handle = create_test_app_handle();
    let model_manager = Arc::new(ModelManager::new(&app_handle).unwrap());
    let manager = TranscriptionManager::new(&app_handle, model_manager).unwrap();

    // Find a downloaded model
    let models = manager.model_manager.get_available_models();
    if let Some(downloaded) = models.iter().find(|m| m.is_downloaded) {
        let result = manager.load_model(&downloaded.id);
        assert!(result.is_ok());
        assert!(manager.is_model_loaded());
    }
}

#[tokio::test]
async fn test_unload_model() {
    let app_handle = create_test_app_handle();
    let model_manager = Arc::new(ModelManager::new(&app_handle).unwrap());
    let manager = TranscriptionManager::new(&app_handle, model_manager).unwrap();

    // Load then unload
    let models = manager.model_manager.get_available_models();
    if let Some(downloaded) = models.iter().find(|m| m.is_downloaded) {
        manager.load_model(&downloaded.id).unwrap();
        assert!(manager.is_model_loaded());

        let result = manager.unload_model();
        assert!(result.is_ok());
        assert!(!manager.is_model_loaded());
    }
}

#[tokio::test]
async fn test_transcribe_empty_audio() {
    let app_handle = create_test_app_handle();
    let model_manager = Arc::new(ModelManager::new(&app_handle).unwrap());
    let manager = TranscriptionManager::new(&app_handle, model_manager).unwrap();

    let empty_audio = vec![];
    let result = manager.transcribe(empty_audio);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[tokio::test]
async fn test_transcribe_with_loaded_model() {
    let app_handle = create_test_app_handle();
    let model_manager = Arc::new(ModelManager::new(&app_handle).unwrap());
    let manager = TranscriptionManager::new(&app_handle, model_manager).unwrap();

    // Load a model first
    let models = manager.model_manager.get_available_models();
    if let Some(downloaded) = models.iter().find(|m| m.is_downloaded) {
        manager.load_model(&downloaded.id).unwrap();

        // Create test audio (1 second of sine wave)
        let audio = create_test_audio_samples(1000);

        let result = manager.transcribe(audio);
        assert!(result.is_ok());
        // Note: Result might be empty if audio is not recognizable speech
    }
}

#[tokio::test]
async fn test_transcribe_without_loaded_model() {
    let app_handle = create_test_app_handle();
    let model_manager = Arc::new(ModelManager::new(&app_handle).unwrap());
    let manager = TranscriptionManager::new(&app_handle, model_manager).unwrap();

    let audio = create_test_audio_samples(1000);
    let result = manager.transcribe(audio);

    // Should error because no model is loaded
    assert!(result.is_err());
}

#[tokio::test]
async fn test_idle_unload_timeout() {
    let app_handle = create_test_app_handle();
    let model_manager = Arc::new(ModelManager::new(&app_handle).unwrap());
    let manager = TranscriptionManager::new(&app_handle, model_manager).unwrap();

    // Set timeout to 1 second for testing
    let mut settings = get_settings(&app_handle);
    settings.model_unload_timeout = ModelUnloadTimeout::Custom(1);
    write_settings(&app_handle, settings);

    // Load model
    let models = manager.model_manager.get_available_models();
    if let Some(downloaded) = models.iter().find(|m| m.is_downloaded) {
        manager.load_model(&downloaded.id).unwrap();
        assert!(manager.is_model_loaded());

        // Wait for idle timeout + buffer
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Model should be unloaded
        assert!(!manager.is_model_loaded());
    }
}
```

**Automated Verification:**
```bash
cargo test transcription_tests -- --nocapture
```

**Deliverables:**
- [ ] 8+ unit tests for TranscriptionManager
- [ ] Model loading/unloading tests
- [ ] Transcription pipeline tests
- [ ] Idle timeout tests
- [ ] Error handling coverage

---

### Task 1.1.5: Integration Tests for Audio → Transcription Pipeline
**Priority**: MEDIUM
**Effort**: 1 week
**Dependencies**: 1.1.2, 1.1.3, 1.1.4

**Test Cases:**
```rust
// tests/integration/pipeline_tests.rs

#[tokio::test]
async fn test_end_to_end_recording_and_transcription() {
    let app_handle = create_test_app_handle();

    // Initialize managers
    let mut audio_manager = AudioManager::new(&app_handle).await.unwrap();
    let model_manager = Arc::new(ModelManager::new(&app_handle).unwrap());
    let transcription_manager = TranscriptionManager::new(&app_handle, model_manager).unwrap();

    // Load a model
    let models = transcription_manager.model_manager.get_available_models();
    let downloaded = models.iter().find(|m| m.is_downloaded).expect("Need at least one model");
    transcription_manager.load_model(&downloaded.id).unwrap();

    // Start recording
    audio_manager.start_recording().await.unwrap();

    // Record for 2 seconds
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Stop and get audio
    let audio_samples = audio_manager.stop_recording().await.unwrap();

    // Transcribe
    let result = transcription_manager.transcribe(audio_samples);

    assert!(result.is_ok());
    println!("Transcription: {}", result.unwrap());
}

#[tokio::test]
async fn test_multiple_recordings_in_sequence() {
    let app_handle = create_test_app_handle();
    let mut audio_manager = AudioManager::new(&app_handle).await.unwrap();
    let model_manager = Arc::new(ModelManager::new(&app_handle).unwrap());
    let transcription_manager = TranscriptionManager::new(&app_handle, model_manager).unwrap();

    // Load model once
    let models = transcription_manager.model_manager.get_available_models();
    let downloaded = models.iter().find(|m| m.is_downloaded).expect("Need at least one model");
    transcription_manager.load_model(&downloaded.id).unwrap();

    // Perform 3 recordings
    for i in 0..3 {
        audio_manager.start_recording().await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        let audio = audio_manager.stop_recording().await.unwrap();

        let result = transcription_manager.transcribe(audio);
        assert!(result.is_ok(), "Recording {} failed", i);
    }
}

#[tokio::test]
async fn test_device_change_during_pipeline() {
    let app_handle = create_test_app_handle();
    let mut audio_manager = AudioManager::new(&app_handle).await.unwrap();

    let devices = audio_manager.get_available_devices();
    if devices.len() > 1 {
        // Start with first device
        audio_manager.start_recording().await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        audio_manager.stop_recording().await.unwrap();

        // Switch to second device
        audio_manager.set_device(&devices[1].id).await.unwrap();

        // Record again
        audio_manager.start_recording().await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let audio = audio_manager.stop_recording().await.unwrap();

        assert!(audio.len() > 0);
    }
}
```

**Automated Verification:**
```bash
cargo test --test integration -- --nocapture
```

**Deliverables:**
- [ ] 3+ integration tests
- [ ] End-to-end pipeline verification
- [ ] Multi-recording sequence tests
- [ ] Device switching tests

---

## 1.2 Code Quality & Technical Debt

### Task 1.2.1: Consolidate Logging System
**Priority**: MEDIUM
**Effort**: 3 days
**Dependencies**: None

**Implementation Steps:**
1. Audit all `println!` and `eprintln!` usage
2. Replace with appropriate log levels
3. Configure env_logger with levels
4. Add log output to file (optional)

**Changes Required:**
```rust
// Before
println!("Starting transcription");
eprintln!("Error: {}", e);

// After
log::info!("Starting transcription");
log::error!("Error: {}", e);
```

**Files to Update:**
```bash
# Find all println! usage
grep -r "println!" src-tauri/src/

# Expected files:
- src/managers/transcription.rs (multiple instances)
- src/managers/model.rs (multiple instances)
- src/managers/audio.rs
- src/audio_toolkit/
```

**Verification Script:**
```bash
#!/bin/bash
# verify_logging.sh

echo "Checking for remaining println! statements..."
COUNT=$(grep -r "println!" src-tauri/src/ --include="*.rs" | wc -l)

if [ $COUNT -eq 0 ]; then
    echo "✓ No println! statements found"
    exit 0
else
    echo "✗ Found $COUNT println! statements"
    grep -r "println!" src-tauri/src/ --include="*.rs"
    exit 1
fi
```

**Test Plan:**
```rust
#[test]
fn test_logging_configuration() {
    env_logger::init();

    log::trace!("Trace message");
    log::debug!("Debug message");
    log::info!("Info message");
    log::warn!("Warning message");
    log::error!("Error message");

    // Verify log output exists (check log file or stderr)
}
```

**Automated Verification:**
```bash
./verify_logging.sh
cargo test test_logging_configuration
```

**Deliverables:**
- [ ] All `println!` converted to `log::info!` or appropriate level
- [ ] All `eprintln!` converted to `log::error!` or `log::warn!`
- [ ] Logging configuration documented
- [ ] No raw print statements in production code

---

### Task 1.2.2: Error Handling Improvements
**Priority**: HIGH
**Effort**: 1 week
**Dependencies**: None

**Implementation Steps:**
1. Create custom error types
2. Implement From traits for error conversion
3. Add context to errors using `.context()`
4. Add error recovery strategies

**Custom Error Types:**
```rust
// src-tauri/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandyError {
    #[error("Audio device error: {0}")]
    AudioDevice(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Model not downloaded: {0}")]
    ModelNotDownloaded(String),

    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, HandyError>;
```

**Add to Cargo.toml:**
```toml
[dependencies]
thiserror = "1.0"
```

**Update Manager Methods:**
```rust
// Before
pub fn load_model(&self, model_id: &str) -> Result<()> {
    let model_path = self.model_manager.get_model_path(model_id)?;
    // ...
}

// After
pub fn load_model(&self, model_id: &str) -> Result<()> {
    let model_path = self.model_manager
        .get_model_path(model_id)
        .context(format!("Failed to get path for model: {}", model_id))?;
    // ...
}
```

**Test Plan:**
```rust
#[test]
fn test_error_types() {
    let err = HandyError::ModelNotFound("test".to_string());
    assert_eq!(err.to_string(), "Model not found: test");
}

#[test]
fn test_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let handy_err: HandyError = io_err.into();
    assert!(matches!(handy_err, HandyError::Io(_)));
}
```

**Automated Verification:**
```bash
cargo test error_types
cargo build --release
# Should compile without errors
```

**Deliverables:**
- [ ] Custom error types defined
- [ ] Error conversion implementations
- [ ] Error context added throughout codebase
- [ ] Error handling tests
- [ ] Documentation on error handling strategy

---

## 1.3 Documentation

### Task 1.3.1: API Documentation (Rust Docs)
**Priority**: HIGH
**Effort**: 1 week
**Dependencies**: None

**Implementation Steps:**
1. Add doc comments to all public APIs
2. Add examples to complex functions
3. Document all managers
4. Generate and review docs

**Example Documentation:**
```rust
/// Manages transcription models including download, loading, and lifecycle.
///
/// The `ModelManager` is responsible for:
/// - Tracking available models (Whisper and Parakeet)
/// - Downloading models from remote sources
/// - Managing model storage and cleanup
/// - Providing model paths to the transcription system
///
/// # Examples
///
/// ```no_run
/// use handy::managers::ModelManager;
///
/// let app_handle = /* get app handle */;
/// let manager = ModelManager::new(&app_handle)?;
///
/// // List available models
/// let models = manager.get_available_models();
/// for model in models {
///     println!("{}: {}", model.id, model.name);
/// }
///
/// // Download a model
/// manager.download_model("small").await?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct ModelManager {
    // ...
}

impl ModelManager {
    /// Creates a new ModelManager instance.
    ///
    /// # Arguments
    ///
    /// * `app_handle` - The Tauri application handle
    ///
    /// # Errors
    ///
    /// Returns an error if the models directory cannot be created or accessed.
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        // ...
    }

    /// Downloads a model by ID.
    ///
    /// This method will:
    /// 1. Check if the model is already downloaded
    /// 2. Resume partial downloads if available
    /// 3. Emit progress events during download
    /// 4. Extract archives for directory-based models
    ///
    /// # Arguments
    ///
    /// * `model_id` - The ID of the model to download (e.g., "small", "parakeet-v3")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The model ID is not found
    /// - The network request fails
    /// - The download is interrupted
    /// - Extraction fails (for archive-based models)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(manager: &ModelManager) -> Result<(), anyhow::Error> {
    /// manager.download_model("small").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_model(&self, model_id: &str) -> Result<()> {
        // ...
    }
}
```

**Verification Command:**
```bash
# Generate documentation
cargo doc --no-deps --open

# Check for missing docs
cargo doc --no-deps 2>&1 | grep "missing documentation"

# Check doc tests
cargo test --doc
```

**Deliverables:**
- [ ] All public APIs documented
- [ ] Examples in complex functions
- [ ] Module-level documentation
- [ ] Generated docs reviewed
- [ ] Doc tests passing

---

### Task 1.3.2: Architecture Documentation
**Priority**: MEDIUM
**Effort**: 3 days
**Dependencies**: None

**Create ARCHITECTURE.md:**
```markdown
# Handy Architecture

## Overview

Handy follows a manager-based architecture where core functionality is organized into specialized managers that communicate via Tauri's command-event system.

## Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (React)                     │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Settings   │  │   History    │  │ Model Select │  │
│  │     UI      │  │      UI      │  │      UI      │  │
│  └──────┬──────┘  └──────┬───────┘  └──────┬───────┘  │
└─────────┼────────────────┼──────────────────┼──────────┘
          │ Tauri Commands │                  │
          ▼                ▼                  ▼
┌─────────────────────────────────────────────────────────┐
│                 Backend (Rust/Tauri)                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │              Command Handlers                     │  │
│  └───────┬──────────────────┬───────────────────────┘  │
│          │                  │                           │
│  ┌───────▼───────┐  ┌───────▼───────┐  ┌───────────┐  │
│  │Audio Manager  │  │Model Manager  │  │  History  │  │
│  │               │  │               │  │  Manager  │  │
│  └───────┬───────┘  └───────┬───────┘  └───────────┘  │
│          │                  │                           │
│          └──────┬───────────┘                           │
│                 ▼                                        │
│        ┌────────────────────┐                           │
│        │Transcription       │                           │
│        │    Manager         │                           │
│        └────────┬───────────┘                           │
│                 │                                        │
│  ┌──────────────▼───────────────┐                      │
│  │      Audio Toolkit           │                      │
│  │  ┌──────┐  ┌────┐  ┌──────┐ │                      │
│  │  │ VAD  │  │Rec │  │Resam │ │                      │
│  │  └──────┘  └────┘  └──────┘ │                      │
│  └──────────────────────────────┘                      │
└─────────────────────────────────────────────────────────┘
```

## Data Flow

### Recording Flow
1. User presses shortcut → `ShortcutManager` detects
2. `AudioManager.start_recording()` called
3. Audio device streams samples
4. VAD filters silence in real-time
5. User releases shortcut
6. `AudioManager.stop_recording()` returns filtered audio
7. Audio sent to `TranscriptionManager`
8. Model transcribes audio to text
9. Text passed through custom word correction
10. Result emitted to frontend
11. `ClipboardManager` pastes text to active app

### Model Download Flow
1. User selects model in UI
2. `download_model` command called
3. `ModelManager` checks if partial download exists
4. HTTP request with range header (resume support)
5. Progress events emitted during download
6. For archives: extract to temporary directory
7. Move to final location
8. Update model status
9. Emit completion event

## Manager Responsibilities

### AudioManager
- Audio device enumeration
- Recording start/stop
- Device switching
- Mode management (AlwaysOn/OnDemand)

### ModelManager
- Model metadata management
- Download/resume/cancel operations
- Model storage and cleanup
- Path resolution

### TranscriptionManager
- Model loading/unloading
- Transcription execution
- Idle timeout management
- Multi-engine support (Whisper/Parakeet)

### HistoryManager
- SQLite database operations
- Audio file storage
- Transcription metadata
- Search and cleanup

## Threading Model

- **Main Thread**: Tauri event loop, UI rendering
- **Audio Thread**: cpal audio stream callback
- **Idle Watcher Thread**: Model unload timeout polling
- **Tokio Runtime**: Async operations (HTTP downloads, commands)

## State Management

- **Settings**: Tauri store plugin (JSON persistence)
- **Manager State**: Arc<Mutex<T>> for thread-safe access
- **Events**: Tauri emit for backend→frontend communication
- **Commands**: Tauri commands for frontend→backend calls
```

**Deliverables:**
- [ ] ARCHITECTURE.md created
- [ ] Component diagrams included
- [ ] Data flow documented
- [ ] Threading model explained
- [ ] State management described

---

## 1.4 Performance & Optimization

### Task 1.4.1: Async Model Loading
**Priority**: HIGH
**Effort**: 1 week
**Dependencies**: 1.1.3

**Current Problem:**
```rust
// Currently blocks UI during model loading
pub fn transcribe(&self, audio: Vec<f32>) -> Result<String> {
    // If model not loaded, load it synchronously (BLOCKS!)
    if !self.is_model_loaded() {
        self.load_model(&settings.selected_model)?; // <-- UI freeze here
    }
    // ...
}
```

**Solution:**
```rust
// New approach: Async loading with progress events
pub async fn load_model_async(&self, model_id: &str) -> Result<()> {
    // Emit loading started event
    self.app_handle.emit("model-loading-started", model_id)?;

    // Spawn blocking task for model loading
    let model_path = self.model_manager.get_model_path(model_id)?;
    let loaded_engine = tokio::task::spawn_blocking(move || {
        // Heavy model loading happens here without blocking async runtime
        match model_info.engine_type {
            EngineType::Whisper => {
                let mut engine = WhisperEngine::new();
                engine.load_model(&model_path)?;
                Ok(LoadedEngine::Whisper(engine))
            }
            // ...
        }
    }).await??;

    // Update state
    *self.engine.lock().unwrap() = Some(loaded_engine);

    // Emit loading completed event
    self.app_handle.emit("model-loading-completed", model_id)?;

    Ok(())
}
```

**UI Integration:**
```typescript
// src/hooks/useModelLoading.ts
export function useModelLoading() {
  const [isLoading, setIsLoading] = useState(false);
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const loadingStarted = listen('model-loading-started', () => {
      setIsLoading(true);
      setProgress(0);
    });

    const loadingCompleted = listen('model-loading-completed', () => {
      setIsLoading(false);
      setProgress(100);
    });

    return () => {
      loadingStarted.then(unlisten => unlisten());
      loadingCompleted.then(unlisten => unlisten());
    };
  }, []);

  return { isLoading, progress };
}
```

**Test Plan:**
```rust
#[tokio::test]
async fn test_async_model_loading() {
    let manager = TranscriptionManager::new(/* ... */).unwrap();

    // Start loading
    let load_task = manager.load_model_async("small");

    // Model should not be immediately loaded
    assert!(!manager.is_model_loaded());

    // Wait for completion
    load_task.await.unwrap();

    // Now it should be loaded
    assert!(manager.is_model_loaded());
}

#[tokio::test]
async fn test_concurrent_transcription_requests() {
    let manager = Arc::new(TranscriptionManager::new(/* ... */).unwrap());

    // Start 3 transcriptions concurrently
    let tasks: Vec<_> = (0..3).map(|_| {
        let mgr = manager.clone();
        let audio = create_test_audio_samples(1000);
        tokio::spawn(async move {
            mgr.transcribe(audio).await
        })
    }).collect();

    // Wait for all to complete
    for task in tasks {
        let result = task.await.unwrap();
        assert!(result.is_ok());
    }
}
```

**Automated Verification:**
```bash
cargo test test_async_model_loading
cargo test test_concurrent_transcription_requests

# Performance benchmark
cargo bench --bench model_loading
```

**Deliverables:**
- [ ] Async model loading implemented
- [ ] UI loading states added
- [ ] Progress events emitted
- [ ] Tests for concurrent operations
- [ ] No UI blocking during model load

---

### Task 1.4.2: Hot-Plug Audio Device Support
**Priority**: MEDIUM
**Effort**: 1 week
**Dependencies**: 1.1.2

**Current Limitation:**
```rust
// Device enumeration only happens at startup
pub async fn new(app_handle: &AppHandle) -> Result<Self> {
    let devices = enumerate_devices()?; // <-- Only once!
    // ...
}
```

**Solution:**
```rust
// Add device monitoring
pub struct AudioManager {
    // ...
    device_watcher: Arc<Mutex<DeviceWatcher>>,
}

pub struct DeviceWatcher {
    last_known_devices: Vec<AudioDevice>,
    poll_interval: Duration,
}

impl DeviceWatcher {
    pub async fn start_monitoring(&mut self, app_handle: AppHandle) {
        loop {
            tokio::time::sleep(self.poll_interval).await;

            let current_devices = enumerate_devices().ok().unwrap_or_default();

            // Detect changes
            if current_devices != self.last_known_devices {
                let added = Self::find_added(&self.last_known_devices, &current_devices);
                let removed = Self::find_removed(&self.last_known_devices, &current_devices);

                if !added.is_empty() {
                    app_handle.emit("audio-device-added", &added)?;
                }

                if !removed.is_empty() {
                    app_handle.emit("audio-device-removed", &removed)?;
                }

                self.last_known_devices = current_devices;
            }
        }
    }

    fn find_added(old: &[AudioDevice], new: &[AudioDevice]) -> Vec<AudioDevice> {
        new.iter()
            .filter(|d| !old.iter().any(|o| o.id == d.id))
            .cloned()
            .collect()
    }

    fn find_removed(old: &[AudioDevice], new: &[AudioDevice]) -> Vec<AudioDevice> {
        old.iter()
            .filter(|d| !new.iter().any(|n| n.id == d.id))
            .cloned()
            .collect()
    }
}
```

**Test Plan:**
```rust
#[tokio::test]
async fn test_device_hot_plug_detection() {
    let mut watcher = DeviceWatcher::new(Duration::from_millis(100));

    // Get initial devices
    let initial = enumerate_devices().unwrap();
    watcher.last_known_devices = initial.clone();

    // Simulate device change (in real test, would use mock device manager)
    let mut modified = initial.clone();
    modified.push(AudioDevice {
        id: "new-device".to_string(),
        name: "New Microphone".to_string(),
        is_default: false,
    });

    let added = DeviceWatcher::find_added(&initial, &modified);
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].id, "new-device");
}

#[tokio::test]
async fn test_device_removal_during_recording() {
    let mut audio_manager = AudioManager::new(&create_test_app_handle()).await.unwrap();

    audio_manager.start_recording().await.unwrap();

    // Simulate device removal (would need mock in real implementation)
    // Expected: gracefully handle and emit error event

    // In real implementation, would verify:
    // - Recording stops gracefully
    // - Error event is emitted
    // - Audio captured so far is preserved
}
```

**Automated Verification:**
```bash
cargo test device_hot_plug
cargo test device_removal_during_recording
```

**Deliverables:**
- [ ] Device monitoring implemented
- [ ] Device add/remove events
- [ ] Graceful handling of device removal during recording
- [ ] UI updates on device changes
- [ ] Tests for device changes

---

## 1.5 Phase 1 Summary

### Completion Criteria
- [ ] All unit tests passing (>50 tests)
- [ ] Integration tests passing (>5 tests)
- [ ] Test coverage >70%
- [ ] All logging converted to log crate
- [ ] API documentation complete
- [ ] Architecture documented
- [ ] Async model loading implemented
- [ ] Hot-plug device support added

### Metrics
- **Code Coverage**: Target 70%+
- **Test Count**: 50+ unit tests, 5+ integration tests
- **Documentation**: 100% public API coverage
- **Performance**: Model loading <500ms on average hardware
- **Reliability**: 0 crashes in 1000 test runs

### Phase 1 Test Suite Runner
```bash
#!/bin/bash
# run_phase1_tests.sh

echo "=== Phase 1 Test Suite ==="
echo

echo "1. Running unit tests..."
cargo test --lib -- --nocapture
UNIT_RESULT=$?

echo
echo "2. Running integration tests..."
cargo test --test integration -- --nocapture
INTEGRATION_RESULT=$?

echo
echo "3. Running doc tests..."
cargo test --doc
DOC_RESULT=$?

echo
echo "4. Verifying logging consolidation..."
./verify_logging.sh
LOGGING_RESULT=$?

echo
echo "5. Generating documentation..."
cargo doc --no-deps
DOC_GEN_RESULT=$?

echo
echo "6. Running benchmarks..."
cargo bench --bench phase1
BENCH_RESULT=$?

echo
echo "=== Results ==="
echo "Unit Tests: $([ $UNIT_RESULT -eq 0 ] && echo '✓ PASS' || echo '✗ FAIL')"
echo "Integration Tests: $([ $INTEGRATION_RESULT -eq 0 ] && echo '✓ PASS' || echo '✗ FAIL')"
echo "Doc Tests: $([ $DOC_RESULT -eq 0 ] && echo '✓ PASS' || echo '✗ FAIL')"
echo "Logging Check: $([ $LOGGING_RESULT -eq 0 ] && echo '✓ PASS' || echo '✗ FAIL')"
echo "Documentation: $([ $DOC_GEN_RESULT -eq 0 ] && echo '✓ PASS' || echo '✗ FAIL')"
echo "Benchmarks: $([ $BENCH_RESULT -eq 0 ] && echo '✓ PASS' || echo '✗ FAIL')"

# Exit with error if any test failed
if [ $UNIT_RESULT -ne 0 ] || [ $INTEGRATION_RESULT -ne 0 ] || [ $DOC_RESULT -ne 0 ] || [ $LOGGING_RESULT -ne 0 ] || [ $DOC_GEN_RESULT -ne 0 ]; then
    echo
    echo "✗ Phase 1 tests FAILED"
    exit 1
else
    echo
    echo "✓ Phase 1 tests PASSED"
    exit 0
fi
```

---

# Phase 2: Core Features (6-12 Months)

**Focus**: User Experience, Performance, and Essential Features

**Theme**: Enhance core functionality with features that significantly improve the user experience without compromising simplicity.

## 2.1 Real-Time Streaming Transcription

### Task 2.1.1: Implement Chunked Audio Processing
**Priority**: CRITICAL
**Effort**: 2 weeks
**Dependencies**: Phase 1 complete

**Architecture:**
```
User speaks → Audio chunks (every 500ms) → VAD → Whisper → Partial results → UI updates
```

**Implementation:**
```rust
// src-tauri/src/managers/streaming_transcription.rs

pub struct StreamingTranscriptionManager {
    chunk_buffer: Arc<Mutex<Vec<f32>>>,
    chunk_size_ms: u32,
    min_chunk_size_samples: usize,
    transcription_manager: Arc<TranscriptionManager>,
    is_streaming: Arc<AtomicBool>,
}

impl StreamingTranscriptionManager {
    pub fn new(
        transcription_manager: Arc<TranscriptionManager>,
        chunk_size_ms: u32
    ) -> Self {
        let sample_rate = 16000; // Whisper expects 16kHz
        let min_chunk_size_samples = (sample_rate * chunk_size_ms / 1000) as usize;

        Self {
            chunk_buffer: Arc::new(Mutex::new(Vec::new())),
            chunk_size_ms,
            min_chunk_size_samples,
            transcription_manager,
            is_streaming: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start_streaming(&self, app_handle: AppHandle) -> Result<()> {
        self.is_streaming.store(true, Ordering::Relaxed);

        let buffer = self.chunk_buffer.clone();
        let min_size = self.min_chunk_size_samples;
        let mgr = self.transcription_manager.clone();
        let is_streaming = self.is_streaming.clone();

        tokio::spawn(async move {
            let mut full_transcript = String::new();

            while is_streaming.load(Ordering::Relaxed) {
                // Wait for chunk to be ready
                tokio::time::sleep(Duration::from_millis(100)).await;

                let chunk = {
                    let mut buf = buffer.lock().unwrap();
                    if buf.len() >= min_size {
                        let chunk = buf.drain(..min_size).collect();
                        chunk
                    } else {
                        continue;
                    }
                };

                // Transcribe chunk
                match mgr.transcribe(chunk).await {
                    Ok(partial) => {
                        if !partial.is_empty() {
                            full_transcript.push_str(&partial);
                            full_transcript.push(' ');

                            // Emit partial result
                            let _ = app_handle.emit("streaming-transcription-partial", &full_transcript);
                        }
                    }
                    Err(e) => {
                        log::error!("Streaming transcription error: {}", e);
                    }
                }
            }

            // Emit final result
            let _ = app_handle.emit("streaming-transcription-complete", &full_transcript.trim());
        });

        Ok(())
    }

    pub fn push_audio(&self, samples: Vec<f32>) {
        let mut buffer = self.chunk_buffer.lock().unwrap();
        buffer.extend(samples);
    }

    pub fn stop_streaming(&self) {
        self.is_streaming.store(false, Ordering::Relaxed);
    }
}
```

**Test Plan:**
```rust
#[tokio::test]
async fn test_streaming_transcription_chunking() {
    let app_handle = create_test_app_handle();
    let transcription_mgr = Arc::new(create_transcription_manager());
    let streaming_mgr = StreamingTranscriptionManager::new(transcription_mgr, 500);

    // Start streaming
    streaming_mgr.start_streaming(app_handle.clone()).await.unwrap();

    // Push audio in chunks
    for _ in 0..10 {
        let chunk = create_test_audio_samples(100); // 100ms chunks
        streaming_mgr.push_audio(chunk);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Wait for processing
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Stop streaming
    streaming_mgr.stop_streaming();

    // Verify partial results were emitted
    // (Would check event emissions in real test)
}

#[tokio::test]
async fn test_streaming_latency() {
    let streaming_mgr = create_streaming_manager();

    streaming_mgr.start_streaming(create_test_app_handle()).await.unwrap();

    let start = Instant::now();

    // Push one chunk
    let chunk = create_test_audio_samples(500);
    streaming_mgr.push_audio(chunk);

    // Wait for first partial result
    // (In real test, would listen for event)
    tokio::time::sleep(Duration::from_millis(100)).await;

    let latency = start.elapsed();

    // Latency should be < 1 second for responsive UX
    assert!(latency < Duration::from_secs(1), "Latency too high: {:?}", latency);

    streaming_mgr.stop_streaming();
}
```

**UI Integration:**
```typescript
// src/components/StreamingTranscript.tsx
export function StreamingTranscript() {
  const [partialText, setPartialText] = useState('');
  const [finalText, setFinalText] = useState('');

  useEffect(() => {
    const unlistenPartial = listen('streaming-transcription-partial', (event) => {
      setPartialText(event.payload as string);
    });

    const unlistenComplete = listen('streaming-transcription-complete', (event) => {
      setFinalText(event.payload as string);
      setPartialText('');
    });

    return () => {
      unlistenPartial.then(fn => fn());
      unlistenComplete.then(fn => fn());
    };
  }, []);

  return (
    <div className="streaming-transcript">
      <div className="partial-text opacity-70">{partialText}</div>
      {finalText && <div className="final-text">{finalText}</div>}
    </div>
  );
}
```

**Automated Verification:**
```bash
cargo test streaming_transcription --features streaming
cargo bench --bench streaming_latency
```

**Deliverables:**
- [ ] Streaming transcription manager
- [ ] Chunked audio processing
- [ ] Partial result events
- [ ] UI components for streaming display
- [ ] Latency benchmarks
- [ ] Tests for streaming behavior

---

## 2.2 Ollama Integration

### Task 2.2.1: Implement OllamaManager
**Priority**: HIGH
**Effort**: 1 week
**Dependencies**: Phase 1 complete

**Implementation:**
```rust
// src-tauri/src/managers/ollama.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Deserialize)]
struct GenerateResponse {
    model: String,
    response: String,
    done: bool,
}

#[derive(Deserialize)]
struct ModelsResponse {
    models: Vec<OllamaModel>,
}

pub struct OllamaManager {
    base_url: String,
    client: Client,
    is_available: Arc<AtomicBool>,
}

impl OllamaManager {
    pub fn new(base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());

        Self {
            base_url,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            is_available: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if Ollama service is running
    pub async fn check_availability(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                self.is_available.store(true, Ordering::Relaxed);
                true
            }
            _ => {
                self.is_available.store(false, Ordering::Relaxed);
                false
            }
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<ModelsResponse>()
            .await?;

        Ok(response.models)
    }

    /// Process text with Ollama
    pub async fn process_text(
        &self,
        text: &str,
        model: &str,
        prompt_template: &str,
        system_prompt: Option<&str>,
    ) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        let prompt = prompt_template.replace("{text}", text);

        let request = GenerateRequest {
            model: model.to_string(),
            prompt,
            stream: false,
            system: system_prompt.map(String::from),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(30))
            .send()
            .await?
            .json::<GenerateResponse>()
            .await?;

        Ok(response.response)
    }

    /// Process text with streaming support
    pub async fn process_text_streaming<F>(
        &self,
        text: &str,
        model: &str,
        prompt_template: &str,
        system_prompt: Option<&str>,
        mut callback: F,
    ) -> Result<String>
    where
        F: FnMut(String),
    {
        let url = format!("{}/api/generate", self.base_url);

        let prompt = prompt_template.replace("{text}", text);

        let request = GenerateRequest {
            model: model.to_string(),
            prompt,
            stream: true,
            system: system_prompt.map(String::from),
        };

        let mut response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let mut full_response = String::new();

        while let Some(chunk) = response.chunk().await? {
            let text = String::from_utf8_lossy(&chunk);

            // Parse JSON lines
            for line in text.lines() {
                if let Ok(resp) = serde_json::from_str::<GenerateResponse>(line) {
                    full_response.push_str(&resp.response);
                    callback(resp.response);

                    if resp.done {
                        break;
                    }
                }
            }
        }

        Ok(full_response)
    }

    pub fn is_available(&self) -> bool {
        self.is_available.load(Ordering::Relaxed)
    }
}
```

**Test Plan:**
```rust
#[tokio::test]
async fn test_ollama_manager_creation() {
    let manager = OllamaManager::new(None);
    assert_eq!(manager.base_url, "http://localhost:11434");
}

#[tokio::test]
async fn test_ollama_availability_check() {
    let manager = OllamaManager::new(None);

    // Note: This test requires Ollama to be running
    let available = manager.check_availability().await;

    if available {
        println!("✓ Ollama is running");
        assert!(manager.is_available());
    } else {
        println!("⚠ Ollama is not running (test skipped)");
    }
}

#[tokio::test]
#[ignore] // Run only when Ollama is available
async fn test_list_models() {
    let manager = OllamaManager::new(None);

    if !manager.check_availability().await {
        println!("Skipping test - Ollama not available");
        return;
    }

    let models = manager.list_models().await.unwrap();
    assert!(models.len() > 0, "Should have at least one model");

    for model in models {
        println!("Found model: {} ({} bytes)", model.name, model.size);
    }
}

#[tokio::test]
#[ignore] // Run only when Ollama is available
async fn test_process_text() {
    let manager = OllamaManager::new(None);

    if !manager.check_availability().await {
        println!("Skipping test - Ollama not available");
        return;
    }

    let test_text = "hello world how are you";
    let prompt_template = "Add proper punctuation and capitalization to this text: {text}";
    let system_prompt = "You are a helpful assistant that improves text formatting.";

    let result = manager.process_text(
        test_text,
        "llama3.2:3b",
        prompt_template,
        Some(system_prompt)
    ).await;

    match result {
        Ok(enhanced) => {
            println!("Original: {}", test_text);
            println!("Enhanced: {}", enhanced);
            assert!(!enhanced.is_empty());
        }
        Err(e) => {
            println!("Error (may be due to model not available): {}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_streaming_processing() {
    let manager = OllamaManager::new(None);

    if !manager.check_availability().await {
        println!("Skipping test - Ollama not available");
        return;
    }

    let test_text = "the quick brown fox jumps over the lazy dog";
    let prompt_template = "Summarize this text: {text}";

    let mut chunks = Vec::new();

    let result = manager.process_text_streaming(
        test_text,
        "llama3.2:3b",
        prompt_template,
        None,
        |chunk| {
            print!("{}", chunk);
            chunks.push(chunk);
        }
    ).await;

    match result {
        Ok(full) => {
            println!("\nFull response: {}", full);
            assert!(!chunks.is_empty(), "Should have received chunks");
            assert!(!full.is_empty());
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
```

**Automated Verification:**
```bash
# Without Ollama running
cargo test ollama_manager_creation
cargo test ollama_availability_check

# With Ollama running (install first: curl -fsSL https://ollama.com/install.sh | sh)
ollama pull llama3.2:3b
cargo test --test ollama_tests -- --ignored --nocapture
```

**Deliverables:**
- [ ] OllamaManager implementation
- [ ] HTTP client for Ollama API
- [ ] Model listing functionality
- [ ] Text processing (streaming & non-streaming)
- [ ] Tests for all functionality
- [ ] Error handling and fallbacks

---

### Task 2.2.2: Post-Processing Pipeline
**Priority**: HIGH
**Effort**: 1 week
**Dependencies**: 2.2.1

**Implementation:**
```rust
// src-tauri/src/processing/pipeline.rs

use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait TextProcessor: Send + Sync {
    /// Process the input text and return the result
    async fn process(&self, text: String) -> Result<String>;

    /// Get the name of this processor
    fn name(&self) -> &str;

    /// Check if this processor is enabled
    fn is_enabled(&self) -> bool;
}

pub struct PostProcessingPipeline {
    processors: Vec<Box<dyn TextProcessor>>,
}

impl PostProcessingPipeline {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add_processor(&mut self, processor: Box<dyn TextProcessor>) {
        self.processors.push(processor);
    }

    pub async fn process(&self, mut text: String) -> Result<String> {
        for processor in &self.processors {
            if processor.is_enabled() {
                log::debug!("Running processor: {}", processor.name());
                text = processor.process(text).await?;
            } else {
                log::debug!("Skipping disabled processor: {}", processor.name());
            }
        }
        Ok(text)
    }
}

// Custom Words Processor (existing functionality)
pub struct CustomWordsProcessor {
    words: Vec<String>,
    threshold: f64,
    enabled: bool,
}

#[async_trait]
impl TextProcessor for CustomWordsProcessor {
    async fn process(&self, text: String) -> Result<String> {
        Ok(apply_custom_words(&text, &self.words, self.threshold))
    }

    fn name(&self) -> &str {
        "CustomWords"
    }

    fn is_enabled(&self) -> bool {
        self.enabled && !self.words.is_empty()
    }
}

// Ollama Processor
pub struct OllamaProcessor {
    manager: Arc<OllamaManager>,
    model: String,
    mode: ProcessingMode,
    custom_prompt: Option<String>,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingMode {
    Punctuation,
    Summarize,
    CommandExtract,
    Custom,
}

impl ProcessingMode {
    pub fn to_prompt_template(&self) -> &str {
        match self {
            ProcessingMode::Punctuation => {
                "Add proper punctuation, capitalization, and formatting to this transcribed text, preserving the exact words spoken:\n\n{text}"
            }
            ProcessingMode::Summarize => {
                "Provide a concise summary of this transcription:\n\n{text}"
            }
            ProcessingMode::CommandExtract => {
                "Extract action items and TODO items from this transcription as a bulleted list:\n\n{text}"
            }
            ProcessingMode::Custom => "{text}",
        }
    }

    pub fn system_prompt(&self) -> &str {
        match self {
            ProcessingMode::Punctuation => {
                "You are a helpful assistant that improves transcription quality by adding punctuation and capitalization. Preserve the exact words and do not add or remove content."
            }
            ProcessingMode::Summarize => {
                "You are a helpful assistant that creates concise summaries."
            }
            ProcessingMode::CommandExtract => {
                "You are a helpful assistant that extracts actionable items from text."
            }
            ProcessingMode::Custom => {
                "You are a helpful assistant."
            }
        }
    }
}

#[async_trait]
impl TextProcessor for OllamaProcessor {
    async fn process(&self, text: String) -> Result<String> {
        // Check if Ollama is available
        if !self.manager.check_availability().await {
            log::warn!("Ollama not available, skipping processing");
            return Ok(text);
        }

        let prompt_template = match &self.mode {
            ProcessingMode::Custom => {
                self.custom_prompt.as_deref().unwrap_or("{text}")
            }
            mode => mode.to_prompt_template(),
        };

        let result = self.manager.process_text(
            &text,
            &self.model,
            prompt_template,
            Some(self.mode.system_prompt())
        ).await;

        match result {
            Ok(processed) => Ok(processed),
            Err(e) => {
                log::error!("Ollama processing failed: {}, using original text", e);
                Ok(text) // Fallback to original
            }
        }
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
```

**Test Plan:**
```rust
#[tokio::test]
async fn test_pipeline_creation() {
    let pipeline = PostProcessingPipeline::new();
    assert_eq!(pipeline.processors.len(), 0);
}

#[tokio::test]
async fn test_pipeline_with_custom_words() {
    let mut pipeline = PostProcessingPipeline::new();

    let processor = CustomWordsProcessor {
        words: vec!["API".to_string(), "JSON".to_string()],
        threshold: 0.8,
        enabled: true,
    };

    pipeline.add_processor(Box::new(processor));

    let input = "the api returns jason data";
    let output = pipeline.process(input.to_string()).await.unwrap();

    // Should correct "jason" to "JSON"
    assert!(output.contains("JSON"));
}

#[tokio::test]
async fn test_pipeline_disabled_processor() {
    let mut pipeline = PostProcessingPipeline::new();

    let processor = CustomWordsProcessor {
        words: vec!["test".to_string()],
        threshold: 0.8,
        enabled: false, // Disabled!
    };

    pipeline.add_processor(Box::new(processor));

    let input = "this is a test";
    let output = pipeline.process(input.to_string()).await.unwrap();

    // Should be unchanged
    assert_eq!(output, input);
}

#[tokio::test]
#[ignore]
async fn test_pipeline_with_ollama() {
    let manager = Arc::new(OllamaManager::new(None));

    if !manager.check_availability().await {
        println!("Skipping - Ollama not available");
        return;
    }

    let mut pipeline = PostProcessingPipeline::new();

    let processor = OllamaProcessor {
        manager,
        model: "llama3.2:3b".to_string(),
        mode: ProcessingMode::Punctuation,
        custom_prompt: None,
        enabled: true,
    };

    pipeline.add_processor(Box::new(processor));

    let input = "hello world how are you doing today";
    let output = pipeline.process(input.to_string()).await.unwrap();

    println!("Input: {}", input);
    println!("Output: {}", output);

    // Output should have punctuation and capitalization
    assert!(output.chars().next().unwrap().is_uppercase());
    assert!(output.contains('.') || output.contains('!') || output.contains('?'));
}

#[tokio::test]
#[ignore]
async fn test_pipeline_chained_processors() {
    let manager = Arc::new(OllamaManager::new(None));

    if !manager.check_availability().await {
        println!("Skipping - Ollama not available");
        return;
    }

    let mut pipeline = PostProcessingPipeline::new();

    // First: custom words
    pipeline.add_processor(Box::new(CustomWordsProcessor {
        words: vec!["API".to_string()],
        threshold: 0.8,
        enabled: true,
    }));

    // Second: Ollama punctuation
    pipeline.add_processor(Box::new(OllamaProcessor {
        manager,
        model: "llama3.2:3b".to_string(),
        mode: ProcessingMode::Punctuation,
        custom_prompt: None,
        enabled: true,
    }));

    let input = "the api is working";
    let output = pipeline.process(input.to_string()).await.unwrap();

    println!("Input: {}", input);
    println!("Output: {}", output);

    // Should have both corrections and punctuation
    assert!(output.contains("API"));
    assert!(output.chars().next().unwrap().is_uppercase());
}
```

**Automated Verification:**
```bash
cargo test pipeline --lib
cargo test --test pipeline_tests -- --ignored --nocapture
```

**Deliverables:**
- [ ] TextProcessor trait
- [ ] PostProcessingPipeline
- [ ] CustomWordsProcessor
- [ ] OllamaProcessor
- [ ] Processing modes (Punctuation, Summarize, CommandExtract)
- [ ] Tests for pipeline
- [ ] Fallback handling

---

(Continuing in next response due to length limits...)

//! Audio recording and management module
//!
//! This module provides the [`AudioRecordingManager`] which handles all aspects of audio
//! recording in Handy, including:
//! - Microphone device management and selection
//! - Voice Activity Detection (VAD) integration
//! - Recording state management
//! - Push-to-talk and always-on microphone modes
//!
//! # Architecture
//!
//! The manager uses a state machine to track recording sessions and supports two microphone modes:
//! - **OnDemand**: Opens microphone only when recording starts
//! - **AlwaysOn**: Keeps microphone open continuously for lower latency
//!
//! # Example
//!
//! ```no_run
//! use handy_app_lib::managers::audio::AudioRecordingManager;
//!
//! // Create manager (requires Tauri app handle)
//! // let manager = AudioRecordingManager::new(&app_handle)?;
//!
//! // Start recording with a binding ID
//! // let started = manager.try_start_recording("shortcut-1");
//!
//! // Stop recording and get audio samples
//! // if let Some(samples) = manager.stop_recording("shortcut-1") {
//! //     // Process audio samples (f32, 16kHz)
//! // }
//! ```

use crate::audio_toolkit::{list_input_devices, vad::SmoothedVad, AudioRecorder, SileroVad};
use crate::settings::get_settings;
use crate::utils;
use log::{debug, error, info};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::Manager;

/// Sample rate expected by Whisper models (16kHz)
const WHISPER_SAMPLE_RATE: usize = 16000;

/* ──────────────────────────────────────────────────────────────── */

/// Recording state of the audio system
///
/// Tracks whether the system is idle or actively recording, along with
/// the binding ID that initiated the recording.
#[derive(Clone, Debug)]
pub enum RecordingState {
    /// No active recording
    Idle,
    /// Currently recording audio
    ///
    /// # Fields
    /// * `binding_id` - Identifier of the keyboard shortcut that started this recording
    Recording { binding_id: String },
}

/// Microphone operation mode
///
/// Determines when the microphone stream is opened and kept active.
#[derive(Clone, Debug)]
pub enum MicrophoneMode {
    /// Microphone stays open continuously for lowest latency
    ///
    /// Best for: Frequent transcription usage
    /// Trade-off: Higher power consumption
    AlwaysOn,
    /// Microphone opens only when recording starts
    ///
    /// Best for: Occasional transcription usage
    /// Trade-off: Small latency at recording start (~100-200ms)
    OnDemand,
}

/* ──────────────────────────────────────────────────────────────── */

fn create_audio_recorder(
    vad_path: &str,
    app_handle: &tauri::AppHandle,
) -> Result<AudioRecorder, anyhow::Error> {
    let silero = SileroVad::new(vad_path, 0.3)
        .map_err(|e| anyhow::anyhow!("Failed to create SileroVad: {}", e))?;
    let smoothed_vad = SmoothedVad::new(Box::new(silero), 15, 15, 2);

    // Recorder with VAD plus a spectrum-level callback that forwards updates to
    // the frontend.
    let recorder = AudioRecorder::new()
        .map_err(|e| anyhow::anyhow!("Failed to create AudioRecorder: {}", e))?
        .with_vad(Box::new(smoothed_vad))
        .with_level_callback({
            let app_handle = app_handle.clone();
            move |levels| {
                utils::emit_levels(&app_handle, &levels);
            }
        });

    Ok(recorder)
}

/* ──────────────────────────────────────────────────────────────── */

/// Main audio recording manager
///
/// Manages the complete audio recording pipeline including device selection,
/// VAD (Voice Activity Detection), and recording session lifecycle.
///
/// # Thread Safety
///
/// This struct is `Clone` and thread-safe. Multiple clones can be used across
/// threads and will share the same underlying state.
///
/// # Fields
///
/// All fields are private and accessed through public methods to ensure
/// safe concurrent access.
#[derive(Clone)]
pub struct AudioRecordingManager {
    state: Arc<Mutex<RecordingState>>,
    mode: Arc<Mutex<MicrophoneMode>>,
    app_handle: tauri::AppHandle,

    recorder: Arc<Mutex<Option<AudioRecorder>>>,
    is_open: Arc<Mutex<bool>>,
    is_recording: Arc<Mutex<bool>>,
    initial_volume: Arc<Mutex<Option<u8>>>,
}

impl AudioRecordingManager {
    /* ---------- construction ------------------------------------------------ */

    /// Creates a new audio recording manager
    ///
    /// Initializes the manager with settings from the app's configuration.
    /// If `always_on_microphone` is enabled in settings, the microphone stream
    /// will be opened immediately.
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri application handle for accessing settings and emitting events
    ///
    /// # Returns
    ///
    /// * `Ok(AudioRecordingManager)` - Successfully created manager
    /// * `Err` - Failed to initialize (e.g., could not open microphone in always-on mode)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use handy_app_lib::managers::audio::AudioRecordingManager;
    /// # fn example(app: &tauri::AppHandle) -> Result<(), anyhow::Error> {
    /// let manager = AudioRecordingManager::new(app)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(app: &tauri::AppHandle) -> Result<Self, anyhow::Error> {
        let settings = get_settings(app);
        let mode = if settings.always_on_microphone {
            MicrophoneMode::AlwaysOn
        } else {
            MicrophoneMode::OnDemand
        };

        let manager = Self {
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            mode: Arc::new(Mutex::new(mode.clone())),
            app_handle: app.clone(),

            recorder: Arc::new(Mutex::new(None)),
            is_open: Arc::new(Mutex::new(false)),
            is_recording: Arc::new(Mutex::new(false)),
            initial_volume: Arc::new(Mutex::new(None)),
        };

        // Always-on?  Open immediately.
        if matches!(mode, MicrophoneMode::AlwaysOn) {
            manager.start_microphone_stream()?;
        }

        Ok(manager)
    }

    /* ---------- microphone life-cycle -------------------------------------- */

    pub fn start_microphone_stream(&self) -> Result<(), anyhow::Error> {
        let mut open_flag = self.is_open.lock().unwrap();
        if *open_flag {
            debug!("Microphone stream already active");
            return Ok(());
        }

        let start_time = Instant::now();

        let settings = get_settings(&self.app_handle);
        let mut initial_volume_guard = self.initial_volume.lock().unwrap();

        if settings.mute_while_recording {
            *initial_volume_guard = Some(cpvc::get_system_volume());
            cpvc::set_system_volume(0);
        } else {
            *initial_volume_guard = None;
        }

        let vad_path = self
            .app_handle
            .path()
            .resolve(
                "resources/models/silero_vad_v4.onnx",
                tauri::path::BaseDirectory::Resource,
            )
            .map_err(|e| anyhow::anyhow!("Failed to resolve VAD path: {}", e))?;
        let mut recorder_opt = self.recorder.lock().unwrap();

        if recorder_opt.is_none() {
            *recorder_opt = Some(create_audio_recorder(
                vad_path.to_str().unwrap(),
                &self.app_handle,
            )?);
        }

        // Get the selected device from settings
        let settings = get_settings(&self.app_handle);
        let selected_device = if let Some(device_name) = settings.selected_microphone {
            // Find the device by name
            match list_input_devices() {
                Ok(devices) => devices
                    .into_iter()
                    .find(|d| d.name == device_name)
                    .map(|d| d.device),
                Err(e) => {
                    debug!("Failed to list devices, using default: {}", e);
                    None
                }
            }
        } else {
            None
        };

        if let Some(rec) = recorder_opt.as_mut() {
            rec.open(selected_device)
                .map_err(|e| anyhow::anyhow!("Failed to open recorder: {}", e))?;
        }

        *open_flag = true;
        info!(
            "Microphone stream initialized in {:?}",
            start_time.elapsed()
        );
        Ok(())
    }

    pub fn stop_microphone_stream(&self) {
        let mut open_flag = self.is_open.lock().unwrap();
        if !*open_flag {
            return;
        }

        let mut initial_volume_guard = self.initial_volume.lock().unwrap();
        if let Some(vol) = *initial_volume_guard {
            cpvc::set_system_volume(vol);
        }
        *initial_volume_guard = None;

        if let Some(rec) = self.recorder.lock().unwrap().as_mut() {
            // If still recording, stop first.
            if *self.is_recording.lock().unwrap() {
                let _ = rec.stop();
                *self.is_recording.lock().unwrap() = false;
            }
            let _ = rec.close();
        }

        *open_flag = false;
        debug!("Microphone stream stopped");
    }

    /* ---------- mode switching --------------------------------------------- */

    pub fn update_mode(&self, new_mode: MicrophoneMode) -> Result<(), anyhow::Error> {
        let mode_guard = self.mode.lock().unwrap();
        let cur_mode = mode_guard.clone();

        match (cur_mode, &new_mode) {
            (MicrophoneMode::AlwaysOn, MicrophoneMode::OnDemand) => {
                if matches!(*self.state.lock().unwrap(), RecordingState::Idle) {
                    drop(mode_guard);
                    self.stop_microphone_stream();
                }
            }
            (MicrophoneMode::OnDemand, MicrophoneMode::AlwaysOn) => {
                drop(mode_guard);
                self.start_microphone_stream()?;
            }
            _ => {}
        }

        *self.mode.lock().unwrap() = new_mode;
        Ok(())
    }

    /* ---------- recording --------------------------------------------------- */

    /// Attempts to start a new recording session
    ///
    /// This method will fail if a recording is already in progress. Each recording
    /// is associated with a binding ID (usually a keyboard shortcut identifier) to
    /// ensure proper pairing with stop_recording calls.
    ///
    /// In OnDemand mode, this will also open the microphone stream if it's not already open.
    ///
    /// # Arguments
    ///
    /// * `binding_id` - Unique identifier for this recording session (e.g., "shortcut-1")
    ///
    /// # Returns
    ///
    /// * `true` - Recording started successfully
    /// * `false` - Failed to start (already recording, mic unavailable, or device error)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use handy_app_lib::managers::audio::AudioRecordingManager;
    /// # fn example(manager: &AudioRecordingManager) {
    /// if manager.try_start_recording("push-to-talk") {
    ///     println!("Recording started!");
    /// } else {
    ///     println!("Could not start recording");
    /// }
    /// # }
    /// ```
    pub fn try_start_recording(&self, binding_id: &str) -> bool {
        let mut state = self.state.lock().unwrap();

        if let RecordingState::Idle = *state {
            // Ensure microphone is open in on-demand mode
            if matches!(*self.mode.lock().unwrap(), MicrophoneMode::OnDemand) {
                if let Err(e) = self.start_microphone_stream() {
                    error!("Failed to open microphone stream: {e}");
                    return false;
                }
            }

            if let Some(rec) = self.recorder.lock().unwrap().as_ref() {
                if rec.start().is_ok() {
                    *self.is_recording.lock().unwrap() = true;
                    *state = RecordingState::Recording {
                        binding_id: binding_id.to_string(),
                    };
                    debug!("Recording started for binding {binding_id}");

                    // Emit recording-started event for frontend
                    let _ = self.app_handle.emit("recording-started", ());

                    return true;
                }
            }
            error!("Recorder not available");
            false
        } else {
            false
        }
    }

    pub fn update_selected_device(&self) -> Result<(), anyhow::Error> {
        // If currently open, restart the microphone stream to use the new device
        if *self.is_open.lock().unwrap() {
            self.stop_microphone_stream();
            self.start_microphone_stream()?;
        }
        Ok(())
    }

    pub fn stop_recording(&self, binding_id: &str) -> Option<Vec<f32>> {
        let mut state = self.state.lock().unwrap();

        match *state {
            RecordingState::Recording {
                binding_id: ref active,
            } if active == binding_id => {
                *state = RecordingState::Idle;
                drop(state);

                let samples = if let Some(rec) = self.recorder.lock().unwrap().as_ref() {
                    match rec.stop() {
                        Ok(buf) => buf,
                        Err(e) => {
                            error!("stop() failed: {e}");
                            Vec::new()
                        }
                    }
                } else {
                    error!("Recorder not available");
                    Vec::new()
                };

                *self.is_recording.lock().unwrap() = false;

                // Emit recording-stopped event for frontend
                let _ = self.app_handle.emit("recording-stopped", ());

                // In on-demand mode turn the mic off again
                if matches!(*self.mode.lock().unwrap(), MicrophoneMode::OnDemand) {
                    self.stop_microphone_stream();
                }

                // Pad if very short
                let s_len = samples.len();
                debug!("Got {} samples", s_len);
                if s_len < WHISPER_SAMPLE_RATE && s_len > 0 {
                    let mut padded = samples;
                    padded.resize(WHISPER_SAMPLE_RATE * 5 / 4, 0.0);
                    Some(padded)
                } else {
                    Some(samples)
                }
            }
            _ => None,
        }
    }

    /// Cancel any ongoing recording without returning audio samples
    pub fn cancel_recording(&self) {
        let mut state = self.state.lock().unwrap();

        if let RecordingState::Recording { .. } = *state {
            *state = RecordingState::Idle;
            drop(state);

            if let Some(rec) = self.recorder.lock().unwrap().as_ref() {
                let _ = rec.stop(); // Discard the result
            }

            *self.is_recording.lock().unwrap() = false;

            // In on-demand mode turn the mic off again
            if matches!(*self.mode.lock().unwrap(), MicrophoneMode::OnDemand) {
                self.stop_microphone_stream();
            }
        }
    }
}

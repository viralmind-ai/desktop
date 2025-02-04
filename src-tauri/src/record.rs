use crate::axtree;
use crate::ffmpeg::{self, FFmpegRecorder};
use crate::input;
use crate::logger::Logger;
use crate::macos_screencapture::MacOSScreenRecorder;

enum Recorder {
    FFmpeg(FFmpegRecorder),
    #[cfg(target_os = "macos")]
    MacOS(MacOSScreenRecorder),
}

impl Recorder {
    fn start(&mut self) -> Result<(), String> {
        match self {
            Recorder::FFmpeg(recorder) => recorder.start(),
            #[cfg(target_os = "macos")]
            Recorder::MacOS(recorder) => recorder.start(),
        }
    }

    fn stop(&mut self) -> Result<(), String> {
        match self {
            Recorder::FFmpeg(recorder) => recorder.stop(),
            #[cfg(target_os = "macos")]
            Recorder::MacOS(recorder) => recorder.stop(),
        }
    }
}

use chrono::Local;
use display_info::DisplayInfo;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

#[derive(Default)]
pub struct QuestState {
    pub objectives_completed: Mutex<i32>,
}

// Global state for recording and logging
lazy_static::lazy_static! {
    static ref RECORDING_STATE: Arc<Mutex<Option<Recorder>>> = Arc::new(Mutex::new(None));
    static ref LOGGER_STATE: Arc<Mutex<Option<Logger>>> = Arc::new(Mutex::new(None));
}

fn get_session_path(app: &tauri::AppHandle) -> Result<(PathBuf, String), String> {
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings");

    std::fs::create_dir_all(&recordings_dir)
        .map_err(|e| format!("Failed to create recordings directory: {}", e))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    Ok((recordings_dir, timestamp))
}

#[tauri::command]
pub async fn start_recording(
    app: tauri::AppHandle,
    quest_state: State<'_, QuestState>,
) -> Result<(), String> {
    // Start screen recording
    let mut rec_state = RECORDING_STATE.lock().map_err(|e| e.to_string())?;
    if rec_state.is_some() {
        return Err("Recording already in progress".to_string());
    }

    // Initialize FFmpeg if not already done
    ffmpeg::init_ffmpeg()?;

    // Get paths for both video and log files
    let (recordings_dir, timestamp) = get_session_path(&app)?;
    let video_path = recordings_dir.join(format!("recording_{}.mp4", timestamp));

    println!("{}", video_path.display());

    // Get primary display info
    let displays = DisplayInfo::all().map_err(|e| format!("Failed to get display info: {}", e))?;
    let primary = displays
        .iter()
        .find(|d| d.is_primary)
        .or_else(|| displays.first())
        .ok_or_else(|| "No display found".to_string())?;

    // Reset quest state and emit recording started event
    *quest_state.objectives_completed.lock().unwrap() = 0;
    app.emit(
        "recording-status",
        serde_json::json!({
            "state": "recording"
        }),
    )
    .unwrap();

    let mut recorder = if cfg!(target_os = "macos") {
        Recorder::MacOS(MacOSScreenRecorder::new(video_path))
    } else {
        let input_format = if cfg!(target_os = "windows") {
            "gdigrab"
        } else if cfg!(target_os = "linux") {
            "x11grab"
        } else {
            return Err("Unsupported platform".to_string());
        };

        let input_device = if cfg!(target_os = "windows") {
            "desktop".to_string()
        } else {
            ":0.0".to_string() // X11 display
        };

        Recorder::FFmpeg(FFmpegRecorder::new_with_input(
            primary.width,
            primary.height,
            30,
            video_path,
            input_format.to_string(),
            input_device,
        ))
    };

    recorder.start()?;
    *rec_state = Some(recorder);

    // Start input logging and listening
    let mut log_state = LOGGER_STATE.lock().map_err(|e| e.to_string())?;
    if log_state.is_none() {
        *log_state = Some(Logger::new(&app)?);
    }

    // Start input listener
    input::start_input_listener(app.clone())?;

    // Start dump-tree polling
    axtree::start_dump_tree_polling(app.clone())?;

    Ok(())
}

#[tauri::command]
pub async fn stop_recording(
    app: tauri::AppHandle,
    _quest_state: State<'_, QuestState>,
) -> Result<(), String> {
    // Emit recording stopping event
    app.emit(
        "recording-status",
        serde_json::json!({
            "state": "stopping"
        }),
    )
    .unwrap();

    // Stop input logging and listening first
    let mut log_state = LOGGER_STATE.lock().map_err(|e| e.to_string())?;
    *log_state = None;

    // Stop input listener
    input::stop_input_listener()?;

    // Stop dump-tree polling
    axtree::stop_dump_tree_polling()?;

    // Stop screen recording last since it might hang
    let mut rec_state = RECORDING_STATE.lock().map_err(|e| e.to_string())?;
    if let Some(mut recorder) = rec_state.take() {
        recorder.stop()?;
    }

    // Emit recording stopped event
    app.emit(
        "recording-status",
        serde_json::json!({
            "state": "stopped"
        }),
    )
    .unwrap();

    Ok(())
}

pub fn log_input(event: serde_json::Value) -> Result<(), String> {
    if let Ok(mut state) = LOGGER_STATE.lock() {
        if let Some(logger) = state.as_mut() {
            logger.log_event(event)?;
        }
    }
    Ok(())
}

pub fn log_ffmpeg(output: &str, is_stderr: bool) -> Result<(), String> {
    if let Ok(mut state) = LOGGER_STATE.lock() {
        if let Some(logger) = state.as_mut() {
            logger.log_ffmpeg(output, is_stderr)?;
        }
    }
    Ok(())
}

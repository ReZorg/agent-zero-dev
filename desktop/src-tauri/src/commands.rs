//! Agent Zero Desktop Application
//! 
//! This module contains IPC commands for communication between
//! the Tauri frontend and the Python backend sidecar.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Manager;

/// Backend status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub port: u16,
    pub ready: bool,
    pub error: Option<String>,
}

/// Application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub version: String,
    pub platform: String,
    pub arch: String,
    pub tauri_version: String,
}

/// Global state for tracking backend status
pub struct BackendState {
    pub running: AtomicBool,
    pub ready: AtomicBool,
    pub pid: std::sync::Mutex<Option<u32>>,
    pub error: std::sync::Mutex<Option<String>>,
}

impl Default for BackendState {
    fn default() -> Self {
        Self {
            running: AtomicBool::new(false),
            ready: AtomicBool::new(false),
            pid: std::sync::Mutex::new(None),
            error: std::sync::Mutex::new(None),
        }
    }
}

/// Get the current backend status
#[tauri::command]
pub fn get_backend_status(state: tauri::State<'_, Arc<BackendState>>) -> BackendStatus {
    let pid = state.pid.lock().unwrap().clone();
    let error = state.error.lock().unwrap().clone();
    
    BackendStatus {
        running: state.running.load(Ordering::SeqCst),
        pid,
        port: 50001,
        ready: state.ready.load(Ordering::SeqCst),
        error,
    }
}

/// Get application information
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        tauri_version: tauri::VERSION.to_string(),
    }
}

/// Open an external URL in the default browser
#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
}

/// Check if the backend health endpoint is responding
#[tauri::command]
pub async fn check_backend_health(port: u16) -> Result<bool, String> {
    let url = format!("http://localhost:{}/api/health", port);
    
    match reqwest::get(&url).await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

/// Set backend as ready (called when health check passes)
#[tauri::command]
pub fn set_backend_ready(state: tauri::State<'_, Arc<BackendState>>, ready: bool) {
    state.ready.store(ready, Ordering::SeqCst);
}

/// Set backend error message
#[tauri::command]
pub fn set_backend_error(state: tauri::State<'_, Arc<BackendState>>, error: Option<String>) {
    *state.error.lock().unwrap() = error;
}

/// Show the main window (called after backend is ready)
#[tauri::command]
pub async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

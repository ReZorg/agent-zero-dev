//! Agent Zero Desktop Application
//! 
//! A Tauri-based desktop shell for Agent Zero that manages a Python
//! backend sidecar process and provides a native application experience.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::{
    BackendState, check_backend_health, get_app_info, get_backend_status,
    open_external_url, set_backend_error, set_backend_ready, show_main_window,
};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

/// Default port for the Agent Zero backend
const BACKEND_PORT: u16 = 50001;

/// Maximum time to wait for backend to start (in seconds)
const BACKEND_STARTUP_TIMEOUT: u64 = 120;

/// Interval between health checks (in milliseconds)
const HEALTH_CHECK_INTERVAL: u64 = 500;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(BackendState::default()))
        .setup(|app| {
            let app_handle = app.handle().clone();
            let state = app.state::<Arc<BackendState>>().inner().clone();
            
            // Spawn the backend startup task
            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_backend(&app_handle, &state).await {
                    log::error!("Failed to start backend: {}", e);
                    *state.error.lock().unwrap() = Some(e.clone());
                    
                    // Show error dialog
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                    }
                }
            });
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Graceful shutdown of backend when window closes
                let app_handle = window.app_handle().clone();
                let state = app_handle.state::<Arc<BackendState>>();
                
                if state.running.load(Ordering::SeqCst) {
                    log::info!("Shutting down backend process...");
                    // The sidecar will be killed automatically by Tauri
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_backend_status,
            get_app_info,
            open_external_url,
            check_backend_health,
            set_backend_ready,
            set_backend_error,
            show_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Start the Python backend sidecar process
async fn start_backend(
    app_handle: &tauri::AppHandle,
    state: &Arc<BackendState>,
) -> Result<(), String> {
    log::info!("Starting Agent Zero backend...");
    
    // Get the sidecar command
    let sidecar_command = app_handle
        .shell()
        .sidecar("agent-zero-backend")
        .map_err(|e| format!("Failed to create sidecar command: {}", e))?
        .args([
            "--desktop-mode",
            "--port", &BACKEND_PORT.to_string(),
            "--host", "127.0.0.1",
        ]);
    
    // Spawn the sidecar process
    let (mut rx, child) = sidecar_command
        .spawn()
        .map_err(|e| format!("Failed to spawn sidecar: {}", e))?;
    
    // Store the PID
    *state.pid.lock().unwrap() = Some(child.pid());
    state.running.store(true, Ordering::SeqCst);
    
    log::info!("Backend process started with PID: {}", child.pid());
    
    // Clone state for the output reader task
    let state_clone = state.clone();
    let app_clone = app_handle.clone();
    
    // Spawn task to read stdout/stderr from the sidecar
    tauri::async_runtime::spawn(async move {
        use tauri_plugin_shell::process::CommandEvent;
        
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let line_str = String::from_utf8_lossy(&line);
                    log::info!("[Backend] {}", line_str);
                    
                    // Check for ready signal
                    if line_str.contains("Agent Zero is running") || 
                       line_str.contains("DESKTOP_READY") {
                        state_clone.ready.store(true, Ordering::SeqCst);
                        
                        // Show the main window
                        if let Some(window) = app_clone.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                CommandEvent::Stderr(line) => {
                    let line_str = String::from_utf8_lossy(&line);
                    log::warn!("[Backend Error] {}", line_str);
                }
                CommandEvent::Terminated(payload) => {
                    log::info!("Backend process terminated: {:?}", payload);
                    state_clone.running.store(false, Ordering::SeqCst);
                    state_clone.ready.store(false, Ordering::SeqCst);
                    *state_clone.pid.lock().unwrap() = None;
                    break;
                }
                _ => {}
            }
        }
    });
    
    // Wait for backend to be ready with timeout
    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(BACKEND_STARTUP_TIMEOUT);
    
    while start_time.elapsed() < timeout {
        // Check health endpoint
        if let Ok(true) = check_backend_health_internal(BACKEND_PORT).await {
            state.ready.store(true, Ordering::SeqCst);
            log::info!("Backend is ready!");
            
            // Show the main window
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
            
            return Ok(());
        }
        
        // Check if process is still running
        if !state.running.load(Ordering::SeqCst) {
            return Err("Backend process terminated unexpectedly".to_string());
        }
        
        tokio::time::sleep(Duration::from_millis(HEALTH_CHECK_INTERVAL)).await;
    }
    
    Err(format!(
        "Backend failed to start within {} seconds",
        BACKEND_STARTUP_TIMEOUT
    ))
}

/// Internal health check function
async fn check_backend_health_internal(port: u16) -> Result<bool, ()> {
    let url = format!("http://127.0.0.1:{}/api/health", port);
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|_| ())?;
    
    match client.get(&url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

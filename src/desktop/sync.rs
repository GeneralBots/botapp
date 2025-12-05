//! Rclone Sync Module for Desktop File Synchronization
//!
//! Provides bidirectional sync between local filesystem and remote S3 storage
//! using rclone as the underlying sync engine.
//!
//! Desktop-only feature: This runs rclone as a subprocess on the user's machine.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{Emitter, Window};

/// Global state for tracking the rclone process
static RCLONE_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

/// Sync status reported to the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub status: String,
    pub is_running: bool,
    pub last_sync: Option<String>,
    pub files_synced: u64,
    pub bytes_transferred: u64,
    pub current_file: Option<String>,
    pub error: Option<String>,
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub local_path: String,
    pub remote_name: String,
    pub remote_path: String,
    pub sync_mode: SyncMode,
    pub exclude_patterns: Vec<String>,
}

/// Sync direction/mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMode {
    /// Local changes pushed to remote
    Push,
    /// Remote changes pulled to local
    Pull,
    /// Bidirectional sync (newest wins)
    Bisync,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            local_path: dirs::home_dir()
                .map(|p| p.join("GeneralBots").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/GeneralBots".to_string()),
            remote_name: "gbdrive".to_string(),
            remote_path: "/".to_string(),
            sync_mode: SyncMode::Bisync,
            exclude_patterns: vec![
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                "*.tmp".to_string(),
                ".git/**".to_string(),
            ],
        }
    }
}

/// Get current sync status
#[tauri::command]
pub fn get_sync_status() -> SyncStatus {
    let process_guard = RCLONE_PROCESS.lock().unwrap();
    let is_running = process_guard.is_some();

    SyncStatus {
        status: if is_running {
            "syncing".to_string()
        } else {
            "idle".to_string()
        },
        is_running,
        last_sync: None,
        files_synced: 0,
        bytes_transferred: 0,
        current_file: None,
        error: None,
    }
}

/// Start rclone sync process
#[tauri::command]
pub async fn start_sync(window: Window, config: Option<SyncConfig>) -> Result<SyncStatus, String> {
    let config = config.unwrap_or_default();

    // Check if already running
    {
        let process_guard = RCLONE_PROCESS.lock().unwrap();
        if process_guard.is_some() {
            return Err("Sync already running".to_string());
        }
    }

    // Ensure local directory exists
    let local_path = PathBuf::from(&config.local_path);
    if !local_path.exists() {
        std::fs::create_dir_all(&local_path)
            .map_err(|e| format!("Failed to create local directory: {}", e))?;
    }

    // Build rclone command
    let mut cmd = Command::new("rclone");

    // Set sync mode
    match config.sync_mode {
        SyncMode::Push => {
            cmd.arg("sync");
            cmd.arg(&config.local_path);
            cmd.arg(format!("{}:{}", config.remote_name, config.remote_path));
        }
        SyncMode::Pull => {
            cmd.arg("sync");
            cmd.arg(format!("{}:{}", config.remote_name, config.remote_path));
            cmd.arg(&config.local_path);
        }
        SyncMode::Bisync => {
            cmd.arg("bisync");
            cmd.arg(&config.local_path);
            cmd.arg(format!("{}:{}", config.remote_name, config.remote_path));
            cmd.arg("--resync"); // First run needs resync
        }
    }

    // Add common options
    cmd.arg("--progress").arg("--verbose").arg("--checksum"); // Use checksums for accuracy

    // Add exclude patterns
    for pattern in &config.exclude_patterns {
        cmd.arg("--exclude").arg(pattern);
    }

    // Configure output capture
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    // Spawn the process
    let child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "rclone not found. Please install rclone: https://rclone.org/install/".to_string()
        } else {
            format!("Failed to start rclone: {}", e)
        }
    })?;

    // Store the process handle
    {
        let mut process_guard = RCLONE_PROCESS.lock().unwrap();
        *process_guard = Some(child);
    }

    // Emit started event
    let _ = window.emit("sync_started", ());

    // Spawn a task to monitor the process
    let window_clone = window.clone();
    std::thread::spawn(move || {
        monitor_sync_process(window_clone);
    });

    Ok(SyncStatus {
        status: "syncing".to_string(),
        is_running: true,
        last_sync: None,
        files_synced: 0,
        bytes_transferred: 0,
        current_file: None,
        error: None,
    })
}

/// Stop rclone sync process
#[tauri::command]
pub fn stop_sync() -> Result<SyncStatus, String> {
    let mut process_guard = RCLONE_PROCESS.lock().unwrap();

    if let Some(mut child) = process_guard.take() {
        // Try graceful termination first
        #[cfg(unix)]
        {
            unsafe {
                libc::kill(child.id() as i32, libc::SIGTERM);
            }
        }

        #[cfg(windows)]
        {
            let _ = child.kill();
        }

        // Wait briefly for graceful shutdown
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Force kill if still running
        let _ = child.kill();
        let _ = child.wait();

        Ok(SyncStatus {
            status: "stopped".to_string(),
            is_running: false,
            last_sync: Some(chrono::Utc::now().to_rfc3339()),
            files_synced: 0,
            bytes_transferred: 0,
            current_file: None,
            error: None,
        })
    } else {
        Err("No sync process running".to_string())
    }
}

/// Configure rclone remote for S3/MinIO
#[tauri::command]
pub fn configure_remote(
    remote_name: String,
    endpoint: String,
    access_key: String,
    secret_key: String,
    bucket: String,
) -> Result<(), String> {
    // Use rclone config create command
    let output = Command::new("rclone")
        .args([
            "config",
            "create",
            &remote_name,
            "s3",
            "provider",
            "Minio",
            "endpoint",
            &endpoint,
            "access_key_id",
            &access_key,
            "secret_access_key",
            &secret_key,
            "acl",
            "private",
        ])
        .output()
        .map_err(|e| format!("Failed to configure rclone: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("rclone config failed: {}", stderr));
    }

    // Set default bucket path
    let _ = Command::new("rclone")
        .args(["config", "update", &remote_name, "bucket", &bucket])
        .output();

    Ok(())
}

/// Check if rclone is installed
#[tauri::command]
pub fn check_rclone_installed() -> Result<String, String> {
    let output = Command::new("rclone")
        .arg("version")
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "rclone not installed".to_string()
            } else {
                format!("Error checking rclone: {}", e)
            }
        })?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        let first_line = version.lines().next().unwrap_or("unknown");
        Ok(first_line.to_string())
    } else {
        Err("rclone check failed".to_string())
    }
}

/// List configured rclone remotes
#[tauri::command]
pub fn list_remotes() -> Result<Vec<String>, String> {
    let output = Command::new("rclone")
        .args(["listremotes"])
        .output()
        .map_err(|e| format!("Failed to list remotes: {}", e))?;

    if output.status.success() {
        let remotes = String::from_utf8_lossy(&output.stdout);
        Ok(remotes
            .lines()
            .map(|s| s.trim_end_matches(':').to_string())
            .filter(|s| !s.is_empty())
            .collect())
    } else {
        Err("Failed to list rclone remotes".to_string())
    }
}

/// Get sync folder path
#[tauri::command]
pub fn get_sync_folder() -> String {
    dirs::home_dir()
        .map(|p| p.join("GeneralBots").to_string_lossy().to_string())
        .unwrap_or_else(|| "~/GeneralBots".to_string())
}

/// Set sync folder path
#[tauri::command]
pub fn set_sync_folder(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    // Store in app config (would need app handle for persistent storage)
    // For now, just validate the path
    Ok(())
}

/// Monitor the sync process and emit events
fn monitor_sync_process(window: Window) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let mut process_guard = RCLONE_PROCESS.lock().unwrap();

        if let Some(ref mut child) = *process_guard {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process finished
                    let success = status.success();
                    *process_guard = None;

                    let status = SyncStatus {
                        status: if success {
                            "completed".to_string()
                        } else {
                            "error".to_string()
                        },
                        is_running: false,
                        last_sync: Some(chrono::Utc::now().to_rfc3339()),
                        files_synced: 0,
                        bytes_transferred: 0,
                        current_file: None,
                        error: if success {
                            None
                        } else {
                            Some(format!("Exit code: {:?}", status.code()))
                        },
                    };

                    let _ = window.emit("sync_completed", &status);
                    break;
                }
                Ok(None) => {
                    // Still running - emit progress
                    let status = SyncStatus {
                        status: "syncing".to_string(),
                        is_running: true,
                        last_sync: None,
                        files_synced: 0,
                        bytes_transferred: 0,
                        current_file: None,
                        error: None,
                    };
                    let _ = window.emit("sync_progress", &status);
                }
                Err(e) => {
                    // Error checking status
                    *process_guard = None;

                    let status = SyncStatus {
                        status: "error".to_string(),
                        is_running: false,
                        last_sync: Some(chrono::Utc::now().to_rfc3339()),
                        files_synced: 0,
                        bytes_transferred: 0,
                        current_file: None,
                        error: Some(format!("Process error: {}", e)),
                    };

                    let _ = window.emit("sync_error", &status);
                    break;
                }
            }
        } else {
            // No process running
            break;
        }
    }
}

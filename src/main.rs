#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! BotApp - Tauri Desktop Application for General Bots
//!
//! This is the entry point for the native desktop application.
//! It wraps botui's pure web UI with Tauri for native capabilities.

use log::info;

mod desktop;

fn main() {
    env_logger::init();
    info!("BotApp starting (Tauri)...");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Drive commands
            desktop::drive::list_files,
            desktop::drive::upload_file,
            desktop::drive::create_folder,
            desktop::drive::delete_path,
            desktop::drive::get_home_dir,
            // Sync commands (rclone-based)
            desktop::sync::get_sync_status,
            desktop::sync::start_sync,
            desktop::sync::stop_sync,
            desktop::sync::configure_remote,
            desktop::sync::check_rclone_installed,
            desktop::sync::list_remotes,
            desktop::sync::get_sync_folder,
            desktop::sync::set_sync_folder,
        ])
        .setup(|_app| {
            info!("BotApp setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

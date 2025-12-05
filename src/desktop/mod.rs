//! Desktop-specific functionality for BotApp
//!
//! This module provides native desktop capabilities:
//! - Drive/file management via Tauri
//! - System tray integration
//! - Rclone-based file synchronization (desktop only)

pub mod drive;
pub mod sync;
pub mod tray;

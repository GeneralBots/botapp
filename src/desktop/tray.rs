//! Tray manager for desktop application.
//!
//! Provides system tray functionality for different operating modes.

use anyhow::Result;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages the system tray icon and its interactions.
#[derive(Clone, Debug)]
pub struct TrayManager {
    hostname: Arc<RwLock<Option<String>>>,
    running_mode: RunningMode,
    tray_active: Arc<RwLock<bool>>,
}

/// The running mode of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunningMode {
    /// Full server mode with all services.
    Server,
    /// Desktop mode with UI and local services.
    Desktop,
    /// Client mode connecting to remote server.
    Client,
}

/// Events that can be triggered from the tray menu.
#[derive(Debug, Clone, Copy)]
pub enum TrayEvent {
    /// Open the main application window.
    Open,
    /// Open settings dialog.
    Settings,
    /// Show about dialog.
    About,
    /// Quit the application.
    Quit,
}

impl TrayManager {
    /// Creates a new tray manager with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            hostname: Arc::new(RwLock::new(None)),
            running_mode: RunningMode::Desktop,
            tray_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Creates a new tray manager with the specified running mode.
    #[must_use]
    pub fn with_mode(mode: RunningMode) -> Self {
        Self {
            hostname: Arc::new(RwLock::new(None)),
            running_mode: mode,
            tray_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Starts the tray manager based on the running mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the tray initialization fails.
    pub async fn start(&self) -> Result<()> {
        match self.running_mode {
            RunningMode::Desktop => {
                self.start_desktop_mode().await?;
            }
            RunningMode::Server => {
                log::info!("Running in server mode - tray icon disabled");
            }
            RunningMode::Client => {
                self.start_client_mode().await;
            }
        }
        Ok(())
    }

    async fn start_desktop_mode(&self) -> Result<()> {
        log::info!("Starting desktop mode tray icon");
        let mut active = self.tray_active.write().await;
        *active = true;
        drop(active);

        #[cfg(target_os = "linux")]
        self.setup_linux_tray();

        #[cfg(target_os = "windows")]
        self.setup_windows_tray();

        #[cfg(target_os = "macos")]
        self.setup_macos_tray();

        Ok(())
    }

    async fn start_client_mode(&self) {
        log::info!("Starting client mode with minimal tray");
        let mut active = self.tray_active.write().await;
        *active = true;
        drop(active);
    }

    #[cfg(target_os = "linux")]
    fn setup_linux_tray(&self) {
        log::info!(
            "Initializing Linux system tray via DBus/StatusNotifierItem for mode: {:?}",
            self.running_mode
        );
    }

    #[cfg(target_os = "windows")]
    fn setup_windows_tray(&self) {
        log::info!(
            "Initializing Windows system tray via Shell_NotifyIcon for mode: {:?}",
            self.running_mode
        );
    }

    #[cfg(target_os = "macos")]
    fn setup_macos_tray(&self) {
        log::info!(
            "Initializing macOS menu bar via NSStatusItem for mode: {:?}",
            self.running_mode
        );
    }

    /// Returns a string representation of the current running mode.
    #[must_use]
    pub fn get_mode_string(&self) -> String {
        match self.running_mode {
            RunningMode::Desktop => "Desktop".to_string(),
            RunningMode::Server => "Server".to_string(),
            RunningMode::Client => "Client".to_string(),
        }
    }

    /// Updates the tray status message.
    ///
    /// # Errors
    ///
    /// Returns an error if the status update fails.
    pub async fn update_status(&self, status: &str) -> Result<()> {
        let active = self.tray_active.read().await;
        let is_active = *active;
        drop(active);

        if is_active {
            log::info!("Tray status: {status}");
        }
        Ok(())
    }

    /// Sets the tray tooltip text.
    ///
    /// # Errors
    ///
    /// Returns an error if setting the tooltip fails.
    pub async fn set_tooltip(&self, tooltip: &str) -> Result<()> {
        let active = self.tray_active.read().await;
        let is_active = *active;
        drop(active);

        if is_active {
            log::debug!("Tray tooltip: {tooltip}");
        }
        Ok(())
    }

    /// Shows a desktop notification.
    ///
    /// # Errors
    ///
    /// Returns an error if showing the notification fails.
    pub async fn show_notification(&self, title: &str, body: &str) -> Result<()> {
        let active = self.tray_active.read().await;
        let is_active = *active;
        drop(active);

        if is_active {
            log::info!("Notification: {title} - {body}");

            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("notify-send")
                    .arg(title)
                    .arg(body)
                    .spawn();
            }

            #[cfg(target_os = "macos")]
            {
                let script = format!("display notification \"{body}\" with title \"{title}\"");
                let _ = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(&script)
                    .spawn();
            }
        }
        Ok(())
    }

    /// Gets the current hostname.
    pub async fn get_hostname(&self) -> Option<String> {
        let hostname = self.hostname.read().await;
        hostname.clone()
    }

    /// Sets the hostname.
    pub async fn set_hostname(&self, new_hostname: String) {
        let mut hostname = self.hostname.write().await;
        *hostname = Some(new_hostname);
    }

    /// Stops the tray manager.
    ///
    /// # Errors
    ///
    /// Returns an error if stopping fails.
    pub async fn stop(&self) {
        let mut active = self.tray_active.write().await;
        *active = false;
        drop(active);
        log::info!("Tray manager stopped");
    }

    /// Returns whether the tray is currently active.
    pub async fn is_active(&self) -> bool {
        let active = self.tray_active.read().await;
        let result = *active;
        drop(active);
        result
    }

    /// Handles a tray event and performs the appropriate action.
    pub fn handle_event(&self, event: TrayEvent) {
        let mode = self.get_mode_string();
        match event {
            TrayEvent::Open => {
                log::info!("Tray event: Open main window (mode: {mode})");
            }
            TrayEvent::Settings => {
                log::info!("Tray event: Open settings (mode: {mode})");
            }
            TrayEvent::About => {
                log::info!("Tray event: Show about dialog (mode: {mode})");
            }
            TrayEvent::Quit => {
                log::info!("Tray event: Quit application (mode: {mode})");
            }
        }
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Monitors the status of services.
#[derive(Debug)]
pub struct ServiceMonitor {
    services: Vec<ServiceStatus>,
}

/// Status of a monitored service.
#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    /// Service name.
    pub name: String,
    /// Whether the service is running.
    pub running: bool,
    /// Service port number.
    pub port: u16,
    /// Service URL.
    pub url: String,
}

impl ServiceMonitor {
    /// Creates a new service monitor with default services.
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: vec![
                ServiceStatus {
                    name: "API".to_string(),
                    running: false,
                    port: 8080,
                    url: "http://localhost:8080".to_string(),
                },
                ServiceStatus {
                    name: "UI".to_string(),
                    running: false,
                    port: 3000,
                    url: "http://localhost:3000".to_string(),
                },
            ],
        }
    }

    /// Adds a service to monitor.
    pub fn add_service(&mut self, name: &str, port: u16) {
        self.services.push(ServiceStatus {
            name: name.to_string(),
            running: false,
            port,
            url: format!("http://localhost:{port}"),
        });
    }

    /// Checks all services and returns their current status.
    pub async fn check_services(&mut self) -> Vec<ServiceStatus> {
        for service in &mut self.services {
            service.running = Self::check_service(&service.url).await;
        }
        self.services.clone()
    }

    /// Checks if a service is running at the given URL.
    pub async fn check_service(url: &str) -> bool {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return false;
        }

        let Ok(client) = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(2))
            .build()
        else {
            return false;
        };

        let health_url = format!("{}/health", url.trim_end_matches('/'));

        client
            .get(&health_url)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
    }

    /// Gets a service by name.
    #[must_use]
    pub fn get_service(&self, name: &str) -> Option<&ServiceStatus> {
        self.services.iter().find(|s| s.name == name)
    }

    /// Returns whether all services are running.
    #[must_use]
    pub fn all_running(&self) -> bool {
        self.services.iter().all(|s| s.running)
    }

    /// Returns whether any service is running.
    #[must_use]
    pub fn any_running(&self) -> bool {
        self.services.iter().any(|s| s.running)
    }
}

impl Default for ServiceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

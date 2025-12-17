//! System Tray functionality for BotApp
//!
//! Provides system tray icon and menu for desktop platforms.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tray manager handles system tray icon and interactions
pub struct TrayManager {
    hostname: Arc<RwLock<Option<String>>>,
    running_mode: RunningMode,
}

/// Running mode for the application
#[derive(Debug, Clone, PartialEq)]
pub enum RunningMode {
    Server,
    Desktop,
    Client,
}

impl TrayManager {
    /// Create a new TrayManager
    pub fn new() -> Self {
        Self {
            hostname: Arc::new(RwLock::new(None)),
            running_mode: RunningMode::Desktop,
        }
    }

    /// Start the tray icon
    pub async fn start(&self) -> Result<()> {
        match self.running_mode {
            RunningMode::Desktop => {
                self.start_desktop_mode().await?;
            }
            RunningMode::Server => {
                log::info!("Running in server mode - tray icon disabled");
            }
            RunningMode::Client => {
                log::info!("Running in client mode - tray icon minimal");
            }
        }
        Ok(())
    }

    async fn start_desktop_mode(&self) -> Result<()> {
        log::info!("Starting desktop mode tray icon");
        // Platform-specific tray implementation would go here
        // For now, this is a placeholder
        Ok(())
    }

    /// Get mode as string
    pub fn get_mode_string(&self) -> String {
        match self.running_mode {
            RunningMode::Desktop => "Desktop".to_string(),
            RunningMode::Server => "Server".to_string(),
            RunningMode::Client => "Client".to_string(),
        }
    }

    /// Update tray status
    pub async fn update_status(&self, status: &str) -> Result<()> {
        log::info!("Tray status update: {}", status);
        Ok(())
    }

    /// Get hostname
    pub async fn get_hostname(&self) -> Option<String> {
        let hostname = self.hostname.read().await;
        hostname.clone()
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Service status monitor
pub struct ServiceMonitor {
    services: Vec<ServiceStatus>,
}

/// Status of a service
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    pub port: u16,
    pub url: String,
}

impl ServiceMonitor {
    /// Create a new service monitor with default services
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

    /// Check all services
    pub async fn check_services(&mut self) -> Vec<ServiceStatus> {
        for service in &mut self.services {
            service.running = Self::check_service(&service.url).await;
        }
        self.services.clone()
    }

    async fn check_service(url: &str) -> bool {
        if url.starts_with("http://") || url.starts_with("https://") {
            match reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
            {
                Ok(client) => {
                    match client
                        .get(format!("{}/health", url))
                        .timeout(std::time::Duration::from_secs(2))
                        .send()
                        .await
                    {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }
}

impl Default for ServiceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

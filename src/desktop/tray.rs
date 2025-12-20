use anyhow::Result;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct TrayManager {
    hostname: Arc<RwLock<Option<String>>>,
    running_mode: RunningMode,
    tray_active: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunningMode {
    Server,
    Desktop,
    Client,
}

#[derive(Debug, Clone)]
pub enum TrayEvent {
    Open,
    Settings,
    About,
    Quit,
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            hostname: Arc::new(RwLock::new(None)),
            running_mode: RunningMode::Desktop,
            tray_active: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_mode(mode: RunningMode) -> Self {
        Self {
            hostname: Arc::new(RwLock::new(None)),
            running_mode: mode,
            tray_active: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        match self.running_mode {
            RunningMode::Desktop => {
                self.start_desktop_mode().await?;
            }
            RunningMode::Server => {
                log::info!("Running in server mode - tray icon disabled");
            }
            RunningMode::Client => {
                self.start_client_mode().await?;
            }
        }
        Ok(())
    }

    async fn start_desktop_mode(&self) -> Result<()> {
        log::info!("Starting desktop mode tray icon");

        let mut active = self.tray_active.write().await;
        *active = true;

        #[cfg(target_os = "linux")]
        {
            self.setup_linux_tray().await?;
        }

        #[cfg(target_os = "windows")]
        {
            self.setup_windows_tray().await?;
        }

        #[cfg(target_os = "macos")]
        {
            self.setup_macos_tray().await?;
        }

        Ok(())
    }

    async fn start_client_mode(&self) -> Result<()> {
        log::info!("Starting client mode with minimal tray");
        let mut active = self.tray_active.write().await;
        *active = true;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn setup_linux_tray(&self) -> Result<()> {
        log::info!("Initializing Linux system tray via DBus/StatusNotifierItem");
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn setup_windows_tray(&self) -> Result<()> {
        log::info!("Initializing Windows system tray via Shell_NotifyIcon");
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn setup_macos_tray(&self) -> Result<()> {
        log::info!("Initializing macOS menu bar via NSStatusItem");
        Ok(())
    }

    pub fn get_mode_string(&self) -> String {
        match self.running_mode {
            RunningMode::Desktop => "Desktop".to_string(),
            RunningMode::Server => "Server".to_string(),
            RunningMode::Client => "Client".to_string(),
        }
    }

    pub async fn update_status(&self, status: &str) -> Result<()> {
        let active = self.tray_active.read().await;
        if *active {
            log::info!("Tray status: {}", status);
        }
        Ok(())
    }

    pub async fn set_tooltip(&self, tooltip: &str) -> Result<()> {
        let active = self.tray_active.read().await;
        if *active {
            log::debug!("Tray tooltip: {}", tooltip);
        }
        Ok(())
    }

    pub async fn show_notification(&self, title: &str, body: &str) -> Result<()> {
        let active = self.tray_active.read().await;
        if *active {
            log::info!("Notification: {} - {}", title, body);

            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("notify-send")
                    .arg(title)
                    .arg(body)
                    .spawn();
            }

            #[cfg(target_os = "macos")]
            {
                let script = format!("display notification \"{}\" with title \"{}\"", body, title);
                let _ = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(&script)
                    .spawn();
            }
        }
        Ok(())
    }

    pub async fn get_hostname(&self) -> Option<String> {
        let hostname = self.hostname.read().await;
        hostname.clone()
    }

    pub async fn set_hostname(&self, hostname: String) {
        let mut h = self.hostname.write().await;
        *h = Some(hostname);
    }

    pub async fn stop(&self) -> Result<()> {
        let mut active = self.tray_active.write().await;
        *active = false;
        log::info!("Tray manager stopped");
        Ok(())
    }

    pub async fn is_active(&self) -> bool {
        let active = self.tray_active.read().await;
        *active
    }

    pub fn handle_event(&self, event: TrayEvent) {
        match event {
            TrayEvent::Open => {
                log::info!("Tray event: Open main window");
            }
            TrayEvent::Settings => {
                log::info!("Tray event: Open settings");
            }
            TrayEvent::About => {
                log::info!("Tray event: Show about dialog");
            }
            TrayEvent::Quit => {
                log::info!("Tray event: Quit application");
            }
        }
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ServiceMonitor {
    services: Vec<ServiceStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    pub port: u16,
    pub url: String,
}

impl ServiceMonitor {
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

    pub fn add_service(&mut self, name: &str, port: u16) {
        self.services.push(ServiceStatus {
            name: name.to_string(),
            running: false,
            port,
            url: format!("http://localhost:{}", port),
        });
    }

    pub async fn check_services(&mut self) -> Vec<ServiceStatus> {
        for service in &mut self.services {
            service.running = Self::check_service(&service.url).await;
        }
        self.services.clone()
    }

    pub async fn check_service(url: &str) -> bool {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return false;
        }

        let client = match reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(2))
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        let health_url = format!("{}/health", url.trim_end_matches('/'));

        match client.get(&health_url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => match client.get(url).send().await {
                Ok(response) => response.status().is_success(),
                Err(_) => false,
            },
        }
    }

    pub fn get_service(&self, name: &str) -> Option<&ServiceStatus> {
        self.services.iter().find(|s| s.name == name)
    }

    pub fn all_running(&self) -> bool {
        self.services.iter().all(|s| s.running)
    }

    pub fn any_running(&self) -> bool {
        self.services.iter().any(|s| s.running)
    }
}

impl Default for ServiceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

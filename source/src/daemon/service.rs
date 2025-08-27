use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use log::{info, warn};

/// Service manager for installing and managing the Synx daemon as a system service
pub struct ServiceManager {
    service_name: String,
    binary_path: PathBuf,
    config_path: Option<PathBuf>,
}

impl ServiceManager {
    pub fn new(service_name: String, binary_path: PathBuf) -> Self {
        Self {
            service_name,
            binary_path,
            config_path: None,
        }
    }

    pub fn with_config_path(mut self, config_path: PathBuf) -> Self {
        self.config_path = Some(config_path);
        self
    }

    /// Install the service based on the current platform
    pub fn install(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.install_systemd_service()
        }

        #[cfg(target_os = "macos")]
        {
            self.install_launchd_service()
        }

        #[cfg(target_os = "windows")]
        {
            self.install_windows_service()
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow!("Service installation not supported on this platform"))
        }
    }

    /// Uninstall the service
    pub fn uninstall(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.uninstall_systemd_service()
        }

        #[cfg(target_os = "macos")]
        {
            self.uninstall_launchd_service()
        }

        #[cfg(target_os = "windows")]
        {
            self.uninstall_windows_service()
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow!("Service uninstallation not supported on this platform"))
        }
    }

    /// Start the service
    pub fn start(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.systemctl_command("start")
        }

        #[cfg(target_os = "macos")]
        {
            self.launchctl_command("load")
        }

        #[cfg(target_os = "windows")]
        {
            self.sc_command("start")
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow!("Service start not supported on this platform"))
        }
    }

    /// Stop the service
    pub fn stop(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.systemctl_command("stop")
        }

        #[cfg(target_os = "macos")]
        {
            self.launchctl_command("unload")
        }

        #[cfg(target_os = "windows")]
        {
            self.sc_command("stop")
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow!("Service stop not supported on this platform"))
        }
    }

    /// Enable the service to start on boot
    pub fn enable(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.systemctl_command("enable")
        }

        #[cfg(target_os = "macos")]
        {
            // launchd services are enabled by default when loaded
            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            // Windows services are configured during installation
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow!("Service enable not supported on this platform"))
        }
    }

    /// Disable the service from starting on boot
    pub fn disable(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.systemctl_command("disable")
        }

        #[cfg(target_os = "macos")]
        {
            // Handled by unloading in launchd
            self.launchctl_command("unload")
        }

        #[cfg(target_os = "windows")]
        {
            // Would need to modify service config
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow!("Service disable not supported on this platform"))
        }
    }

    /// Get service status
    pub fn status(&self) -> Result<ServiceStatus> {
        #[cfg(target_os = "linux")]
        {
            self.get_systemd_status()
        }

        #[cfg(target_os = "macos")]
        {
            self.get_launchd_status()
        }

        #[cfg(target_os = "windows")]
        {
            self.get_windows_status()
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow!("Service status not supported on this platform"))
        }
    }
}

// Linux systemd implementation
#[cfg(target_os = "linux")]
impl ServiceManager {
    fn install_systemd_service(&self) -> Result<()> {
        let service_content = self.generate_systemd_service()?;
        let service_file = format!("/etc/systemd/system/{}.service", self.service_name);
        
        // Write service file
        fs::write(&service_file, service_content)
            .map_err(|e| anyhow!("Failed to write systemd service file: {}", e))?;
        
        info!("Created systemd service file: {}", service_file);
        
        // Reload systemd
        self.systemctl_command("daemon-reload")?;
        
        Ok(())
    }

    fn uninstall_systemd_service(&self) -> Result<()> {
        let service_file = format!("/etc/systemd/system/{}.service", self.service_name);
        
        // Stop and disable first
        let _ = self.stop();
        let _ = self.disable();
        
        // Remove service file
        if Path::new(&service_file).exists() {
            fs::remove_file(&service_file)
                .map_err(|e| anyhow!("Failed to remove systemd service file: {}", e))?;
            
            info!("Removed systemd service file: {}", service_file);
        }
        
        // Reload systemd
        self.systemctl_command("daemon-reload")?;
        
        Ok(())
    }

    fn systemctl_command(&self, command: &str) -> Result<()> {
        let output = Command::new("systemctl")
            .arg(command)
            .arg(&self.service_name)
            .output()
            .map_err(|e| anyhow!("Failed to execute systemctl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("systemctl {} failed: {}", command, stderr));
        }

        info!("Executed: systemctl {} {}", command, self.service_name);
        Ok(())
    }

    fn get_systemd_status(&self) -> Result<ServiceStatus> {
        let output = Command::new("systemctl")
            .args(&["is-active", &self.service_name])
            .output()
            .map_err(|e| anyhow!("Failed to check service status: {}", e))?;

        let status_str = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
        
        let status = match status_str.as_str() {
            "active" => ServiceStatus::Running,
            "inactive" => ServiceStatus::Stopped,
            "failed" => ServiceStatus::Failed,
            _ => ServiceStatus::Unknown,
        };

        Ok(status)
    }

    fn generate_systemd_service(&self) -> Result<String> {
        let config_arg = if let Some(ref config_path) = self.config_path {
            format!(" --config {}", config_path.display())
        } else {
            String::new()
        };

        let service_content = format!(
r#"[Unit]
Description=Synx Code Validation Daemon
Documentation=https://github.com/A5873/synx
After=network.target
Wants=network.target

[Service]
Type=notify
ExecStart={binary_path} daemon{config_arg}
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=10
User=synx
Group=synx
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/synx /var/run/synx
PrivateTmp=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictNamespaces=true
LockPersonality=true
MemoryDenyWriteExecute=true
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
"#,
            binary_path = self.binary_path.display(),
            config_arg = config_arg
        );

        Ok(service_content)
    }
}

// macOS launchd implementation
#[cfg(target_os = "macos")]
impl ServiceManager {
    fn install_launchd_service(&self) -> Result<()> {
        let plist_content = self.generate_launchd_plist()?;
        let plist_file = format!("/Library/LaunchDaemons/com.synx.{}.plist", self.service_name);
        
        // Write plist file
        fs::write(&plist_file, plist_content)
            .map_err(|e| anyhow!("Failed to write launchd plist file: {}", e))?;
        
        info!("Created launchd plist file: {}", plist_file);
        
        // Set proper permissions
        Command::new("chown")
            .args(&["root:wheel", &plist_file])
            .output()
            .map_err(|e| anyhow!("Failed to set plist ownership: {}", e))?;

        Command::new("chmod")
            .args(&["644", &plist_file])
            .output()
            .map_err(|e| anyhow!("Failed to set plist permissions: {}", e))?;
        
        Ok(())
    }

    fn uninstall_launchd_service(&self) -> Result<()> {
        let plist_file = format!("/Library/LaunchDaemons/com.synx.{}.plist", self.service_name);
        
        // Unload first
        let _ = self.stop();
        
        // Remove plist file
        if Path::new(&plist_file).exists() {
            fs::remove_file(&plist_file)
                .map_err(|e| anyhow!("Failed to remove launchd plist file: {}", e))?;
            
            info!("Removed launchd plist file: {}", plist_file);
        }
        
        Ok(())
    }

    fn launchctl_command(&self, command: &str) -> Result<()> {
        let plist_file = format!("/Library/LaunchDaemons/com.synx.{}.plist", self.service_name);
        
        let output = Command::new("launchctl")
            .arg(command)
            .arg(&plist_file)
            .output()
            .map_err(|e| anyhow!("Failed to execute launchctl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("launchctl {} warning: {}", command, stderr);
        }

        info!("Executed: launchctl {} {}", command, plist_file);
        Ok(())
    }

    fn get_launchd_status(&self) -> Result<ServiceStatus> {
        let label = format!("com.synx.{}", self.service_name);
        
        let output = Command::new("launchctl")
            .args(&["list", &label])
            .output()
            .map_err(|e| anyhow!("Failed to check service status: {}", e))?;

        if output.status.success() {
            // Service exists, check if it's running by looking for PID
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("\"PID\"") && !output_str.contains("\"PID\" = 0") {
                Ok(ServiceStatus::Running)
            } else {
                Ok(ServiceStatus::Stopped)
            }
        } else {
            Ok(ServiceStatus::NotInstalled)
        }
    }

    fn generate_launchd_plist(&self) -> Result<String> {
        let config_arg = if let Some(ref config_path) = self.config_path {
            format!("\n        <string>--config</string>\n        <string>{}</string>", config_path.display())
        } else {
            String::new()
        };

        let plist_content = format!(
r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.synx.{service_name}</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>{binary_path}</string>
        <string>daemon</string>{config_arg}
    </array>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>StandardOutPath</key>
    <string>/var/log/synx-daemon.log</string>
    
    <key>StandardErrorPath</key>
    <string>/var/log/synx-daemon.log</string>
    
    <key>UserName</key>
    <string>_synx</string>
    
    <key>GroupName</key>
    <string>_synx</string>
    
    <key>WorkingDirectory</key>
    <string>/var/lib/synx</string>
    
    <key>ThrottleInterval</key>
    <integer>10</integer>
</dict>
</plist>
"#,
            service_name = self.service_name,
            binary_path = self.binary_path.display(),
            config_arg = config_arg
        );

        Ok(plist_content)
    }
}

// Windows service implementation
#[cfg(target_os = "windows")]
impl ServiceManager {
    fn install_windows_service(&self) -> Result<()> {
        let config_arg = if let Some(ref config_path) = self.config_path {
            format!(" --config \"{}\"", config_path.display())
        } else {
            String::new()
        };

        let output = Command::new("sc")
            .args(&[
                "create",
                &self.service_name,
                "binPath=",
                &format!("\"{}\" daemon{}", self.binary_path.display(), config_arg),
                "start=",
                "auto",
                "DisplayName=",
                "Synx Code Validation Daemon",
            ])
            .output()
            .map_err(|e| anyhow!("Failed to create Windows service: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("sc create failed: {}", stderr));
        }

        info!("Created Windows service: {}", self.service_name);
        Ok(())
    }

    fn uninstall_windows_service(&self) -> Result<()> {
        // Stop first
        let _ = self.stop();

        let output = Command::new("sc")
            .args(&["delete", &self.service_name])
            .output()
            .map_err(|e| anyhow!("Failed to delete Windows service: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("sc delete failed: {}", stderr));
        }

        info!("Deleted Windows service: {}", self.service_name);
        Ok(())
    }

    fn sc_command(&self, command: &str) -> Result<()> {
        let output = Command::new("sc")
            .arg(command)
            .arg(&self.service_name)
            .output()
            .map_err(|e| anyhow!("Failed to execute sc: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("sc {} failed: {}", command, stderr));
        }

        info!("Executed: sc {} {}", command, self.service_name);
        Ok(())
    }

    fn get_windows_status(&self) -> Result<ServiceStatus> {
        let output = Command::new("sc")
            .args(&["query", &self.service_name])
            .output()
            .map_err(|e| anyhow!("Failed to query service status: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
        
        if output_str.contains("running") {
            Ok(ServiceStatus::Running)
        } else if output_str.contains("stopped") {
            Ok(ServiceStatus::Stopped)
        } else if output_str.contains("1060") {  // Service does not exist
            Ok(ServiceStatus::NotInstalled)
        } else {
            Ok(ServiceStatus::Unknown)
        }
    }
}

/// Status of a system service
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Failed,
    NotInstalled,
    Unknown,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Running => write!(f, "running"),
            ServiceStatus::Stopped => write!(f, "stopped"),
            ServiceStatus::Failed => write!(f, "failed"),
            ServiceStatus::NotInstalled => write!(f, "not installed"),
            ServiceStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Install the Synx daemon as a system service
pub fn install_service(service_name: &str, binary_path: &Path, config_path: Option<&Path>) -> Result<()> {
    let mut manager = ServiceManager::new(service_name.to_string(), binary_path.to_path_buf());
    
    if let Some(config) = config_path {
        manager = manager.with_config_path(config.to_path_buf());
    }
    
    manager.install()?;
    manager.enable()?;
    
    info!("Synx daemon service installed successfully");
    println!("✅ Service '{}' installed successfully", service_name);
    println!("   To start: sudo systemctl start {} (Linux) or sudo launchctl load /Library/LaunchDaemons/com.synx.{}.plist (macOS)", service_name, service_name);
    
    Ok(())
}

/// Uninstall the Synx daemon service
pub fn uninstall_service(service_name: &str) -> Result<()> {
    let manager = ServiceManager::new(service_name.to_string(), PathBuf::new());
    
    manager.uninstall()?;
    
    info!("Synx daemon service uninstalled successfully");
    println!("✅ Service '{}' uninstalled successfully", service_name);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_service_manager_creation() {
        let manager = ServiceManager::new("test-service".to_string(), PathBuf::from("/usr/bin/synx"));
        assert_eq!(manager.service_name, "test-service");
    }

    #[test]
    fn test_service_status_display() {
        assert_eq!(ServiceStatus::Running.to_string(), "running");
        assert_eq!(ServiceStatus::Stopped.to_string(), "stopped");
        assert_eq!(ServiceStatus::Failed.to_string(), "failed");
        assert_eq!(ServiceStatus::NotInstalled.to_string(), "not installed");
        assert_eq!(ServiceStatus::Unknown.to_string(), "unknown");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_systemd_service_generation() {
        let manager = ServiceManager::new("test-service".to_string(), PathBuf::from("/usr/bin/synx"));
        let service_content = manager.generate_systemd_service().unwrap();
        
        assert!(service_content.contains("[Unit]"));
        assert!(service_content.contains("[Service]"));
        assert!(service_content.contains("[Install]"));
        assert!(service_content.contains("ExecStart=/usr/bin/synx daemon"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_launchd_plist_generation() {
        let manager = ServiceManager::new("test-service".to_string(), PathBuf::from("/usr/bin/synx"));
        let plist_content = manager.generate_launchd_plist().unwrap();
        
        assert!(plist_content.contains("<?xml version=\"1.0\""));
        assert!(plist_content.contains("<key>Label</key>"));
        assert!(plist_content.contains("<key>ProgramArguments</key>"));
        assert!(plist_content.contains("/usr/bin/synx"));
    }
}

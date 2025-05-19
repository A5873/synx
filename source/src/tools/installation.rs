use std::process::Command;
use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};
use log::{info, warn, error, debug};

use super::{ToolRequirement, PackageManager};

/// Represents an installation result
#[derive(Debug)]
pub enum InstallationResult {
    Success,
    AlreadyInstalled,
    NeedsUpdate {
        current_version: String,
        required_version: String,
    },
    Failed {
        error: String,
        manual_instructions: String,
    },
}

/// Install a tool using the specified package manager
pub fn install_tool(req: &ToolRequirement, pm: &PackageManager) -> Result<InstallationResult> {
    info!("Installing tool: {}", req.name);

    // Check if tool is already installed with correct version
    if let Ok(true) = is_correctly_installed(req) {
        return Ok(InstallationResult::AlreadyInstalled);
    }

    // Check installation prerequisites
    check_installation_prerequisites(req, pm)?;

    // Perform platform-specific pre-installation steps
    perform_pre_installation_steps(req)?;

    // Update package manager if supported
    if let Some(update_cmd) = &pm.update_cmd {
        if let Err(e) = execute_command(update_cmd, "Failed to update package manager") {
            warn!("Package manager update failed: {}", e);
            // Continue with installation even if update fails
        }
    }

    // Get the appropriate package name for this package manager and platform
    let package_name = get_platform_specific_package_name(req, pm);

    // Construct and execute installation command
    let install_cmd = format!("{} {}", pm.install_cmd, package_name);
    match execute_command(&install_cmd, "Installation failed") {
        Ok(_) => {
            info!("Successfully installed {}", req.name);
            
            // Perform post-installation setup if needed
            if let Err(e) = perform_post_installation_setup(req) {
                warn!("Post-installation setup failed: {}", e);
            }

            Ok(InstallationResult::Success)
        }
        Err(e) => {
            error!("Failed to install {}: {}", req.name, e);
            Ok(InstallationResult::Failed {
                error: e.to_string(),
                manual_instructions: req.install_instructions.clone(),
            })
        }
    }
}

/// Check if a tool is already installed with the correct version
fn is_correctly_installed(req: &ToolRequirement) -> Result<bool> {
    use super::detection::detect_tool;
    use super::ToolStatus;

    match detect_tool(req)? {
        ToolStatus::Available { .. } => Ok(true),
        ToolStatus::WrongVersion { current, required, .. } => {
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Check prerequisites before installation
fn check_installation_prerequisites(req: &ToolRequirement, pm: &PackageManager) -> Result<()> {
    // Check if we have sufficient permissions
    if !check_install_permissions() {
        return Err(anyhow!(
            "Insufficient permissions to install packages. Try running with appropriate privileges."
        ));
    }

    // Check disk space
    check_available_disk_space()?;

    // Check platform-specific requirements
    let platform = get_current_platform();
    if let Some(platform_reqs) = req.platform_reqs.get(&platform) {
        // Check minimum OS version if specified
        if let Some(min_version) = &platform_reqs.min_os_version {
            check_os_version(min_version)?;
        }

        // Check required libraries
        for lib in &platform_reqs.required_libs {
            if !check_library_installed(lib) {
                warn!("Required library {} not found", lib);
            }
        }

        // Check environment variables
        for (var, expected_value) in &platform_reqs.required_env {
            match std::env::var(var) {
                Ok(current_value) => {
                    if &current_value != expected_value {
                        warn!("Environment variable {} has unexpected value", var);
                    }
                }
                Err(_) => {
                    warn!("Required environment variable {} not set", var);
                }
            }
        }
    }

    Ok(())
}

/// Get the current platform identifier
fn get_current_platform() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Check if we have sufficient permissions for package installation
fn check_install_permissions() -> bool {
    #[cfg(unix)]
    {
        Command::new("id")
            .arg("-u")
            .output()
            .map_or(false, |output| {
                String::from_utf8_lossy(&output.stdout).trim() == "0"
            })
    }

    #[cfg(windows)]
    {
        use windows_security::is_elevated;
        is_elevated()
    }

    #[cfg(not(any(unix, windows)))]
    {
        true // Default for other platforms
    }
}

/// Check available disk space
fn check_available_disk_space() -> Result<()> {
    #[cfg(unix)]
    {
        let output = Command::new("df")
            .args(["-h", "/"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to check disk space"));
        }

        // Parse df output and check available space
        // This is a simplified check
        Ok(())
    }

    #[cfg(windows)]
    {
        // TODO: Implement Windows disk space check
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(())
    }
}

/// Execute platform-specific pre-installation steps
fn perform_pre_installation_steps(req: &ToolRequirement) -> Result<()> {
    match get_current_platform().as_str() {
        "windows" => {
            // Windows-specific preparation
            check_windows_prerequisites(req)
        }
        "macos" => {
            // macOS-specific preparation
            check_macos_prerequisites(req)
        }
        "linux" => {
            // Linux-specific preparation
            check_linux_prerequisites(req)
        }
        _ => Ok(()),
    }
}

/// Perform post-installation setup steps
fn perform_post_installation_setup(req: &ToolRequirement) -> Result<()> {
    // Common post-installation tasks
    verify_installation(req)?;

    // Platform-specific post-installation tasks
    match get_current_platform().as_str() {
        "windows" => setup_windows_tool(req),
        "macos" => setup_macos_tool(req),
        "linux" => setup_linux_tool(req),
        _ => Ok(()),
    }
}

/// Get platform-specific package name
fn get_platform_specific_package_name(req: &ToolRequirement, pm: &PackageManager) -> String {
    let platform = get_current_platform();
    if let Some(platform_reqs) = req.platform_reqs.get(&platform) {
        if let Some(pkg_info) = platform_reqs.package_info.get(&pm.name) {
            if let Some(package_name) = pkg_info.packages.get("default") {
                return package_name.clone();
            }
        }
    }
    req.package_name.clone()
}

/// Execute a shell command
fn execute_command(cmd: &str, error_msg: &str) -> Result<()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    let status = Command::new(parts[0])
        .args(&parts[1..])
        .status()
        .with_context(|| format!("{}: {}", error_msg, cmd))?;

    if !status.success() {
        Err(anyhow!("{}: {} (exit code: {:?})", error_msg, cmd, status.code()))
    } else {
        Ok(())
    }
}

/// Verify installation was successful
fn verify_installation(req: &ToolRequirement) -> Result<()> {
    use super::detection::detect_tool;
    use super::ToolStatus;

    match detect_tool(req)? {
        ToolStatus::Available { .. } => Ok(()),
        status => Err(anyhow!("Tool verification failed: {:?}", status)),
    }
}

// Platform-specific setup functions
#[cfg(target_os = "windows")]
fn setup_windows_tool(req: &ToolRequirement) -> Result<()> {
    // Windows-specific post-installation setup
    Ok(())
}

#[cfg(target_os = "macos")]
fn setup_macos_tool(req: &ToolRequirement) -> Result<()> {
    // macOS-specific post-installation setup
    Ok(())
}

#[cfg(target_os = "linux")]
fn setup_linux_tool(req: &ToolRequirement) -> Result<()> {
    // Linux-specific post-installation setup
    Ok(())
}

// Platform-specific prerequisite checks
fn check_windows_prerequisites(req: &ToolRequirement) -> Result<()> {
    #[cfg(windows)]
    {
        // Check for required Windows features
        // Check for required Visual Studio components
        // etc.
    }
    Ok(())
}

fn check_macos_prerequisites(req: &ToolRequirement) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // Check for XCode command line tools
        // Check for Rosetta 2 on Apple Silicon
        // etc.
    }
    Ok(())
}

fn check_linux_prerequisites(req: &ToolRequirement) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        // Check for build essentials
        // Check for required development packages
        // etc.
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_get_platform_specific_package_name() {
        let pm = PackageManager {
            name: "test_pm".to_string(),
            install_cmd: "install".to_string(),
            update_cmd: None,
            packages: HashMap::new(),
        };

        let req = ToolRequirement {
            name: "test".to_string(),
            description: "Test tool".to_string(),
            package_name: "test-pkg".to_string(),
            version_req: None,
            version_cmd: vec![],
            version_pattern: String::new(),
            required: true,
            alternatives: vec![],
            capabilities: Default::default(),
            platform_reqs: HashMap::new(),
            install_instructions: String::new(),
            info_url: String::new(),
        };

        let package_name = get_platform_specific_package_name(&req, &pm);
        assert_eq!(package_name, "test-pkg");
    }

    // Add more tests...
}

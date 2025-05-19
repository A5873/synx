use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, Context, anyhow};
use semver::{Version, VersionReq};
use regex::Regex;
use which::which;
use log::{debug, warn, error};

use super::{
    ToolStatus,
    ToolRequirement,
    ToolCapabilities,
    get_command_path,
    get_command_version,
    check_tool_capabilities,
};

/// Detect the status of a tool based on its requirements
pub fn detect_tool(req: &ToolRequirement) -> Result<ToolStatus> {
    debug!("Detecting tool: {}", req.name);

    // First check if the tool is available in PATH
    let tool_path = match get_command_path(&req.package_name) {
        Some(path) => path,
        None => return Ok(ToolStatus::NotInstalled),
    };

    // Check platform support
    if !is_platform_supported(req)? {
        return Ok(ToolStatus::Unsupported(
            format!("{} is not supported on this platform", req.name)
        ));
    }

    // Check tool version if required
    if let Some(version_req) = &req.version_req {
        match get_tool_version(&req.package_name, &req.version_cmd, &req.version_pattern) {
            Some(current_version) => {
                let required = VersionReq::parse(version_req)
                    .context("Failed to parse version requirement")?;

                if !required.matches(&current_version) {
                    return Ok(ToolStatus::WrongVersion {
                        path: tool_path,
                        current: current_version,
                        required,
                    });
                }
            }
            None => {
                warn!("Could not determine version for tool: {}", req.name);
            }
        }
    }

    // Check tool capabilities
    let missing_capabilities = check_tool_capabilities(&req.package_name, &req.capabilities);
    if !missing_capabilities.is_empty() {
        return Ok(ToolStatus::MissingCapabilities {
            path: tool_path,
            missing: missing_capabilities,
        });
    }

    // Check if all dependencies are available
    let missing_deps = check_dependencies(req)?;
    if !missing_deps.is_empty() {
        warn!("Missing dependencies for {}: {:?}", req.name, missing_deps);
    }

    // All checks passed
    Ok(ToolStatus::Available {
        path: tool_path,
        version: get_tool_version(&req.package_name, &req.version_cmd, &req.version_pattern),
    })
}

/// Check if the tool is supported on the current platform
fn is_platform_supported(req: &ToolRequirement) -> Result<bool> {
    let platform = get_current_platform();
    
    // Check if we have platform-specific requirements
    if let Some(platform_reqs) = req.platform_reqs.get(&platform) {
        // Check minimum OS version if specified
        if let Some(min_version) = &platform_reqs.min_os_version {
            if !check_os_version(min_version)? {
                return Ok(false);
            }
        }
        Ok(true)
    } else {
        // If no platform-specific requirements are defined, assume supported
        Ok(true)
    }
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

/// Check if current OS version meets the requirement
fn check_os_version(min_version: &str) -> Result<bool> {
    match get_current_platform().as_str() {
        "windows" => check_windows_version(min_version),
        "macos" => check_macos_version(min_version),
        "linux" => check_linux_version(min_version),
        _ => Ok(false),
    }
}

#[cfg(target_os = "windows")]
fn check_windows_version(min_version: &str) -> Result<bool> {
    use windows_version::OsVersion;
    let current = OsVersion::current()?;
    let required = min_version.parse::<f32>()?;
    Ok(current.version() >= required)
}

#[cfg(not(target_os = "windows"))]
fn check_windows_version(_min_version: &str) -> Result<bool> {
    Ok(false)
}

#[cfg(target_os = "macos")]
fn check_macos_version(min_version: &str) -> Result<bool> {
    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()?;
    let version = String::from_utf8_lossy(&output.stdout);
    Ok(version.trim() >= min_version)
}

#[cfg(not(target_os = "macos"))]
fn check_macos_version(_min_version: &str) -> Result<bool> {
    Ok(false)
}

#[cfg(target_os = "linux")]
fn check_linux_version(min_version: &str) -> Result<bool> {
    let os_release = std::fs::read_to_string("/etc/os-release")
        .context("Failed to read OS release information")?;
    Ok(os_release.contains(min_version))
}

#[cfg(not(target_os = "linux"))]
fn check_linux_version(_min_version: &str) -> Result<bool> {
    Ok(false)
}

/// Get the version of a tool using its version command and pattern
fn get_tool_version(cmd: &str, version_cmd: &[String], version_pattern: &str) -> Option<Version> {
    let output = Command::new(cmd)
        .args(version_cmd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(version_pattern).ok()?;
    let captures = re.captures(&version_output)?;
    let version_str = captures.get(1)?.as_str();

    Version::parse(version_str).ok()
}

/// Check tool dependencies
fn check_dependencies(req: &ToolRequirement) -> Result<Vec<String>> {
    let mut missing_deps = Vec::new();

    for dep in &req.capabilities.dependencies {
        if which(dep).is_err() {
            missing_deps.push(dep.clone());
        }
    }

    Ok(missing_deps)
}

/// Check specific tool capabilities
fn check_specific_capabilities(tool: &str, capability: &str) -> bool {
    match (tool, capability) {
        ("gcc", "sanitize") => check_gcc_sanitizer_support(),
        ("python3", "venv") => check_python_venv_support(),
        ("node", "npm") => check_npm_available(),
        // Add more tool-specific capability checks
        _ => true, // Default to true for unknown capabilities
    }
}

fn check_gcc_sanitizer_support() -> bool {
    Command::new("gcc")
        .args(["-fsanitize=address", "-x", "c", "-", "-o", "/dev/null"])
        .status()
        .map_or(false, |status| status.success())
}

fn check_python_venv_support() -> bool {
    Command::new("python3")
        .args(["-c", "import venv"])
        .status()
        .map_or(false, |status| status.success())
}

fn check_npm_available() -> bool {
    which("npm").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_get_current_platform() {
        let platform = get_current_platform();
        assert!(!platform.is_empty());
    }

    #[test]
    fn test_check_dependencies() {
        let req = ToolRequirement {
            name: "test".to_string(),
            description: "Test tool".to_string(),
            package_name: "test".to_string(),
            version_req: None,
            version_cmd: vec![],
            version_pattern: String::new(),
            required: true,
            alternatives: vec![],
            capabilities: ToolCapabilities {
                features: vec![],
                optional_features: vec![],
                dependencies: vec!["ls".to_string()], // Use a command that should exist
            },
            platform_reqs: HashMap::new(),
            install_instructions: String::new(),
            info_url: String::new(),
        };

        let missing = check_dependencies(&req).unwrap();
        assert!(missing.is_empty());
    }

    // Add more tests...
}

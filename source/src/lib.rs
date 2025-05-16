pub mod config;
pub mod detectors;
pub mod validators;

use std::path::Path;
use anyhow::{Result, Context, anyhow};
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;

/// Custom error type for validation failures
#[derive(Debug)]
pub enum ValidationError {
    /// File not found
    FileNotFound(String),
    /// Unsupported file type
    UnsupportedType(String),
    /// Validation failed
    ValidationFailed(String),
    /// Configuration error
    ConfigError(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::FileNotFound(path) => write!(f, "File not found: {}", path),
            ValidationError::UnsupportedType(file_type) => write!(f, "Unsupported file type: {}", file_type),
            ValidationError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ValidationError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Helper function to print status messages
pub fn print_status(message: &str, success: bool, verbose: bool) -> Result<()> {
    if !verbose && success {
        return Ok(());
    }
    
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    
    if success {
        stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)))?;
        writeln!(&mut stdout, "✓ {}", message)?;
    } else {
        stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Red)))?;
        writeln!(&mut stdout, "✗ {}", message)?;
    }
    stdout.reset()?;
    
    Ok(())
}

/// Main function to validate a file
pub fn validate_file(
    file_path: &Path, 
    verbose: bool, 
    config_path: Option<&Path>
) -> Result<()> {
    // Check if file exists
    if !file_path.exists() {
        return Err(anyhow!(ValidationError::FileNotFound(
            file_path.to_string_lossy().to_string()
        )));
    }
    // Load configuration
    let config = config::Config::load(config_path)?;
    
    // Detect file type
    let file_type = detectors::detect_file_type(file_path)
        .context(format!("Failed to detect file type for {:?}", file_path))?;
    
    if verbose {
        println!("Detected file type: {}", file_type);
    }
    
    // Get validator for the file type
    let validator = validators::get_validator(&file_type)
        .context(format!("No validator found for file type: {}", file_type))?;
    
    // Run validation
    validator.validate(file_path, verbose, &config)
        .context(format!("Validation failed for {:?}", file_path))
}

/// Check if a validator is available for a file
pub fn has_validator_for_file(file_path: &Path) -> Result<bool> {
    if !file_path.exists() {
        return Ok(false);
    }
    
    let file_type = detectors::detect_file_type(file_path)?;
    match validators::get_validator(&file_type) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Get a list of all supported file extensions
pub fn get_supported_extensions() -> Vec<&'static str> {
    vec![
        "py", "js", "ts", "html", "htm", "css", "json", 
        "yaml", "yml", "toml", "sh", "bash", "zsh",
        "md", "markdown", "rs", "c", "cpp", "cc", "cxx",
        "Dockerfile",
    ]
}

/// Validate multiple files
pub fn validate_files(
    file_paths: &[&Path],
    verbose: bool,
    config_path: Option<&Path>,
) -> Result<(usize, usize)> {
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for file_path in file_paths {
        match validate_file(file_path, verbose, config_path) {
            Ok(_) => {
                success_count += 1;
            },
            Err(e) => {
                println!("Error validating {:?}: {}", file_path, e);
                failure_count += 1;
            }
        }
    }
    
    Ok((success_count, failure_count))
}

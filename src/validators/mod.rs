use std::path::Path;
use std::process::Command;
use anyhow::{Result, anyhow, Context};
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;
use tempfile;

use crate::config::Config;
use crate::detectors::FileType;

/// Trait for file validators
pub trait Validator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()>;
    fn name(&self) -> &str;
}

/// Get validator for a specific file type
pub fn get_validator(file_type: &FileType) -> Result<Box<dyn Validator>> {
    match file_type {
        FileType::Python => Ok(Box::new(PythonValidator)),
        FileType::JavaScript => Ok(Box::new(JavaScriptValidator)),
        FileType::Json => Ok(Box::new(JsonValidator)),
        FileType::Yaml => Ok(Box::new(YamlValidator)),
        FileType::Html => Ok(Box::new(HtmlValidator)),
        FileType::Css => Ok(Box::new(CssValidator)),
        FileType::Dockerfile => Ok(Box::new(DockerfileValidator)),
        FileType::Shell => Ok(Box::new(ShellValidator)),
        FileType::Markdown => Ok(Box::new(MarkdownValidator)),
        FileType::Toml => Ok(Box::new(TomlValidator)),
        FileType::Rust => Ok(Box::new(RustValidator)),
        _ => Err(anyhow!("No validator available for {:?}", file_type)),
    }
}

/// Helper function to print colorized output
fn print_colored(message: &str, success: bool, verbose: bool) -> Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    
    if success {
        if verbose {
            stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)))?;
            writeln!(&mut stdout, "✓ {}", message)?;
            stdout.reset()?;
        }
    } else {
        stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Red)))?;
        writeln!(&mut stdout, "✗ {}", message)?;
        stdout.reset()?;
    }
    
    Ok(())
}

/// Helper function to check if a command is available
fn is_command_available(command: &str) -> bool {
    match Command::new("which").arg(command).output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Helper function to run a command and handle its output
fn run_command(cmd: &mut Command, validator_name: &str, file_path: &Path, verbose: bool) -> Result<()> {
    if verbose {
        println!("Running {} on {:?}", validator_name, file_path);
    }
    
    // Try to execute the command
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                print_colored(&format!("{}: No issues found in {:?}", validator_name, file_path), true, verbose)?;
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                print_colored(&format!("{} found issues in {:?}:", validator_name, file_path), false, false)?;
                
                // Some tools output to stdout, some to stderr
                if !stderr.is_empty() {
                    println!("{}", stderr);
                } else if !stdout.is_empty() {
                    println!("{}", stdout);
                }
                
                Err(anyhow!("{} validation failed", validator_name))
            }
        },
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                print_colored(&format!("{} is not installed", validator_name), false, false)?;
                println!("Please install the required tool to validate this file type.");
                Err(anyhow!("Validator tool not found: {}", cmd.get_program().to_string_lossy()))
            } else {
                Err(anyhow!("Failed to execute {}: {}", validator_name, e))
            }
        }
    }
}

// Validator implementations

/// Python syntax validator that uses Python's built-in py_compile module
struct PythonValidator;
impl Validator for PythonValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("python") {
            if !validator_config.enabled {
                if verbose {
                    println!("Python validator is disabled in config");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        let mut cmd = Command::new("python");
        cmd.args(["-m", "py_compile", file_path.to_str().unwrap()]);
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "Python Syntax Validator"
    }
}

/// JavaScript syntax validator that uses Node.js's syntax check
struct JavaScriptValidator;
impl Validator for JavaScriptValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("javascript") {
            if !validator_config.enabled {
                if verbose {
                    println!("JavaScript validator is disabled in config");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        // First check if node is available
        if !is_command_available("node") {
            print_colored("Node.js is not installed", false, false)?;
            println!("Please install Node.js to validate JavaScript files.");
            return Err(anyhow!("Node.js is not installed"));
        }
        
        let mut cmd = Command::new("node");
        cmd.args(["--check", file_path.to_str().unwrap()]);
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "JavaScript Syntax Validator"
    }
}

struct JsonValidator;
impl Validator for JsonValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("json") {
            if !validator_config.enabled {
                if verbose {
                    println!("JSON validator is disabled in config");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        // Check if jq is available
        if !is_command_available("jq") {
            print_colored("jq is not installed", false, false)?;
            println!("Please install jq to validate JSON files.");
            return Err(anyhow!("jq is not installed"));
        }
        
        let mut cmd = Command::new("jq");
        cmd.args([".", file_path.to_str().unwrap()]);
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "JSON Validator"
    }
}

struct YamlValidator;
impl Validator for YamlValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        let mut cmd = Command::new("yamllint");
        cmd.args(["-f", "parsable", file_path.to_str().unwrap()]);
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "YAML Validator"
    }
}

struct HtmlValidator;
impl Validator for HtmlValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        let mut cmd = Command::new("tidy");
        cmd.args(["-q", "-e", file_path.to_str().unwrap()]);
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "HTML Validator"
    }
}

struct CssValidator;
impl Validator for CssValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        // Using a simple CSS validator (could use stylelint in a real implementation)
        let mut cmd = Command::new("csslint");
        cmd.arg(file_path.to_str().unwrap());
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "CSS Validator"
    }
}

struct DockerfileValidator;
impl Validator for DockerfileValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        let mut cmd = Command::new("hadolint");
        cmd.arg(file_path.to_str().unwrap());
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "Dockerfile Validator"
    }
}

struct ShellValidator;
impl Validator for ShellValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        let mut cmd = Command::new("shellcheck");
        cmd.arg(file_path.to_str().unwrap());
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "Shell Script Validator"
    }
}

struct MarkdownValidator;
impl Validator for MarkdownValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        let mut cmd = Command::new("mdl");
        cmd.arg(file_path.to_str().unwrap());
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "Markdown Validator"
    }
}

/// TOML validator that uses the toml crate to parse TOML files
struct TomlValidator;
impl Validator for TomlValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("toml") {
            if !validator_config.enabled {
                if verbose {
                    println!("TOML validator is disabled in config");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        // Read the file
        let content = std::fs::read_to_string(file_path)
            .context(format!("Failed to read TOML file: {:?}", file_path))?;
        
        // Try to parse it with toml
        match toml::from_str::<toml::Value>(&content) {
            Ok(_) => {
                print_colored(&format!("{}: No issues found in {:?}", self.name(), file_path), true, verbose)?;
                Ok(())
            },
            Err(err) => {
                print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
                println!("{}", err);
                Err(anyhow!("TOML validation failed: {}", err))
            }
        }
    }
    
    fn name(&self) -> &str {
        "TOML Validator"
    }
}

/// Rust syntax validator that uses rustc with --emit=check
struct RustValidator;
impl Validator for RustValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("rust") {
            if !validator_config.enabled {
                if verbose {
                    println!("Rust validator is disabled in config");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        // Check if rustc is available
        if !is_command_available("rustc") {
            print_colored("Rust compiler is not installed", false, false)?;
            println!("Please install Rust to validate Rust files.");
            return Err(anyhow!("Rust compiler is not installed"));
        }
        
        // Create a temporary directory for compilation artifacts
        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory for Rust compilation")?;
        
        let mut cmd = Command::new("rustc");
        cmd.args([
            "--emit=check",
            "--crate-type=lib",
            "--color=always",
            "-o", temp_dir.path().join("output").to_str().unwrap(),
            file_path.to_str().unwrap()
        ]);
        
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "Rust Validator"
    }
}

/// Add additional test helpers for validators
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;
    
    #[test]
    fn test_toml_validator() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.toml");
        
        // Valid TOML
        std::fs::write(&file_path, r#"
        [package]
        name = "test"
        version = "0.1.0"
        "#).unwrap();
        
        let config = Config::default();
        let validator = TomlValidator;
        assert!(validator.validate(&file_path, false, &config).is_ok());
        
        // Invalid TOML
        std::fs::write(&file_path, r#"
        [package
        name = "test"
        "#).unwrap();
        
        assert!(validator.validate(&file_path, false, &config).is_err());
    }
}

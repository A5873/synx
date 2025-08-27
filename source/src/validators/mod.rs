use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::collections::HashMap;

pub mod scan;
pub use scan::{scan_directory, ScanResult};
mod display;
pub use display::display_scan_results;
mod error_display;
pub use error_display::{ValidationError, ErrorType, ErrorDisplay, parse_validation_output, display_validation_errors};

// Import the configuration module

#[derive(Default)]
pub struct ValidationOptions {
    pub strict: bool,
    pub verbose: bool,
    pub timeout: u64,
    pub config: Option<FileValidationConfig>,
}

#[derive(Debug, Clone)]
pub struct FileValidationConfig {
    pub file_mappings: Option<HashMap<String, String>>,
}

impl Default for FileValidationConfig {
    fn default() -> Self {
        Self {
            file_mappings: None,
        }
    }
}

pub fn validate_file(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let file_type = detect_file_type(file_path)?;
    
    // Check for custom validation rules
    if let Some(config) = &options.config {
        if let Some(mapped_type) = process_mappings(config, &file_type) {
            // Use the mapped file type for validation
            let validator = get_validator_for_type(&mapped_type);
            return validator(file_path, options);
        }
    }
    
    // Use default validator for the file type
    let validator = get_validator_for_type(&file_type);
    validator(file_path, options)
}

fn process_mappings(config: &FileValidationConfig, file_type: &str) -> Option<String> {
    config.file_mappings.as_ref()
        .and_then(|mappings| mappings.get(file_type).cloned())
}

fn detect_file_type(file_path: &Path) -> Result<String> {
    if let Some(ext) = file_path.extension() {
        if let Some(ext_str) = ext.to_str() {
            return Ok(ext_str.to_lowercase());
        }
    }
    let mime = tree_magic_mini::from_filepath(file_path)
        .ok_or_else(|| anyhow!("Failed to detect file type"))?;
    Ok(mime.split("/").last().unwrap_or("unknown").to_string())
}

fn get_validator_for_type(file_type: &str) -> fn(&Path, &ValidationOptions) -> Result<bool> {
    match file_type {
        "rs" => validate_rust,
        "cpp" | "cxx" | "cc" => validate_cpp,
        "c" => validate_c,
        "cs" => validate_csharp,
        "py" | "python" => validate_python,
        "js" | "javascript" => validate_javascript,
        "java" => validate_java,
        "go" => validate_go,
        "ts" | "tsx" => validate_typescript,
        "json" => validate_json,
        "yaml" | "yml" => validate_yaml,
        "html" | "htm" => validate_html,
        "css" => validate_css,
        "sh" | "bash" => validate_shell,
        "dockerfile" => validate_dockerfile,
        _ => validate_unknown,
    }
}

fn validate_rust(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    if options.verbose {
        eprintln!("Validating Rust file: {}", file_path.display());
    }
    
    // First, try to find if this file is part of a Cargo project
    if let Some(cargo_dir) = find_cargo_project_root(file_path) {
        validate_rust_with_cargo(file_path, &cargo_dir, options)
    } else {
        if options.verbose {
            eprintln!("No Cargo project found for {}, using standalone validation", file_path.display());
        }
        // Fall back to standalone rustc validation with basic syntax checking
        validate_rust_standalone(file_path, options)
    }
}

/// Find the Cargo project root by looking for Cargo.toml in parent directories
fn find_cargo_project_root(file_path: &Path) -> Option<PathBuf> {
    // Convert to absolute path first
    let abs_path = std::fs::canonicalize(file_path).ok()?;
    
    // Start from the file's directory
    let start_dir = if abs_path.is_file() {
        abs_path.parent()?
    } else {
        &abs_path
    };
    
    let mut current = start_dir;
    
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Some(current.to_path_buf());
        }
        
        current = current.parent()?;
    }
}

/// Validate Rust file using Cargo (for project files)
fn validate_rust_with_cargo(file_path: &Path, cargo_dir: &Path, options: &ValidationOptions) -> Result<bool> {
    if options.verbose {
        eprintln!("Using Cargo validation for {} in project {}", file_path.display(), cargo_dir.display());
    }
    
    let mut cmd = Command::new("cargo");
    cmd.current_dir(cargo_dir)
       .arg("check")
       .arg("--message-format=short");
    
    // For now, just run a general cargo check instead of trying to target specific binaries
    // This will validate the entire project, which includes our file
    
    if options.strict {
        // In strict mode, also run clippy if available
        let clippy_available = Command::new("cargo")
            .arg("clippy")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
            
        if clippy_available {
            cmd = Command::new("cargo");
            cmd.current_dir(cargo_dir)
               .arg("clippy")
               .arg("--message-format=short")
               .arg("--")
               .arg("-D").arg("warnings");
        } else {
            // Fall back to cargo check with warnings as errors
            cmd.env("RUSTFLAGS", "-D warnings");
        }
    }
    
    let output = cmd.output()?;
    let success = output.status.success();
    
    if !success && options.verbose {
        eprintln!("Rust validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        if !output.stdout.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
    
    Ok(success)
}

/// Validate standalone Rust file using rustc (for files outside projects)
fn validate_rust_standalone(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("rustc");
    cmd.arg("--crate-type=lib")
       .arg("--error-format=short")
       .arg("-A").arg("dead_code")
       .arg("-A").arg("unused_variables")
       .arg("-A").arg("unused_imports")
       .arg(file_path);

    if options.strict {
        cmd.arg("-D").arg("warnings");
    }

    let output = cmd.output()?;
    let success = output.status.success();
    
    if !success && options.verbose {
        eprintln!("Rust validation errors (standalone mode):");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

// Add other validator functions...

fn validate_unknown(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    if options.verbose {
        eprintln!("No validator available for file: {}", file_path.display());
    }
    Ok(!options.strict)
}

fn validate_cpp(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("g++");
    cmd.arg("-fsyntax-only")
       .arg("-Wall")
       .arg("-pedantic");

    if options.strict {
        cmd.arg("-Werror")
           .arg("-Wextra")
           .arg("-Wconversion");
    }

    cmd.arg(file_path);
    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("C++ validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_c(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("gcc");
    cmd.arg("-fsyntax-only")
       .arg("-Wall")
       .arg("-pedantic");

    if options.strict {
        cmd.arg("-Werror")
           .arg("-Wextra")
           .arg("-Wconversion");
    }

    cmd.arg(file_path);
    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("C validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_csharp(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("dotnet");
    cmd.arg("build")
       .arg(file_path);

    if options.strict {
        cmd.arg("/warnaserror");
    }

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("C# validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_python(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("python3");
    cmd.arg("-m").arg("py_compile").arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    // Enhanced error reporting with colorized output
    if !success {
        let error_output = if !output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stderr).to_string()
        } else {
            String::from_utf8_lossy(&output.stdout).to_string()
        };
        
        if options.verbose {
            // Parse and display structured errors
            let errors = parse_validation_output(file_path, &error_output, "python");
            if !errors.is_empty() {
                let _ = display_validation_errors(&errors);
            } else {
                // Fallback to simple error display
                eprintln!("Python validation errors:");
                eprintln!("{}", error_output);
            }
        }
    }

    Ok(success)
}

fn validate_javascript(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("node");
    cmd.arg("--check").arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    // Enhanced error reporting with colorized output
    if !success {
        let error_output = if !output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stderr).to_string()
        } else {
            String::from_utf8_lossy(&output.stdout).to_string()
        };
        
        if options.verbose {
            // Parse and display structured errors
            let errors = parse_validation_output(file_path, &error_output, "javascript");
            if !errors.is_empty() {
                let _ = display_validation_errors(&errors);
            } else {
                // Fallback to simple error display
                eprintln!("JavaScript validation errors:");
                eprintln!("{}", error_output);
            }
        }
    }

    Ok(success)
}

fn validate_java(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("javac");
    cmd.arg("-Werror").arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("Java validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_go(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("go");
    cmd.arg("vet").arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("Go validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_typescript(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("tsc");
    cmd.arg("--noEmit").arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("TypeScript validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_json(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("jq");
    cmd.arg(".").arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("JSON validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_yaml(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("yamllint");
    cmd.arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("YAML validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_html(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("tidy");
    cmd.arg("-q").arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("HTML validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_css(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("stylelint");
    cmd.arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("CSS validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_shell(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("shellcheck");
    cmd.arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("Shell script validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_dockerfile(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("hadolint");
    cmd.arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("Dockerfile validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

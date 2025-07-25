use anyhow::{Result, anyhow};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use std::collections::HashMap;

pub mod scan;
pub use scan::{scan_directory, ScanResult};
mod display;
pub use display::display_scan_results;

// Import the configuration module
use crate::config;

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
    let mut cmd = Command::new("rustc");
    cmd.arg("--crate-type=lib")
       .arg("--error-format=short")
       .arg("-A").arg("dead_code")
       .arg(file_path);

    if options.strict {
        cmd.arg("-D").arg("warnings");
    }

    let output = cmd.output()?;
    let success = output.status.success();
    
    if !success && options.verbose {
        eprintln!("Rust validation errors:");
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

    if !success && options.verbose {
        eprintln!("Python validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_javascript(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("node");
    cmd.arg("--check").arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("JavaScript validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
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

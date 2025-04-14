use std::path::Path;
use std::fs::File;
use std::io::Read;
use anyhow::{Result, Context};
use tree_magic_mini as magic;

/// FileType enum representing the detected file types
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FileType {
    Python,
    JavaScript,
    TypeScript,
    Html,
    Css,
    Json,
    Yaml,
    Toml,
    Dockerfile,
    Shell,
    Markdown,
    C,
    Cpp,
    Rust,
    Unknown(String),
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Python => write!(f, "Python"),
            FileType::JavaScript => write!(f, "JavaScript"),
            FileType::TypeScript => write!(f, "TypeScript"),
            FileType::Html => write!(f, "HTML"),
            FileType::Css => write!(f, "CSS"),
            FileType::Json => write!(f, "JSON"),
            FileType::Yaml => write!(f, "YAML"),
            FileType::Toml => write!(f, "TOML"),
            FileType::Dockerfile => write!(f, "Dockerfile"),
            FileType::Shell => write!(f, "Shell"),
            FileType::Markdown => write!(f, "Markdown"),
            FileType::C => write!(f, "C"),
            FileType::Cpp => write!(f, "C++"),
            FileType::Rust => write!(f, "Rust"),
            FileType::Unknown(ext) => write!(f, "Unknown ({})", ext),
        }
    }
}

/// Map a MIME type to a FileType
fn mime_to_file_type(mime: &str) -> Option<FileType> {
    match mime {
        "text/x-python" | "application/x-python-code" => Some(FileType::Python),
        "application/javascript" | "text/javascript" => Some(FileType::JavaScript),
        "application/typescript" | "text/typescript" => Some(FileType::TypeScript),
        "text/html" => Some(FileType::Html),
        "text/css" => Some(FileType::Css),
        "application/json" => Some(FileType::Json),
        "application/yaml" | "text/yaml" => Some(FileType::Yaml),
        "application/toml" | "text/toml" => Some(FileType::Toml),
        "text/markdown" => Some(FileType::Markdown),
        "text/x-c" => Some(FileType::C),
        "text/x-c++" => Some(FileType::Cpp),
        "text/x-rust" => Some(FileType::Rust),
        "application/x-shellscript" | "text/x-shellscript" => Some(FileType::Shell),
        _ => None,
    }
}

/// Check if the file has a shebang line indicating a shell script
fn check_for_shebang(path: &Path) -> Result<Option<FileType>> {
    let mut file = File::open(path).context("Failed to open file")?;
    let mut buffer = [0; 1024];
    let n = file.read(&mut buffer).context("Failed to read file")?;
    let content = String::from_utf8_lossy(&buffer[..n]);
    
    if content.starts_with("#!/bin/bash") || 
       content.starts_with("#!/bin/sh") || 
       content.starts_with("#!/usr/bin/env bash") || 
       content.starts_with("#!/usr/bin/env sh") {
        return Ok(Some(FileType::Shell));
    }
    
    if content.starts_with("#!/usr/bin/env python") || 
       content.starts_with("#!/usr/bin/python") {
        return Ok(Some(FileType::Python));
    }
    
    if content.starts_with("#!/usr/bin/env node") || 
       content.starts_with("#!/usr/bin/node") {
        return Ok(Some(FileType::JavaScript));
    }
    
    Ok(None)
}

/// Detect file type based on extension, content, and custom mappings
pub fn detect_file_type(path: &Path) -> Result<FileType> {
    // Load config for custom mappings
    let config = crate::config::Config::load(None)
        .context("Failed to load configuration for file type detection")?;
    // First try to detect by extension
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        
        match ext.as_str() {
            "py" => return Ok(FileType::Python),
            "js" => return Ok(FileType::JavaScript),
            "ts" => return Ok(FileType::TypeScript),
            "html" | "htm" => return Ok(FileType::Html),
            "css" => return Ok(FileType::Css),
            "json" => return Ok(FileType::Json),
            "yaml" | "yml" => return Ok(FileType::Yaml),
            "toml" => return Ok(FileType::Toml),
            "md" | "markdown" => return Ok(FileType::Markdown),
            "c" => return Ok(FileType::C),
            "cpp" | "cc" | "cxx" => return Ok(FileType::Cpp),
            "rs" => return Ok(FileType::Rust),
            "sh" | "bash" | "zsh" => return Ok(FileType::Shell),
            _ => {}
        }
    }
    
    // Check special file names (e.g., Dockerfile)
    let file_name = path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    
    // Check custom mappings from config
    if let Some(file_type) = config.file_mappings.get(&file_name) {
        match file_type.to_lowercase().as_str() {
            "python" => return Ok(FileType::Python),
            "javascript" => return Ok(FileType::JavaScript),
            "typescript" => return Ok(FileType::TypeScript),
            "html" => return Ok(FileType::Html),
            "css" => return Ok(FileType::Css),
            "json" => return Ok(FileType::Json),
            "yaml" => return Ok(FileType::Yaml),
            "toml" => return Ok(FileType::Toml),
            "dockerfile" => return Ok(FileType::Dockerfile),
            "shell" => return Ok(FileType::Shell),
            "markdown" => return Ok(FileType::Markdown),
            "c" => return Ok(FileType::C),
            "cpp" => return Ok(FileType::Cpp),
            "rust" => return Ok(FileType::Rust),
            _ => {}
        }
    }
    
    // Common special files
    match file_name.as_str() {
        "Dockerfile" => return Ok(FileType::Dockerfile),
        "Makefile" | "makefile" => return Ok(FileType::Shell),
        ".gitignore" | ".dockerignore" => return Ok(FileType::Shell),
        _ => {}
    }
    
    // Check for shebang line
    if let Ok(Some(file_type)) = check_for_shebang(path) {
        return Ok(file_type);
    }
    
    // Use tree_magic_mini for content-based detection
    let mime = magic::from_filepath(path).unwrap_or_default();
    
    if let Some(file_type) = mime_to_file_type(&mime) {
        return Ok(file_type);
    }
    
    // If all detection methods fail, return Unknown with the extension if any
    if let Some(extension) = path.extension() {
        Ok(FileType::Unknown(extension.to_string_lossy().to_string()))
    } else {
        Ok(FileType::Unknown(format!("no-extension (mime: {})", mime)))
    }
}

// Additional helper functions for file type detection can be added here

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let file_path = dir.join(filename);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    #[test]
    fn test_extension_detection() {
        let dir = tempdir().unwrap();
        
        // Create test files with different extensions
        let py_file = create_test_file(dir.path(), "test.py", "print('hello')");
        let js_file = create_test_file(dir.path(), "test.js", "console.log('hello')");
        let json_file = create_test_file(dir.path(), "test.json", "{}");
        
        // Test detection
        assert_eq!(detect_file_type(&py_file).unwrap(), FileType::Python);
        assert_eq!(detect_file_type(&js_file).unwrap(), FileType::JavaScript);
        assert_eq!(detect_file_type(&json_file).unwrap(), FileType::Json);
    }

    #[test]
    fn test_special_file_detection() {
        let dir = tempdir().unwrap();
        
        // Create special files
        let dockerfile = create_test_file(dir.path(), "Dockerfile", "FROM ubuntu:20.04");
        
        // Test detection
        assert_eq!(detect_file_type(&dockerfile).unwrap(), FileType::Dockerfile);
    }

    #[test]
    fn test_shebang_detection() {
        let dir = tempdir().unwrap();
        
        // Create files with shebangs
        let bash_file = create_test_file(dir.path(), "script", "#!/bin/bash\necho hello");
        let py_file = create_test_file(dir.path(), "script.x", "#!/usr/bin/env python\nprint('hello')");
        
        // Test detection
        assert_eq!(detect_file_type(&bash_file).unwrap(), FileType::Shell);
        assert_eq!(detect_file_type(&py_file).unwrap(), FileType::Python);
    }

    #[test]
    fn test_content_detection() {
        let dir = tempdir().unwrap();
        
        // Create files without extensions but with recognizable content
        let html_file = create_test_file(dir.path(), "webpage", "<!DOCTYPE html><html><body>Hello</body></html>");
        
        // Test detection
        assert_eq!(detect_file_type(&html_file).unwrap(), FileType::Html);
    }
}

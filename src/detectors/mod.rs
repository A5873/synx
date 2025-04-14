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

/// Map a MIME type to a FileType with improved detection
fn mime_to_file_type(mime: &str) -> Option<FileType> {
    // First check for exact matches
    match mime {
        "text/x-python" | "application/x-python-code" | "text/x-python-script" => Some(FileType::Python),
        "application/javascript" | "text/javascript" | "application/x-javascript" => Some(FileType::JavaScript),
        "application/typescript" | "text/typescript" => Some(FileType::TypeScript),
        "text/html" | "application/xhtml+xml" | "text/xml" | "application/xml" | "text/plain+html" => Some(FileType::Html),
        "text/css" => Some(FileType::Css),
        "application/json" | "application/ld+json" => Some(FileType::Json),
        "application/yaml" | "text/yaml" | "application/x-yaml" => Some(FileType::Yaml),
        "application/toml" | "text/toml" | "application/x-toml" => Some(FileType::Toml),
        "text/markdown" | "text/x-markdown" => Some(FileType::Markdown),
        "text/x-c" | "text/x-csrc" => Some(FileType::C),
        "text/x-c++" | "text/x-c++src" => Some(FileType::Cpp),
        "text/x-rust" | "text/rust" => Some(FileType::Rust),
        "application/x-shellscript" | "text/x-shellscript" | "text/x-sh" => Some(FileType::Shell),
        _ => {
            // Check for partial matches if exact match fails
            if mime.contains("html") || mime.contains("xhtml") {
                return Some(FileType::Html);
            } else if mime.contains("xml") {
                return Some(FileType::Html); // XML is close enough to HTML for basic validation
            } else if mime.contains("json") {
                return Some(FileType::Json);
            } else if mime.contains("yaml") {
                return Some(FileType::Yaml);
            } else if mime.contains("python") {
                return Some(FileType::Python);
            } else if mime.contains("javascript") || mime.contains("js") || mime.contains("ecmascript") {
                return Some(FileType::JavaScript);
            } else if mime.contains("typescript") || mime.contains("ts") {
                return Some(FileType::TypeScript);
            } else if mime.contains("markdown") || mime.contains("md") {
                return Some(FileType::Markdown);
            } else if mime.contains("shell") || mime.contains("sh") {
                return Some(FileType::Shell);
            } else {
                None
            }
        }
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

/// Check if file content matches JavaScript patterns
fn is_likely_javascript(content: &str) -> bool {
    let content_lower = content.to_lowercase();
    
    // Common JavaScript keywords and patterns
    let js_patterns = [
        // Functions
        "function ", "() =>", "function(", "() {", ") {",
        // Variable declarations
        "var ", "let ", "const ", 
        // Common statements
        "return ", "if (", "for (", "while (", "switch (", "try {", "catch (", 
        // Modern JS features
        "async ", "await ", "class ", "extends ", "static ", "get ", "set ",
        "import ", "export ", "from ", "default ", 
        // Common methods
        ".map(", ".filter(", ".reduce(", ".forEach(", ".then(", ".catch(",
        // JS built-ins
        "console.log", "document.", "window.", "Object.", "Array.", "String.",
        "Promise.", "fetch(", "JSON.", "Math.", 
        // Common JS libraries
        "$(", "jQuery", "React", "Vue", "angular", "lodash", "underscore",
        "require(", "module.exports", "exports."
    ];
    
    // Count how many JS patterns we find
    let pattern_count = js_patterns.iter()
        .filter(|&pattern| content_lower.contains(pattern))
        .count();
    
    // If we find multiple JS patterns, it's likely JavaScript
    pattern_count >= 3
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
    // Try to detect content by checking the file contents directly
    // This is more reliable than MIME type for some file types
    let file = File::open(path).ok();
    if let Some(mut file) = file {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            // Check for JavaScript first since MIME detection often misses it
            if is_likely_javascript(&content) {
                return Ok(FileType::JavaScript);
            }
            
            let content_lower = content.to_lowercase();
            
            // Comprehensive HTML detection
            // 1. Check for full HTML documents
            // 1. Check for full HTML documents
            if content_lower.contains("<!doctype html>") || 
               content_lower.contains("<html") || 
               (content_lower.contains("<head") && content_lower.contains("<body")) {
                return Ok(FileType::Html);
            }
            
            // 2. Check for XML documents that might be XHTML
            if content_lower.contains("<?xml") && 
               (content_lower.contains("<!doctype") || content_lower.contains("<html")) {
                return Ok(FileType::Html);
            }
            
            // 3. Check for HTML fragments by looking for common HTML tags
            // Count HTML-like tags to reduce false positives
            let html_tag_count = [
                "<div", "</div>", 
                "<span", "</span>", 
                "<p>", "</p>", 
                "<h1", "<h2", "<h3", "<h4", "<h5", "<h6",
                "</h1>", "</h2>", "</h3>", "</h4>", "</h5>", "</h6>",
                "<a href", "<img src", "<table", "<tr", "<td", 
                "<ul", "<ol", "<li", "<form", "<input", "<button",
                "<header", "<footer", "<nav", "<section", "<article"
            ].iter()
             .filter(|&tag| content_lower.contains(tag))
             .count();
             
            // If we found multiple HTML tags, it's likely HTML content
            if html_tag_count >= 2 {
                return Ok(FileType::Html);
            }
            
            // 4. Check for individual HTML markers with attributes, which are very likely HTML
            let html_attribute_patterns = [
                "class=\"", "id=\"", "style=\"", "href=\"", "src=\"", 
                "alt=\"", "title=\"", "data-", "aria-"
            ];
            
            if content_lower.contains("<") && content_lower.contains(">") &&
               html_attribute_patterns.iter().any(|&attr| content_lower.contains(attr)) {
                return Ok(FileType::Html);
            }
            
            // JavaScript detection moved to the beginning for better results
            
            // 6. Check for shell scripts by shebang
            if content_lower.contains("#!/bin/bash") || 
               content_lower.contains("#!/bin/sh") {
                return Ok(FileType::Shell);
            }
        }
    }
    
    // Use tree_magic_mini for content-based detection as a fallback
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
    use std::path::PathBuf;
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
        
        // ---------- HTML Content Detection Tests ----------
        
        // Test 1: Standard HTML5 document
        let html5_file = create_test_file(dir.path(), "webpage_html5", r#"<!DOCTYPE html>
<html lang="en">
<head>
    <title>Test HTML5</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body>
    <h1>Hello World</h1>
    <p>This is a test HTML5 file.</p>
</body>
</html>"#);
        
        // Test 2: XHTML document
        let xhtml_file = create_test_file(dir.path(), "webpage_xhtml", r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en">
<head>
    <title>Test XHTML</title>
    <meta http-equiv="Content-Type" content="text/html; charset=UTF-8" />
</head>
<body>
    <h1>Hello XHTML World</h1>
    <p>This is a test XHTML file.</p>
</body>
</html>"#);
        
        // Test 3: Minimal HTML
        let minimal_html_file = create_test_file(dir.path(), "webpage_minimal", r#"<html>
<body>
<p>Minimal HTML with no DOCTYPE</p>
</body>
</html>"#);

        // Test 4: HTML fragment (not a complete document)
        let html_fragment_file = create_test_file(dir.path(), "html_fragment", r#"<div class="container">
    <h2>This is just an HTML fragment</h2>
    <p>Not a complete HTML document, but should still be detected</p>
</div>"#);
        
        // ---------- JavaScript Content Detection Tests ----------
        
        // Test 5: Modern JavaScript with ES6 features
        let modern_js_file = create_test_file(dir.path(), "script_modern", r#"// Modern ES6 JavaScript
const greet = (name) => {
    const message = `Hello, ${name}!`;
    console.log(message);
    return message;
};

class Person {
    constructor(name, age) {
        this.name = name;
        this.age = age;
    }
    
    introduce() {
        return `Hi, I'm ${this.name} and I'm ${this.age} years old.`;
    }
}

// Array methods
const numbers = [1, 2, 3, 4, 5];
const doubled = numbers.map(n => n * 2);
"#);
        
        // Test 6: Traditional JavaScript
        let traditional_js_file = create_test_file(dir.path(), "script_traditional", r#"// Traditional JavaScript
function calculateTotal(items) {
    var total = 0;
    for (var i = 0; i < items.length; i++) {
        total += items[i].price;
    }
    return total;
}

var car = {
    make: "Toyota",
    model: "Corolla",
    year: 2020,
    getInfo: function() {
        return this.make + " " + this.model + " (" + this.year + ")";
    }
};
"#);
        
        // Test 7: jQuery-style JavaScript
        let jquery_js_file = create_test_file(dir.path(), "script_jquery", r#"// jQuery-style JavaScript
$(document).ready(function() {
    $('.button').click(function() {
        var id = $(this).data('id');
        $.ajax({
            url: '/api/items/' + id,
            method: 'GET',
            success: function(data) {
                $('#result').html(data.name);
            },
            error: function(xhr) {
                console.error('Request failed');
            }
        });
    });
});
"#);
        
        // ---------- Run all the tests ----------
        
        // HTML detection tests
        assert_eq!(detect_file_type(&html5_file).unwrap(), FileType::Html, "Failed to detect HTML5");
        assert_eq!(detect_file_type(&xhtml_file).unwrap(), FileType::Html, "Failed to detect XHTML");
        assert_eq!(detect_file_type(&minimal_html_file).unwrap(), FileType::Html, "Failed to detect minimal HTML");
        assert_eq!(detect_file_type(&html_fragment_file).unwrap(), FileType::Html, "Failed to detect HTML fragment");
        
        // JavaScript detection tests
        assert_eq!(detect_file_type(&modern_js_file).unwrap(), FileType::JavaScript, "Failed to detect modern JS");
        assert_eq!(detect_file_type(&traditional_js_file).unwrap(), FileType::JavaScript, "Failed to detect traditional JS");
        assert_eq!(detect_file_type(&jquery_js_file).unwrap(), FileType::JavaScript, "Failed to detect jQuery-style JS");
    }
} // end of tests module

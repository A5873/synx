//! Example plugin implementations
//! 
//! This module contains sample plugins to demonstrate the plugin system
//! and validate the API design.

use super::{
    Plugin, ValidatorPlugin, FormatterPlugin, AnalyzerPlugin,
    PluginMetadata, PluginContext, PluginResult, PluginCategory,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use slog::{debug, info, warn};

/// Python validator plugin using flake8 and mypy
pub struct PythonValidatorPlugin {
    metadata: PluginMetadata,
    use_mypy: bool,
    use_flake8: bool,
}

impl PythonValidatorPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "python_validator".to_string(),
                name: "Python Validator".to_string(),
                version: "1.0.0".to_string(),
                description: "Validates Python files using flake8 and mypy".to_string(),
                authors: vec!["Synx Team".to_string()],
                supported_extensions: vec!["py".to_string(), "pyx".to_string(), "pyi".to_string()],
                categories: vec![PluginCategory::Validator, PluginCategory::Linter],
                min_synx_version: "0.1.0".to_string(),
                dependencies: vec!["flake8".to_string(), "mypy".to_string()],
            },
            use_mypy: true,
            use_flake8: true,
        }
    }

    /// Check if external tools are available
    fn check_tool_availability(&self) -> (bool, bool) {
        let flake8_available = Command::new("flake8").arg("--version").output().is_ok();
        let mypy_available = Command::new("mypy").arg("--version").output().is_ok();
        (flake8_available, mypy_available)
    }
}

impl Plugin for PythonValidatorPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&mut self, context: &PluginContext) -> Result<()> {
        info!(context.logger, "Initializing Python validator plugin");

        // Check tool availability
        let (flake8_available, mypy_available) = self.check_tool_availability();

        if !flake8_available && !mypy_available {
            return Err(anyhow!("Neither flake8 nor mypy is available"));
        }

        self.use_flake8 = flake8_available;
        self.use_mypy = mypy_available;

        if !self.use_flake8 {
            warn!(context.logger, "flake8 not available, skipping style checks");
        }
        if !self.use_mypy {
            warn!(context.logger, "mypy not available, skipping type checks");
        }

        // Read configuration
        if let Some(settings) = context.config.get("python_validator") {
            if let Some(use_mypy) = settings.get("use_mypy").and_then(|v| v.as_bool()) {
                self.use_mypy = self.use_mypy && use_mypy;
            }
            if let Some(use_flake8) = settings.get("use_flake8").and_then(|v| v.as_bool()) {
                self.use_flake8 = self.use_flake8 && use_flake8;
            }
        }

        info!(context.logger, "Python validator initialized (flake8: {}, mypy: {})", 
              self.use_flake8, self.use_mypy);
        Ok(())
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
            self.metadata.supported_extensions.contains(&extension.to_lowercase())
        } else {
            false
        }
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "use_mypy": {
                    "type": "boolean",
                    "description": "Enable mypy type checking",
                    "default": true
                },
                "use_flake8": {
                    "type": "boolean", 
                    "description": "Enable flake8 style checking",
                    "default": true
                },
                "mypy_config": {
                    "type": "string",
                    "description": "Path to mypy configuration file"
                },
                "flake8_config": {
                    "type": "string",
                    "description": "Path to flake8 configuration file"
                }
            }
        }))
    }
}

#[async_trait]
impl ValidatorPlugin for PythonValidatorPlugin {
    async fn validate_file(
        &self,
        file_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        debug!(context.logger, "Validating Python file: {:?}", file_path);

        let mut result = PluginResult::success("Python validation completed");
        let mut issues = Vec::new();
        let mut total_issues = 0;

        // Run flake8 for style checking
        if self.use_flake8 {
            debug!(context.logger, "Running flake8 on {:?}", file_path);
            
            let output = Command::new("flake8")
                .arg(file_path)
                .arg("--format=json")
                .current_dir(&context.working_dir)
                .output();

            match output {
                Ok(output) => {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        
                        if !stdout.is_empty() {
                            if let Ok(flake8_issues) = serde_json::from_str::<serde_json::Value>(&stdout) {
                                if let Some(issues_array) = flake8_issues.as_array() {
                                    total_issues += issues_array.len();
                                    for issue in issues_array {
                                        if let Some(code) = issue.get("code").and_then(|v| v.as_str()) {
                                            if let Some(message) = issue.get("text").and_then(|v| v.as_str()) {
                                                issues.push(format!("{}: {}", code, message));
                                            }
                                        }
                                    }
                                }
                            }
                        } else if !stderr.is_empty() {
                            issues.push(format!("flake8 error: {}", stderr));
                        }
                    }
                }
                Err(e) => {
                    result = result.with_warning(format!("Failed to run flake8: {}", e));
                }
            }
        }

        // Run mypy for type checking
        if self.use_mypy {
            debug!(context.logger, "Running mypy on {:?}", file_path);
            
            let output = Command::new("mypy")
                .arg(file_path)
                .arg("--no-error-summary")
                .arg("--show-error-codes")
                .current_dir(&context.working_dir)
                .output();

            match output {
                Ok(output) => {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        
                        if !stdout.is_empty() {
                            let mypy_lines: Vec<&str> = stdout.lines().collect();
                            total_issues += mypy_lines.len();
                            for line in mypy_lines {
                                if !line.trim().is_empty() {
                                    issues.push(format!("mypy: {}", line));
                                }
                            }
                        } else if !stderr.is_empty() {
                            issues.push(format!("mypy error: {}", stderr));
                        }
                    }
                }
                Err(e) => {
                    result = result.with_warning(format!("Failed to run mypy: {}", e));
                }
            }
        }

        // Determine final result
        if total_issues > 0 {
            result = PluginResult::failure(format!("Found {} issues in Python file", total_issues));
            for issue in issues {
                result = result.with_error(issue);
            }
        }

        result = result
            .with_metric("issues_found", total_issues as f64)
            .with_metric("flake8_enabled", if self.use_flake8 { 1.0 } else { 0.0 })
            .with_metric("mypy_enabled", if self.use_mypy { 1.0 } else { 0.0 });

        Ok(result)
    }

    fn supported_rules(&self) -> Vec<String> {
        let mut rules = Vec::new();
        
        if self.use_flake8 {
            rules.extend([
                "style_check".to_string(),
                "pep8".to_string(),
                "complexity".to_string(),
            ]);
        }
        
        if self.use_mypy {
            rules.extend([
                "type_check".to_string(),
                "static_analysis".to_string(),
            ]);
        }
        
        rules
    }
}

/// JSON formatter plugin using jq
pub struct JsonFormatterPlugin {
    metadata: PluginMetadata,
    indent_size: u32,
    sort_keys: bool,
}

impl JsonFormatterPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "json_formatter".to_string(),
                name: "JSON Formatter".to_string(),
                version: "1.0.0".to_string(),
                description: "Formats JSON files with consistent indentation and key sorting".to_string(),
                authors: vec!["Synx Team".to_string()],
                supported_extensions: vec!["json".to_string(), "jsonc".to_string()],
                categories: vec![PluginCategory::Formatter],
                min_synx_version: "0.1.0".to_string(),
                dependencies: vec![],
            },
            indent_size: 2,
            sort_keys: false,
        }
    }
}

impl Plugin for JsonFormatterPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&mut self, context: &PluginContext) -> Result<()> {
        info!(context.logger, "Initializing JSON formatter plugin");

        // Read configuration
        if let Some(settings) = context.config.get("json_formatter") {
            if let Some(indent) = settings.get("indent_size").and_then(|v| v.as_u64()) {
                self.indent_size = indent as u32;
            }
            if let Some(sort) = settings.get("sort_keys").and_then(|v| v.as_bool()) {
                self.sort_keys = sort;
            }
        }

        info!(context.logger, "JSON formatter initialized (indent: {}, sort_keys: {})", 
              self.indent_size, self.sort_keys);
        Ok(())
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
            self.metadata.supported_extensions.contains(&extension.to_lowercase())
        } else {
            false
        }
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "indent_size": {
                    "type": "integer",
                    "description": "Number of spaces for indentation",
                    "default": 2,
                    "minimum": 1,
                    "maximum": 8
                },
                "sort_keys": {
                    "type": "boolean",
                    "description": "Sort object keys alphabetically",
                    "default": false
                }
            }
        }))
    }
}

#[async_trait]
impl FormatterPlugin for JsonFormatterPlugin {
    async fn format_file(
        &self,
        file_path: &Path,
        context: &PluginContext,
        check_only: bool,
    ) -> Result<PluginResult> {
        debug!(context.logger, "Formatting JSON file: {:?} (check_only: {})", file_path, check_only);

        // Read the file
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| anyhow!("Failed to read file: {}", e))?;

        // Parse JSON
        let parsed: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Invalid JSON: {}", e))?;

        // Format JSON
        let formatted = if self.sort_keys {
            // Custom formatting with sorted keys
            format_json_with_sorted_keys(&parsed, self.indent_size)?
        } else {
            serde_json::to_string_pretty(&parsed)?
        };

        let needs_formatting = content.trim() != formatted.trim();

        if check_only {
            let result = if needs_formatting {
                PluginResult::failure("File needs formatting")
                    .with_data(serde_json::json!({
                        "needs_formatting": true,
                        "original_length": content.len(),
                        "formatted_length": formatted.len()
                    }))
            } else {
                PluginResult::success("File is properly formatted")
                    .with_data(serde_json::json!({
                        "needs_formatting": false
                    }))
            };
            
            Ok(result.with_metric("needs_formatting", if needs_formatting { 1.0 } else { 0.0 }))
        } else {
            if needs_formatting {
                // Write formatted content back
                std::fs::write(file_path, &formatted)
                    .map_err(|e| anyhow!("Failed to write formatted file: {}", e))?;

                Ok(PluginResult::success("File formatted successfully")
                    .with_metric("characters_changed", (content.len() as i64 - formatted.len() as i64).abs() as f64)
                    .with_metric("formatted", 1.0))
            } else {
                Ok(PluginResult::success("File was already properly formatted")
                    .with_metric("formatted", 0.0))
            }
        }
    }

    fn supported_options(&self) -> Vec<String> {
        vec![
            "indent_size".to_string(),
            "sort_keys".to_string(),
        ]
    }
}

/// Basic code analyzer plugin
pub struct BasicAnalyzerPlugin {
    metadata: PluginMetadata,
}

impl BasicAnalyzerPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "basic_analyzer".to_string(),
                name: "Basic Code Analyzer".to_string(),
                version: "1.0.0".to_string(),
                description: "Provides basic code metrics like line count, file size, etc.".to_string(),
                authors: vec!["Synx Team".to_string()],
                supported_extensions: vec![], // Supports all files
                categories: vec![PluginCategory::Analyzer],
                min_synx_version: "0.1.0".to_string(),
                dependencies: vec![],
            },
        }
    }
}

impl Plugin for BasicAnalyzerPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&mut self, context: &PluginContext) -> Result<()> {
        info!(context.logger, "Initializing basic analyzer plugin");
        Ok(())
    }

    fn can_handle(&self, _file_path: &Path) -> bool {
        true // Can analyze any file
    }
}

#[async_trait]
impl AnalyzerPlugin for BasicAnalyzerPlugin {
    async fn analyze_file(
        &self,
        file_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        debug!(context.logger, "Analyzing file: {:?}", file_path);

        // Get file metadata
        let metadata = std::fs::metadata(file_path)
            .map_err(|e| anyhow!("Failed to get file metadata: {}", e))?;

        let file_size = metadata.len();

        // Read file content
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| anyhow!("Failed to read file: {}", e))?;

        // Calculate basic metrics
        let line_count = content.lines().count();
        let char_count = content.chars().count();
        let word_count = content.split_whitespace().count();
        let blank_lines = content.lines().filter(|line| line.trim().is_empty()).count();

        let result = PluginResult::success("File analysis completed")
            .with_metric("file_size_bytes", file_size as f64)
            .with_metric("line_count", line_count as f64)
            .with_metric("character_count", char_count as f64)
            .with_metric("word_count", word_count as f64)
            .with_metric("blank_lines", blank_lines as f64)
            .with_metric("non_blank_lines", (line_count - blank_lines) as f64)
            .with_data(serde_json::json!({
                "file_path": file_path.display().to_string(),
                "extension": file_path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                "metrics": {
                    "size_bytes": file_size,
                    "lines": line_count,
                    "characters": char_count,
                    "words": word_count,
                    "blank_lines": blank_lines,
                    "code_lines": line_count - blank_lines
                }
            }));

        Ok(result)
    }

    async fn analyze_project(
        &self,
        project_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        debug!(context.logger, "Analyzing project: {:?}", project_path);

        let mut total_files = 0;
        let mut total_size = 0u64;
        let mut total_lines = 0;
        let mut file_extensions: HashMap<String, u32> = HashMap::new();

        // Walk through all files in the project
        for entry in walkdir::WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(metadata) = entry.metadata() {
                total_files += 1;
                total_size += metadata.len();

                if let Some(extension) = entry.path().extension().and_then(|s| s.to_str()) {
                    *file_extensions.entry(extension.to_lowercase()).or_default() += 1;
                }

                // Count lines for text files
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    total_lines += content.lines().count();
                }
            }
        }

        let result = PluginResult::success("Project analysis completed")
            .with_metric("total_files", total_files as f64)
            .with_metric("total_size_bytes", total_size as f64)
            .with_metric("total_lines", total_lines as f64)
            .with_metric("avg_file_size", if total_files > 0 { total_size as f64 / total_files as f64 } else { 0.0 })
            .with_data(serde_json::json!({
                "project_path": project_path.display().to_string(),
                "summary": {
                    "files": total_files,
                    "size_bytes": total_size,
                    "lines": total_lines,
                    "extensions": file_extensions
                }
            }));

        Ok(result)
    }

    fn supported_metrics(&self) -> Vec<String> {
        vec![
            "file_size_bytes".to_string(),
            "line_count".to_string(),
            "character_count".to_string(),
            "word_count".to_string(),
            "blank_lines".to_string(),
            "non_blank_lines".to_string(),
        ]
    }
}

/// Helper function to format JSON with sorted keys
fn format_json_with_sorted_keys(value: &serde_json::Value, indent_size: u32) -> Result<String> {
    fn format_value(value: &serde_json::Value, indent: u32, indent_size: u32) -> String {
        let current_indent = " ".repeat(indent as usize);
        let next_indent = " ".repeat((indent + indent_size) as usize);

        match value {
            serde_json::Value::Object(map) => {
                if map.is_empty() {
                    "{}".to_string()
                } else {
                    let mut sorted_keys: Vec<_> = map.keys().collect();
                    sorted_keys.sort();
                    
                    let mut result = "{\n".to_string();
                    for (i, key) in sorted_keys.iter().enumerate() {
                        let val = &map[*key];
                        result.push_str(&format!("{}\"{}\":", next_indent, key));
                        
                        if matches!(val, serde_json::Value::Object(_) | serde_json::Value::Array(_)) {
                            result.push('\n');
                            result.push_str(&next_indent);
                        } else {
                            result.push(' ');
                        }
                        
                        result.push_str(&format_value(val, indent + indent_size, indent_size));
                        
                        if i < sorted_keys.len() - 1 {
                            result.push(',');
                        }
                        result.push('\n');
                    }
                    result.push_str(&format!("{}}}", current_indent));
                    result
                }
            }
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    "[]".to_string()
                } else {
                    let mut result = "[\n".to_string();
                    for (i, item) in arr.iter().enumerate() {
                        result.push_str(&next_indent);
                        result.push_str(&format_value(item, indent + indent_size, indent_size));
                        if i < arr.len() - 1 {
                            result.push(',');
                        }
                        result.push('\n');
                    }
                    result.push_str(&format!("{}]", current_indent));
                    result
                }
            }
            _ => serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
        }
    }

    Ok(format_value(value, 0, indent_size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use slog::Drain;
    use std::collections::HashMap;

    fn create_test_logger() -> slog::Logger {
        let drain = slog::Discard;
        slog::Logger::root(drain, slog::o!())
    }

    #[tokio::test]
    async fn test_basic_analyzer() {
        let mut analyzer = BasicAnalyzerPlugin::new();
        let logger = create_test_logger();
        
        let context = PluginContext {
            working_dir: std::env::current_dir().unwrap(),
            config: HashMap::new(),
            security_policy: crate::tools::policy::SecurityPolicy::default(),
            resource_limits: super::ResourceLimits::default(),
            logger,
        };

        analyzer.initialize(&context).unwrap();

        // Create a temporary file for testing
        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "Line 1\nLine 2\n\nLine 4").unwrap();

        let result = analyzer.analyze_file(&test_file, &context).await.unwrap();
        assert!(result.success);
        assert_eq!(result.metrics.get("line_count"), Some(&4.0));
        assert_eq!(result.metrics.get("blank_lines"), Some(&1.0));
        assert_eq!(result.metrics.get("non_blank_lines"), Some(&3.0));
    }

    #[test]
    fn test_json_formatting() {
        let json_str = r#"{"b": 2, "a": 1, "c": [3, 2, 1]}"#;
        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        
        let formatted = format_json_with_sorted_keys(&parsed, 2).unwrap();
        
        // Check that keys are sorted and properly formatted
        assert!(formatted.contains("\"a\": 1"));
        assert!(formatted.contains("\"b\": 2"));
        assert!(formatted.contains("\"c\":"));
        
        // Check that 'a' comes before 'b' in the formatted output
        let a_pos = formatted.find("\"a\":").unwrap();
        let b_pos = formatted.find("\"b\":").unwrap();
        assert!(a_pos < b_pos);
    }
}

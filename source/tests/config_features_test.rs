use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;
use anyhow::Result;
use tempfile::tempdir;

// Import the config module from the main crate
use synx::config::Config;

#[test]
fn test_default_feature_values() -> Result<()> {
    // Test that default configuration has expected values for new features
    let config = Config::default();
    
    // Check cache settings
    assert_eq!(config.enable_cache, false);
    assert_eq!(config.cache_duration, 15);
    
    // Check security defaults
    assert_eq!(config.security.enable_sandbox, Some(false));
    assert_eq!(config.security.verify_checksums, Some(false));
    assert_eq!(config.security.auto_update_checksums, Some(false));
    assert_eq!(config.security.enable_audit, Some(false));
    assert!(config.security.audit_log.is_none());
    assert_eq!(config.security.max_file_size, Some(5242880)); // 5MB
    assert!(config.security.allowed_dirs.is_none());
    
    // Check resource limits defaults
    let resource_limits = config.security.resource_limits.as_ref().unwrap();
    assert_eq!(resource_limits.max_memory, Some(256));
    assert_eq!(resource_limits.max_cpu, Some(30));
    assert_eq!(resource_limits.max_io_rate, Some(5));
    assert_eq!(resource_limits.max_execution_time, Some(15));
    
    // Check TUI defaults
    assert_eq!(config.tui.show_examples, Some(true));
    assert_eq!(config.tui.side_by_side, Some(true));
    assert_eq!(config.tui.show_line_numbers, Some(true));
    assert_eq!(config.tui.syntax_highlighting, Some(true));
    assert_eq!(config.tui.show_shortcuts, Some(true));
    assert_eq!(config.tui.default_tab, Some("issues".to_string()));
    assert_eq!(config.tui.max_suggestions, Some(3));
    
    // Check color scheme defaults
    let colors = config.tui.colors.as_ref().unwrap();
    assert_eq!(colors.background, Some("default".to_string()));
    assert_eq!(colors.foreground, Some("white".to_string()));
    assert_eq!(colors.border, Some("blue".to_string()));
    assert_eq!(colors.selected, Some("yellow".to_string()));
    assert_eq!(colors.error, Some("red".to_string()));
    assert_eq!(colors.warning, Some("yellow".to_string()));
    assert_eq!(colors.info, Some("blue".to_string()));
    
    Ok(())
}

#[test]
fn test_security_config() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("security_config.toml");
    
    // Create a test configuration file with security settings
    let mut file = File::create(&config_path)?;
    writeln!(file, r#"
[security]
enable_sandbox = true
verify_checksums = true
auto_update_checksums = false
enable_audit = true
audit_log = "/var/log/synx/audit.log"
max_file_size = 10485760  # 10MB
allowed_dirs = [
    "${PWD}/source",
    "${PWD}/tests"
]

[security.resource_limits]
max_memory = 512
max_cpu = 50
max_io_rate = 10
max_execution_time = 30
"#)?;
    
    // Load the configuration
    let config = Config::new(
        None, None, None, None, None, None, None,
        Some(config_path.to_str().unwrap())
    )?;
    
    // Check security settings
    assert_eq!(config.security.enable_sandbox, Some(true));
    assert_eq!(config.security.verify_checksums, Some(true));
    assert_eq!(config.security.auto_update_checksums, Some(false));
    assert_eq!(config.security.enable_audit, Some(true));
    assert_eq!(config.security.audit_log, Some("/var/log/synx/audit.log".to_string()));
    assert_eq!(config.security.max_file_size, Some(10485760));
    
    // Check allowed directories
    let allowed_dirs = config.security.allowed_dirs.as_ref().unwrap();
    assert_eq!(allowed_dirs.len(), 2);
    assert_eq!(allowed_dirs[0], "${PWD}/source");
    assert_eq!(allowed_dirs[1], "${PWD}/tests");
    
    // Check resource limits
    let limits = config.security.resource_limits.as_ref().unwrap();
    assert_eq!(limits.max_memory, Some(512));
    assert_eq!(limits.max_cpu, Some(50));
    assert_eq!(limits.max_io_rate, Some(10));
    assert_eq!(limits.max_execution_time, Some(30));
    
    Ok(())
}

#[test]
fn test_tui_config() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("tui_config.toml");
    
    // Create a test configuration file with TUI settings
    let mut file = File::create(&config_path)?;
    writeln!(file, r#"
[tui]
show_examples = true
side_by_side = true
show_line_numbers = true
syntax_highlighting = true
show_shortcuts = true
default_tab = "explanation"
max_suggestions = 5

[tui.colors]
background = "black"
foreground = "white"
border = "blue"
selected = "yellow"
error = "bright_red"
warning = "bright_yellow"
info = "cyan"
line_numbers = "gray"
keywords = "bright_blue"
strings = "bright_green"
comments = "gray"
types = "bright_purple"
correct = "bright_green"
incorrect = "bright_red"
"#)?;
    
    // Load the configuration
    let config = Config::new(
        None, None, None, None, None, None, None,
        Some(config_path.to_str().unwrap())
    )?;
    
    // Check TUI settings
    assert_eq!(config.tui.show_examples, Some(true));
    assert_eq!(config.tui.side_by_side, Some(true));
    assert_eq!(config.tui.show_line_numbers, Some(true));
    assert_eq!(config.tui.syntax_highlighting, Some(true));
    assert_eq!(config.tui.show_shortcuts, Some(true));
    assert_eq!(config.tui.default_tab, Some("explanation".to_string()));
    assert_eq!(config.tui.max_suggestions, Some(5));
    
    // Check color scheme
    let colors = config.tui.colors.as_ref().unwrap();
    assert_eq!(colors.background, Some("black".to_string()));
    assert_eq!(colors.foreground, Some("white".to_string()));
    assert_eq!(colors.border, Some("blue".to_string()));
    assert_eq!(colors.selected, Some("yellow".to_string()));
    assert_eq!(colors.error, Some("bright_red".to_string()));
    assert_eq!(colors.warning, Some("bright_yellow".to_string()));
    assert_eq!(colors.info, Some("cyan".to_string()));
    assert_eq!(colors.line_numbers, Some("gray".to_string()));
    assert_eq!(colors.keywords, Some("bright_blue".to_string()));
    assert_eq!(colors.strings, Some("bright_green".to_string()));
    assert_eq!(colors.comments, Some("gray".to_string()));
    assert_eq!(colors.types, Some("bright_purple".to_string()));
    assert_eq!(colors.correct, Some("bright_green".to_string()));
    assert_eq!(colors.incorrect, Some("bright_red".to_string()));
    
    Ok(())
}

#[test]
fn test_cache_settings() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("cache_config.toml");
    
    // Create a test configuration file with cache settings
    let mut file = File::create(&config_path)?;
    writeln!(file, r#"
[general]
strict = false
verbose = true
enable_cache = true
cache_duration = 60
"#)?;
    
    // Load the configuration
    let config = Config::new(
        None, None, None, None, None, None, None,
        Some(config_path.to_str().unwrap())
    )?;
    
    // Check cache settings
    assert_eq!(config.enable_cache, true);
    assert_eq!(config.cache_duration, 60);
    
    // Test command-line override
    let config_with_override = Config::new(
        None, None, None, None, None, 
        Some(false), // Disable cache via command line
        Some(30),    // Set different cache duration
        Some(config_path.to_str().unwrap())
    )?;
    
    // Command-line args should override file
    assert_eq!(config_with_override.enable_cache, false);
    assert_eq!(config_with_override.cache_duration, 30);
    
    Ok(())
}

#[test]
fn test_integrated_features() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("integrated_config.toml");
    
    // Create a test configuration file with all new features
    let mut file = File::create(&config_path)?;
    writeln!(file, r#"
[general]
strict = true
verbose = true
enable_cache = true
cache_duration = 45

[validators.rust]
edition = "2021"
clippy = true
clippy_flags = ["--deny=warnings"]
custom_rules = ["unused_variable", "unused_import"]

[validators.python]
mypy_strict = true
pylint_threshold = 9.0
formatter = "black"
line_length = 88

[validators.javascript]
eslint_config = "./custom_eslint.json"
node_version = "18"
formatter = "prettier"

[security]
enable_sandbox = true
verify_checksums = true
max_file_size = 8388608  # 8MB

[security.resource_limits]
max_memory = 512
max_cpu = 40
max_execution_time = 25

[tui]
show_examples = true
default_tab = "explanation"
max_suggestions = 4

[tui.colors]
background = "black"
error = "bright_red"
keywords = "bright_blue"
"#)?;
    
    // Load the configuration
    let config = Config::new(
        None, None, None, None, None, None, None,
        Some(config_path.to_str().unwrap())
    )?;
    
    // Check general settings
    assert_eq!(config.strict, true);
    assert_eq!(config.verbose, true);
    assert_eq!(config.enable_cache, true);
    assert_eq!(config.cache_duration, 45);
    
    // Check validator settings
    assert_eq!(config.validators.rust.edition, Some("2021".to_string()));
    assert_eq!(config.validators.rust.clippy, Some(true));
    let custom_rules = config.validators.rust.custom_rules.as_ref().unwrap();
    assert_eq!(custom_rules.len(), 2);
    assert_eq!(custom_rules[0], "unused_variable");
    
    assert_eq!(config.validators.python.formatter, Some("black".to_string()));
    assert_eq!(config.validators.python.line_length, Some(88));
    
    assert_eq!(config.validators.javascript.formatter, Some("prettier".to_string()));
    
    // Check security settings
    assert_eq!(config.security.enable_sandbox, Some(true));
    assert_eq!(config.security.verify_checksums, Some(true));
    assert_eq!(config.security.max_file_size, Some(8388608));
    
    // Check resource limits
    let limits = config.security.resource_limits.as_ref().unwrap();
    assert_eq!(limits.max_memory, Some(512));
    assert_eq!(limits.max_cpu, Some(40));
    assert_eq!(limits.max_execution_time, Some(25));
    
    // Check TUI settings
    assert_eq!(config.tui.default_tab, Some("explanation".to_string()));
    assert_eq!(config.tui.max_suggestions, Some(4));
    
    // Check color scheme
    let colors = config.tui.colors.as_ref().unwrap();
    assert_eq!(colors.background, Some("black".to_string()));
    assert_eq!(colors.error, Some("bright_red".to_string()));
    assert_eq!(colors.keywords, Some("bright_blue".to_string()));
    
    // Verify that saving and reloading preserves all settings
    let save_path = temp_dir.path().join("saved_integrated.toml");
    config.save_to_file(&save_path)?;
    
    // Load the saved configuration
    let loaded_config = Config::new(
        None, None, None, None, None, None, None,
        Some(save_path.to_str().unwrap())
    )?;
    
    // Verify all settings were preserved
    assert_eq!(loaded_config.enable_cache, true);
    assert_eq!(loaded_config.cache_duration, 45);
    assert_eq!(loaded_config.security.enable_sandbox, Some(true));
    assert_eq!(loaded_config.validators.python.formatter, Some("black".to_string()));
    assert_eq!(loaded_config.validators.python.line_length, Some(88));
    assert_eq!(loaded_config.tui.default_tab, Some("explanation".to_string()));
    
    Ok(())
}

#[test]
fn test_backward_compatibility() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("old_config.toml");
    
    // Create a test configuration file with only old settings
    let mut file = File::create(&config_path)?;
    writeln!(file, r#"
[general]
strict = true
verbose = true
watch = false
watch_interval = 5

[validators.rust]
edition = "2021"
clippy = true

[validators.python]
mypy_strict = true
pylint_threshold = 9.0
"#)?;
    
    // Load the configuration
    let config = Config::new(
        None, None, None, None, None, None, None,
        Some(config_path.to_str().unwrap())
    )?;
    
    // Check that old settings were loaded correctly
    assert_eq!(config.strict, true);
    assert_eq!(config.verbose, true);
    assert_eq!(config.watch, false);
    assert_eq!(config.watch_interval, 5);
    assert_eq!(config.validators.rust.edition, Some("2021".to_string()));
    assert_eq!(config.validators.rust.clippy, Some(true));
    assert_eq!(config.validators.python.mypy_strict, Some(true));
    assert_eq!(config.validators.python.pylint_threshold, Some(9.0));
    
    // Check that new settings have default values
    assert_eq!(config.enable_cache, false); // Default value
    assert_eq!(config.cache_duration, 15);  // Default value
    
    // Security settings should have default values
    assert_eq!(config.security.enable_sandbox, Some(false));
    assert_eq!(config.security.verify_checksums, Some(false));
    
    // TUI settings should have default values
    assert_eq!(config.tui.show_examples, Some(true));
    assert_eq!(config.tui.default_tab, Some("issues".to_string()));
    
    Ok(())
}


use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;
use anyhow::Result;
use tempfile::tempdir;

// Import the config module from the main crate
use synx::config::{Config, load_configuration, convert_to_config_file};

#[test]
fn test_default_config() -> Result<()> {
    // Test that default configuration has expected values
    let config = Config::default();
    
    // Check general settings
    assert_eq!(config.strict, false);
    assert_eq!(config.verbose, false);
    assert_eq!(config.watch, false);
    assert_eq!(config.watch_interval, 2);
    
    // Check that we have default validator configs
    assert!(config.validators.rust.edition.is_some());
    assert_eq!(config.validators.rust.edition, Some("2021".to_string()));
    assert_eq!(config.validators.rust.clippy, Some(false));
    
    // Check C config defaults
    assert_eq!(config.validators.c.standard, Some("c11".to_string()));
    assert_eq!(config.validators.c.check_memory, Some(false));
    
    // Check that custom validators map is empty
    assert!(config.validators.custom.is_empty());
    
    Ok(())
}

#[test]
fn test_load_from_file() -> Result<()> {
    // Create a temporary directory for the test
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("synx.toml");
    
    // Create a test configuration file
    let mut file = File::create(&config_path)?;
    writeln!(file, r#"
[general]
strict = true
verbose = true
watch = false
watch_interval = 10

[validators.rust]
edition = "2021"
clippy = true
clippy_flags = ["--deny=warnings"]

[validators.python]
mypy_strict = true
pylint_threshold = 9.0
"#)?;
    
    // Load the configuration
    let config = load_configuration(Some(config_path.to_str().unwrap()))?;
    
    // Check that general settings were loaded
    assert_eq!(config.general.strict, Some(true));
    assert_eq!(config.general.verbose, Some(true));
    assert_eq!(config.general.watch, Some(false));
    assert_eq!(config.general.watch_interval, Some(10));
    
    // Check that validator settings were loaded
    assert_eq!(config.validators.rust.clippy, Some(true));
    assert_eq!(config.validators.python.mypy_strict, Some(true));
    assert_eq!(config.validators.python.pylint_threshold, Some(9.0));
    
    Ok(())
}

#[test]
fn test_config_merging() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("synx.toml");
    
    // Create a test configuration file
    let mut file = File::create(&config_path)?;
    writeln!(file, r#"
[general]
strict = true
# verbose is not specified, should use default

[validators.rust]
edition = "2018"
# clippy is not specified, should use default
"#)?;
    
    // Create a config with command-line options
    let config = Config::new(
        Some(false), // Override strict from config file
        Some(true),  // Set verbose explicitly
        None,        // Use default for watch
        Some(5),     // Override watch_interval
        Some(config_path.to_str().unwrap())
    )?;
    
    // Check merging precedence
    assert_eq!(config.strict, false); // Command-line overrides config file
    assert_eq!(config.verbose, true); // Command-line sets this
    assert_eq!(config.watch, false);  // Default value
    assert_eq!(config.watch_interval, 5); // Command-line sets this
    
    // Check validator merging
    assert_eq!(config.validators.rust.edition, Some("2018".to_string())); // From config file
    assert_eq!(config.validators.rust.clippy, Some(false)); // Default value
    
    Ok(())
}

#[test]
fn test_validator_specific_settings() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("synx.toml");
    
    // Create a test configuration with multiple validators
    let mut file = File::create(&config_path)?;
    writeln!(file, r#"
[validators.c]
standard = "c99"
check_memory = true
include_paths = ["/usr/include", "/opt/include"]

[validators.javascript]
eslint_config = "./custom_eslint.json"
node_version = "18"

[validators.typescript]
eslint_config = "./custom_eslint.json"
tsconfig = "./tsconfig.strict.json"

[validators.custom.xml]
command = "xmllint"
args = ["--noout"]
strict_args = ["--dtdvalid", "schema.dtd"]
success_pattern = "validates$"
"#)?;
    
    // Load the configuration
    let config = Config::new(
        None, None, None, None,
        Some(config_path.to_str().unwrap())
    )?;
    
    // Check C validator settings
    assert_eq!(config.validators.c.standard, Some("c99".to_string()));
    assert_eq!(config.validators.c.check_memory, Some(true));
    assert!(config.validators.c.include_paths.is_some());
    let include_paths = config.validators.c.include_paths.as_ref().unwrap();
    assert_eq!(include_paths.len(), 2);
    assert_eq!(include_paths[0], "/usr/include");
    
    // Check JavaScript validator settings
    assert_eq!(config.validators.javascript.eslint_config, Some("./custom_eslint.json".to_string()));
    assert_eq!(config.validators.javascript.node_version, Some("18".to_string()));
    
    // Check TypeScript validator settings
    assert_eq!(config.validators.typescript.eslint_config, Some("./custom_eslint.json".to_string()));
    assert_eq!(config.validators.typescript.tsconfig, Some("./tsconfig.strict.json".to_string()));
    
    // Check custom validator
    assert!(config.validators.custom.contains_key("xml"));
    let xml_config = &config.validators.custom["xml"];
    assert_eq!(xml_config.command, "xmllint");
    assert_eq!(xml_config.args.as_ref().unwrap(), &vec!["--noout".to_string()]);
    
    Ok(())
}

#[test]
fn test_missing_config_file() -> Result<()> {
    // Try to load from a nonexistent file
    let result = load_configuration(Some("nonexistent_file.toml"));
    
    // Should return a default config, not an error
    assert!(result.is_ok());
    let config = result?;
    
    // Default general settings should be present
    assert!(config.general.strict.is_none());
    assert!(config.general.verbose.is_none());
    
    Ok(())
}

#[test]
fn test_invalid_config_syntax() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("invalid.toml");
    
    // Create an invalid TOML file
    let mut file = File::create(&config_path)?;
    writeln!(file, r#"
[general
strict = true  # Missing closing bracket
verbose = true
"#)?;
    
    // Try to load the invalid configuration
    let result = load_configuration(Some(config_path.to_str().unwrap()));
    
    // Should return an error
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_config_file_save() -> Result<()> {
    // Create a configuration
    let mut config = Config::default();
    config.strict = true;
    config.verbose = true;
    config.validators.rust.clippy = Some(true);
    
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let save_path = temp_dir.path().join("saved_config.toml");
    
    // Save the configuration
    config.save_to_file(&save_path)?;
    
    // Check that the file exists
    assert!(save_path.exists());
    
    // Read back the configuration
    let loaded_config = load_configuration(Some(save_path.to_str().unwrap()))?;
    
    // Check that settings were preserved
    assert_eq!(loaded_config.general.strict, Some(true));
    assert_eq!(loaded_config.general.verbose, Some(true));
    assert_eq!(loaded_config.validators.rust.clippy, Some(true));
    
    Ok(())
}


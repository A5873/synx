use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::collections::HashMap;
use anyhow::{Result, Context, anyhow};
use serde::{Deserialize, Serialize};

/// Configuration structure for synx
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// General configuration settings
    #[serde(default)]
    pub general: GeneralConfig,
    
    /// Validator-specific configurations
    #[serde(default)]
    pub validators: HashMap<String, ValidatorConfig>,
    
    /// Custom file extensions mappings
    #[serde(default)]
    pub file_mappings: HashMap<String, String>,
}

/// General configuration settings
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct GeneralConfig {
    /// Show verbose output by default
    #[serde(default)]
    pub verbose: bool,
    
    /// Use strict mode by default (fail on warnings)
    #[serde(default)]
    pub strict: bool,
    
    /// Default validator timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_timeout() -> u64 {
    30
}

/// Configuration for specific validators
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ValidatorConfig {
    /// Custom command to run instead of the default
    pub command: Option<String>,
    
    /// Custom arguments to pass to the validator
    pub args: Option<Vec<String>>,
    
    /// Whether to treat warnings as errors
    #[serde(default)]
    pub strict: bool,
    
    /// Custom timeout for this validator in seconds
    pub timeout: Option<u64>,
    
    /// Whether to use this validator
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load configuration from a file or use defaults
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        let config_file = if let Some(path) = config_path {
            path.to_path_buf()
        } else {
            get_default_config_path().context("Failed to get default config path")?
        };
        
        if !config_file.exists() {
            if config_path.is_some() {
                return Err(anyhow!("Specified config file does not exist: {:?}", config_file));
            }
            return Ok(Self::default());
        }
        
        let mut file = File::open(&config_file)
            .context(format!("Failed to open config file: {:?}", config_file))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context(format!("Failed to read config file: {:?}", config_file))?;
        
        let config: Config = toml::from_str(&contents)
            .context(format!("Failed to parse TOML in config file: {:?}", config_file))?;
        
        Ok(config)
    }
    
    /// Generate a default configuration file
    pub fn generate_default_config() -> Result<PathBuf> {
        let config_path = get_default_config_path()
            .context("Failed to determine default config path")?;
        
        // Create the config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create config directory: {:?}", parent))?;
        }
        
        // Only create the file if it doesn't exist
        if !config_path.exists() {
            let default_config = Config::default();
            let toml_string = toml::to_string_pretty(&default_config)
                .context("Failed to serialize default config")?;
            
            let mut file = File::create(&config_path)
                .context(format!("Failed to create config file: {:?}", config_path))?;
            
            file.write_all(toml_string.as_bytes())
                .context("Failed to write default config")?;
            
            println!("Created default config at {:?}", config_path);
        }
        
        Ok(config_path)
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut validators = HashMap::new();
        
        // Add default configurations for common validators
        validators.insert("python".to_string(), ValidatorConfig {
            command: Some("python".to_string()),
            args: Some(vec!["-m".to_string(), "py_compile".to_string()]),
            strict: false,
            timeout: Some(10),
            enabled: true,
        });
        
        validators.insert("javascript".to_string(), ValidatorConfig {
            command: Some("node".to_string()),
            args: Some(vec!["--check".to_string()]),
            strict: false,
            timeout: Some(10),
            enabled: true,
        });
        
        validators.insert("json".to_string(), ValidatorConfig {
            command: Some("jq".to_string()),
            args: Some(vec![".".to_string()]),
            strict: true,
            timeout: Some(5),
            enabled: true,
        });
        
        validators.insert("yaml".to_string(), ValidatorConfig {
            command: Some("yamllint".to_string()),
            args: Some(vec!["-f".to_string(), "parsable".to_string()]),
            strict: false,
            timeout: Some(10),
            enabled: true,
        });
        
        // Add HTML validator
        validators.insert("html".to_string(), ValidatorConfig {
            command: Some("tidy".to_string()),
            args: Some(vec!["-q".to_string(), "-e".to_string()]),
            strict: false,
            timeout: Some(10),
            enabled: true,
        });
        
        // Add CSS validator
        validators.insert("css".to_string(), ValidatorConfig {
            command: Some("csslint".to_string()),
            args: None,
            strict: false,
            timeout: Some(10),
            enabled: true,
        });
        
        // Add Dockerfile validator
        validators.insert("dockerfile".to_string(), ValidatorConfig {
            command: Some("hadolint".to_string()),
            args: None,
            strict: true,
            timeout: Some(10),
            enabled: true,
        });
        
        // Add Shell validator
        validators.insert("shell".to_string(), ValidatorConfig {
            command: Some("shellcheck".to_string()),
            args: None,
            strict: false,
            timeout: Some(10),
            enabled: true,
        });
        
        // Add Markdown validator
        validators.insert("markdown".to_string(), ValidatorConfig {
            command: Some("mdl".to_string()),
            args: None,
            strict: false,
            timeout: Some(10),
            enabled: true,
        });
        
        // Add Rust validator
        validators.insert("rust".to_string(), ValidatorConfig {
            command: Some("rustc".to_string()),
            args: Some(vec!["--emit=check".to_string(), "--crate-type=lib".to_string()]),
            strict: true,
            timeout: Some(15),
            enabled: true,
        });
        
        // Create some default file mappings
        let mut file_mappings = HashMap::new();
        file_mappings.insert("Dockerfile".to_string(), "dockerfile".to_string());
        file_mappings.insert("Jenkinsfile".to_string(), "groovy".to_string());
        file_mappings.insert("Makefile".to_string(), "makefile".to_string());
        
        Self {
            general: GeneralConfig {
                verbose: false,
                strict: false,
                timeout: 30,
            },
            validators,
            file_mappings,
        }
    }
}

/// Get the default configuration file path
pub fn get_default_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not determine home directory"))?;
    
    Ok(home_dir.join(".config").join("synx").join("config.toml"))
}

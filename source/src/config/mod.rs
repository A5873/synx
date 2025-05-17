use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow, Context};
use dirs;

// Main configuration struct that includes all settings
#[derive(Debug, Clone)]
pub struct Config {
    // General settings
    pub strict: bool,
    pub verbose: bool,
    pub watch: bool,
    pub watch_interval: u64,
    
    // Path to config file (if specified)
    pub config_path: Option<PathBuf>,
    
    // Language-specific settings
    pub validators: ValidatorConfigs,
}

// Container for all language-specific configurations
#[derive(Debug, Clone, Default)]
pub struct ValidatorConfigs {
    pub rust: RustConfig,
    pub cpp: CppConfig,
    pub c: CConfig,
    pub csharp: CSharpConfig,
    pub python: PythonConfig,
    pub javascript: JavaScriptConfig,
    pub typescript: TypeScriptConfig,
    pub go: GoConfig,
    pub java: JavaConfig,
    pub html: HtmlConfig,
    pub css: CssConfig,
    pub yaml: YamlConfig,
    pub json: JsonConfig,
    pub shell: ShellConfig,
    pub dockerfile: DockerfileConfig,
    // Custom validators map for extensibility
    pub custom: HashMap<String, CustomValidatorConfig>,
}

// Language-specific configuration structs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RustConfig {
    pub edition: Option<String>,       // Rust edition to use (e.g., "2021")
    pub clippy: Option<bool>,          // Whether to run clippy
    pub clippy_flags: Option<Vec<String>>, // Additional clippy flags
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CppConfig {
    pub standard: Option<String>,      // C++ standard to use (e.g., "c++17")
    pub include_paths: Option<Vec<String>>, // Additional include paths
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CConfig {
    pub standard: Option<String>,      // C standard to use (e.g., "c11")
    pub check_memory: Option<bool>,    // Whether to check for memory leaks
    pub include_paths: Option<Vec<String>>, // Additional include paths
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CSharpConfig {
    pub use_dotnet: Option<bool>,      // Whether to prefer dotnet CLI over Mono
    pub framework: Option<String>,     // Target framework (e.g., "net6.0")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PythonConfig {
    pub mypy_strict: Option<bool>,     // Whether to use strict type checking
    pub pylint_threshold: Option<f64>, // Pylint score threshold
    pub ignore_rules: Option<Vec<String>>, // Rules to ignore
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JavaScriptConfig {
    pub eslint_config: Option<String>, // Path to custom ESLint config
    pub node_version: Option<String>,  // Target Node.js version
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TypeScriptConfig {
    pub eslint_config: Option<String>, // Path to custom ESLint config
    pub tsconfig: Option<String>,      // Path to tsconfig.json
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoConfig {
    pub test: Option<bool>,            // Whether to run tests
    pub lint_flags: Option<Vec<String>>, // Additional golangci-lint flags
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JavaConfig {
    pub checkstyle_config: Option<String>, // Path to checkstyle config
    pub version: Option<String>,       // Java version to target
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HtmlConfig {
    pub tidy_flags: Option<Vec<String>>, // Additional tidy flags
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CssConfig {
    pub csslint_flags: Option<Vec<String>>, // Additional csslint flags
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YamlConfig {
    pub custom_config: Option<String>, // Path to custom yamllint config
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonConfig {
    pub allow_comments: Option<bool>,  // Whether to allow comments in JSON
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShellConfig {
    pub shell_type: Option<String>,    // Shell type (bash, sh, zsh)
    pub ignore_rules: Option<Vec<String>>, // Shellcheck rules to ignore
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DockerfileConfig {
    pub ignore_rules: Option<Vec<String>>, // Hadolint rules to ignore
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomValidatorConfig {
    pub command: String,               // Command to run
    pub args: Option<Vec<String>>,     // Arguments for the command
    pub strict_args: Option<Vec<String>>, // Additional args for strict mode
    pub success_pattern: Option<String>, // Regex pattern for success
}

// Implement Default for each config struct
impl Default for RustConfig {
    fn default() -> Self {
        Self {
            edition: Some("2021".to_string()),
            clippy: Some(false),
            clippy_flags: None,
        }
    }
}

impl Default for CppConfig {
    fn default() -> Self {
        Self {
            standard: Some("c++17".to_string()),
            include_paths: None,
        }
    }
}

impl Default for CConfig {
    fn default() -> Self {
        Self {
            standard: Some("c11".to_string()),
            check_memory: Some(false),
            include_paths: None,
        }
    }
}

impl Default for CSharpConfig {
    fn default() -> Self {
        Self {
            use_dotnet: Some(true),
            framework: None,
        }
    }
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            mypy_strict: Some(false),
            pylint_threshold: Some(7.0),
            ignore_rules: None,
        }
    }
}

impl Default for JavaScriptConfig {
    fn default() -> Self {
        Self {
            eslint_config: None,
            node_version: None,
        }
    }
}

impl Default for TypeScriptConfig {
    fn default() -> Self {
        Self {
            eslint_config: None,
            tsconfig: None,
        }
    }
}

impl Default for GoConfig {
    fn default() -> Self {
        Self {
            test: Some(false),
            lint_flags: None,
        }
    }
}

impl Default for JavaConfig {
    fn default() -> Self {
        Self {
            checkstyle_config: None,
            version: None,
        }
    }
}

impl Default for HtmlConfig {
    fn default() -> Self {
        Self {
            tidy_flags: None,
        }
    }
}

impl Default for CssConfig {
    fn default() -> Self {
        Self {
            csslint_flags: None,
        }
    }
}

impl Default for YamlConfig {
    fn default() -> Self {
        Self {
            custom_config: None,
        }
    }
}

impl Default for JsonConfig {
    fn default() -> Self {
        Self {
            allow_comments: Some(false),
        }
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            shell_type: None,
            ignore_rules: None,
        }
    }
}

impl Default for DockerfileConfig {
    fn default() -> Self {
        Self {
            ignore_rules: None,
        }
    }
}

// TOML config file structure
#[derive(Debug, Deserialize, Serialize)]
struct ConfigFile {
    general: Option<GeneralConfig>,
    validators: Option<ValidatorsConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeneralConfig {
    strict: Option<bool>,
    verbose: Option<bool>,
    watch: Option<bool>,
    watch_interval: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ValidatorsConfig {
    rust: Option<RustConfig>,
    cpp: Option<CppConfig>,
    c: Option<CConfig>,
    csharp: Option<CSharpConfig>,
    python: Option<PythonConfig>,
    javascript: Option<JavaScriptConfig>,
    typescript: Option<TypeScriptConfig>,
    go: Option<GoConfig>,
    java: Option<JavaConfig>,
    html: Option<HtmlConfig>,
    css: Option<CssConfig>,
    yaml: Option<YamlConfig>,
    json: Option<JsonConfig>,
    shell: Option<ShellConfig>,
    dockerfile: Option<DockerfileConfig>,
    custom: Option<HashMap<String, CustomValidatorConfig>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            strict: false,
            verbose: false,
            watch: false,
            watch_interval: 2,
            config_path: None,
            validators: ValidatorConfigs::default(),
        }
    }
}

impl Config {
    // Create a new Config with the specified options and load configuration files
    pub fn new(
        strict: Option<bool>,
        verbose: Option<bool>,
        watch: Option<bool>,
        watch_interval: Option<u64>,
        config_path: Option<&str>,
    ) -> Result<Self> {
        // Start with defaults
        let mut config = Config::default();
        
        // Load configuration from files
        let loaded_config = load_configuration(config_path)?;
        
        // Override with loaded configuration values
        if let Some(general) = &loaded_config.general {
            if let Some(strict_val) = general.strict {
                config.strict = strict_val;
            }
            if let Some(verbose_val) = general.verbose {
                config.verbose = verbose_val;
            }
            if let Some(watch_val) = general.watch {
                config.watch = watch_val;
            }
            if let Some(interval) = general.watch_interval {
                config.watch_interval = interval;
            }
        }
        
        // Load validator configurations
        if let Some(validators) = &loaded_config.validators {
            if let Some(rust_config) = &validators.rust {
                merge_into(&mut config.validators.rust, rust_config);
            }
            if let Some(cpp_config) = &validators.cpp {
                merge_into(&mut config.validators.cpp, cpp_config);
            }
            if let Some(c_config) = &validators.c {
                merge_into(&mut config.validators.c, c_config);
            }
            if let Some(csharp_config) = &validators.csharp {
                merge_into(&mut config.validators.csharp, csharp_config);
            }
            if let Some(python_config) = &validators.python {
                merge_into(&mut config.validators.python, python_config);
            }
            if let Some(js_config) = &validators.javascript {
                merge_into(&mut config.validators.javascript, js_config);
            }
            if let Some(ts_config) = &validators.typescript {
                merge_into(&mut config.validators.typescript, ts_config);
            }
            if let Some(go_config) = &validators.go {
                merge_into(&mut config.validators.go, go_config);
            }
            if let Some(java_config) = &validators.java {
                merge_into(&mut config.validators.java, java_config);
            }
            if let Some(html_config) = &validators.html {
                merge_into(&mut config.validators.html, html_config);
            }
            if let Some(css_config) = &validators.css {
                merge_into(&mut config.validators.css, css_config);
            }
            if let Some(yaml_config) = &validators.yaml {
                merge_into(&mut config.validators.yaml, yaml_config);
            }
            if let Some(json_config) = &validators.json {
                merge_into(&mut config.validators.json, json_config);
            }
            if let Some(shell_config) = &validators.shell {
                merge_into(&mut config.validators.shell, shell_config);
            }
            if let Some(dockerfile_config) = &validators.dockerfile {
                merge_into(&mut config.validators.dockerfile, dockerfile_config);
            }
            if let Some(custom_configs) = &validators.custom {
                for (name, custom_config) in custom_configs {
                    config.validators.custom.insert(name.clone(), custom_config.clone());
                }
            }
        }
        
        // Override with command-line options (highest precedence)
        if let Some(strict_val) = strict {
            config.strict = strict_val;
        }
        if let Some(verbose_val) = verbose {
            config.verbose = verbose_val;
        }
        if let Some(watch_val) = watch {
            config.watch = watch_val;
        }
        if let Some(interval) = watch_interval {
            config.watch_interval = interval;
        }
        if let Some(path) = config_path {
            config.config_path = Some(PathBuf::from(path));
        }
        
        Ok(config)
    }
    
    // Save the current configuration to a file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let config_file = convert_to_config_file(self);
        let toml_string = toml::to_string_pretty(&config_file)
            .context("Failed to serialize configuration to TOML")?;
        fs::write(path, toml_string)
            .context(format!("Failed to write configuration to {}", path.display()))?;
        Ok(())
    }
    
    // Create a default configuration file
    pub fn create_default_config(path: &Path) -> Result<()> {
        let config = Config::default();
        config.save_to_file(path)
    }
}

// Helper function to merge configs
fn merge_into<T: Clone>(target: &mut T, source: &T) {
    *target = source.clone();
}

// Convert Config to ConfigFile for serialization
fn convert_to_config_file(config: &Config) -> ConfigFile {
    ConfigFile {
        general: Some(GeneralConfig {
            strict: Some(config.strict),
            verbose: Some(config.verbose),
            watch: Some(config.watch),
            watch_interval: Some(config.watch_interval),
        }),
        validators: Some(ValidatorsConfig {
            rust: Some(

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

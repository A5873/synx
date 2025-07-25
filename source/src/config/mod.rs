use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::env;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow, Context};
use log::{debug, info, warn};
use dirs;

// Main configuration struct that includes all settings
#[derive(Debug, Clone)]
pub struct Config {
    // General settings
    pub strict: bool,
    pub verbose: bool,
    pub watch: bool,
    pub watch_interval: u64,
    pub timeout: u64,
    
    // Paths to config files that were loaded
    pub loaded_config_paths: Vec<PathBuf>,
    
    // Custom file extensions mappings
    pub file_mappings: HashMap<String, String>,
    
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
    file_mappings: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeneralConfig {
    strict: Option<bool>,
    verbose: Option<bool>,
    watch: Option<bool>,
    watch_interval: Option<u64>,
    timeout: Option<u64>,
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
        // Create default file mappings
        let mut file_mappings = HashMap::new();
        file_mappings.insert("Dockerfile".to_string(), "dockerfile".to_string());
        file_mappings.insert("Jenkinsfile".to_string(), "groovy".to_string());
        file_mappings.insert("Makefile".to_string(), "makefile".to_string());

        Config {
            strict: false,
            verbose: false,
            watch: false,
            watch_interval: 2,
            timeout: 30,
            loaded_config_paths: Vec::new(),
            file_mappings,
            validators: ValidatorConfigs::default(),
        }
    }
}

impl Config {
    /// Create a new Config with the specified options and load configuration files
    /// Loads configurations in the following order (increasing precedence):
    /// 1. System configuration (/etc/synx/config.toml)
    /// 2. User configuration (~/.config/synx/config.toml)
    /// 3. Project configuration (.synx.toml in current directory)
    /// 4. Explicit config path (if provided)
    /// 5. Command-line arguments (highest precedence)
    pub fn new(
        strict: Option<bool>,
        verbose: Option<bool>,
        watch: Option<bool>,
        watch_interval: Option<u64>,
        timeout: Option<u64>,
        explicit_config_path: Option<&str>,
    ) -> Result<Self> {
        // Start with defaults
        let mut config = Config::default();
        
        // Load all configuration files in order of precedence
        let configs = load_all_configurations(explicit_config_path)?;
        
        // Track which config files were loaded
        let mut loaded_paths = Vec::new();
        
        // Process each config in order of precedence
        for (config_file, path) in configs {
            if let Some(path) = path {
                debug!("Loading configuration from {}", path.display());
                loaded_paths.push(path);
            }
            
            // Apply the configuration
            config.merge_from_config_file(&config_file)?;
        }
        
        // Store the loaded config paths
        config.loaded_config_paths = loaded_paths;
        
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
        if let Some(timeout_val) = timeout {
            config.timeout = timeout_val;
        }
        
        Ok(config)
    }
    
    /// Merges settings from a config file into this config
    fn merge_from_config_file(&mut self, config_file: &ConfigFile) -> Result<()> {
        // Merge general settings
        if let Some(general) = &config_file.general {
            if let Some(strict_val) = general.strict {
                self.strict = strict_val;
            }
            if let Some(verbose_val) = general.verbose {
                self.verbose = verbose_val;
            }
            if let Some(watch_val) = general.watch {
                self.watch = watch_val;
            }
            if let Some(interval) = general.watch_interval {
                self.watch_interval = interval;
            }
            if let Some(timeout) = general.timeout {
                self.timeout = timeout;
            }
        }

        // Merge file mappings
        if let Some(mappings) = &config_file.file_mappings {
            for (name, mapping) in mappings {
                self.file_mappings.insert(name.clone(), mapping.clone());
            }
        }
        
        // Merge validator configurations
        if let Some(validators) = &config_file.validators {
            self.merge_validator_configs(validators)?;
        }
        
        Ok(())
    }
    
    /// Merges validator configurations from the provided ValidatorsConfig
    fn merge_validator_configs(&mut self, validators: &ValidatorsConfig) -> Result<()> {
        if let Some(rust_config) = &validators.rust {
            merge_into(&mut self.validators.rust, rust_config);
        }
        if let Some(cpp_config) = &validators.cpp {
            merge_into(&mut self.validators.cpp, cpp_config);
        }
        if let Some(c_config) = &validators.c {
            merge_into(&mut self.validators.c, c_config);
        }
        if let Some(csharp_config) = &validators.csharp {
            merge_into(&mut self.validators.csharp, csharp_config);
        }
        if let Some(python_config) = &validators.python {
            merge_into(&mut self.validators.python, python_config);
        }
        if let Some(js_config) = &validators.javascript {
            merge_into(&mut self.validators.javascript, js_config);
        }
        if let Some(ts_config) = &validators.typescript {
            merge_into(&mut self.validators.typescript, ts_config);
        }
        if let Some(go_config) = &validators.go {
            merge_into(&mut self.validators.go, go_config);
        }
        if let Some(java_config) = &validators.java {
            merge_into(&mut self.validators.java, java_config);
        }
        if let Some(html_config) = &validators.html {
            merge_into(&mut self.validators.html, html_config);
        }
        if let Some(css_config) = &validators.css {
            merge_into(&mut self.validators.css, css_config);
        }
        if let Some(yaml_config) = &validators.yaml {
            merge_into(&mut self.validators.yaml, yaml_config);
        }
        if let Some(json_config) = &validators.json {
            merge_into(&mut self.validators.json, json_config);
        }
        if let Some(shell_config) = &validators.shell {
            merge_into(&mut self.validators.shell, shell_config);
        }
        if let Some(dockerfile_config) = &validators.dockerfile {
            merge_into(&mut self.validators.dockerfile, dockerfile_config);
        }
        if let Some(custom_configs) = &validators.custom {
            for (name, custom_config) in custom_configs {
                self.validators.custom.insert(name.clone(), custom_config.clone());
            }
        }
        
        Ok(())
    }
    
    /// Save the current configuration to a file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let config_file = convert_to_config_file(self);
        let toml_string = toml::to_string_pretty(&config_file)
            .context("Failed to serialize configuration to TOML")?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory {}", parent.display()))?;
        }
        
        fs::write(path, toml_string)
            .context(format!("Failed to write configuration to {}", path.display()))?;
        
        info!("Configuration saved to {}", path.display());
        Ok(())
    }
    
    /// Generate a default configuration file at the default path
    pub fn generate_default_config() -> Result<PathBuf> {
        let config_path = get_default_config_path()
            .context("Failed to determine default config path")?;
        
        let config = Config::default();
        config.save_to_file(&config_path)?;
        
        println!("Created default config at {:?}", config_path);
        Ok(config_path)
    }
}

/// Get the default configuration file path
pub fn get_default_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not determine home directory"))?;
    
    Ok(home_dir.join(".config").join("synx").join("config.toml"))
}

/// Helper function to load configuration file paths in order of precedence
fn load_all_configurations(explicit_path: Option<&str>) -> Result<Vec<(ConfigFile, Option<PathBuf>)>> {
    let mut result = Vec::new();
    
    // 1. Try system-wide configuration
    let system_path = Path::new("/etc/synx/config.toml");
    if system_path.exists() {
        match load_config_file(system_path) {
            Ok(config) => {
                debug!("Loaded system configuration from {}", system_path.display());
                result.push((config, Some(system_path.to_path_buf())));
            }
            Err(e) => {
                warn!("Failed to load system configuration: {}", e);
            }
        }
    }
    
    // 2. Try user configuration
    if let Some(home_dir) = dirs::home_dir() {
        let user_path = home_dir.join(".config").join("synx").join("config.toml");
        if user_path.exists() {
            match load_config_file(&user_path) {
                Ok(config) => {
                    debug!("Loaded user configuration from {}", user_path.display());
                    result.push((config, Some(user_path)));
                }
                Err(e) => {
                    warn!("Failed to load user configuration: {}", e);
                }
            }
        }
    }
    
    // 3. Try project configuration
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_path = current_dir.join(".synx.toml");
    if project_path.exists() {
        match load_config_file(&project_path) {
            Ok(config) => {
                debug!("Loaded project configuration from {}", project_path.display());
                result.push((config, Some(project_path)));
            }
            Err(e) => {
                warn!("Failed to load project configuration: {}", e);
            }
        }
    }
    
    // 4. Try explicit configuration path if provided
    if let Some(path_str) = explicit_path {
        let explicit_path = Path::new(path_str);
        if explicit_path.exists() {
            match load_config_file(explicit_path) {
                Ok(config) => {
                    debug!("Loaded explicit configuration from {}", explicit_path.display());
                    result.push((config, Some(explicit_path.to_path_buf())));
                }
                Err(e) => {
                    warn!("Failed to load explicit configuration: {}", e);
                }
            }
        } else {
            warn!("Specified configuration file does not exist: {}", explicit_path.display());
        }
    }
    
    Ok(result)
}

// Helper function to load a configuration file
fn load_config_file(path: &Path) -> Result<ConfigFile> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read configuration file: {}", path.display()))?;
    
    let config: ConfigFile = toml::from_str(&content)
        .context(format!("Failed to parse TOML in file: {}", path.display()))?;
    
    Ok(config)
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
            timeout: Some(config.timeout),
        }),
        validators: Some(ValidatorsConfig {
            rust: Some(config.validators.rust.clone()),
            cpp: Some(config.validators.cpp.clone()),
            c: Some(config.validators.c.clone()),
            csharp: Some(config.validators.csharp.clone()),
            python: Some(config.validators.python.clone()),
            javascript: Some(config.validators.javascript.clone()),
            typescript: Some(config.validators.typescript.clone()),
            go: Some(config.validators.go.clone()),
            java: Some(config.validators.java.clone()),
            html: Some(config.validators.html.clone()),
            css: Some(config.validators.css.clone()),
            yaml: Some(config.validators.yaml.clone()),
            json: Some(config.validators.json.clone()),
            shell: Some(config.validators.shell.clone()),
            dockerfile: Some(config.validators.dockerfile.clone()),
            custom: if config.validators.custom.is_empty() {
                None
            } else {
                Some(config.validators.custom.clone())
            },
        }),
        file_mappings: if config.file_mappings.is_empty() {
            None
        } else {
            Some(config.file_mappings.clone())
        },
    }
}

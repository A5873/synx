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
    pub enable_cache: bool,
    pub cache_duration: u64,
    
    // Paths to config files that were loaded
    pub loaded_config_paths: Vec<PathBuf>,
    
    // Custom file extensions mappings
    pub file_mappings: HashMap<String, String>,
    
    // Language-specific settings
    pub validators: ValidatorConfigs,
    
    // Security settings
    pub security: SecurityConfig,
    
    // TUI settings
    pub tui: TuiConfig,
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
    pub custom_rules: Option<Vec<String>>, // Custom rules to enable
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
    pub formatter: Option<String>,     // Code formatter to use (e.g., "black")
    pub line_length: Option<u64>,      // Line length for formatter
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JavaScriptConfig {
    pub eslint_config: Option<String>, // Path to custom ESLint config
    pub node_version: Option<String>,  // Target Node.js version
    pub formatter: Option<String>,     // Code formatter to use (e.g., "prettier")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TypeScriptConfig {
    pub eslint_config: Option<String>, // Path to custom ESLint config
    pub tsconfig: Option<String>,      // Path to tsconfig.json
    pub formatter: Option<String>,     // Code formatter to use (e.g., "prettier")
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

// Security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub enable_sandbox: Option<bool>,         // Whether to enable sandbox for secure operation
    pub verify_checksums: Option<bool>,       // Whether to verify checksums of configuration files
    pub auto_update_checksums: Option<bool>,  // Whether to auto-update checksums when files change
    pub enable_audit: Option<bool>,           // Whether to enable audit logging
    pub audit_log: Option<String>,            // Path to audit log file
    pub max_file_size: Option<u64>,           // Maximum file size for processing (in bytes)
    pub allowed_dirs: Option<Vec<String>>,    // Allowed directories for file operations
    pub resource_limits: Option<ResourceLimits>, // Resource limits
}

// Resource limits for security
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceLimits {
    pub max_memory: Option<u64>,           // Maximum memory usage (in MB)
    pub max_cpu: Option<u64>,              // Maximum CPU usage (in percent)
    pub max_io_rate: Option<u64>,          // Maximum I/O rate (in MB/s)
    pub max_execution_time: Option<u64>,   // Maximum execution time (in seconds)
}

// TUI configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TuiConfig {
    pub show_examples: Option<bool>,      // Whether to show examples
    pub side_by_side: Option<bool>,       // Whether to use side-by-side mode for examples
    pub show_line_numbers: Option<bool>,  // Whether to show line numbers
    pub syntax_highlighting: Option<bool>,// Whether to use syntax highlighting
    pub show_shortcuts: Option<bool>,     // Whether to show keyboard shortcuts
    pub default_tab: Option<String>,      // Default starting tab
    pub max_suggestions: Option<u64>,     // Maximum suggestions to show
    pub colors: Option<ColorScheme>,      // Color scheme
}

// TUI color scheme
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColorScheme {
    pub background: Option<String>,       // Background color
    pub foreground: Option<String>,       // Foreground color
    pub border: Option<String>,           // Border color
    pub selected: Option<String>,         // Selected item color
    pub error: Option<String>,            // Error color
    pub warning: Option<String>,          // Warning color
    pub info: Option<String>,             // Info color
    pub line_numbers: Option<String>,     // Line numbers color
    pub keywords: Option<String>,         // Keywords color
    pub strings: Option<String>,          // Strings color
    pub comments: Option<String>,         // Comments color
    pub types: Option<String>,            // Types color
    pub correct: Option<String>,          // Correct example color
    pub incorrect: Option<String>,        // Incorrect example color
}

// Implement Default for each config struct
impl Default for RustConfig {
    fn default() -> Self {
        Self {
            edition: Some("2021".to_string()),
            clippy: Some(false),
            clippy_flags: None,
            custom_rules: None,
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
            formatter: None,
            line_length: None,
        }
    }
}

impl Default for JavaScriptConfig {
    fn default() -> Self {
        Self {
            eslint_config: None,
            node_version: None,
            formatter: None,
        }
    }
}

impl Default for TypeScriptConfig {
    fn default() -> Self {
        Self {
            eslint_config: None,
            tsconfig: None,
            formatter: None,
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

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_sandbox: Some(false),
            verify_checksums: Some(false),
            auto_update_checksums: Some(false),
            enable_audit: Some(false),
            audit_log: None,
            max_file_size: Some(5242880), // 5MB default
            allowed_dirs: None,
            resource_limits: Some(ResourceLimits::default()),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: Some(256),  // 256MB
            max_cpu: Some(30),      // 30%
            max_io_rate: Some(5),   // 5MB/s
            max_execution_time: Some(15), // 15 seconds
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            show_examples: Some(true),
            side_by_side: Some(true),
            show_line_numbers: Some(true),
            syntax_highlighting: Some(true),
            show_shortcuts: Some(true),
            default_tab: Some("issues".to_string()),
            max_suggestions: Some(3),
            colors: Some(ColorScheme::default()),
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background: Some("default".to_string()),
            foreground: Some("white".to_string()),
            border: Some("blue".to_string()),
            selected: Some("yellow".to_string()),
            error: Some("red".to_string()),
            warning: Some("yellow".to_string()),
            info: Some("blue".to_string()),
            line_numbers: Some("dark_gray".to_string()),
            keywords: Some("cyan".to_string()),
            strings: Some("green".to_string()),
            comments: Some("dark_gray".to_string()),
            types: Some("magenta".to_string()),
            correct: Some("green".to_string()),
            incorrect: Some("red".to_string()),
        }
    }
}

// TOML config file structure
#[derive(Debug, Deserialize, Serialize)]
struct ConfigFile {
    general: Option<GeneralConfig>,
    validators: Option<ValidatorsConfig>,
    file_mappings: Option<HashMap<String, String>>,
    security: Option<SecurityConfig>,
    tui: Option<TuiConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeneralConfig {
    strict: Option<bool>,
    verbose: Option<bool>,
    watch: Option<bool>,
    watch_interval: Option<u64>,
    timeout: Option<u64>,
    enable_cache: Option<bool>,
    cache_duration: Option<u64>,
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
            enable_cache: false,
            cache_duration: 15,
            loaded_config_paths: Vec::new(),
            file_mappings,
            validators: ValidatorConfigs::default(),
            security: SecurityConfig::default(),
            tui: TuiConfig::default(),
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
        enable_cache: Option<bool>,
        cache_duration: Option<u64>,
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
        if let Some(cache_enabled) = enable_cache {
            config.enable_cache = cache_enabled;
        }
        if let Some(cache_dur) = cache_duration {
            config.cache_duration = cache_dur;
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
            if let Some(cache_enabled) = general.enable_cache {
                self.enable_cache = cache_enabled;
            }
            if let Some(cache_dur) = general.cache_duration {
                self.cache_duration = cache_dur;
            }
        }

        // Merge security settings
        if let Some(security) = &config_file.security {
            if let Some(enable_sandbox) = security.enable_sandbox {
                self.security.enable_sandbox = Some(enable_sandbox);
            }
            if let Some(verify_checksums) = security.verify_checksums {
                self.security.verify_checksums = Some(verify_checksums);
            }
            if let Some(auto_update_checksums) = security.auto_update_checksums {
                self.security.auto_update_checksums = Some(auto_update_checksums);
            }
            if let Some(enable_audit) = security.enable_audit {
                self.security.enable_audit = Some(enable_audit);
            }
            if let Some(audit_log) = &security.audit_log {
                self.security.audit_log = Some(audit_log.clone());
            }
            if let Some(max_file_size) = security.max_file_size {
                self.security.max_file_size = Some(max_file_size);
            }
            if let Some(allowed_dirs) = &security.allowed_dirs {
                self.security.allowed_dirs = Some(allowed_dirs.clone());
            }
            if let Some(resource_limits) = &security.resource_limits {
                if self.security.resource_limits.is_none() {
                    self.security.resource_limits = Some(ResourceLimits::default());
                }
                
                if let Some(limits) = &mut self.security.resource_limits {
                    if let Some(max_memory) = resource_limits.max_memory {
                        limits.max_memory = Some(max_memory);
                    }
                    if let Some(max_cpu) = resource_limits.max_cpu {
                        limits.max_cpu = Some(max_cpu);
                    }
                    if let Some(max_io_rate) = resource_limits.max_io_rate {
                        limits.max_io_rate = Some(max_io_rate);
                    }
                    if let Some(max_execution_time) = resource_limits.max_execution_time {
                        limits.max_execution_time = Some(max_execution_time);
                    }
                }
            }
        }
        
        // Merge TUI settings
        if let Some(tui) = &config_file.tui {
            if let Some(show_examples) = tui.show_examples {
                self.tui.show_examples = Some(show_examples);
            }
            if let Some(side_by_side) = tui.side_by_side {
                self.tui.side_by_side = Some(side_by_side);
            }
            if let Some(show_line_numbers) = tui.show_line_numbers {
                self.tui.show_line_numbers = Some(show_line_numbers);
            }
            if let Some(syntax_highlighting) = tui.syntax_highlighting {
                self.tui.syntax_highlighting = Some(syntax_highlighting);
            }
            if let Some(show_shortcuts) = tui.show_shortcuts {
                self.tui.show_shortcuts = Some(show_shortcuts);
            }
            if let Some(default_tab) = &tui.default_tab {
                self.tui.default_tab = Some(default_tab.clone());
            }
            if let Some(max_suggestions) = tui.max_suggestions {
                self.tui.max_suggestions = Some(max_suggestions);
            }
            
            if let Some(colors) = &tui.colors {
                if self.tui.colors.is_none() {
                    self.tui.colors = Some(ColorScheme::default());
                }
                
                if let Some(scheme) = &mut self.tui.colors {
                    if let Some(background) = &colors.background {
                        scheme.background = Some(background.clone());
                    }
                    if let Some(foreground) = &colors.foreground {
                        scheme.foreground = Some(foreground.clone());
                    }
                    if let Some(border) = &colors.border {
                        scheme.border = Some(border.clone());
                    }
                    if let Some(selected) = &colors.selected {
                        scheme.selected = Some(selected.clone());
                    }
                    if let Some(error) = &colors.error {
                        scheme.error = Some(error.clone());
                    }
                    if let Some(warning) = &colors.warning {
                        scheme.warning = Some(warning.clone());
                    }
                    if let Some(info) = &colors.info {
                        scheme.info = Some(info.clone());
                    }
                    if let Some(line_numbers) = &colors.line_numbers {
                        scheme.line_numbers = Some(line_numbers.clone());
                    }
                    if let Some(keywords) = &colors.keywords {
                        scheme.keywords = Some(keywords.clone());
                    }
                    if let Some(strings) = &colors.strings {
                        scheme.strings = Some(strings.clone());
                    }
                    if let Some(comments) = &colors.comments {
                        scheme.comments = Some(comments.clone());
                    }
                    if let Some(types) = &colors.types {
                        scheme.types = Some(types.clone());
                    }
                    if let Some(correct) = &colors.correct {
                        scheme.correct = Some(correct.clone());
                    }
                    if let Some(incorrect) = &colors.incorrect {
                        scheme.incorrect = Some(incorrect.clone());
                    }
                }
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
            enable_cache: Some(config.enable_cache),
            cache_duration: Some(config.cache_duration),
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
        security: Some(config.security.clone()),
        tui: Some(config.tui.clone()),
    }
}

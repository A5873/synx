use clap::{Parser, Subcommand};
use std::process;

mod banner;
mod intelligence;

/// CLI arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Files to validate (when no subcommand is used)
    files: Vec<String>,

    /// Watch files for changes and revalidate
    #[arg(short = 'w', long)]
    watch: bool,

    /// Show verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Path to config file (default: ~/.config/synx/config.toml)
    #[arg(short = 'c', long)]
    config: Option<String>,

    /// Watch interval in seconds (default: 2)
    #[arg(long, default_value_t = 2)]
    interval: u64,

    /// Initialize default configuration file
    #[arg(long)]
    init_config: bool,

    /// Strict mode - treat warnings as errors
    #[arg(short = 's', long)]
    strict: bool,

    /// Show detailed error information with code context
    #[arg(long)]
    show_errors: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan directories recursively for code files
    Scan {
        /// Directories to scan
        #[arg(required = true)]
        paths: Vec<String>,
        
        /// Exclude patterns (glob patterns)
        #[arg(long, short = 'e')]
        exclude: Vec<String>,
        
        /// Number of parallel workers
        #[arg(long, short = 'j', default_value_t = 4)]
        parallel: usize,
        
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        
        /// Generate report file
        #[arg(long, short = 'r')]
        report: Option<String>,
    },
    /// Configuration management commands
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Cache management commands
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
    /// Intelligence and analytics commands
    Intelligence {
        #[command(subcommand)]
        action: IntelligenceAction,
    },
    /// Daemon management commands
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Performance monitoring and optimization commands
    Performance {
        #[command(subcommand)]
        action: PerformanceAction,
    },
    /// Interactive real-time monitoring (htop-style)
    Monitor {
        /// Paths to monitor for validation
        paths: Vec<String>,
        /// Enable automatic validation
        #[arg(long)]
        auto_validate: bool,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Generate default configuration file
    Init,
    /// Show current configuration
    Show,
    /// Validate configuration file
    Validate {
        /// Path to config file to validate
        path: Option<String>,
    },
}

#[derive(Subcommand)]
enum CacheAction {
    /// Show cache statistics
    Stats,
    /// Clear validation cache
    Clear,
    /// Show cache location
    Info,
}

#[derive(Subcommand)]
enum IntelligenceAction {
    /// Analyze a single file for complexity and quality metrics
    Analyze {
        /// File to analyze
        path: String,
        /// Output format (text, json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Analyze entire project for intelligence insights
    Project {
        /// Project directory to analyze
        path: String,
        /// Output format (text, json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Generate detailed report file
        #[arg(long, short = 'r')]
        report: Option<String>,
    },
    /// Show intelligence engine statistics
    Stats,
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the daemon (run in foreground)
    Start {
        /// Directories to watch
        #[arg(short = 'w', long, value_delimiter = ',')]
        watch_paths: Vec<String>,
        /// Daemon configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Run in foreground (don't daemonize)
        #[arg(long)]
        foreground: bool,
    },
    /// Stop the daemon
    Stop,
    /// Get daemon status
    Status,
    /// Restart the daemon
    Restart,
    /// Install daemon as system service
    Install {
        /// Service name
        #[arg(long, default_value = "synx-daemon")]
        service_name: String,
        /// Path to synx binary
        #[arg(long)]
        binary_path: Option<String>,
        /// Daemon configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
    },
    /// Uninstall daemon system service
    Uninstall {
        /// Service name
        #[arg(long, default_value = "synx-daemon")]
        service_name: String,
    },
    /// Generate default daemon configuration
    InitConfig {
        /// Configuration file path
        #[arg(long, default_value = "synx-daemon.toml")]
        path: String,
    },
    /// Show daemon statistics
    Stats,
}

#[derive(Subcommand)]
enum PerformanceAction {
    /// Show performance statistics
    Stats,
    /// Clear performance cache
    Clear,
    /// Optimize performance caches
    Optimize,
    /// Run performance benchmark
    Benchmark {
        /// Directory to benchmark
        path: String,
        /// Number of iterations
        #[arg(long, default_value_t = 3)]
        iterations: usize,
    },
}

fn main() {
    // Initialize logging
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    // Handle init config command
    if args.init_config {
        match synx::config::Config::generate_default_config() {
            Ok(path) => {
                println!("✅ Created default configuration at: {}", path.display());
                process::exit(0);
            }
            Err(e) => {
                eprintln!("❌ Failed to create config: {}", e);
                process::exit(1);
            }
        }
    }

    // Create configuration
    let config = match synx::config::Config::new(
        Some(args.strict),
        Some(args.verbose),
        Some(args.watch),
        Some(args.interval),
        None, // timeout - use default
        args.config.as_deref(),
    ) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ Failed to load configuration: {}", e);
            process::exit(2);
        }
    };

    // Handle subcommands
    match &args.command {
        Some(Commands::Scan { paths, exclude, parallel, format, report }) => {
            handle_scan_command(paths, exclude, *parallel, format, report, &config);
        }
        Some(Commands::Config { action }) => {
            handle_config_command(action, &config);
        }
        Some(Commands::Cache { action }) => {
            handle_cache_command(action);
        }
        Some(Commands::Intelligence { action }) => {
            handle_intelligence_command(action, &config);
        }
        Some(Commands::Daemon { action }) => {
            handle_daemon_command(action, &config);
        }
        Some(Commands::Performance { action }) => {
            handle_performance_command(action, &config);
        }
        Some(Commands::Monitor { paths, auto_validate }) => {
            handle_monitor_command(paths, *auto_validate, &config);
        }
        None => {
            // Legacy mode: validate individual files
            if args.verbose {
                println!("Validating files: {:?}", args.files);
            }
            
            match synx::run(&args.files, &config) {
                Ok(true) => {
                    if args.verbose {
                        println!("\n✅ All validations passed successfully!");
                    }
                    process::exit(0);
                }
                Ok(false) => {
                    if args.verbose {
                        println!("\n❌ Some validations failed!");
                    }
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("\n❌ Error: {}", e);
                    process::exit(2);
                }
            }
        }
    }
}

fn handle_scan_command(
    paths: &[String],
    exclude: &[String], 
    _parallel: usize,
    format: &str,
    report: &Option<String>,
    config: &synx::config::Config,
) {
    for path in paths {
        println!("🔍 Scanning directory: {}", path);
        
        let path_buf = std::path::PathBuf::from(path);
        if !path_buf.exists() {
            eprintln!("❌ Path does not exist: {}", path);
            process::exit(1);
        }
        
        if !path_buf.is_dir() {
            eprintln!("❌ Path is not a directory: {}", path);
            process::exit(1);
        }
        
        // Create validation options
        let validation_options = synx::validators::ValidationOptions {
            strict: config.strict,
            verbose: config.verbose,
            timeout: 30,
            config: Some(synx::validators::FileValidationConfig::default()),
        };
        
        // Run the scan
        match synx::validators::scan_directory(&path_buf, &validation_options, exclude) {
            Ok(result) => {
                // Display results based on format
                match format {
                    "json" => {
                        let json_output = serde_json::json!({
                            "total_files": result.total_files,
                            "valid_files": result.valid_files,
                            "invalid_files": result.invalid_files.len(),
                            "results_by_type": result.results_by_type
                        });
                        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
                    }
                    _ => {
                        // Default text output
                        synx::validators::display_scan_results(&result, &path_buf);
                    }
                }
                
                // Save report if specified
                if let Some(report_path) = report {
                    match save_report(&result, report_path, format) {
                        Ok(()) => println!("📊 Report saved to: {}", report_path),
                        Err(e) => eprintln!("❌ Failed to save report: {}", e),
                    }
                }
                
                // Exit with appropriate code
                if result.invalid_files.is_empty() {
                    process::exit(0);
                } else {
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("❌ Scan failed: {}", e);
                process::exit(2);
            }
        }
    }
}

fn handle_config_command(action: &ConfigAction, config: &synx::config::Config) {
    match action {
        ConfigAction::Init => {
            match synx::config::Config::generate_default_config() {
                Ok(path) => {
                    println!("✅ Created default configuration at: {}", path.display());
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("❌ Failed to create config: {}", e);
                    process::exit(1);
                }
            }
        }
        ConfigAction::Show => {
            println!("📝 Current Configuration:");
            println!("=======================\n");
            
            println!("General Settings:");
            println!("  Strict mode: {}", config.strict);
            println!("  Verbose output: {}", config.verbose);
            println!("  Watch mode: {}", config.watch);
            println!("  Watch interval: {}s", config.watch_interval);
            println!("  Timeout: {}s", config.timeout);
            
            println!("\nLoaded Configuration Files:");
            if config.loaded_config_paths.is_empty() {
                println!("  (No configuration files loaded - using defaults)");
            } else {
                for path in &config.loaded_config_paths {
                    println!("  - {}", path.display());
                }
            }
            
            if !config.file_mappings.is_empty() {
                println!("\nFile Mappings:");
                for (name, mapping) in &config.file_mappings {
                    println!("  {} -> {}", name, mapping);
                }
            }
            
            process::exit(0);
        }
        ConfigAction::Validate { path } => {
            let config_path = if let Some(path) = path {
                std::path::PathBuf::from(path)
            } else {
                match synx::config::get_default_config_path() {
                    Ok(path) => path,
                    Err(e) => {
                        eprintln!("❌ Failed to get default config path: {}", e);
                        process::exit(1);
                    }
                }
            };
            
            if !config_path.exists() {
                eprintln!("❌ Configuration file does not exist: {}", config_path.display());
                process::exit(1);
            }
            
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match toml::from_str::<toml::Value>(&content) {
                        Ok(_) => {
                            println!("✅ Configuration file is valid: {}", config_path.display());
                            process::exit(0);
                        }
                        Err(e) => {
                            eprintln!("❌ Invalid configuration file: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to read configuration file: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}

fn handle_cache_command(action: &CacheAction) {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from(".cache"))
        .join("synx");
    let cache_file = cache_dir.join("validation_cache.json");
    
    match action {
        CacheAction::Info => {
            println!("📁 Cache Information:");
            println!("===================\n");
            println!("Cache directory: {}", cache_dir.display());
            println!("Cache file: {}", cache_file.display());
            println!("Cache exists: {}", cache_file.exists());
            
            if cache_file.exists() {
                if let Ok(metadata) = std::fs::metadata(&cache_file) {
                    println!("Cache size: {} bytes", metadata.len());
                    if let Ok(modified) = metadata.modified() {
                        println!("Last modified: {:?}", modified);
                    }
                }
            }
            process::exit(0);
        }
        CacheAction::Stats => {
            if !cache_file.exists() {
                println!("📊 No cache file found. Run a scan to create cache.");
                process::exit(0);
            }
            
            match std::fs::read_to_string(&cache_file) {
                Ok(content) => {
                    match serde_json::from_str::<std::collections::HashMap<std::path::PathBuf, serde_json::Value>>(&content) {
                        Ok(cache_data) => {
                            println!("📊 Cache Statistics:");
                            println!("==================\n");
                            println!("Total cached files: {}", cache_data.len());
                            
                            let valid_count = cache_data.values()
                                .filter(|entry| entry.get("is_valid").and_then(|v| v.as_bool()).unwrap_or(false))
                                .count();
                            let invalid_count = cache_data.len() - valid_count;
                            
                            println!("Valid files: {}", valid_count);
                            println!("Invalid files: {}", invalid_count);
                            
                            if let Ok(metadata) = std::fs::metadata(&cache_file) {
                                println!("Cache file size: {} bytes", metadata.len());
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to parse cache file: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to read cache file: {}", e);
                    process::exit(1);
                }
            }
            process::exit(0);
        }
        CacheAction::Clear => {
            if cache_file.exists() {
                match std::fs::remove_file(&cache_file) {
                    Ok(()) => {
                        println!("✅ Cache cleared successfully");
                        process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to clear cache: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                println!("📋 No cache file found - nothing to clear");
                process::exit(0);
            }
        }
    }
}

fn handle_intelligence_command(action: &IntelligenceAction, _config: &synx::config::Config) {
    match action {
        IntelligenceAction::Analyze { path, format } => {
            println!("🧠 Analyzing file: {}", path);
            
            let file_path = std::path::PathBuf::from(path);
            if !file_path.exists() {
                eprintln!("❌ File does not exist: {}", path);
                process::exit(1);
            }
            
            // Read file content
            let _content = match std::fs::read_to_string(&file_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("❌ Failed to read file: {}", e);
                    process::exit(1);
                }
            };
            
            // Create intelligence engine
            let mut intelligence = match intelligence::IntelligenceEngine::new() {
                Ok(engine) => engine,
                Err(e) => {
                    eprintln!("❌ Failed to initialize intelligence engine: {}", e);
                    process::exit(1);
                }
            };
            
            // Generate file report
            match intelligence.analyze_file(&file_path) {
                Ok(report) => {
                    match format.as_str() {
                        "json" => {
                            match serde_json::to_string_pretty(&report) {
                                Ok(json) => println!("{}", json),
                                Err(e) => {
                                    eprintln!("❌ Failed to serialize report: {}", e);
                                    process::exit(1);
                                }
                            }
                        }
                        _ => {
                            // Default text format
                            println!("{}", intelligence::format_file_report(&report));
                        }
                    }
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("❌ Analysis failed: {}", e);
                    process::exit(1);
                }
            }
        }
        IntelligenceAction::Project { path, format, report } => {
            println!("🧠 Analyzing project: {}", path);
            
            let project_path = std::path::PathBuf::from(path);
            if !project_path.exists() {
                eprintln!("❌ Project path does not exist: {}", path);
                process::exit(1);
            }
            
            if !project_path.is_dir() {
                eprintln!("❌ Project path is not a directory: {}", path);
                process::exit(1);
            }
            
            // Create intelligence engine
            let mut intelligence = match intelligence::IntelligenceEngine::new() {
                Ok(engine) => engine,
                Err(e) => {
                    eprintln!("❌ Failed to initialize intelligence engine: {}", e);
                    process::exit(1);
                }
            };
            
            // Generate project report
            match intelligence.analyze_project(&project_path) {
                Ok(project_report) => {
                    match format.as_str() {
                        "json" => {
                            match serde_json::to_string_pretty(&project_report) {
                                Ok(json) => println!("{}", json),
                                Err(e) => {
                                    eprintln!("❌ Failed to serialize report: {}", e);
                                    process::exit(1);
                                }
                            }
                        }
                        _ => {
                            // Default text format
                            println!("{}", intelligence::format_project_report(&project_report));
                        }
                    }
                    
                    // Save detailed report if requested
                    if let Some(report_path) = report {
                        let detailed_report = intelligence::generate_detailed_report(&project_report);
                        match std::fs::write(report_path, detailed_report) {
                            Ok(()) => println!("📊 Detailed report saved to: {}", report_path),
                            Err(e) => eprintln!("❌ Failed to save report: {}", e),
                        }
                    }
                    
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("❌ Project analysis failed: {}", e);
                    process::exit(1);
                }
            }
        }
        IntelligenceAction::Stats => {
            println!("🧠 Intelligence Engine Statistics");
            println!("================================\n");
            
            // Create intelligence engine to get stats
            let intelligence = match intelligence::IntelligenceEngine::new() {
                Ok(engine) => engine,
                Err(e) => {
                    eprintln!("❌ Failed to initialize intelligence engine: {}", e);
                    process::exit(1);
                }
            };
            let stats = intelligence.get_statistics();
            
            println!("Engine Status: Active");
            println!("Supported Languages: {}", stats.supported_languages.join(", "));
            println!("Available Metrics: {}", stats.available_metrics.len());
            println!("Quality Factors: {}", stats.quality_factors.len());
            println!("Suggestion Rules: {}", stats.suggestion_rules);
            
            println!("\nMetrics Available:");
            for metric in &stats.available_metrics {
                println!("  • {}", metric);
            }
            
            println!("\nQuality Factors:");
            for factor in &stats.quality_factors {
                println!("  • {}", factor);
            }
            
            process::exit(0);
        }
    }
}

#[tokio::main]
async fn handle_daemon_command(action: &DaemonAction, _config: &synx::config::Config) {
    use synx::daemon::{DaemonConfig, SynxDaemon, ServiceManager, install_service, uninstall_service};
    use std::path::PathBuf;
    
    match action {
        DaemonAction::Start { watch_paths, config, foreground } => {
            // Show banner for long-running daemon operations
            banner::print_banner();
            println!("🚀 Starting Synx Daemon");
            
            // Load daemon configuration
            let mut daemon_config = if let Some(config_path) = config {
                match DaemonConfig::from_file(config_path) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("❌ Failed to load daemon config: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                match DaemonConfig::load_default() {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("❌ Failed to load default daemon config: {}", e);
                        process::exit(1);
                    }
                }
            };
            
            // Override watch paths if provided
            if !watch_paths.is_empty() {
                daemon_config.watch_paths = watch_paths.iter().map(|p| PathBuf::from(p)).collect();
            }
            
            // Set foreground mode
            daemon_config.daemonize = !foreground;
            
            // Load synx configuration
            let synx_config = match synx::config::Config::new(None, None, None, None, None, None) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("❌ Failed to load synx config: {}", e);
                    process::exit(1);
                }
            };
            
            // Create and start daemon
            let mut daemon = match SynxDaemon::new(daemon_config, synx_config) {
                Ok(daemon) => daemon,
                Err(e) => {
                    eprintln!("❌ Failed to create daemon: {}", e);
                    process::exit(1);
                }
            };
            
            if let Err(e) = daemon.start().await {
                eprintln!("❌ Daemon failed: {}", e);
                process::exit(1);
            }
        }
        
        DaemonAction::Stop => {
            println!("🛑 Stopping Synx Daemon");
            // In a real implementation, this would send a signal to the running daemon
            // For now, we'll just show how to stop a service
            let manager = ServiceManager::new("synx-daemon".to_string(), PathBuf::new());
            match manager.stop() {
                Ok(()) => println!("✅ Daemon stopped successfully"),
                Err(e) => {
                    eprintln!("❌ Failed to stop daemon: {}", e);
                    process::exit(1);
                }
            }
        }
        
        DaemonAction::Status => {
            println!("📊 Synx Daemon Status");
            let manager = ServiceManager::new("synx-daemon".to_string(), PathBuf::new());
            match manager.status() {
                Ok(status) => {
                    println!("Status: {}", status);
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("❌ Failed to get daemon status: {}", e);
                    process::exit(1);
                }
            }
        }
        
        DaemonAction::Restart => {
            println!("🔄 Restarting Synx Daemon");
            let manager = ServiceManager::new("synx-daemon".to_string(), PathBuf::new());
            
            // Stop then start
            if let Err(e) = manager.stop() {
                eprintln!("⚠️ Warning: Failed to stop daemon: {}", e);
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            match manager.start() {
                Ok(()) => println!("✅ Daemon restarted successfully"),
                Err(e) => {
                    eprintln!("❌ Failed to restart daemon: {}", e);
                    process::exit(1);
                }
            }
        }
        
        DaemonAction::Install { service_name, binary_path, config } => {
            println!("📦 Installing Synx Daemon as system service");
            
            let binary_path = if let Some(path) = binary_path {
                PathBuf::from(path)
            } else {
                // Try to find the current binary
                std::env::current_exe().unwrap_or_else(|_| PathBuf::from("synx"))
            };
            
            let config_path = config.as_ref().map(|c| std::path::Path::new(c));
            
            match install_service(service_name, &binary_path, config_path) {
                Ok(()) => {
                    println!("✅ Service installed successfully");
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("❌ Failed to install service: {}", e);
                    process::exit(1);
                }
            }
        }
        
        DaemonAction::Uninstall { service_name } => {
            println!("🗑️ Uninstalling Synx Daemon service");
            
            match uninstall_service(service_name) {
                Ok(()) => {
                    println!("✅ Service uninstalled successfully");
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("❌ Failed to uninstall service: {}", e);
                    process::exit(1);
                }
            }
        }
        
        DaemonAction::InitConfig { path } => {
            println!("⚙️ Generating default daemon configuration");
            
            match DaemonConfig::generate_default_config(path) {
                Ok(()) => {
                    println!("✅ Created default daemon configuration at: {}", path);
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("❌ Failed to create daemon config: {}", e);
                    process::exit(1);
                }
            }
        }
        
        DaemonAction::Stats => {
            println!("📈 Synx Daemon Statistics");
            println!("=========================\n");
            println!("This command would show daemon runtime statistics.");
            println!("Currently not implemented - daemon must be running to provide stats.");
            process::exit(0);
        }
    }
}

fn handle_performance_command(action: &PerformanceAction, _config: &synx::config::Config) {
    use synx::performance::{PerformanceConfig, PerformanceEngine};
    
    match action {
        PerformanceAction::Stats => {
            println!("⚡ Performance Statistics");
            println!("========================\n");
            
            let perf_config = PerformanceConfig::default();
            match PerformanceEngine::new(perf_config) {
                Ok(engine) => {
                    match engine.get_stats() {
                        Ok(stats) => {
                            println!("Thread Pool Size: {}", stats.thread_pool_size);
                            println!("Memory Usage: {}MB", stats.memory_usage);
                            println!("\nCache Statistics:");
                            println!("  Total Entries: {}", stats.cache_stats.total_entries);
                            println!("  Cache Hits: {}", stats.cache_stats.hits);
                            println!("  Cache Misses: {}", stats.cache_stats.misses);
                            println!("  Hit Ratio: {:.1}%", stats.cache_stats.hit_ratio * 100.0);
                            println!("  Memory Usage: {:.2}MB", stats.cache_stats.total_memory_mb);
                            
                            println!("\nValidation Metrics:");
                            println!("  Total Files: {}", stats.validation_metrics.total_files);
                            println!("  Average Time: {:.2}ms", stats.validation_metrics.average_validation_time_ms);
                            println!("  Files/Second: {:.2}", stats.validation_metrics.files_per_second);
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to get performance stats: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to create performance engine: {}", e);
                    process::exit(1);
                }
            }
            
            process::exit(0);
        }
        
        PerformanceAction::Clear => {
            println!("🧹 Clearing performance caches...");
            
            let perf_config = PerformanceConfig::default();
            match PerformanceEngine::new(perf_config) {
                Ok(mut engine) => {
                    match engine.reset() {
                        Ok(()) => {
                            println!("✅ Performance caches cleared successfully");
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to clear caches: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to create performance engine: {}", e);
                    process::exit(1);
                }
            }
            
            process::exit(0);
        }
        
        PerformanceAction::Optimize => {
            println!("⚙️ Optimizing performance caches...");
            
            let perf_config = PerformanceConfig::default();
            match PerformanceEngine::new(perf_config) {
                Ok(mut engine) => {
                    match engine.optimize_cache() {
                        Ok(()) => {
                            println!("✅ Performance caches optimized successfully");
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to optimize caches: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to create performance engine: {}", e);
                    process::exit(1);
                }
            }
            
            process::exit(0);
        }
        
        PerformanceAction::Benchmark { path, iterations } => {
            println!("🏃 Running performance benchmark on: {}", path);
            println!("Iterations: {}\n", iterations);
            
            let path_buf = std::path::PathBuf::from(path);
            if !path_buf.exists() {
                eprintln!("❌ Path does not exist: {}", path);
                process::exit(1);
            }
            
            if !path_buf.is_dir() {
                eprintln!("❌ Path is not a directory: {}", path);
                process::exit(1);
            }
            
            let mut total_times = Vec::new();
            let validation_options = synx::validators::ValidationOptions {
                strict: false,
                verbose: false,
                timeout: 30,
                config: Some(synx::validators::FileValidationConfig::default()),
            };
            
            for i in 1..=*iterations {
                println!("🔄 Running iteration {} of {}...", i, iterations);
                
                let start = std::time::Instant::now();
                match synx::validators::scan_directory(&path_buf, &validation_options, &[]) {
                    Ok(result) => {
                        let elapsed = start.elapsed();
                        total_times.push(elapsed);
                        
                        println!("  ✅ Completed in {:.2}s ({} files)", 
                               elapsed.as_secs_f64(), result.total_files);
                    }
                    Err(e) => {
                        eprintln!("  ❌ Iteration {} failed: {}", i, e);
                        process::exit(1);
                    }
                }
            }
            
            // Calculate statistics
            let total_time: std::time::Duration = total_times.iter().sum();
            let avg_time = total_time / total_times.len() as u32;
            let min_time = total_times.iter().min().unwrap();
            let max_time = total_times.iter().max().unwrap();
            
            println!("\n📊 Benchmark Results:");
            println!("====================\n");
            println!("Average Time: {:.2}s", avg_time.as_secs_f64());
            println!("Minimum Time: {:.2}s", min_time.as_secs_f64());
            println!("Maximum Time: {:.2}s", max_time.as_secs_f64());
            println!("Total Time: {:.2}s", total_time.as_secs_f64());
            
            process::exit(0);
        }
    }
}

fn handle_monitor_command(paths: &[String], _auto_validate: bool, _config: &synx::config::Config) {
    // Show banner for interactive TUI
    banner::print_banner();
    println!("🖥️ Starting Interactive TUI Monitor");
    
    // Convert paths to PathBuf
    let watch_paths: Vec<std::path::PathBuf> = paths.iter().map(|p| std::path::PathBuf::from(p)).collect();
    
    // Validate paths exist
    for path in &watch_paths {
        if !path.exists() {
            eprintln!("❌ Path does not exist: {}", path.display());
            process::exit(1);
        }
    }
    
    // Create a basic validation report for the TUI
    let mut file_issues = std::collections::HashMap::new();
    
    // For demonstration, create some sample issues
    for path in &watch_paths {
        if path.is_file() {
            let sample_issue = synx::tui::ValidationIssue {
                file_path: path.clone(),
                issue_type: "demo_issue".to_string(),
                severity: synx::analysis::IssueSeverity::Low,
                message: "This is a demonstration issue for TUI testing".to_string(),
                line_start: 1,
                line_end: 1,
                suggested_fix: Some("Fix suggestion".to_string()),
                context: std::collections::HashMap::new(),
            };
            file_issues.insert(path.clone(), vec![sample_issue]);
        }
    }
    
    if file_issues.is_empty() {
        println!("⚠️ No files found to monitor. Creating a dummy validation report.");
        // Create a dummy file issue for testing
        let dummy_path = std::path::PathBuf::from("dummy_file.txt");
        let sample_issue = synx::tui::ValidationIssue {
            file_path: dummy_path.clone(),
            issue_type: "demo_issue".to_string(),
            severity: synx::analysis::IssueSeverity::Low,
            message: "This is a demonstration issue for TUI testing".to_string(),
            line_start: 1,
            line_end: 1,
            suggested_fix: Some("Fix suggestion".to_string()),
            context: std::collections::HashMap::new(),
        };
        file_issues.insert(dummy_path, vec![sample_issue]);
    }
    
    let validation_report = synx::tui::ValidationReport {
        file_issues,
    };
    
    // Start the basic TUI
    match synx::tui::run_interactive_mode(validation_report) {
        Ok(results) => {
            println!("✅ Interactive TUI exited successfully");
            println!("  Fixed issues: {}", results.fixed_issues);
            println!("  Ignored issues: {}", results.ignored_issues);
            println!("  Remaining issues: {}", results.remaining_issues);
            process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Interactive TUI failed: {}", e);
            process::exit(1);
        }
    }
}

fn save_report(
    result: &synx::validators::ScanResult,
    path: &str,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = match format {
        "json" => {
            let json_output = serde_json::json!({
                "total_files": result.total_files,
                "valid_files": result.valid_files,
                "invalid_files": result.invalid_files.len(),
                "invalid_file_paths": result.invalid_files,
                "skipped_files": result.skipped_files,
                "results_by_type": result.results_by_type
            });
            serde_json::to_string_pretty(&json_output)?
        }
        _ => {
            // Default text format
            format!(
                "Synx Validation Report\n======================\n\nTotal files scanned: {}\nValid files: {}\nInvalid files: {}\nSkipped files: {}\n\nInvalid files:\n{}\n",
                result.total_files,
                result.valid_files,
                result.invalid_files.len(),
                result.skipped_files.len(),
                result.invalid_files.iter()
                    .map(|p| format!("  - {}", p.display()))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    };
    
    std::fs::write(path, content)?;
    Ok(())
}

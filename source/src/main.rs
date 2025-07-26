use clap::{Parser, Subcommand};
use std::process;

mod banner;

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
}

fn main() {
    // Initialize logging
    env_logger::init();
    
    // Show the banner on startup
    banner::print_banner();

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

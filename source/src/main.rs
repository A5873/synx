use clap::Parser;
use std::process;
use std::path::PathBuf;

mod banner;

/// CLI arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Files to validate
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

    if args.verbose {
        println!("Validating files: {:?}", args.files);
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

    // Run validation
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

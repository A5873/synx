use clap::Parser;
use std::process;

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
    // Show the banner on startup
    banner::print_banner();

    // Parse command line arguments
    let args = Args::parse();

    if args.verbose {
        println!("Validating files: {:?}", args.files);
    }

    let config = synx::Config {
        strict: args.strict,
        verbose: args.verbose,
        watch: args.watch,
        watch_interval: args.interval,
    };

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

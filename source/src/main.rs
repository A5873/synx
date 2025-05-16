mod banner;
use clap::Parser;

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

    // Your existing main logic here...
    println!("Validating files: {:?}", args.files);
}

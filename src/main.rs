use anyhow::{Result, Context};
use clap::{Parser, arg};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::sync::mpsc::channel;
use notify::{Watcher, RecursiveMode, event::{Event, EventKind, ModifyKind}};
use synx::{validate_file, config};
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;
use std::collections::HashSet;

/// A CLI-first universal syntax validator and linter dispatcher
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Files to validate
    #[arg(required_unless_present_any = ["watch", "init_config"])]
    files: Vec<PathBuf>,

    /// Watch files for changes and revalidate
    #[arg(short, long)]
    watch: bool,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Path to config file (default: ~/.config/synx/config.toml)
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Watch interval in seconds (default: 2)
    #[arg(long, default_value = "2")]
    interval: u64,
    
    /// Initialize default configuration file
    #[arg(long)]
    init_config: bool,
    
    /// Strict mode - treat warnings as errors
    #[arg(short, long)]
    strict: bool,
}

fn print_header(message: &str) -> Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)).set_bold(true))?;
    writeln!(&mut stdout, "\n{}", message)?;
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)))?;
    writeln!(&mut stdout, "{}", "-".repeat(message.len()))?;
    stdout.reset()?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize default config if requested
    if args.init_config {
        let config_path = config::Config::generate_default_config()?;
        println!("Initialized default configuration at: {:?}", config_path);
        if args.files.is_empty() {
            return Ok(());
        }
    }
    
    // Get configuration
    let config_path = args.config.as_deref();
    
    if args.verbose {
        print_header("Synx - Universal Syntax Validator")?;
        println!("Files to check: {:?}", args.files);
        if let Some(config) = &args.config {
            println!("Using custom config file: {:?}", config);
        } else {
            println!("Using default config");
        }
        if args.watch {
            println!("Watch mode enabled (interval: {}s)", args.interval);
        }
        if args.strict {
            println!("Strict mode enabled");
        }
        println!();
    }
    
    if args.watch {
        // Print initial header in watch mode
        print_header("Starting initial validation...")?;
        
        // Run validation once for all files first
        let files_to_watch: HashSet<PathBuf> = args.files.iter().cloned().collect();
        
        for file in &args.files {
            // Skip reporting errors in watch mode to keep running
            if let Err(e) = validate_file(file.as_path(), args.verbose, config_path) {
                if args.verbose {
                    println!("Error: {}", e);
                }
            }
        }
        
        // Set up a channel to receive events
        let (tx, rx) = channel();
        
        // Create a watcher with default configuration
        let mut watcher = notify::recommended_watcher(tx)
            .context("Failed to create file watcher")?;
        
        // Watch each file
        for file in &args.files {
            if !file.exists() {
                println!("Warning: File not found: {:?}", file);
                continue;
            }
            
            watcher.watch(file, RecursiveMode::NonRecursive)
                .context(format!("Failed to watch file: {:?}", file))?;
                
            if args.verbose {
                println!("Watching: {:?}", file);
            }
        }
        
        println!("\nWatching for changes (press Ctrl+C to exit)...");
        
        // Process events
        loop {
            match rx.recv_timeout(Duration::from_secs(args.interval)) {
                Ok(event) => {
                    // Process file events
                    if let Ok(event) = event {
                        handle_file_event(event, &files_to_watch, args.verbose, config_path)?;
                    }
                },
                Err(_) => {
                    // Timeout occurred, just continue the loop
                    if args.verbose {
                        println!("Watching...");
                    }
                }
            }
        }
    } else {
        // Run validation once for each file
        let mut has_errors = false;
        
        for file in &args.files {
            if let Err(e) = validate_file(file.as_path(), args.verbose, config_path) {
                has_errors = true;
                if args.verbose {
                    println!("Error: {}", e);
                }
            }
        }
        
        // Return non-zero exit code if any errors occurred
        if has_errors {
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Handle file system events for watched files
fn handle_file_event(
    event: Event,
    files_to_watch: &HashSet<PathBuf>,
    verbose: bool,
    config_path: Option<&Path>,
) -> Result<()> {
    // Only process modify events
    if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
        for path in event.paths {
            // Check if this is one of our watched files
            if files_to_watch.contains(&path) {
                print_header(&format!("File changed: {:?}", path))?;
                
                // Re-validate the file
                if let Err(e) = validate_file(&path, verbose, config_path) {
                    if verbose {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }
    
    Ok(())
}

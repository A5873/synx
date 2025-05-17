use std::path::Path;
use anyhow::{Result, anyhow};
use notify::{Watcher, RecursiveMode};
use std::time::Duration;
use std::sync::mpsc::channel;

mod validators;
use validators::{ValidationOptions, validate_file};

pub struct Config {
    pub strict: bool,
    pub verbose: bool,
    pub watch: bool,
    pub watch_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            strict: false,
            verbose: false,
            watch: false,
            watch_interval: 2,
        }
    }
}

/// Run validation on the specified files
pub fn run(files: &[String], config: &Config) -> Result<bool> {
    let mut success = true;

    if config.watch {
        watch_files(files, config)?;
    } else {
        success = validate_files(files, config)?;
    }

    Ok(success)
}

/// Watch files for changes and revalidate
fn watch_files(files: &[String], config: &Config) -> Result<bool> {
    println!("Watching files for changes. Press Ctrl+C to stop.");

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        tx.send(res).unwrap();
    })?;

    for file in files {
        watcher.watch(Path::new(file), RecursiveMode::NonRecursive)?;
    }

    loop {
        match rx.recv_timeout(Duration::from_secs(config.watch_interval)) {
            Ok(_) => {
                println!("\nChange detected, revalidating...");
                validate_files(files, config)?;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(e) => return Err(anyhow!("Watch error: {}", e)),
        }
    }
}

/// Validate multiple files
fn validate_files(files: &[String], config: &Config) -> Result<bool> {
    let options = ValidationOptions {
        strict: config.strict,
        verbose: config.verbose,
    };

    let mut success = true;
    for file in files {
        let path = Path::new(file);
        if !path.exists() {
            eprintln!("File not found: {}", file);
            success = false;
            continue;
        }

        match validate_file(path, &options) {
            Ok(valid) => {
                if !valid {
                    success = false;
                } else if config.verbose {
                    println!("✓ {}", file);
                }
            }
            Err(e) => {
                eprintln!("Error validating {}: {}", file, e);
                success = false;
            }
        }
    }

    Ok(success)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_validate_valid_json() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("valid.json");
        let mut file = File::create(&file_path)?;
        writeln!(file, "{{ \"key\": \"value\" }}")?;

        let config = Config::default();
        let result = run(&[file_path.to_string_lossy().to_string()], &config)?;
        assert!(result);
        Ok(())
    }

    #[test]
    fn test_validate_invalid_json() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("invalid.json");
        let mut file = File::create(&file_path)?;
        writeln!(file, "{{ invalid json }}")?;

        let config = Config::default();
        let result = run(&[file_path.to_string_lossy().to_string()], &config)?;
        assert!(!result);
        Ok(())
    }
}

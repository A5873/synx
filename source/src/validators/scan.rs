use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;
use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use colored::*;
use console::Emoji;

use super::{ValidationOptions, validate_file};

static SCAN_MARK: Emoji<'_, '_> = Emoji("🔍", ">");
static FILE_MARK: Emoji<'_, '_> = Emoji("📄", "-");
static ERROR_MARK: Emoji<'_, '_> = Emoji("❌", "x");

#[derive(Default)]
pub struct ScanResult {
    pub total_files: usize,
    pub valid_files: usize,
    pub invalid_files: Vec<PathBuf>,
    pub skipped_files: Vec<PathBuf>,
    pub results_by_type: HashMap<String, TypeResult>,
}

#[derive(Default)]
pub struct TypeResult {
    pub total: usize,
    pub valid: usize,
    pub invalid: Vec<PathBuf>,
}

pub fn scan_directory(
    dir_path: &Path,
    options: &ValidationOptions,
    exclude_patterns: &[String],
) -> Result<ScanResult> {
    let mut result = ScanResult::default();
    
    println!("\n{} {} {}", 
        SCAN_MARK,
        "Starting scan of".bright_blue(),
        dir_path.display().to_string().bright_white().underline()
    );

    // Count total files first for progress bar
    let total_files = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();

    let progress = ProgressBar::new(total_files as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template(&format!("{} {}",
                "[{elapsed_precise}]".bright_black(),
                " {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"))?
            .progress_chars("█▇▆▅▄▃▂▁  ")
    );

    for entry in WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        
        // Skip files matching exclude patterns
        if exclude_patterns.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .map(|p| p.matches(path.to_str().unwrap_or("")))
                .unwrap_or(false)
        }) {
            result.skipped_files.push(path.to_path_buf());
            if options.verbose {
                println!("  {} {} {}", 
                    FILE_MARK,
                    "Skipping".yellow(),
                    path.display().to_string().yellow()
                );
            }
            continue;
        }

        result.total_files += 1;

        match validate_file(path, options) {
            Ok(true) => {
                result.valid_files += 1;
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let type_result = result.results_by_type
                    .entry(ext)
                    .or_default();
                type_result.total += 1;
                type_result.valid += 1;

                if options.verbose {
                    println!("  {} {} {}", 
                        FILE_MARK,
                        "Valid".green(),
                        path.display().to_string().bright_white()
                    );
                }
            }
            Ok(false) => {
                result.invalid_files.push(path.to_path_buf());
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let type_result = result.results_by_type
                    .entry(ext)
                    .or_default();
                type_result.total += 1;
                type_result.invalid.push(path.to_path_buf());

                if options.verbose {
                    println!("  {} {} {}", 
                        ERROR_MARK,
                        "Invalid".red(),
                        path.display().to_string().red()
                    );
                }
            }
            Err(e) => {
                if options.verbose {
                    println!("  {} {} {} - {}", 
                        ERROR_MARK,
                        "Error".red().bold(),
                        path.display().to_string().red(),
                        e.to_string().bright_black()
                    );
                }
                result.invalid_files.push(path.to_path_buf());
            }
        }

        progress.inc(1);
        let invalid_count = result.invalid_files.len();
        if invalid_count > 0 {
            progress.set_message(format!("({} {})", 
                invalid_count.to_string().red(),
                if invalid_count == 1 { "issue found" } else { "issues found" }
            ));
        }
    }

    progress.finish();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_scan_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create some test files
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        
        // Valid files
        File::create(temp_dir.path().join("src/valid.rs")).unwrap()
            .write_all(b"fn main() { println!(\"Hello\"); }\n").unwrap();
        
        File::create(temp_dir.path().join("src/valid.py")).unwrap()
            .write_all(b"print('Hello')\n").unwrap();
        
        // Invalid files
        File::create(temp_dir.path().join("src/invalid.rs")).unwrap()
            .write_all(b"fn main() { println!(\"Hello\" }\n").unwrap();
        
        File::create(temp_dir.path().join("src/invalid.py")).unwrap()
            .write_all(b"print('Hello'\n").unwrap();
        
        let options = ValidationOptions {
            strict: true,
            verbose: false,
            timeout: 30,
            config: None,
        };
        
        let result = scan_directory(temp_dir.path(), &options, &[]).unwrap();
        
        assert_eq!(result.total_files, 4);
        assert!(result.valid_files > 0);
        assert!(!result.invalid_files.is_empty());
    }
}

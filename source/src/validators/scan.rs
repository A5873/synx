use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;
use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use colored::*;
use console::Emoji;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use blake3::Hasher;
use std::fs;
use std::io::Read;

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

#[derive(Debug, Default, serde::Serialize)]
pub struct TypeResult {
    pub total: usize,
    pub valid: usize,
    pub invalid: Vec<PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    hash: String,
    is_valid: bool,
    timestamp: u64,
}

struct ValidationCache {
    entries: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    cache_file: PathBuf,
}

impl ValidationCache {
    fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("synx");
        fs::create_dir_all(&cache_dir).ok();
        let cache_file = cache_dir.join("validation_cache.json");
        
        let entries = if cache_file.exists() {
            fs::read_to_string(&cache_file)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            HashMap::new()
        };
        
        Self {
            entries: Arc::new(Mutex::new(entries)),
            cache_file,
        }
    }
    
    fn get_file_hash(path: &Path) -> Option<String> {
        let mut file = fs::File::open(path).ok()?;
        let mut hasher = Hasher::new();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).ok()?;
        hasher.update(&buffer);
        Some(hasher.finalize().to_hex().to_string())
    }
    
    fn is_valid_cached(&self, path: &Path) -> Option<bool> {
        let hash = Self::get_file_hash(path)?;
        let entries = self.entries.lock().ok()?;
        
        if let Some(entry) = entries.get(path) {
            if entry.hash == hash {
                return Some(entry.is_valid);
            }
        }
        None
    }
    
    fn cache_result(&self, path: &Path, is_valid: bool) {
        if let Some(hash) = Self::get_file_hash(path) {
            if let Ok(mut entries) = self.entries.lock() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                
                entries.insert(path.to_path_buf(), CacheEntry {
                    hash,
                    is_valid,
                    timestamp,
                });
            }
        }
    }
    
    fn save(&self) {
        if let Ok(entries) = self.entries.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*entries) {
                fs::write(&self.cache_file, json).ok();
            }
        }
    }
}

pub fn scan_directory(
    dir_path: &Path,
    options: &ValidationOptions,
    exclude_patterns: &[String],
) -> Result<ScanResult> {
    let start_time = Instant::now();
    
    println!("\n{} {} {}", 
        SCAN_MARK,
        "Starting parallel scan of".bright_blue(),
        dir_path.display().to_string().bright_white().underline()
    );

    let cache = ValidationCache::new();
    
    // Collect all file paths first
    let files: Vec<PathBuf> = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !exclude_patterns.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .map(|p| p.matches(e.path().to_str().unwrap_or("")))
                .unwrap_or(false)
        }))
        .map(|e| e.path().to_path_buf())
        .collect();

    let total_files = files.len();
    println!("  Found {} files to validate", total_files.to_string().bright_white());
    
    if total_files == 0 {
        return Ok(ScanResult::default());
    }

    let progress = Arc::new(Mutex::new(ProgressBar::new(total_files as u64)));
    {
        let p = progress.lock().unwrap();
        p.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{} {}",
                    "[{elapsed_precise}]".bright_black(),
                    " {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"))?
                .progress_chars("█▇▆▅▄▃▂▁  ")
        );
    }

    // Thread-safe collections for results
    let valid_files = Arc::new(Mutex::new(Vec::new()));
    let invalid_files = Arc::new(Mutex::new(Vec::new()));
    let skipped_files = Arc::new(Mutex::new(Vec::new()));
    let results_by_type = Arc::new(Mutex::new(HashMap::<String, TypeResult>::new()));
    let cache_hits = Arc::new(Mutex::new(0usize));
    
    // Process files in parallel
    files.par_iter().for_each(|path| {
        let mut cached = false;
        
        // Check cache first
        let validation_result = if let Some(is_valid) = cache.is_valid_cached(path) {
            cached = true;
            *cache_hits.lock().unwrap() += 1;
            Ok(is_valid)
        } else {
            validate_file(path, options)
        };

        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string();

        match validation_result {
            Ok(true) => {
                valid_files.lock().unwrap().push(path.clone());
                
                let mut type_results = results_by_type.lock().unwrap();
                let type_result = type_results.entry(ext).or_default();
                type_result.total += 1;
                type_result.valid += 1;
                
                if !cached {
                    cache.cache_result(path, true);
                }

                if options.verbose {
                    let cache_indicator = if cached { " (cached)".bright_black() } else { "".normal() };
                    println!("  {} {} {}{}", 
                        FILE_MARK,
                        "Valid".green(),
                        path.display().to_string().bright_white(),
                        cache_indicator
                    );
                }
            }
            Ok(false) => {
                invalid_files.lock().unwrap().push(path.clone());
                
                let mut type_results = results_by_type.lock().unwrap();
                let type_result = type_results.entry(ext).or_default();
                type_result.total += 1;
                type_result.invalid.push(path.clone());
                
                if !cached {
                    cache.cache_result(path, false);
                }

                if options.verbose {
                    let cache_indicator = if cached { " (cached)".bright_black() } else { "".normal() };
                    println!("  {} {} {}{}", 
                        ERROR_MARK,
                        "Invalid".red(),
                        path.display().to_string().red(),
                        cache_indicator
                    );
                }
            }
            Err(e) => {
                invalid_files.lock().unwrap().push(path.clone());
                
                if options.verbose {
                    println!("  {} {} {} - {}", 
                        ERROR_MARK,
                        "Error".red().bold(),
                        path.display().to_string().red(),
                        e.to_string().bright_black()
                    );
                }
            }
        }

        // Update progress
        {
            let p = progress.lock().unwrap();
            p.inc(1);
            let invalid_count = invalid_files.lock().unwrap().len();
            if invalid_count > 0 {
                p.set_message(format!("({} {})", 
                    invalid_count.to_string().red(),
                    if invalid_count == 1 { "issue found" } else { "issues found" }
                ));
            }
        }
    });

    progress.lock().unwrap().finish();
    
    // Save cache to disk
    cache.save();
    
    let elapsed = start_time.elapsed();
    let cache_hit_count = *cache_hits.lock().unwrap();
    
    // Construct final result
    let valid_files_vec = Arc::try_unwrap(valid_files).unwrap().into_inner().unwrap();
    let invalid_files_vec = Arc::try_unwrap(invalid_files).unwrap().into_inner().unwrap();
    let skipped_files_vec = Arc::try_unwrap(skipped_files).unwrap().into_inner().unwrap();
    let results_by_type_map = Arc::try_unwrap(results_by_type).unwrap().into_inner().unwrap();
    
    println!("\n{} Scan completed in {:.2}s ({} cache hits)", 
        "✓".green(),
        elapsed.as_secs_f64(),
        cache_hit_count.to_string().bright_blue()
    );
    
    Ok(ScanResult {
        total_files,
        valid_files: valid_files_vec.len(),
        invalid_files: invalid_files_vec,
        skipped_files: skipped_files_vec,
        results_by_type: results_by_type_map,
    })
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

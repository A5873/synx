use colored::*;
use super::scan::ScanResult;
use std::path::Path;
use console::{style, Emoji};

static CHECK_MARK: Emoji<'_, '_> = Emoji("✓", "√");
static CROSS_MARK: Emoji<'_, '_> = Emoji("✗", "x");
static WARN_MARK: Emoji<'_, '_> = Emoji("⚠", "!");
static FILE_MARK: Emoji<'_, '_> = Emoji("📄", "-");
static FOLDER_MARK: Emoji<'_, '_> = Emoji("📁", "+");
static SEARCH_MARK: Emoji<'_, '_> = Emoji("🔍", ">");

pub fn display_scan_results(result: &ScanResult, root_dir: &Path) {
    println!("\n{} {} Scan Results for: {}", 
        SEARCH_MARK,
        "Directory".bright_blue().bold(),
        root_dir.display().to_string().bright_white().underline()
    );

    println!("\n{} Summary:", FOLDER_MARK);
    println!("  {} Total Files:    {}", 
        FILE_MARK,
        result.total_files.to_string().bright_white()
    );
    println!("  {} Valid Files:    {}", 
        CHECK_MARK,
        result.valid_files.to_string().green()
    );
    println!("  {} Invalid Files:  {}", 
        CROSS_MARK,
        result.invalid_files.len().to_string().red()
    );
    println!("  {} Skipped Files:  {}", 
        WARN_MARK,
        result.skipped_files.len().to_string().yellow()
    );

    if !result.results_by_type.is_empty() {
        println!("\n{} Results by File Type:", FOLDER_MARK);
        for (ext, type_result) in &result.results_by_type {
            let success_rate = (type_result.valid as f32 / type_result.total as f32 * 100.0) as i32;
            let status_color = match success_rate {
                90..=100 => "green",
                70..=89 => "yellow",
                _ => "red"
            };
            
            println!("  {} .{:<8} [{:>3}%] {} valid, {} total", 
                FILE_MARK,
                ext,
                style(format!("{}", success_rate)).to_string().as_str(),
                type_result.valid.to_string().green(),
                type_result.total
            );
        }
    }

    if !result.invalid_files.is_empty() {
        println!("\n{} Invalid Files:", CROSS_MARK);
        for file in &result.invalid_files {
            if let Some(relative) = file.strip_prefix(root_dir).ok() {
                println!("  {} {}", 
                    CROSS_MARK,
                    relative.display().to_string().red()
                );
            }
        }
    }

    if !result.skipped_files.is_empty() {
        println!("\n{} Skipped Files:", WARN_MARK);
        for file in &result.skipped_files {
            if let Some(relative) = file.strip_prefix(root_dir).ok() {
                println!("  {} {}", 
                    WARN_MARK,
                    relative.display().to_string().yellow()
                );
            }
        }
    }

    // Print final summary with color-coded status
    let status = if result.invalid_files.is_empty() {
        "PASSED".green().bold()
    } else {
        "FAILED".red().bold()
    };
    
    println!("\n{} Final Status: {}", FOLDER_MARK, status);
    println!("{}", "=".repeat(60).bright_black());
}

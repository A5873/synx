//! Memory analysis module for Synx
//!
//! This module provides comprehensive memory analysis capabilities:
//! - Valgrind integration for deep memory analysis
//! - AddressSanitizer for memory error detection
//! - Heap profiling and memory usage tracking
//! - Memory leak detection
//! - Memory access pattern analysis
//! - Language-specific memory tools integration

use std::path::Path;
use std::process::Command;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::fs::{self, File};
use tempfile::TempDir;
use anyhow::{Result, Context, anyhow};

use crate::detectors::FileType;
use crate::config::Config;
use crate::analysis::{
    Analyzer, AnalysisOptions, AnalysisResult, AnalysisDetails, MemoryDetails, MemoryLeak,
    AnalysisIssue, IssueSeverity, AnalysisLevel, AnalysisType
};

/// Valgrind Memcheck analyzer for deep memory analysis
pub struct Valgrind;

impl Analyzer for Valgrind {
    fn name(&self) -> &str {
        "Valgrind Memory Analyzer"
    }
    
    fn is_available(&self) -> bool {
        // Check if valgrind is available on the system
        let output = Command::new("which")
            .arg("valgrind")
            .output();
            
        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    fn analyze(&self, file_path: &Path, options: &AnalysisOptions) -> Result<AnalysisResult> {
        let start_time = Instant::now();
        let verbose = options.verbose;
        
        // We need to compile and run the code to analyze it
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        
        // Determine the file type for proper compilation
        let file_type = crate::detectors::detect_file_type(file_path)?;
        
        // Prepare the executable
        let executable_path = self.prepare_executable(&file_type, file_path, &temp_dir, verbose)?;
        
        // Create output directory for analysis data if requested
        let output_dir = if let Some(dir) = &options.output_dir {
            dir.clone()
        } else {
            temp_dir.path().to_string_lossy().to_string()
        };
        
        // Set up valgrind log file
        let log_file = format!("{}/valgrind_memcheck.log", output_dir);
        
        // Configure valgrind options based on analysis level
        let extra_options = match options.analysis_level {
            AnalysisLevel::Basic => "--leak-check=yes",
            AnalysisLevel::Standard => "--leak-check=full --show-leak-kinds=all",
            AnalysisLevel::Comprehensive => "--leak-check=full --show-leak-kinds=all --track-origins=yes --expensive-definedness-checks=yes",
            AnalysisLevel::Intensive => "--leak-check=full --show-leak-kinds=all --track-origins=yes --expensive-definedness-checks=yes --undef-value-errors=yes --keep-stacktraces=alloc-and-free",
        };
        
        // Run valgrind memcheck
        if verbose {
            println!("Running Valgrind Memcheck with options: {}", extra_options);
        }
        
        let mut valgrind_cmd = Command::new("valgrind");
        valgrind_cmd
            .args([
                "--tool=memcheck",
                extra_options,
                "--log-file", &log_file,
                "--xml=yes",
                executable_path.as_str(),
            ])
            .current_dir(&temp_dir.path());
            
        let valgrind_output = valgrind_cmd
            .output()
            .context("Failed to run Valgrind Memcheck")?;
            
        // Also run massif for heap profiling if comprehensive analysis requested
        let mut massif_results = None;
        let mut report_files = Vec::new();
        
        if matches!(options.analysis_level, AnalysisLevel::Comprehensive | AnalysisLevel::Intensive) {
            let massif_file = format!("{}/massif.out", output_dir);
            
            if verbose {
                println!("Running Valgrind Massif for heap profiling");
            }
            
            let mut massif_cmd = Command::new("valgrind");
            massif_cmd
                .args([
                    "--tool=massif",
                    "--detailed-freq=10",
                    "--max-snapshots=100",
                    "--threshold=0.1",
                    "--massif-out-file", &massif_file,
                    executable_path.as_str(),
                ])
                .current_dir(&temp_dir.path());
                
            let _massif_output = massif_cmd
                .output()
                .context("Failed to run Valgrind Massif")?;
                
            // Generate massif visualization
            let massif_viz_file = format!("{}/massif_visualization.txt", output_dir);
            let mut ms_print_cmd = Command::new("ms_print");
            ms_print_cmd
                .args([&massif_file])
                .stdout(std::process::Stdio::piped())
                .current_dir(&temp_dir.path());
                
            let ms_print_output = ms_print_cmd
                .output()
                .context("Failed to run ms_print")?;
                
            if ms_print_output.status.success() {
                let viz_text = String::from_utf8_lossy(&ms_print_output.stdout).to_string();
                fs::write(&massif_viz_file, &viz_text).ok();
                
                if options.save_results {
                    report_files.push(massif_viz_file);
                    report_files.push(massif_file);
                }
                
                // Extract peak memory usage from massif results
                massif_results = Some(viz_text);
            }
        }
        
        // Read the valgrind log file
        let log_content = fs::read_to_string(&log_file)
            .context("Failed to read Valgrind log file")?;
            
        if options.save_results {
            report_files.push(log_file);
        }
        
        // Parse the valgrind results
        let memory_details = self.parse_valgrind_results(&log_content, massif_results.as_deref())?;
        
        // Identify potential memory issues
        let issues = self.identify_memory_issues(&memory_details);
        
        let duration = start_time.elapsed();
        
        // Create the analysis result
        let result = AnalysisResult {
            analyzer_name: self.name().to_string(),
            success: true,
            duration,
            summary: format!(
                "Peak memory: {} bytes, Allocations: {}, Leaks: {}, Errors: {}",
                memory_details.peak_memory,
                memory_details.total_allocations,
                memory_details.leaks.len(),
                if issues.is_empty() { 0 } else { issues.len() }
            ),
            details: AnalysisDetails::Memory(memory_details),
            report_files,
            issues,
        };
        
        Ok(result)
    }
    
    fn required_dependencies(&self) -> Vec<&str> {
        vec!["valgrind"]
    }
    
    fn installation_instructions(&self) -> String {
        r#"To install Valgrind:
- On Debian/Ubuntu: sudo apt-get install valgrind
- On Fedora/RHEL: sudo dnf install valgrind
- On Arch Linux: sudo pacman -S valgrind
- On macOS: brew install valgrind
- On Windows: Valgrind is not available natively, but you can use WSL"#
            .to_string()
    }
}

impl Valgrind {
    /// Prepare an executable from the source file for analysis
    fn prepare_executable(&self, file_type: &FileType, file_path: &Path, temp_dir: &TempDir, verbose: bool) -> Result<String> {
        let executable_path = temp_dir.path().join("executable");
        
        match file_type {
            FileType::C => {
                // Compile C code with valgrind-friendly flags
                let mut cmd = Command::new("gcc");
                cmd
                    .args([
                        "-g", // Debug info
                        "-O0", // No optimization for better analysis
                        "-o", executable_path.to_str().unwrap(),
                        file_path.to_str().unwrap(),
                    ]);
                    
                let output = cmd.output().context("Failed to compile C code")?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow!("Failed to compile C code: {}", stderr));
                }
            },
            FileType::Cpp => {
                // Compile C++ code with valgrind-friendly flags
                let mut cmd = Command::new("g++");
                cmd
                    .args([
                        "-g", // Debug info
                        "-O0", // No optimization for better analysis
                        "-o", executable_path.to_str().unwrap(),
                        file_path.to_str().unwrap(),
                    ]);
                    
                let output = cmd.output().context("Failed to compile C++ code")?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow!("Failed to compile C++ code: {}", stderr));
                }
            },
            FileType::Rust => {
                // Create a simple Cargo project for Rust files
                let src_dir = temp_dir.path().join("src");
                fs::create_dir_all(&src_dir).context("Failed to create src directory")?;
                
                // Copy the Rust file to src
                let dest_path = src_dir.join(file_path.file_name().unwrap());
                fs::copy(file_path, &dest_path).context("Failed to copy Rust file")?;
                
                // Create a simple Cargo.toml
                let cargo_toml = r#"
[package]
name = "memory_analysis"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
                let cargo_toml_path = temp_dir.path().join("Cargo.toml");
                fs::write(&cargo_toml_path, cargo_toml).context("Failed to write Cargo.toml")?;
                
                // Build the Rust code
                let mut cmd = Command::new("cargo");
                cmd
                    .args(["build", "--release"])
                    .current_dir(temp_dir.path());
                    
                let output = cmd.output().context("Failed to build Rust code")?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow!("Failed to build Rust code: {}", stderr));
                }
                
                // Set the executable path to the target/release binary
                let executable_path = temp_dir.path().join("target/release/memory_analysis");
                return Ok(executable_path.to_string_lossy().to_string());
            },
            _ => {
                return Err(anyhow!("File type {:?} is not supported for memory analysis with Valgrind", file_type));
            }
        }
        
        Ok(executable_path.to_string_lossy().to_string())
    }
    
    /// Parse valgrind results into structured memory details
    fn parse_valgrind_results(&self, log_content: &str, massif_results: Option<&str>) -> Result<MemoryDetails> {
        let mut peak_memory = 0;
        let mut total_allocations = 0;
        let mut total_deallocations = 0;
        let mut leaks = Vec::new();
        let mut allocation_hotspots = Vec::new();
        let mut heap_usage_timeline = Vec::new();
        
        // Parse XML output from valgrind memcheck
        // This is a simplified parser that extracts key information
        
        // Count errors and leaks
        for line in log_content.lines() {
            if line.contains("<error>") {
                if let Some(kind_line) = find_xml_tag(log_content, "kind") {
                    let kind = extract_xml_content(kind_line, "kind");
                    
                    if kind.contains("Leak_") {
                        // Extract leak information
                        let leak_size = if let Some(size_line) = find_xml_tag_after(log_content, "leakedbytes", line) {
                            extract_xml_content(size_line, "leakedbytes")
                                .parse::<u64>()
                                .unwrap_or(0)
                        } else {
                            0
                        };
                        
                        let allocation_location = if let Some(frame_line) = find_xml_tag_after(log_content, "frame", line) {
                            if let Some(file_line) = find_xml_tag_after(log_content, "file", frame_line) {
                                extract_xml_content(file_line, "file")
                            } else {
                                "Unknown location".to_string()
                            }
                        } else {
                            "Unknown location".to_string()
                        };
                        
                        // Extract stack trace
                        let mut stack_trace = Vec::new();
                        let mut pos = line;
                        while let Some(frame_pos) = log_content[pos.len()..].find("<frame>") {
                            let frame_start = pos.len() + frame_pos;
                            if let Some(fn_line) = find_xml_tag_after(log_content, "fn", &log_content[frame_start..]) {
                                let fn_name = extract_xml_content(fn_line, "fn");
                                stack_trace.push(fn_name);
                            }
                            pos = &log_content[frame_start + 1..];
                            if stack_trace.len() >= 10 || pos.find("</frame>").is_none() {
                                break;
                            }
                        }
                        
                        // Add the leak to our collection
                        leaks.push(MemoryLeak {
                            size: leak_size,
                            allocation_location,
                            stack_trace,
                        });
                    }
                }
            }
            
            // Count allocations
            if line.contains("heap_tree=peak") {
                if let Some(bytes_line) = find_xml_tag_after(log_content, "bytes", line) {
                    let bytes = extract_xml_content(bytes_line, "bytes")
                        .parse::<u64>()
                        .unwrap_or(0);
                    peak_memory = peak_memory.max(bytes);
                }
                
                // Extract allocation hotspots
                let mut pos = line;
                while let Some(block_pos) = log_content[pos.len()..].find("<block>") {
                    let block_start = pos.len() + block_pos;
                    
                    // Get function name and allocation size
                    let fn_name = if let Some(fn_line) = find_xml_tag_after(log_content, "fn", &log_content[block_start..]) {
                        extract_xml_content(fn_line, "fn")
                    } else {
                        "Unknown function".to_string()
                    };
                    
                    let alloc_bytes = if let Some(bytes_line) = find_xml_tag_after(log_content, "bytes", &log_content[block_start..]) {
                        extract_xml_content(bytes_line, "bytes")
                            .parse::<u64>()
                            .unwrap_or(0)
                    } else {
                        0
                    };
                    
                    // Add to hotspots
                    if alloc_bytes > 0 {
                        // Check if the function is already in the hotspots
                        if let Some(pos) = allocation_hotspots.iter().position(|(name, _)| name == &fn_name) {
                            allocation_hotspots[pos].1 += alloc_bytes;
                        } else {
                            allocation_hotspots.push((fn_name, alloc_bytes));
                        }
                    }
                    
                    pos = &log_content[block_start + 1..];
                    if pos.find("</block>").is_none() {
                        break;
                    }
                }
            }
            
            // Count total allocation/deallocation events
            if line.contains("<event>") {
                if line.contains("<kind>Heap") {
                    let kind = if let Some(kind_line) = find_xml_tag_after(log_content, "kind", line) {
                        extract_xml_content(kind_line, "kind")
                    } else {
                        String::new()
                    };
                    
                    if kind.contains("Alloc") {
                        total_allocations += 1;
                    } else if kind.contains("Free") {
                        total_deallocations += 1;
                    }
                }
            }
        }
        
        // Parse the massif output for heap usage timeline if available
        if let Some(massif_text) = massif_results {
            // Extract snapshot data
            let mut timestamp = 0u64;
            for line in massif_text.lines() {
                if line.starts_with("snapshot=") {
                    timestamp += 1; // Use snapshot number as timestamp
                } else if line.starts_with("mem_heap_B=") {
                    if let Some(mem_str) = line.strip_prefix("mem_heap_B=") {
                        if let Ok(mem) = mem_str.trim().parse::<u64>() {
                            heap_usage_timeline.push((Duration::from_secs(timestamp), mem));
                            peak_memory = peak_memory.max(mem);
                        }
                    }
                }
            }
        }
        
        // Sort allocation hotspots by size (descending)
        allocation_hotspots.sort_by(|a, b| b.1.cmp(&a.1));
        
        Ok(MemoryDetails {
            peak_memory,
            total_allocations,
            total_deallocations,
            leaks,
            allocation_hotspots,
            heap_usage_timeline,
        })
    }
    
    /// Identify memory issues from the analysis results
    fn identify_memory_issues(&self, memory_details: &MemoryDetails) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        // Check for memory leaks
        for leak in &memory_details.leaks {
            let severity = if leak.size > 1_000_000 {
                IssueSeverity::High
            } else if leak.size > 10_000 {
                IssueSeverity::Medium
            } else {
                IssueSeverity::Low
            };
            
            let stack_trace = if !leak.stack_trace.is_empty() {
                format!("\nStack trace:\n{}", leak.stack_trace.join("\n"))
            } else {
                String::new()
            };
            
            issues.push(AnalysisIssue {
                severity,
                description: format!(
                    "Memory leak of {} bytes at {}{}",
                    leak.size, leak.allocation_location, stack_trace
                ),
                location: Some(leak.allocation_location.clone()),
                suggestion: Some("Ensure all allocated memory is properly freed.".to_string()),
            });
        }
        
        // Check for suspicious allocation patterns
        if memory_details.total_allocations > 0 && 
           memory_details.total_deallocations == 0 {
            issues.push(AnalysisIssue {
                severity: IssueSeverity::High,
                description: format!(
                    "No memory deallocations detected despite {} allocations. This suggests systematic memory leaks.",
                    memory_details.total_allocations
                ),
                location: None,
                suggestion: Some("Implement proper memory management and deallocation.".to_string()),
            });
        }
        
        // Check for high memory usage
        if memory_details.peak_memory > 1_000_000_000 { // 1 GB
            issues.push(AnalysisIssue {
                severity: IssueSeverity::Medium,
                description: format!(
                    "High peak memory usage: {} bytes ({}MB)",
                    memory_details.peak_memory,
                    memory_details.peak_memory / 1_000_000
                ),
                location: None,
                suggestion: Some("Consider optimizing memory usage for large data structures.".to_string()),
            });
        }
        
        // Check for memory allocation hotspots
        if !memory_details.allocation_hotspots.is_empty() {
            let (hotspot_fn, bytes) = &memory_details.allocation_hotspots[0];
            if *bytes > 1_000_000 {
                issues.push(AnalysisIssue {
                    severity: IssueSeverity::Medium,
                    description: format!(
                        "Memory allocation hotspot: function '{}' allocated {} bytes ({}MB)",
                        hotspot_fn, bytes, bytes / 1_000_000
                    ),
                    location: Some(hotspot_fn.clone()),
                    suggestion: Some("Consider optimizing this function to reduce memory usage.".to_string()),
                });
            }
        }
        
        issues
    }
}

/// Helper functions for parsing XML
fn find_xml_tag(content: &str, tag_name: &str) -> Option<&str> {
    let start_tag = format!("<{}>", tag_name);
    let end_tag = format!("</{}>", tag_name);
    
    if let Some(start_pos) = content.find(&start_tag) {
        let start = start_pos + start_tag.len();
        if let Some(end_pos) = content[start..].find(&end_tag) {
            let end = start + end_pos;
            return Some(&content[start_pos..end + end_tag.len()]);
        }
    }
    None
}

fn find_xml_tag_after(content: &str, tag_name: &str, after: &str) -> Option<&str> {
    let offset = after.as_ptr() as usize - content.as_ptr() as usize;
    let remaining = &content[offset..];
    
    let start_tag = format!("<{}>", tag_name);
    let end_tag = format!("</{}>", tag_name);
    
    if let Some(start_pos) = remaining.find(&start_tag) {
        let start = start_pos + start_tag.len();
        if let Some(end_pos) = remaining[start..].find(&end_tag) {
            let end = start + end_pos;
            return Some(&remaining[start_pos..end + end_tag.len()]);
        }
    }
    None
}

fn extract_xml_content(tag: &str, tag_name: &str) -> String {
    let start_tag = format!("<{}>", tag_name);
    let end_tag = format!("</{}>", tag_name);
    
    if let Some(start_pos) = tag.find(&start_tag) {
        let start = start_pos + start_tag.len();
        if let Some(end_pos) = tag[start..].find(&end_tag) {
            let end = start + end_pos;
            return tag[start..end].to_string();
        }
    }
    String::new()
}

/// AddressSanitizer (ASan) analyzer for memory error detection
pub struct AddressSanitizer;

impl Analyzer for AddressSanitizer {
    fn name(&self) -> &str {
        "AddressSanitizer Memory Analyzer"
    }
    
    fn is_available(&self) -> bool {
        // Check if gcc/clang with sanitizer support is available
        let is_gcc_available = Command::new("gcc")
            .args(["--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
            
        let is_clang_available = Command::new("clang")
            .args(["--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
            
        is_gcc_available || is_clang_available
    }
    
    fn analyze(&self, file_path: &Path, options: &AnalysisOptions) -> Result<AnalysisResult> {
        let start_time = Instant::now();
        let verbose = options.verbose;
        
        // We need to compile and run the code with ASan enabled
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        
        // Determine the file type for proper compilation
        let file_type = crate::detectors::detect_file_type(file_path)?;
        
        // Prepare the executable with ASan enabled
        let executable_path = self.prepare_executable(&file_type, file_path, &temp_dir, verbose)?;
        
        // Create output directory for analysis data if requested
        let output_dir = if let Some(dir) = &options.output_dir {
            dir.clone()
        } else {
            temp_dir.path().to_string_lossy().to_string()
        };
        
        // Set up environment variables for ASan
        let asan_options = match options.analysis_level {
            AnalysisLevel::Basic => "detect_leaks=1",
            AnalysisLevel::Standard => "detect_leaks=1:detect_stack_use_after_return=1",
            AnalysisLevel::Comprehensive => "detect_leaks=1:detect_stack_use_after_return=1:check_initialization_order=1:strict_init_order=1",
            AnalysisLevel::Intensive => "detect_leaks=1:detect_stack_use_after_return=1:check_initialization_order=1:strict_init_order=1:detect_invalid_pointer_pairs=2:strict_string_checks=1",
        };
        
        // Run the executable with ASan enabled
        if verbose {
            println!("Running AddressSanitizer with options: {}", asan_options);
        }
        
        let output_file = format!("{}/asan_output.txt", output_dir);
        let mut asan_cmd = Command::new(&executable_path);
        asan_cmd
            .env("ASAN_OPTIONS", asan_options)
            .stdout(std::fs::File::create(&output_file).context("Failed to create output file")?)
            .stderr(std::process::Stdio::piped())
            .current_dir(&temp_dir.path());
            
        let asan_output = asan_cmd
            .output()
            .context("Failed to run executable with ASan")?;
            
        // Read ASan output from stderr
        let asan_stderr = String::from_utf8_lossy(&asan_output.stderr).to_string();
        
        // Also append the output file content if available
        let mut asan_output_text = asan_stderr.clone();
        if let Ok(file_content) = fs::read_to_string(&output_file) {
            if !file_content.is_empty() {
                asan_output_text.push_str("\n");
                asan_output_text.push_str(&file_content);
            }
        }
        
        // If we have results, save them as a report file
        let mut report_files = Vec::new();
        if options.save_results {
            report_files.push(output_file);
        }
        
        // Parse the ASan results
        let memory_details = self.parse_asan_results(&asan_output_text)?;
        
        // Identify potential memory issues
        let issues = self.identify_memory_issues(&memory_details);
        
        let duration = start_time.elapsed();
        
        // Create the analysis result
        let result = AnalysisResult {
            analyzer_name: self.name().to_string(),
            success: !asan_output.status.success() || !memory_details.leaks.is_empty(),
            duration,
            summary: format!(
                "Memory errors: {}, Leaks: {}",
                if issues.is_empty() { 0 } else { issues.len() - memory_details.leaks.len() },
                memory_details.leaks.len()
            ),
            details: AnalysisDetails::Memory(memory_details),
            report_files,
            issues,
        };
        
        Ok(result)
    }
    
    fn required_dependencies(&self) -> Vec<&str> {
        vec!["gcc", "clang"]
    }
    
    fn installation_instructions(&self) -> String {
        r#"To use AddressSanitizer:
- On Debian/Ubuntu: sudo apt-get install gcc clang
- On Fedora/RHEL: sudo dnf install gcc clang
- On Arch Linux: sudo pacman -S gcc clang
- On macOS: Install Xcode or brew install gcc llvm
- On Windows: Install MSVC with clang components or MinGW"#
            .to_string()
    }
}

impl AddressSanitizer {
    /// Prepare an executable from the source file with ASan enabled
    fn prepare_executable(&self, file_type: &FileType, file_path: &Path, temp_dir: &TempDir, verbose: bool) -> Result<String> {
        let executable_path = temp_dir.path().join("executable_asan");
        
        match file_type {
            FileType::C => {
                // Choose compiler (prefer clang but fall back to gcc)
                let compiler = if Command::new("clang").output().is_ok() {
                    "clang"
                } else {
                    "gcc"
                };
                
                // Compile C code with ASan flags
                let mut cmd = Command::new(compiler);
                cmd
                    .args([
                        "-fsanitize=address", // ASan
                        "-g",                 // Debug info
                        "-O1",                // Optimize slightly for better ASan results
                        "-fno-omit-frame-pointer", // Better stack traces
                        "-o", executable_path.to_str().unwrap(),
                        file_path.to_str().unwrap(),
                    ]);
                    
                let output = cmd.output().context("Failed to compile C code with ASan")?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow!("Failed to compile C code with ASan: {}", stderr));
                }
            },
            FileType::Cpp => {
                // Choose compiler (prefer clang++ but fall back to g++)
                let compiler = if Command::new("clang++").output().is_ok() {
                    "clang++"
                } else {
                    "g++"
                };
                
                // Compile C++ code with ASan flags
                let mut cmd = Command::new(compiler);
                cmd
                    .args([
                        "-fsanitize=address", // ASan
                        "-g",                 // Debug info
                        "-O1",                // Optimize slightly for better ASan results
                        "-fno-omit-frame-pointer", // Better stack traces
                        "-o", executable_path.to_str().unwrap(),
                        file_path.to_str().unwrap(),
                    ]);
                    
                let output = cmd.output().context("Failed to compile C++ code with ASan")?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow!("Failed to compile C++ code with ASan: {}", stderr));
                }
            },
            FileType::Rust => {
                // Create a simple Cargo project for Rust files
                let src_dir = temp_dir.path().join("src");
                fs::create_dir_all(&src_dir).context("Failed to create src directory")?;
                
                // Copy the Rust file to src
                let dest_path = src_dir.join(file_path.file_name().unwrap());
                fs::copy(file_path, &dest_path).context("Failed to copy Rust file")?;
                
                // Create a simple Cargo.toml with ASan configuration
                let cargo_toml = r#"
[package]
name = "asan_analysis"
version = "0.1.0"
edition = "2021"

[dependencies]

[profile.dev]
overflow-checks = true

[profile.release]
debug = 1
overflow-checks = true
"#;
                let cargo_toml_path = temp_dir.path().join("Cargo.toml");
                fs::write(&cargo_toml_path, cargo_toml).context("Failed to write Cargo.toml")?;
                
                // Create .cargo/config.toml to enable ASan
                let cargo_config_dir = temp_dir.path().join(".cargo");
                fs::create_dir_all(&cargo_config_dir).context("Failed to create .cargo directory")?;
                
                let cargo_config = r#"
[target.x86_64-unknown-linux-gnu]
rustflags = ["-Zsanitizer=address"]

[build]
rustflags = ["-Zsanitizer=address"]
"#;
                let cargo_config_path = cargo_config_dir.join("config.toml");
                fs::write(&cargo_config_path, cargo_config).context("Failed to write .cargo/config.toml")?;
                
                // Build the Rust code with ASan enabled
                let mut cmd = Command::new("cargo");
                cmd
                    .args(["+nightly", "build", "--target", "x86_64-unknown-linux-gnu"])
                    .env("RUSTFLAGS", "-Zsanitizer=address")
                    .current_dir(temp_dir.path());
                    
                let output = cmd.output().context("Failed to build Rust code with ASan")?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow!("Failed to build Rust code with ASan: {}", stderr));
                }
                
                // Set the executable path to the target binary
                let executable_path = temp_dir.path().join("target/x86_64-unknown-linux-gnu/debug/asan_analysis");
                return Ok(executable_path.to_string_lossy().to_string());
            },
            _ => {
                return Err(anyhow!("File type {:?} is not supported for memory analysis with AddressSanitizer", file_type));
            }
        }
        
        Ok(executable_path.to_string_lossy().to_string())
    }
    
    /// Parse ASan output into structured memory details
    fn parse_asan_results(&self, asan_output: &str) -> Result<MemoryDetails> {
        let mut peak_memory = 0;
        let mut total_allocations = 0;
        let mut total_deallocations = 0;
        let mut leaks = Vec::new();
        let mut allocation_hotspots = Vec::new();
        let mut heap_usage_timeline = Vec::new();
        
        // Check for memory leaks
        if asan_output.contains("LeakSanitizer") {
            // Extract memory leak information
            for (i, line) in asan_output.lines().enumerate() {
                if line.contains("Direct leak of") || line.contains("Indirect leak of") {
                    // Extract leak size
                    let leak_size = if let Some(size_str) = line.split_whitespace().nth(3) {
                        size_str.parse::<u64>().unwrap_or(0)
                    } else {
                        0
                    };
                    
                    // Extract allocation location and stack trace
                    let mut allocation_location = String::new();
                    let mut stack_trace = Vec::new();
                    
                    // Look for the stack trace in subsequent lines
                    let lines: Vec<&str> = asan_output.lines().collect();
                    for j in (i + 1)..lines.len() {
                        let trace_line = lines[j];
                        
                        if trace_line.starts_with("    #") {
                            // This is a stack frame
                            let frame = trace_line.trim();
                            
                            if allocation_location.is_empty() && frame.contains(" in ") {
                                // Extract location from the first frame
                                if let Some(location) = frame.split(" in ").nth(1) {
                                    allocation_location = location.to_string();
                                }
                            }
                            
                            stack_trace.push(frame.to_string());
                        } else if trace_line.trim().is_empty() || 
                                  trace_line.contains("Direct leak") || 
                                  trace_line.contains("Indirect leak") {
                            // End of this stack trace
                            break;
                        }
                    }
                    
                    // Set a default location if none was found
                    if allocation_location.is_empty() {
                        allocation_location = "Unknown location".to_string();
                    }
                    
                    // Add the leak to our collection
                    leaks.push(MemoryLeak {
                        size: leak_size,
                        allocation_location,
                        stack_trace,
                    });
                }
            }
        }
        
        // Parse other memory errors (use-after-free, buffer overflow, etc.)
        // These are reported as issues, not as memory leaks
        
        // Rough estimation of memory usage based on output
        // This is a heuristic since ASan doesn't directly report peak memory
        if asan_output.contains("Allocator statistics") {
            for line in asan_output.lines() {
                if line.contains("Peak memory usage:") {
                    if let Some(size_str) = line.split(":").nth(1) {
                        let size_str = size_str.trim();
                        if let Some(size_num) = size_str.split_whitespace().next() {
                            if let Ok(size) = size_num.parse::<u64>() {
                                peak_memory = size;
                            }
                        }
                    }
                } else if line.contains("Number of allocations:") {
                    if let Some(alloc_str) = line.split(":").nth(1) {
                        let alloc_str = alloc_str.trim();
                        if let Ok(allocs) = alloc_str.parse::<u64>() {
                            total_allocations = allocs;
                        }
                    }
                }
            }
        }
        
        Ok(MemoryDetails {
            peak_memory,
            total_allocations,
            total_deallocations,
            leaks,
            allocation_hotspots,
            heap_usage_timeline,
        })
    }
    
    /// Identify memory issues from the ASan output
    fn identify_memory_issues(&self, memory_details: &MemoryDetails) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        // Add memory leaks as issues
        for leak in &memory_details.leaks {
            let severity = if leak.size > 1_000_000 {
                IssueSeverity::High
            } else if leak.size > 10_000 {
                IssueSeverity::Medium
            } else {
                IssueSeverity::Low
            };
            
            let stack_trace = if !leak.stack_trace.is_empty() {
                format!("\nStack trace:\n{}", leak.stack_trace.join("\n"))
            } else {
                String::new()
            };
            
            issues.push(AnalysisIssue {
                severity,
                description: format!(
                    "Memory leak of {} bytes at {}{}",
                    leak.size, leak.allocation_location, stack_trace
                ),
                location: Some(leak.allocation_location.clone()),
                suggestion: Some("Ensure all allocated memory is properly freed.".to_string()),
            });
        }
        
        issues
    }
    
    /// Parse memory error details from ASan output
    fn parse_memory_errors(&self, asan_output: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        // Parse specific memory errors reported by ASan
        
        // 1. Heap-use-after-free
        if let Some(pos) = asan_output.find("ERROR: AddressSanitizer: heap-use-after-free") {
            let error_start = pos;
            if let Some(error_end) = asan_output[error_start..].find("\n\n") {
                let error_text = &asan_output[error_start..error_start + error_end];
                
                // Extract location information
                let location = if let Some(loc_start) = error_text.find("#0 ") {
                    if let Some(loc_end) = error_text[loc_start..].find("\n") {
                        error_text[loc_start..loc_start+loc_end].to_string()
                    } else {
                        "Unknown location".to_string()
                    }
                } else {
                    "Unknown location".to_string()
                };
                
                issues.push(AnalysisIssue {
                    severity: IssueSeverity::Critical,
                    description: format!("Heap use-after-free detected: {}", error_text),
                    location: Some(location),
                    suggestion: Some("Memory was used after being freed. Ensure proper memory management and avoid dangling pointers.".to_string()),
                });
            }
        }
        
        // 2. Heap-buffer-overflow
        if let Some(pos) = asan_output.find("ERROR: AddressSanitizer: heap-buffer-overflow") {
            let error_start = pos;
            if let Some(error_end) = asan_output[error_start..].find("\n\n") {
                let error_text = &asan_output[error_start..error_start + error_end];
                
                // Extract location information
                let location = if let Some(loc_start) = error_text.find("#0 ") {
                    if let Some(loc_end) = error_text[loc_start..].find("\n") {
                        error_text[loc_start..loc_start+loc_end].to_string()
                    } else {
                        "Unknown location".to_string()
                    }
                } else {
                    "Unknown location".to_string()
                };
                
                issues.push(AnalysisIssue {
                    severity: IssueSeverity::Critical,
                    description: format!("Heap buffer overflow detected: {}", error_text),
                    location: Some(location),
                    suggestion: Some("Buffer overflow detected. Ensure array bounds are properly checked and allocations are sufficiently sized.".to_string()),
                });
            }
        }
        
        // 3. Stack-use-after-return
        if let Some(pos) = asan_output.find("ERROR: AddressSanitizer: stack-use-after-return") {
            let error_start = pos;
            if let Some(error_end) = asan_output[error_start..].find("\n\n") {
                let error_text = &asan_output[error_start..error_start + error_end];
                
                // Extract location information
                let location = if let Some(loc_start) = error_text.find("#0 ") {
                    if let Some(loc_end) = error_text[loc_start..].find("\n") {
                        error_text[loc_start..loc_start+loc_end].to_string()
                    } else {
                        "Unknown location".to_string()
                    }
                } else {
                    "Unknown location".to_string()
                };
                
                issues.push(AnalysisIssue {
                    severity: IssueSeverity::Critical,
                    description: format!("Stack use-after-return detected: {}", error_text),
                    location: Some(location),
                    suggestion: Some("A stack variable is being used after its scope has ended. Avoid returning pointers/references to stack variables.".to_string()),
                });
            }
        }
        
        // 4. Global-buffer-overflow
        if let Some(pos) = asan_output.find("ERROR: AddressSanitizer: global-buffer-overflow") {
            let error_start = pos;
            if let Some(error_end) = asan_output[error_start..].find("\n\n") {
                let error_text = &asan_output[error_start..error_start + error_end];
                
                // Extract location information
                let location = if let Some(loc_start) = error_text.find("#0 ") {
                    if let Some(loc_end) = error_text[loc_start..].find("\n") {
                        error_text[loc_start..loc_start+loc_end].to_string()
                    } else {
                        "Unknown location".to_string()
                    }
                } else {
                    "Unknown location".to_string()
                };
                
                issues.push(AnalysisIssue {
                    severity: IssueSeverity::Critical,
                    description: format!("Global buffer overflow detected: {}", error_text),
                    location: Some(location),
                    suggestion: Some("Global buffer overflow detected. Ensure array bounds are properly checked.".to_string()),
                });
            }
        }
        
        issues
    }
}

/// Rust-specific memory analyzer using dhat and other Rust tools
pub struct RustMemoryLeakDetector;

impl Analyzer for RustMemoryLeakDetector {
    fn name(&self) -> &str {
        "Rust Memory Analyzer"
    }
    
    fn is_available(&self) -> bool {
        // Check if cargo is available
        let output = Command::new("cargo")
            .args(["--version"])
            .output();
            
        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    fn analyze(&self, file_path: &Path, options: &AnalysisOptions) -> Result<AnalysisResult> {
        let start_time = Instant::now();
        let verbose = options.verbose;
        
        // Check file type
        let file_type = crate::detectors::detect_file_type(file_path)?;
        if file_type != FileType::Rust {
            return Err(anyhow!("RustMemoryLeakDetector only supports Rust files"));
        }
        
        // Create a temporary Cargo project
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        
        // Set up project structure
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).context("Failed to create src directory")?;
        
        // Copy the Rust file to src
        let dest_path = src_dir.join(file_path.file_name().unwrap());
        fs::copy(file_path, &dest_path).context("Failed to copy Rust file")?;
        
        // Create a wrapper main.rs if the file isn't already main.rs
        if file_path.file_name().unwrap() != "main.rs" {
            let mod_name = file_path.file_stem().unwrap().to_str().unwrap();
            let main_rs_path = src_dir.join("main.rs");
            let main_rs_content = format!(r#"
mod {};

fn main() {{
    println!("Running memory analysis");
    // Try to execute any public functions or tests in the module
}}
"#, mod_name);
            
            fs::write(&main_rs_path, main_rs_content).context("Failed to write main.rs")?;
        }
        
        // Create Cargo.toml with dhat dependency
        let cargo_toml = r#"
[package]
name = "rust_memory_analysis"
version = "0.1.0"
edition = "2021"

[dependencies]
dhat = "0.3.2"

[features]
dhat-heap = ["dhat/heap"]

[profile.dhat]
inherits = "release"
debug = true
"#;
        
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml_path, cargo_toml).context("Failed to write Cargo.toml")?;
        
        // Add dhat initialization to the main.rs
        if let Ok(mut content) = fs::read_to_string(&src_dir.join("main.rs")) {
            let dhat_init = r#"
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

"#;
            
            content.insert_str(0, dhat_init);
            
            // Also insert dhat init at the start of main function
            if let Some(main_pos) = content.find("fn main() {") {
                content.insert_str(main_pos + 12, "\n    let _profiler = dhat::Profiler::new_heap();\n");
            }
            
            fs::write(&src_dir.join("main.rs"), content).context("Failed to update main.rs with dhat initialization")?;
        }
        
        // Create output directory for analysis data if requested
        let output_dir = if let Some(dir) = &options.output_dir {
            dir.clone()
        } else {
            temp_dir.path().to_string_lossy().to_string()
        };
        
        let dhat_out_file = format!("{}/dhat-heap.json", output_dir);
        
        // Build and run with dhat-heap enabled
        if verbose {
            println!("Building Rust code with dhat-heap profiling...");
        }
        
        // Build with dhat feature
        let mut build_cmd = Command::new("cargo");
        build_cmd
            .args(["build", "--profile=dhat", "--features=dhat-heap"])
            .current_dir(&temp_dir);
            
        let build_output = build_cmd
            .output()
            .context("Failed to build Rust code with dhat")?;
            
        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            return Err(anyhow!("Failed to build Rust code with dhat: {}", stderr));
        }
        
        // Run the executable with dhat heap profiling
        if verbose {
            println!("Running Rust code with dhat-heap profiling...");
        }
        
        let mut run_cmd = Command::new("cargo");
        run_cmd
            .args(["run", "--profile=dhat", "--features=dhat-heap"])
            .env("DHAT_OUTPUT_FILE", &dhat_out_file)
            .current_dir(&temp_dir);
            
        let run_output = run_cmd
            .output()
            .context("Failed to run Rust code with dhat")?;
            
        // Save report files
        let mut report_files = Vec::new();
        if options.save_results {
            report_files.push(dhat_out_file.clone());
        }
        
        // Check if dhat output file was created
        let dhat_results = if let Ok(content) = fs::read_to_string(&dhat_out_file) {
            content
        } else {
            if verbose {
                println!("Warning: dhat output file was not created");
            }
            String::new()
        };
        
        // Also run cargo-llvm-cov for coverage-based analysis
        let coverage_report_file = format!("{}/coverage.txt", output_dir);
        if is_command_available("cargo-llvm-cov") {
            if verbose {
                println!("Running cargo-llvm-cov for coverage analysis...");
            }
            
            let mut cov_cmd = Command::new("cargo");
            cov_cmd
                .args(["llvm-cov", "--no-report"])
                .current_dir(&temp_dir);
                
            let _cov_output = cov_cmd.output().ok();
            
            // Add coverage report to report files
            if options.save_results {
                report_files.push(coverage_report_file);
            }
        }
        
        // Run drop-check analysis on the code
        let drop_check_results = self.run_drop_check(&temp_dir, verbose);
        
        // Run memory safety analysis
        let memory_safety_results = self.run_memory_safety_analysis(&temp_dir, verbose);
        
        // Parse the dhat results
        let memory_details = self.parse_dhat_results(&dhat_results, drop_check_results.as_deref())?;
        
        // Identify potential memory issues
        let mut issues = self.identify_memory_issues(&memory_details);
        
        // Add any memory safety issues
        if let Some(safety_results) = memory_safety_results {
            for issue in self.parse_safety_issues(&safety_results) {
                issues.push(issue);
            }
        }
        
        let duration = start_time.elapsed();
        
        // Create the analysis result
        let result = AnalysisResult {
            analyzer_name: self.name().to_string(),
            success: run_output.status.success() && issues.is_empty(),
            duration,
            summary: format!(
                "Peak memory: {} bytes, Allocations: {}, Potential issues: {}",
                memory_details.peak_memory,
                memory_details.total_allocations,
                issues.len()
            ),
            details: AnalysisDetails::Memory(memory_details),
            report_files,
            issues,
        };
        
        Ok(result)
    }
    
    fn required_dependencies(&self) -> Vec<&str> {
        vec!["cargo", "rustc"]
    }
    
    fn installation_instructions(&self) -> String {
        r#"To use Rust Memory Analyzer:
- Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
- Add dhat: cargo install dhat-heap
- For more advanced analysis: cargo install cargo-llvm-cov"#
            .to_string()
    }
}

impl RustMemoryLeakDetector {
    /// Parse dhat-heap results into structured memory details
    fn parse_dhat_results(&self, dhat_json: &str, drop_check_results: Option<&str>) -> Result<MemoryDetails> {
        let mut peak_memory = 0;
        let mut total_allocations = 0;
        let mut total_deallocations = 0;
        let mut leaks = Vec::new();
        let mut allocation_hotspots = Vec::new();
        let mut heap_usage_timeline = Vec::new();
        
        // Parse dhat-heap JSON output if it's available
        if !dhat_json.is_empty() {
            // The dhat output is structured JSON
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(dhat_json) {
                // Extract total memory usage
                if let Some(totals) = json_value.get("totals") {
                    if let Some(total_bytes) = totals.get("bytes") {
                        if let Some(bytes) = total_bytes.as_u64() {
                            peak_memory = bytes;
                        }
                    }
                    if let Some(allocs) = totals.get("allocs") {
                        if let Some(count) = allocs.as_u64() {
                            total_allocations = count;
                        }
                    }
                }
                
                // Extract allocation hotspots
                if let Some(allocations) = json_value.get("allocations") {
                    if let Some(allocs_array) = allocations.as_array() {
                        for alloc in allocs_array {
                            if let (Some(fun), Some(size)) = (alloc.get("function"), alloc.get("bytes")) {
                                if let (Some(fun_str), Some(size_num)) = (fun.as_str(), size.as_u64()) {
                                    allocation_hotspots.push((fun_str.to_string(), size_num));
                                }
                            }
                        }
                    }
                }
                
                // Look for potential leaks (allocations that were never freed)
                if let Some(allocations) = json_value.get("allocations") {
                    if let Some(allocs_array) = allocations.as_array() {
                        for alloc in allocs_array {
                            // Check if the allocation was never freed
                            if let Some(frees) = alloc.get("frees") {
                                if let Some(frees_num) = frees.as_u64() {
                                    if frees_num == 0 {
                                        // This is potentially a leak
                                        let size = alloc.get("bytes")
                                                        .and_then(|v| v.as_u64())
                                                        .unwrap_or(0);
                                                        
                                        let function = alloc.get("function")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("Unknown function")
                                                            .to_string();
                                                            
                                        // Extract stack trace if available
                                        let mut stack_trace = Vec::new();
                                        if let Some(stack) = alloc.get("stack") {
                                            if let Some(stack_array) = stack.as_array() {
                                                for frame in stack_array {
                                                    if let Some(frame_str) = frame.as_str() {
                                                        stack_trace.push(frame_str.to_string());
                                                    }
                                                }
                                            }
                                        }
                                        
                                        leaks.push(MemoryLeak {
                                            size,
                                            allocation_location: function,
                                            stack_trace,
                                        });
                                    } else {
                                        // Count deallocations
                                        total_deallocations += frees_num;
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Extract heap usage timeline if available
                if let Some(timeline) = json_value.get("timeline") {
                    if let Some(timeline_array) = timeline.as_array() {
                        for (i, point) in timeline_array.iter().enumerate() {
                            if let Some(bytes) = point.get("bytes").and_then(|v| v.as_u64()) {
                                // Use the index as a timestamp
                                heap_usage_timeline.push((Duration::from_secs(i as u64), bytes));
                            }
                        }
                    }
                }
            }
        }
        
        // Sort allocation hotspots by size (descending)
        allocation_hotspots.sort_by(|a, b| b.1.cmp(&a.1));
        
        Ok(MemoryDetails {
            peak_memory,
            total_allocations,
            total_deallocations,
            leaks,
            allocation_hotspots,
            heap_usage_timeline,
        })
    }
    
    /// Run drop checker analysis on the Rust code
    fn run_drop_check(&self, temp_dir: &TempDir, verbose: bool) -> Option<String> {
        if !is_command_available("rustc") {
            return None;
        }
        
        if verbose {
            println!("Running drop checker analysis...");
        }
        
        // Run rustc with -Z print-drop-order
        let mut cmd = Command::new("rustc");
        cmd
            .args(["+nightly", "-Z", "print-drop-order", "src/lib.rs"])
            .current_dir(temp_dir);
            
        match cmd.output() {
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                
                if stderr.contains("drop order") || stdout.contains("drop order") {
                    Some(format!("{}\n{}", stdout, stderr))
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    }
    
    /// Run memory safety analysis
    fn run_memory_safety_analysis(&self, temp_dir: &TempDir, verbose: bool) -> Option<String> {
        if !is_command_available("cargo-miri") {
            return None;
        }
        
        if verbose {
            println!("Running memory safety analysis with Miri...");
        }
        
        // Run miri to check for undefined behavior
        let mut cmd = Command::new("cargo");
        cmd
            .args(["+nightly", "miri", "run"])
            .current_dir(temp_dir);
            
        match cmd.output() {
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                
                Some(format!("{}\n{}", stdout, stderr))
            },
            Err(_) => None,
        }
    }
    
    /// Parse memory safety issues from Miri output
    fn parse_safety_issues(&self, safety_output: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        // Look for Miri errors
        if safety_output.contains("error: Undefined Behavior") {
            // Extract the specific error message
            let mut error_msg = String::new();
            let mut in_error = false;
            
            for line in safety_output.lines() {
                if line.contains("error: Undefined Behavior") {
                    in_error = true;
                    error_msg.push_str(line);
                    error_msg.push('\n');
                } else if in_error {
                    if line.trim().is_empty() {
                        in_error = false;
                    } else {
                        error_msg.push_str(line);
                        error_msg.push('\n');
                    }
                }
            }
            
            // Extract location information if available
            let location = if let Some(loc_start) = safety_output.find("  -->") {
                if let Some(loc_end) = safety_output[loc_start..].find('\n') {
                    Some(safety_output[loc_start..loc_start + loc_end].trim().to_string())
                } else {
                    None
                }
            } else {
                None
            };
            
            // Classify the issue by severity and type
            let (severity, suggestion) = if error_msg.contains("dangling reference") || 
                                        error_msg.contains("dangling pointer") {
                (
                    IssueSeverity::Critical,
                    "Dangling reference detected. Ensure that references don't outlive their referents.".to_string()
                )
            } else if error_msg.contains("memory leak") {
                (
                    IssueSeverity::High,
                    "Memory leak detected. Ensure all allocated memory is properly freed or dropped.".to_string()
                )
            } else if error_msg.contains("data race") {
                (
                    IssueSeverity::Critical,
                    "Data race detected. Use proper synchronization primitives like Mutex or RwLock.".to_string()
                )
            } else if error_msg.contains("use after free") {
                (
                    IssueSeverity::Critical,
                    "Use after free detected. Ensure that objects are not used after being dropped.".to_string()
                )
            } else if error_msg.contains("uninitialized") {
                (
                    IssueSeverity::High,
                    "Use of uninitialized memory detected. Ensure all variables are properly initialized.".to_string()
                )
            } else if error_msg.contains("invalid memory") || error_msg.contains("out of bounds") {
                (
                    IssueSeverity::Critical,
                    "Memory access violation detected. Check array bounds and pointer arithmetic.".to_string()
                )
            } else {
                (
                    IssueSeverity::High,
                    "Undefined behavior detected. Review your code for memory safety issues.".to_string()
                )
            };
            
            issues.push(AnalysisIssue {
                severity,
                description: format!("Memory safety issue detected: {}", error_msg),
                location,
                suggestion: Some(suggestion),
            });
        }
        
        issues
    }
    
    /// Identify memory issues from the analysis results
    fn identify_memory_issues(&self, memory_details: &MemoryDetails) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        // Check for memory leaks
        for leak in &memory_details.leaks {
            let severity = if leak.size > 1_000_000 {
                IssueSeverity::High
            } else if leak.size > 10_000 {
                IssueSeverity::Medium
            } else {
                IssueSeverity::Low
            };
            
            let stack_trace = if !leak.stack_trace.is_empty() {
                format!("\nStack trace:\n{}", leak.stack_trace.join("\n"))
            } else {
                String::new()
            };
            
            issues.push(AnalysisIssue {
                severity,
                description: format!(
                    "Memory leak of {} bytes in Rust code at {}{}",
                    leak.size, leak.allocation_location, stack_trace
                ),
                location: Some(leak.allocation_location.clone()),
                suggestion: Some("Check for missing Drop implementations, forgotten Rc/Arc cycles, or forgotten Box/Vec allocations.".to_string()),
            });
        }
        
        // Check for memory allocation hotspots
        if !memory_details.allocation_hotspots.is_empty() {
            let (hotspot_fn, bytes) = &memory_details.allocation_hotspots[0];
            if *bytes > 1_000_000 {
                issues.push(AnalysisIssue {
                    severity: IssueSeverity::Medium,
                    description: format!(
                        "Memory allocation hotspot: function '{}' allocated {} bytes ({}MB)",
                        hotspot_fn, bytes, bytes / 1_000_000
                    ),
                    location: Some(hotspot_fn.clone()),
                    suggestion: Some("Consider optimizing allocations in this function. Use custom allocators, pre-allocation, or memory pooling.".to_string()),
                });
            }
        }
        
        // Check for allocation/deallocation imbalance
        if memory_details.total_allocations > 0 && 
           memory_details.total_deallocations == 0 {
            issues.push(AnalysisIssue {
                severity: IssueSeverity::High,
                description: format!(
                    "No memory deallocations detected despite {} allocations. This suggests systematic memory leaks.",
                    memory_details.total_allocations
                ),
                location: None,
                suggestion: Some("Implement proper Drop traits or ensure manual memory management is correct.".to_string()),
            });
        }
        
        issues
    }
}

/// Python memory analyzer with memory_profiler and tracemalloc integration
pub struct PythonMemoryAnalyzer;

impl Analyzer for PythonMemoryAnalyzer {
    fn name(&self) -> &str {
        "Python Memory Analyzer"
    }
    
    fn is_available(&self) -> bool {
        // Check if Python is available
        let output = Command::new("python")
            .args(["--version"])
            .output();
            
        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    fn analyze(&self, file_path: &Path, options: &AnalysisOptions) -> Result<AnalysisResult> {
        let start_time = Instant::now();
        let verbose = options.verbose;
        
        // Check file type
        let file_type = crate::detectors::detect_file_type(file_path)?;
        if file_type != FileType::Python {
            return Err(anyhow!("PythonMemoryAnalyzer only supports Python files"));
        }
        
        // Create a temporary directory for analysis
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        
        // Create output directory for analysis data if requested
        let output_dir = if let Some(dir) = &options.output_dir {
            dir.clone()
        } else {
            temp_dir.path().to_string_lossy().to_string()
        };
        
        // Copy the Python file to the temp directory
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let dest_path = temp_dir.path().join(file_name);
        fs::copy(file_path, &dest_path).context("Failed to copy Python file")?;
        
        // Create a wrapper script for memory profiling
        let wrapper_path = temp_dir.path().join("memory_profile_wrapper.py");
        let wrapper_content = self.generate_memory_profiler_wrapper(file_name, &output_dir);
        fs::write(&wrapper_path, wrapper_content).context("Failed to write memory profiler wrapper")?;
        
        // Create a wrapper script for tracemalloc
        let tracemalloc_path = temp_dir.path().join("tracemalloc_wrapper.py");
        let tracemalloc_content = self.generate_tracemalloc_wrapper(file_name, &output_dir);
        fs::write(&tracemalloc_path, tracemalloc_content).context("Failed to write tracemalloc wrapper")?;
        
        // Run both wrappers
        let mut report_files = Vec::new();
        let memory_profiler_output_file = format!("{}/memory_profile.txt", output_dir);
        let tracemalloc_output_file = format!("{}/tracemalloc_snapshot.txt", output_dir);
        let object_lifetime_file = format!("{}/object_lifetime.txt", output_dir);
        
        if verbose {
            println!("Running Python memory profiler...");
        }
        
        // Run memory profiler
        let mut prof_cmd = Command::new("python");
        prof_cmd.arg(&wrapper_path).current_dir(&temp_dir);
        
        let prof_output = prof_cmd.output().context("Failed to run memory profiler")?;
        
        if prof_output.status.success() && options.save_results {
            report_files.push(memory_profiler_output_file.clone());
        }
        
        // Run tracemalloc
        if verbose {
            println!("Running tracemalloc for detailed memory tracking...");
        }
        
        let mut trace_cmd = Command::new("python");
        trace_cmd.arg(&tracemalloc_path).current_dir(&temp_dir);
        
        let trace_output = trace_cmd.output().context("Failed to run tracemalloc")?;
        
        if trace_output.status.success() && options.save_results {
            report_files.push(tracemalloc_output_file.clone());
            report_files.push(object_lifetime_file.clone());
        }
        
        // Read the output files
        let memory_profile_results = fs::read_to_string(&memory_profiler_output_file).unwrap_or_default();
        let tracemalloc_results = fs::read_to_string(&tracemalloc_output_file).unwrap_or_default();
        let object_lifetime_results = fs::read_to_string(&object_lifetime_file).unwrap_or_default();
        
        // Parse the results
        let memory_details = self.parse_python_memory_results(
            &memory_profile_results, 
            &tracemalloc_results,
            &object_lifetime_results
        )?;
        
        // Identify potential memory issues
        let issues = self.identify_memory_issues(&memory_details);
        
        let duration = start_time.elapsed();
        
        // Create the analysis result
        let result = AnalysisResult {
            analyzer_name: self.name().to_string(),
            success: true,
            duration,
            summary: format!(
                "Peak memory: {} bytes, Potential memory issues: {}",
                memory_details.peak_memory,
                issues.len()
            ),
            details: AnalysisDetails::Memory(memory_details),
            report_files,
            issues,
        };
        
        Ok(result)
    }
    
    fn required_dependencies(&self) -> Vec<&str> {
        vec!["python"]
    }
    
    fn installation_instructions(&self) -> String {
        r#"To use Python Memory Analyzer:
- Install Python: https://www.python.org/downloads/
- Install memory_profiler: pip install memory_profiler
- For more advanced analysis: pip install objgraph"#
            .to_string()
    }
}

impl PythonMemoryAnalyzer {
    /// Generate a wrapper script for memory_profiler
    fn generate_memory_profiler_wrapper(&self, file_name: &str, output_dir: &str) -> String {
        format!(r##"
#!/usr/bin/env python

import sys
import os
import importlib.util
import time

# Try to import memory_profiler
try:
    import memory_profiler
except ImportError:
    print("Warning: memory_profiler not installed. Run: pip install memory_profiler")
    sys.exit(1)

# Set up the output file
output_file = "{}/memory_profile.txt"

# Get the file path
file_path = "{}"

# Create a wrapper function to profile
@memory_profiler.profile(stream=open(output_file, 'w+'))
def run_code():
    # Load the module
    try:
        # Remove the .py extension if present
        module_name = file_path
        if module_name.endswith('.py'):
            module_name = module_name[:-3]
            
        # Import the module
        spec = importlib.util.spec_from_file_location(module_name, file_path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        
        # Try to run the main function if it exists
        if hasattr(module, 'main') and callable(module.main):
            module.main()
    except Exception as e:
        print(f"Error running code: {{e}}")
        sys.exit(1)

# Run the profiled code
run_code()

# Also record memory usage over time
try:
    with open(output_file, "a") as f:
        f.write("\n\n--- Memory Usage Timeline ---\n")
        
        # Record memory usage over time
        usage_data = []
        start_time = time.time()
        
        # Record baseline memory usage
        baseline = memory_profiler.memory_usage(max_usage=True)
        f.write(f"Baseline: {{baseline}} MiB\n")
        
        # Try to run the module again with memory tracking
        try:
            # Import the module
            spec = importlib.util.spec_from_file_location("module", file_path)
            module = importlib.util.module_from_spec(spec)
            
            # Track memory usage
            mem_usage = memory_profiler.memory_usage((spec.loader.exec_module, (module,)), interval=0.1, timeout=10)
            
            # Write memory usage timeline
            for i, mem in enumerate(mem_usage):
                elapsed = i * 0.1  # 0.1 second interval
                f.write(f"{{elapsed:.1f}}s: {{mem}} MiB\n")
                
        except Exception as e:
            f.write(f"Error during memory timeline tracking: {{e}}\n")
except Exception as e:
    print(f"Error in memory timeline tracking: {{e}}")
"##, output_dir, file_name)
    }
    
    /// Parse Python memory profiling results
    fn parse_python_memory_results(&self, memory_profile: &str, tracemalloc: &str, object_lifetime: &str) -> Result<MemoryDetails> {
        let mut peak_memory = 0;
        let mut total_allocations = 0;
        let mut total_deallocations = 0;
        let mut leaks = Vec::new();
        let mut allocation_hotspots = Vec::new();
        let mut heap_usage_timeline = Vec::new();
        
        // Parse memory_profiler output
        for line in memory_profile.lines() {
            // Extract peak memory usage
            if line.contains("MiB") {
                if let Some(mem_str) = line.split_whitespace().next() {
                    if let Ok(mem) = mem_str.parse::<f64>() {
                        // Convert MiB to bytes
                        let mem_bytes = (mem * 1024.0 * 1024.0) as u64;
                        peak_memory = peak_memory.max(mem_bytes);
                    }
                }
            }
            
            // Extract timeline data
            if line.contains("s:") && line.contains("MiB") {
                let parts: Vec<&str> = line.split(":").collect();
                if parts.len() >= 2 {
                    if let Ok(time_str) = parts[0].trim().replace("s", "").parse::<f64>() {
                        let time_duration = Duration::from_millis((time_str * 1000.0) as u64);
                        
                        if let Some(mem_str) = parts[1].trim().split_whitespace().next() {
                            if let Ok(mem) = mem_str.parse::<f64>() {
                                // Convert MiB to bytes
                                let mem_bytes = (mem * 1024.0 * 1024.0) as u64;
                                heap_usage_timeline.push((time_duration, mem_bytes));
                            }
                        }
                    }
                }
            }
        }

        // Parse tracemalloc output
        if !tracemalloc.is_empty() {
            let mut parsing_allocations = false;
            let mut parsing_by_file = false;
            let mut parsing_by_category = false;
            
            for line in tracemalloc.lines() {
                // Count allocations
                if line.contains("blocks:") {
                    // Extract block counts
                    if let Some(count_str) = line.split("- ").nth(1).and_then(|s| s.split(" ").next()) {
                        if let Ok(count) = count_str.parse::<u64>() {
                            total_allocations += count;
                        }
                    }
                    
                    // Extract file information for hotspots
                    if parsing_by_file {
                        if let Some(kb_str) = line.split_whitespace().next() {
                            if let Ok(kb) = kb_str.parse::<f64>() {
                                let bytes = (kb * 1024.0) as u64;
                                
                                if let Some(file_loc) = line.split(": ").nth(1) {
                                    allocation_hotspots.push((file_loc.trim().to_string(), bytes));
                                }
                            }
                        }
                    }
                }
                
                // Detect sections
                if line.contains("Top memory allocations:") {
                    parsing_allocations = true;
                    parsing_by_file = false;
                    parsing_by_category = false;
                } else if line.contains("Memory allocations by file:") {
                    parsing_allocations = false;
                    parsing_by_file = true;
                    parsing_by_category = false;
                } else if line.contains("Memory allocations by category:") {
                    parsing_allocations = false;
                    parsing_by_file = false;
                    parsing_by_category = true;
                }
            }
        }
        
        // Parse object lifetime and reference tracking results
        if !object_lifetime.is_empty() {
            for line in object_lifetime.lines() {
                // Check for potential memory leaks
                if line.contains("Warning: Large number of") && line.contains("objects:") {
                    let parts: Vec<&str> = line.split("objects:").collect();
                    if parts.len() >= 2 {
                        if let Ok(count) = parts[1].trim().parse::<u64>() {
                            // Object type is mentioned in the warning
                            let object_type = if let Some(type_str) = line.split("of ").nth(1) {
                                if let Some(end) = type_str.find(" objects") {
                                    type_str[..end].to_string()
                                } else {
                                    "Unknown".to_string()
                                }
                            } else {
                                "Unknown".to_string()
                            };
                            
                            // Create a memory leak entry
                            // Size is unknown, so use the count as a proxy (multiply by estimated size)
                            let estimated_size = count * 64; // Rough estimate: 64 bytes per object
                            
                            leaks.push(MemoryLeak {
                                size: estimated_size,
                                allocation_location: format!("Large number of {} objects", object_type),
                                stack_trace: vec![format!("Count: {}", count)],
                            });
                        }
                    }
                }
                
                // Check for circular references
                if line.contains("Found circular reference:") {
                    let ref_info = line.split(": ").nth(1).unwrap_or("Unknown circular reference");
                    
                    // Create a memory leak entry for circular references
                    leaks.push(MemoryLeak {
                        size: 0, // Size unknown for circular references
                        allocation_location: "Circular reference detected".to_string(),
                        stack_trace: vec![ref_info.to_string()],
                    });
                }
            }
        }
        
        // Calculate peak memory if heap usage timeline is available
        if !heap_usage_timeline.is_empty() {
            let max_usage = heap_usage_timeline.iter().map(|(_, size)| *size).max().unwrap_or(0);
            peak_memory = peak_memory.max(max_usage);
        }
        
        // Sort allocation hotspots by size (descending)
        allocation_hotspots.sort_by(|a, b| b.1.cmp(&a.1));
        
        Ok(MemoryDetails {
            peak_memory,
            total_allocations,
            total_deallocations,
            leaks,
            allocation_hotspots,
            heap_usage_timeline,
        })
    }
    
    /// Identify memory issues from the analysis results
    fn identify_memory_issues(&self, memory_details: &MemoryDetails) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        // Check for memory leaks
        for leak in &memory_details.leaks {
            let severity = if leak.allocation_location.contains("Circular reference") {
                IssueSeverity::High
            } else if leak.size > 1_000_000 {
                IssueSeverity::High
            } else if leak.size > 10_000 {
                IssueSeverity::Medium
            } else {
                IssueSeverity::Low
            };
            
            let stack_trace = if !leak.stack_trace.is_empty() {
                format!("\nDetails:\n{}", leak.stack_trace.join("\n"))
            } else {
                String::new()
            };
            
            let suggestion = if leak.allocation_location.contains("Circular reference") {
                "Circular references prevent garbage collection. Use weak references (weakref) to break reference cycles.".to_string()
            } else if leak.allocation_location.contains("Large number of") {
                "Large number of objects suggests a potential memory leak. Check object lifecycle management and ensure objects are properly dereferenced.".to_string()
            } else {
                "Ensure objects are properly dereferenced when no longer needed. Consider using context managers or explicit del statements.".to_string()
            };
            
            issues.push(AnalysisIssue {
                severity,
                description: format!(
                    "Potential memory issue: {}{}",
                    leak.allocation_location, stack_trace
                ),
                location: Some(leak.allocation_location.clone()),
                suggestion: Some(suggestion),
            });
        }
        
        // Check for allocation hotspots
        if !memory_details.allocation_hotspots.is_empty() {
            let top_hotspots = memory_details.allocation_hotspots.iter().take(3);
            
            for (i, (hotspot, bytes)) in top_hotspots.enumerate() {
                if *bytes > 1_000_000 {
                    issues.push(AnalysisIssue {
                        severity: IssueSeverity::Medium,
                        description: format!(
                            "Memory allocation hotspot #{}: {} allocated {} bytes ({}MB)",
                            i+1, hotspot, bytes, bytes / 1_000_000
                        ),
                        location: Some(hotspot.clone()),
                        suggestion: Some("Consider optimizing memory usage in this area. Use generators, iterate instead of loading entire collections, or implement data streaming.".to_string()),
                    });
                }
            }
        }
        
        // Check for high overall memory usage
        if memory_details.peak_memory > 100_000_000 { // 100 MB
            issues.push(AnalysisIssue {
                severity: IssueSeverity::Medium,
                description: format!(
                    "High peak memory usage: {} bytes ({}MB)",
                    memory_details.peak_memory,
                    memory_details.peak_memory / 1_000_000
                ),
                location: None,
                suggestion: Some("Consider optimizing overall memory usage. Use streaming for large data, chunking for processing, or more memory-efficient data structures.".to_string()),
            });
        }
        
        // Check for memory growth patterns from the timeline
        if memory_details.heap_usage_timeline.len() > 2 {
            let first = memory_details.heap_usage_timeline.first().unwrap().1;
            let last = memory_details.heap_usage_timeline.last().unwrap().1;
            
            // Check for consistently increasing memory usage (potential leak)
            let mut consistently_increasing = true;
            for i in 1..memory_details.heap_usage_timeline.len() {
                let prev = memory_details.heap_usage_timeline[i-1].1;
                let curr = memory_details.heap_usage_timeline[i].1;
                
                if curr < prev {
                    consistently_increasing = false;
                    break;
                }
            }
            
            if consistently_increasing && last > first * 2 {
                issues.push(AnalysisIssue {
                    severity: IssueSeverity::High,
                    description: format!(
                        "Memory usage consistently increases over time from {}MB to {}MB, suggesting a memory leak",
                        first / 1_000_000,
                        last / 1_000_000
                    ),
                    location: None,
                    suggestion: Some("Check for unbounded growth in collections, caches without size limits, or failure to release resources. Consider using memory_profiler line by line to identify leaking code.".to_string()),
                });
            }
        }
        
        issues
    }
}

/// Helper function to check if a command is available
fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

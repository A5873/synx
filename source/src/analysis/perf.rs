//! Performance analysis module for Synx
//!
//! This module provides CPU and performance profiling capabilities:
//! - CPU profiling using perf_events
//! - Hardware performance counters
//! - Function-level timing
//! - Cache behavior analysis
//! - Branch prediction analysis

use std::path::Path;
use std::process::Command;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::fs::{self};
use tempfile::TempDir;
use anyhow::{Result, Context, anyhow};

use crate::detectors::FileType;
use crate::analysis::{
    Analyzer, AnalysisOptions, AnalysisResult, AnalysisDetails, PerformanceDetails,
    AnalysisIssue, IssueSeverity, AnalysisLevel, AnalysisType, HotspotFunction
};

/// CPU Profiler using Linux perf or compatible tools
pub struct PerfCpuProfiler;

impl PerfCpuProfiler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Analyzer for PerfCpuProfiler {
    fn name(&self) -> &str {
        "CPU Profiler (perf)"
    }
    
    fn is_available(&self) -> bool {
        // Check if perf is available on the system
        let output = Command::new("which")
            .arg("perf")
            .output();
            
        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    fn analyze(&self, file_path: &Path, options: &AnalysisOptions) -> Result<AnalysisResult> {
        let start_time = Instant::now();
        let verbose = options.verbose;
        
        // We need to compile and run the code to profile it
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        
        // Determine the file type for proper compilation
        let file_type = crate::detectors::detect_file_type(file_path)?;
        
        // Prepare the executable
        let executable_path = self.prepare_executable(&file_type, file_path, &temp_dir, verbose)?;
        
        // Create output directory for perf data if requested
        let output_dir = if let Some(dir) = &options.output_dir {
            dir.clone()
        } else {
            temp_dir.path().to_string_lossy().to_string()
        };
        
        let perf_data_path = format!("{}/perf.data", output_dir);
        
        // Determine the perf events to record based on analysis level
        let perf_events = match options.analysis_level {
            AnalysisLevel::Basic => "cycles,instructions",
            AnalysisLevel::Standard => "cycles,instructions,cache-misses,branch-misses",
            AnalysisLevel::Comprehensive => "cycles,instructions,cache-references,cache-misses,branch-instructions,branch-misses,L1-dcache-loads,L1-dcache-load-misses",
            AnalysisLevel::Intensive => "cycles,instructions,cache-references,cache-misses,branch-instructions,branch-misses,L1-dcache-loads,L1-dcache-load-misses,L1-dcache-stores,L1-dcache-store-misses,L1-icache-loads,L1-icache-load-misses,dTLB-loads,dTLB-load-misses,iTLB-loads,iTLB-load-misses,branch-loads,branch-load-misses",
        };
        
        // Run perf record to gather CPU performance data
        if verbose {
            println!("Running perf record with events: {}", perf_events);
        }
        
        let mut perf_cmd = Command::new("perf");
        perf_cmd
            .args(["record", "-e", perf_events, "-o", &perf_data_path])
            .arg(&executable_path)
            .current_dir(&temp_dir.path());
            
        let perf_output = perf_cmd
            .output()
            .context("Failed to run perf record")?;
            
        if !perf_output.status.success() && verbose {
            let stderr = String::from_utf8_lossy(&perf_output.stderr);
            println!("Warning: perf record exited with errors: {}", stderr);
        }
        
        // Run perf report to get function-level profiling data
        let mut report_cmd = Command::new("perf");
        report_cmd
            .args(["report", "-i", &perf_data_path, "--stdio"])
            .current_dir(&temp_dir.path());
            
        let report_output = report_cmd
            .output()
            .context("Failed to run perf report")?;
            
        let report_text = String::from_utf8_lossy(&report_output.stdout).to_string();
        
        // Run perf stat to get overall performance metrics
        let mut stat_cmd = Command::new("perf");
        stat_cmd
            .args(["stat", "-e", perf_events, "-o", "perf_stat.txt"])
            .arg(&executable_path)
            .current_dir(&temp_dir.path());
            
        let stat_output = stat_cmd
            .output()
            .context("Failed to run perf stat")?;
            
        // Read perf stat results
        let stat_text = if stat_output.status.success() {
            fs::read_to_string(temp_dir.path().join("perf_stat.txt"))
                .unwrap_or_else(|_| "Failed to read perf stat output".to_string())
        } else {
            "Perf stat failed to run".to_string()
        };
        
        // Parse the performance results
        let performance_details = self.parse_perf_results(&report_text, &stat_text)?;
        
        // Generate a call graph if requested for comprehensive analysis
        let mut report_files = Vec::new();
        if matches!(options.analysis_level, AnalysisLevel::Comprehensive | AnalysisLevel::Intensive) {
            // Generate a call graph using perf
            let callgraph_path = format!("{}/callgraph.svg", output_dir);
            
            let mut callgraph_cmd = Command::new("perf");
            callgraph_cmd
                .args(["report", "-i", &perf_data_path, "--call-graph", "fractal,0.5,caller", "-g", "graph", "--stdio"])
                .current_dir(&temp_dir.path());
                
            let _callgraph_output = callgraph_cmd
                .output()
                .context("Failed to generate call graph")?;
                
            if options.save_results {
                report_files.push(callgraph_path);
            }
        }
        
        // Identify potential performance issues
        let issues = self.identify_performance_issues(&performance_details);
        
        let duration = start_time.elapsed();
        
        // Create the analysis result
        let result = AnalysisResult {
            analyzer_name: self.name().to_string(),
            success: true,
            duration,
            summary: format!(
                "CPU usage: {:.2}%, Instructions: {}, Cycles: {}, Cache Misses: {}, Branch Misses: {}",
                performance_details.cpu_usage,
                performance_details.cpu_events.get("instructions").unwrap_or(&0),
                performance_details.cpu_events.get("cycles").unwrap_or(&0),
                performance_details.cpu_events.get("cache-misses").unwrap_or(&0),
                performance_details.cpu_events.get("branch-misses").unwrap_or(&0)
            ),
            details: AnalysisDetails::Performance(performance_details),
            report_files,
            issues,
        };
        
        Ok(result)
    }
    
    fn required_dependencies(&self) -> Vec<&str> {
        vec!["perf"]
    }
    
    fn installation_instructions(&self) -> String {
        r#"To install perf:
- On Debian/Ubuntu: sudo apt-get install linux-tools-common linux-tools-generic linux-tools-$(uname -r)
- On Fedora/RHEL: sudo dnf install perf
- On Arch Linux: sudo pacman -S perf
- On macOS: perf is not available, consider using Instruments or DTrace instead
- On Windows: perf is not available, consider using Windows Performance Toolkit instead"#
            .to_string()
    }
}

impl PerfCpuProfiler {
    /// Prepare an executable from the source file for profiling
    fn prepare_executable(&self, file_type: &FileType, file_path: &Path, temp_dir: &TempDir, verbose: bool) -> Result<String> {
        let executable_path = temp_dir.path().join("executable");
        
        match file_type {
            FileType::C => {
                // Compile C code with perf-friendly flags
                let mut cmd = Command::new("gcc");
                cmd
                    .args([
                        "-g", // Debug info
                        "-O0", // No optimization for better profiling
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
                // Compile C++ code with perf-friendly flags
                let mut cmd = Command::new("g++");
                cmd
                    .args([
                        "-g", // Debug info
                        "-O0", // No optimization for better profiling
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
name = "perf_analysis"
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
                let executable_path = temp_dir.path().join("target/release/perf_analysis");
                return Ok(executable_path.to_string_lossy().to_string());
            },
            _ => {
                return Err(anyhow!("File type {:?} is not supported for perf profiling", file_type));
            }
        }
        
        Ok(executable_path.to_string_lossy().to_string())
    }
    
    /// Parse perf results into structured performance details
    fn parse_perf_results(&self, report_text: &str, stat_text: &str) -> Result<PerformanceDetails> {
        let mut cpu_usage = 0.0;
        let mut execution_time = Duration::from_secs(0);
        let mut user_time = Duration::from_secs(0);
        let mut system_time = Duration::from_secs(0);
        let mut function_times = HashMap::new();
        let mut hotspots = Vec::new();
        let mut cpu_events = HashMap::new();
        
        // Parse CPU usage from stat output
        if let Some(cpu_line) = stat_text.lines().find(|line| line.contains("CPU utilization")) {
            if let Some(percentage) = cpu_line
                .split(':')
                .nth(1)
                .and_then(|s| s.trim().split('%').next())
                .and_then(|s| s.trim().parse::<f64>().ok())
            {
                cpu_usage = percentage;
            }
        }
        
        // Parse execution time from stat output
        if let Some(time_line) = stat_text.lines().find(|line| line.contains("seconds time elapsed")) {
            if let Some(seconds) = time_line
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
            {
                execution_time = Duration::from_secs_f64(seconds);
            }
        }
        
        // Parse user and system time from stat output
        if let Some(user_line) = stat_text.lines().find(|line| line.contains("seconds user")) {
            if let Some(seconds) = user_line
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
            {
                user_time = Duration::from_secs_f64(seconds);
            }
        }
        
        if let Some(sys_line) = stat_text.lines().find(|line| line.contains("seconds sys")) {
            if let Some(seconds) = sys_line
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
            {
                system_time = Duration::from_secs_f64(seconds);
            }
        }
        
        // Parse CPU events from stat output
        for line in stat_text.lines() {
            // Lines with events typically start with a number
            if let Some(first_char) = line.chars().next() {
                if first_char.is_digit(10) {
                    // Parse event data
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let count_str = parts[0].replace(",", "");
                        if let Ok(count) = count_str.parse::<u64>() {
                            let event_name = if parts.len() >= 3 {
                                parts[1].to_string()
                            } else {
                                "unknown".to_string()
                            };
                            cpu_events.insert(event_name, count);
                        }
                    }
                }
            }
        }
        
        // Parse hotspots from report text
        for line in report_text.lines() {
            if line.contains("%") && !line.starts_with("#") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Ok(percentage) = parts[0].trim_end_matches('%').parse::<f64>() {
                        if percentage >= 1.0 {  // Only include functions with at least 1% CPU time
                            let function_name = if parts.len() >= 4 {
                                parts[3].to_string()
                            } else {
                                parts[2].to_string()
                            };
                            
                            hotspots.push(HotspotFunction {
                                name: function_name.clone(),
                                cpu_percentage: percentage,
                                call_count: 0,  // We don't have this info from perf report
                            });
                            
                            function_times.insert(function_name, percentage);
                        }
                    }
                }
            }
        }
        
        Ok(PerformanceDetails {
            cpu_usage,
            execution_time,
            user_time,
            system_time,
            function_times,
            hotspots,
            cpu_events,
        })
    }

    /// Identify potential performance issues from profiling data
    fn identify_performance_issues(&self, perf_details: &PerformanceDetails) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        // Check for CPU usage issues
        if perf_details.cpu_usage > 90.0 {
            issues.push(AnalysisIssue {
                title: "High CPU Usage".to_string(),
                description: format!("CPU usage is very high at {:.1}%", perf_details.cpu_usage),
                severity: IssueSeverity::Medium,
                location: None,
                suggestion: Some("Consider optimizing the most CPU-intensive functions".to_string()),
                analysis_type: AnalysisType::Performance,
                locations: Vec::new(),
            });
        }
        
        // Check for cache issues
        if let Some(cache_misses) = perf_details.cpu_events.get("cache-misses") {
            if let Some(cache_refs) = perf_details.cpu_events.get("cache-references") {
                if *cache_refs > 0 {
                    let miss_rate = (*cache_misses as f64 / *cache_refs as f64) * 100.0;
                    if miss_rate > 20.0 {
                        issues.push(AnalysisIssue {
                            title: "High Cache Miss Rate".to_string(),
                            description: format!("Cache miss rate is {:.1}%", miss_rate),
                            severity: IssueSeverity::Medium,
                            location: None,
                            suggestion: Some("Consider improving data locality and access patterns".to_string()),
                            analysis_type: AnalysisType::Performance,
                            locations: Vec::new(),
                        });
                    }
                }
            }
        }
        
        // Check for branch misprediction issues
        if let Some(branch_misses) = perf_details.cpu_events.get("branch-misses") {
            if let Some(branch_insts) = perf_details.cpu_events.get("branch-instructions") {
                if *branch_insts > 0 {
                    let miss_rate = (*branch_misses as f64 / *branch_insts as f64) * 100.0;
                    if miss_rate > 10.0 {
                        issues.push(AnalysisIssue {
                            title: "High Branch Misprediction Rate".to_string(),
                            description: format!("Branch misprediction rate is {:.1}%", miss_rate),
                            severity: IssueSeverity::Medium,
                            location: None,
                            suggestion: Some("Consider simplifying conditional logic or implementing branch prediction hints".to_string()),
                            analysis_type: AnalysisType::Performance,
                            locations: Vec::new(),
                        });
                    }
                }
            }
        }
        
        // Check for instruction per cycle (IPC) issues
        if let (Some(instructions), Some(cycles)) = (
            perf_details.cpu_events.get("instructions"),
            perf_details.cpu_events.get("cycles")
        ) {
            if *cycles > 0 {
                let ipc = *instructions as f64 / *cycles as f64;
                if ipc < 0.7 {
                    issues.push(AnalysisIssue {
                        title: "Low Instructions Per Cycle".to_string(),
                        description: format!("IPC is low at {:.2}", ipc),
                        severity: IssueSeverity::Medium,
                        location: None,
                        suggestion: Some("Consider reducing memory stalls, improving parallelism, or reducing dependencies between instructions".to_string()),
                        analysis_type: AnalysisType::Performance,
                        locations: Vec::new(),
                    });
                }
            }
        }
        
        issues
    }
}

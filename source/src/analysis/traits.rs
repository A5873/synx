//! Analysis traits and types module for Synx
//!
//! This module provides the core traits and types used throughout the analysis system,
//! including the Analyzer trait, configuration types, and result types.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use anyhow::Result;

/// Severity level for analysis issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Low severity issues are informational and might not require immediate attention.
    Low,
    /// Medium severity issues should be addressed but don't critically impact operation.
    Medium,
    /// High severity issues require immediate attention and may significantly impact operation.
    High,
    /// Critical issues that must be fixed immediately to avoid system failure.
    Critical,
}

/// Depth level for analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisLevel {
    /// Basic analysis with minimal checks.
    Basic,
    /// Standard level analysis with common checks.
    Standard,
    /// Comprehensive analysis with in-depth checks.
    Comprehensive,
    /// Intensive analysis that checks everything possible (may be resource intensive).
    Intensive,
}

/// Type of analysis being performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisType {
    /// Memory usage and leak detection analysis.
    Memory,
    /// Performance and optimization analysis.
    Performance,
}

/// Configuration options for analysis operations.
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    /// Whether to display verbose output during analysis.
    pub verbose: bool,
    /// Optional directory to save analysis output files.
    pub output_dir: Option<String>,
    /// The depth level of analysis to perform.
    pub analysis_level: AnalysisLevel,
    /// Whether to save analysis result files.
    pub save_results: bool,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            output_dir: None,
            analysis_level: AnalysisLevel::Standard,
            save_results: false,
        }
    }
}

/// Represents a memory leak identified during analysis.
#[derive(Debug, Clone)]
pub struct MemoryLeak {
    /// Size of the leak in bytes.
    pub size: u64,
    /// Location where the allocation occurred.
    pub allocation_location: String,
    /// Stack trace at allocation time, if available.
    pub stack_trace: Vec<String>,
}

/// Represents a function identified as a performance hotspot.
#[derive(Debug, Clone)]
pub struct HotspotFunction {
    /// Name of the function.
    pub name: String,
    /// CPU time percentage.
    pub cpu_percentage: f64,
    /// Number of calls.
    pub call_count: u64,
}

/// Details specific to memory analysis.
#[derive(Debug, Clone)]
pub struct MemoryDetails {
    /// Peak memory usage in bytes.
    pub peak_memory: u64,
    /// Total number of allocations.
    pub total_allocations: u64,
    /// Total number of deallocations.
    pub total_deallocations: u64,
    /// Identified memory leaks.
    pub leaks: Vec<MemoryLeak>,
    /// Memory hotspots (function name, bytes allocated).
    pub allocation_hotspots: Vec<(String, u64)>,
    /// Memory usage over time (duration since start, bytes).
    pub heap_usage_timeline: Vec<(Duration, u64)>,
}

/// Details specific to performance analysis.
#[derive(Debug, Clone)]
pub struct PerformanceDetails {
    /// CPU utilization percentage.
    pub cpu_usage: f64,
    /// Total execution time.
    pub execution_time: Duration,
    /// Time spent in user mode.
    pub user_time: Duration,
    /// Time spent in system mode.
    pub system_time: Duration,
    /// Function execution times (function name, percentage).
    pub function_times: HashMap<String, f64>,
    /// Performance hotspots identified.
    pub hotspots: Vec<HotspotFunction>,
    /// Various CPU events (event name, count).
    pub cpu_events: HashMap<String, u64>,
}

/// Detailed information about an analysis finding.
#[derive(Debug, Clone)]
pub enum AnalysisDetails {
    /// Memory-related analysis details.
    Memory(MemoryDetails),
    /// Performance-related analysis details.
    Performance(PerformanceDetails),
}

/// Represents a specific issue identified during analysis.
#[derive(Debug, Clone)]
pub struct AnalysisIssue {
    /// Severity level of the issue.
    pub severity: IssueSeverity,
    /// Description of the issue.
    pub description: String,
    /// Location of the issue (file path, line number, etc.).
    pub location: Option<String>,
    /// Suggested fix or remediation steps, if available.
    pub suggestion: Option<String>,
    /// Title of the issue.
    pub title: String,
    /// Type of analysis that identified this issue.
    pub analysis_type: AnalysisType,
    /// Locations affected by this issue.
    pub locations: Vec<String>,
}

/// Results from an analysis operation.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Name of the analyzer that produced these results.
    pub analyzer_name: String,
    /// Whether the analysis completed successfully.
    pub success: bool,
    /// Duration of the analysis operation.
    pub duration: Duration,
    /// Summary of the analysis results.
    pub summary: String,
    /// Detailed analysis information.
    pub details: AnalysisDetails,
    /// Paths to any report files generated during analysis.
    pub report_files: Vec<String>,
    /// List of issues found during analysis.
    pub issues: Vec<AnalysisIssue>,
}

/// Trait for types that can analyze code or binaries.
pub trait Analyzer {
    /// Returns the name of this analyzer.
    fn name(&self) -> &str;
    
    /// Checks if the analyzer is available on the current system.
    fn is_available(&self) -> bool;
    
    /// Analyzes the file at the given path with the specified options.
    fn analyze(&self, file_path: &Path, options: &AnalysisOptions) -> Result<AnalysisResult>;
    
    /// Returns a list of required dependencies for this analyzer.
    fn required_dependencies(&self) -> Vec<&str>;
    
    /// Returns instructions for installing the analyzer's dependencies.
    fn installation_instructions(&self) -> String;
}


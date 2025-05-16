//! Analysis module for Synx providing performance and dynamic analysis capabilities
//! 
//! This module provides a framework for analyzing code performance, memory usage,
//! and runtime behavior across different programming languages. It supports:
//! 
//! 1. **Performance Analysis**:
//!    - CPU profiling
//!    - Memory allocation tracking
//!    - Cache behavior analysis
//!    - System call profiling
//!    - I/O operation monitoring
//! 
//! 2. **Dynamic Analysis**:
//!    - Runtime behavior monitoring
//!    - Call graph generation
//!    - Data flow tracking
//!    - Resource usage analysis
//! 
//! The implementation is designed to be language-agnostic with specific
//! adapters for each supported language.

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

// Re-export sub-modules
pub mod perf;
pub mod memory;
pub mod cache;
pub mod syscall;
pub mod io;
pub mod runtime;
pub mod callgraph;
pub mod dataflow;
pub mod resource;

/// Main trait for analysis capabilities
pub trait Analyzer {
    /// Returns the name of the analyzer
    fn name(&self) -> &str;
    
    /// Checks if the analyzer is available on the current system
    fn is_available(&self) -> bool;
    
    /// Runs the analysis on a specific file
    fn analyze(&self, file_path: &Path, options: &AnalysisOptions) -> Result<AnalysisResult>;
    
    /// Returns the required dependencies for this analyzer
    fn required_dependencies(&self) -> Vec<&str>;
    
    /// Returns installation instructions for this analyzer
    fn installation_instructions(&self) -> String;
}

/// Analysis options for configuring analysis runs
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    /// Whether to produce verbose output
    pub verbose: bool,
    
    /// The depth of analysis to perform
    pub analysis_level: AnalysisLevel,
    
    /// The type of analysis to perform
    pub analysis_type: AnalysisType,
    
    /// Maximum time to run the analysis (in seconds)
    pub timeout: u64,
    
    /// Whether to save the analysis results to a file
    pub save_results: bool,
    
    /// Output directory for analysis results
    pub output_dir: Option<String>,
    
    /// Additional tool-specific options
    pub tool_options: HashMap<String, String>,
}

/// The depth/intensity of analysis to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisLevel {
    /// Basic analysis with minimal overhead
    Basic,
    
    /// Standard analysis with moderate overhead
    Standard,
    
    /// Comprehensive analysis with potentially high overhead
    Comprehensive,
    
    /// Intensive analysis that may significantly slow down execution
    Intensive,
}

/// The type of analysis to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisType {
    /// Performance analysis (CPU, memory, etc.)
    Performance,
    
    /// Memory analysis (leaks, usage patterns, etc.)
    Memory,
    
    /// Thread safety analysis
    Threading,
    
    /// I/O and system call analysis
    SystemInteraction,
    
    /// Data flow analysis
    DataFlow,
    
    /// Comprehensive analysis (all types)
    Comprehensive,
}

/// Result from an analysis run
#[derive(Debug)]
pub struct AnalysisResult {
    /// Name of the analyzer that produced this result
    pub analyzer_name: String,
    
    /// Whether the analysis was successful
    pub success: bool,
    
    /// Duration of the analysis
    pub duration: Duration,
    
    /// Summary of the analysis results
    pub summary: String,
    
    /// Detailed analysis results
    pub details: AnalysisDetails,
    
    /// Path to any generated report files
    pub report_files: Vec<String>,
    
    /// Any issues found during analysis
    pub issues: Vec<AnalysisIssue>,
}

/// Detailed analysis results
#[derive(Debug)]
pub enum AnalysisDetails {
    /// Performance analysis results
    Performance(PerformanceDetails),
    
    /// Memory analysis results
    Memory(MemoryDetails),
    
    /// Thread analysis results
    Threading(ThreadingDetails),
    
    /// I/O and system call analysis results
    SystemInteraction(SystemInteractionDetails),
    
    /// Data flow analysis results
    DataFlow(DataFlowDetails),
    
    /// Simple text output
    Text(String),
    
    /// No details available
    None,
}

/// Performance analysis details
#[derive(Debug)]
pub struct PerformanceDetails {
    /// CPU usage percentage
    pub cpu_usage: f64,
    
    /// Total execution time
    pub execution_time: Duration,
    
    /// CPU time spent in user mode
    pub user_time: Duration,
    
    /// CPU time spent in system mode
    pub system_time: Duration,
    
    /// CPU time breakdown by function
    pub function_times: HashMap<String, Duration>,
    
    /// Hotspot functions
    pub hotspots: Vec<(String, f64)>,
    
    /// CPU events (instructions, cache misses, etc.)
    pub cpu_events: HashMap<String, u64>,
}

/// Memory analysis details
#[derive(Debug)]
pub struct MemoryDetails {
    /// Peak memory usage
    pub peak_memory: u64,
    
    /// Total memory allocations
    pub total_allocations: u64,
    
    /// Total deallocations
    pub total_deallocations: u64,
    
    /// Memory leaks detected
    pub leaks: Vec<MemoryLeak>,
    
    /// Allocation hotspots
    pub allocation_hotspots: Vec<(String, u64)>,
    
    /// Heap usage over time
    pub heap_usage_timeline: Vec<(Duration, u64)>,
}

/// Thread analysis details
#[derive(Debug)]
pub struct ThreadingDetails {
    /// Number of threads created
    pub thread_count: u32,
    
    /// Data races detected
    pub data_races: Vec<DataRace>,
    
    /// Deadlocks detected
    pub deadlocks: Vec<Deadlock>,
    
    /// Thread contention points
    pub contention_points: Vec<ContentionPoint>,
    
    /// Thread synchronization operations
    pub sync_operations: u64,
}

/// System interaction analysis details
#[derive(Debug)]
pub struct SystemInteractionDetails {
    /// System calls made
    pub syscalls: HashMap<String, u64>,
    
    /// File operations
    pub file_operations: Vec<FileOperation>,
    
    /// Network operations
    pub network_operations: Vec<NetworkOperation>,
    
    /// Total bytes read
    pub bytes_read: u64,
    
    /// Total bytes written
    pub bytes_written: u64,
}

/// Data flow analysis details
#[derive(Debug)]
pub struct DataFlowDetails {
    /// Call graph edges
    pub call_graph: Vec<(String, String)>,
    
    /// Data dependencies
    pub data_dependencies: Vec<DataDependency>,
    
    /// Variable usages
    pub variable_usages: HashMap<String, u64>,
    
    /// Control flow paths
    pub control_flow_paths: Vec<Vec<String>>,
}

/// Memory leak information
#[derive(Debug)]
pub struct MemoryLeak {
    /// Size of the leak in bytes
    pub size: u64,
    
    /// Location where the allocation occurred
    pub allocation_location: String,
    
    /// Stack trace of the allocation
    pub stack_trace: Vec<String>,
}

/// Data race information
#[derive(Debug)]
pub struct DataRace {
    /// Variable involved in the race
    pub variable: String,
    
    /// First thread involved
    pub thread1: String,
    
    /// Second thread involved
    pub thread2: String,
    
    /// Location in the code
    pub location: String,
}

/// Deadlock information
#[derive(Debug)]
pub struct Deadlock {
    /// Threads involved in the deadlock
    pub threads: Vec<String>,
    
    /// Resources involved
    pub resources: Vec<String>,
    
    /// Lock acquisition sequence
    pub lock_sequence: Vec<String>,
}

/// Thread contention point
#[derive(Debug)]
pub struct ContentionPoint {
    /// Location in the code
    pub location: String,
    
    /// Resource being contended
    pub resource: String,
    
    /// Number of contentions
    pub contention_count: u64,
    
    /// Average wait time
    pub average_wait_time: Duration,
}

/// File operation information
#[derive(Debug)]
pub struct FileOperation {
    /// Path of the file
    pub path: String,
    
    /// Type of operation (read, write, etc.)
    pub operation_type: String,
    
    /// Number of bytes
    pub bytes: u64,
    
    /// Duration of the operation
    pub duration: Duration,
}

/// Network operation information
#[derive(Debug)]
pub struct NetworkOperation {
    /// Remote address
    pub address: String,
    
    /// Type of operation (connect, send, receive, etc.)
    pub operation_type: String,
    
    /// Number of bytes
    pub bytes: u64,
    
    /// Duration of the operation
    pub duration: Duration,
}

/// Data dependency information
#[derive(Debug)]
pub struct DataDependency {
    /// Source variable
    pub source: String,
    
    /// Target variable
    pub target: String,
    
    /// Type of dependency
    pub dependency_type: String,
}

/// Issue found during analysis
#[derive(Debug)]
pub struct AnalysisIssue {
    /// Severity of the issue
    pub severity: IssueSeverity,
    
    /// Description of the issue
    pub description: String,
    
    /// Location in the code
    pub location: Option<String>,
    
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Severity of an analysis issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    /// Informational message
    Info,
    
    /// Low severity issue
    Low,
    
    /// Medium severity issue
    Medium,
    
    /// High severity issue
    High,
    
    /// Critical issue
    Critical,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            analysis_level: AnalysisLevel::Standard,
            analysis_type: AnalysisType::Comprehensive,
            timeout: 60,
            save_results: false,
            output_dir: None,
            tool_options: HashMap::new(),
        }
    }
}

/// Factory for creating appropriate analyzers based on file type
pub struct AnalyzerFactory;

impl AnalyzerFactory {
    /// Create a set of analyzers appropriate for the given file type and analysis type
    pub fn create_analyzers(file_type: &FileType, analysis_type: AnalysisType) -> Vec<Box<dyn Analyzer>> {
        let mut analyzers: Vec<Box<dyn Analyzer>> = Vec::new();
        
        // Add appropriate analyzers based on file type and analysis type
        match file_type {
            FileType::C | FileType::Cpp => {
                if matches!(analysis_type, AnalysisType::Performance | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(perf::PerfCpuProfiler {}));
                    analyzers.push(Box::new(cache::CacheGrind {}));
                }
                
                if matches!(analysis_type, AnalysisType::Memory | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(memory::Valgrind {}));
                    analyzers.push(Box::new(memory::AddressSanitizer {}));
                }
                
                if matches!(analysis_type, AnalysisType::Threading | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(runtime::HelgrindAnalyzer {}));
                    analyzers.push(Box::new(runtime::ThreadSanitizer {}));
                }
                
                if matches!(analysis_type, AnalysisType::SystemInteraction | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(syscall::SyscallTracer {}));
                    analyzers.push(Box::new(io::IoProfiler {}));
                }
                
                if matches!(analysis_type, AnalysisType::DataFlow | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(callgraph::CallGraphGenerator {}));
                    analyzers.push(Box::new(dataflow::CDataFlowAnalyzer {}));
                }
            },
            
            FileType::Rust => {
                if matches!(analysis_type, AnalysisType::Performance | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(perf::PerfCpuProfiler {}));
                    analyzers.push(Box::new(resource::RustProfiler {}));
                }
                
                if matches!(analysis_type, AnalysisType::Memory | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(memory::Dhat {}));
                    analyzers.push(Box::new(memory::RustMemoryLeakDetector {}));
                }
                
                if matches!(analysis_type, AnalysisType::Threading | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(runtime::RustThreadAnalyzer {}));
                }
                
                if matches!(analysis_type, AnalysisType::SystemInteraction | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(io::RustIoAnalyzer {}));
                }
                
                if matches!(analysis_type, AnalysisType::DataFlow | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(callgraph::RustCallGraphGenerator {}));
                    analyzers.push(Box::new(dataflow::RustDataFlowAnalyzer {}));
                }
            },
            
            FileType::Python => {
                if matches!(analysis_type, AnalysisType::Performance | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(perf::PythonProfiler {}));
                    analyzers.push(Box::new(resource::PyVizTracer {}));
                }
                
                if matches!(analysis_type, AnalysisType::Memory | AnalysisType::Comprehensive) {
                    analyzers.push(Box::new(memory::MemoryProfiler {}));
                    analyzers.push(Box::new(memory::Pympler {}));
                }
                
                if matches!(analysis_type, AnalysisType::Threading | AnalysisType


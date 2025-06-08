//! Analysis module for performance, memory, and code quality
//!
//! This module provides tools for code analysis, performance profiling,
//! and memory usage detection in a secure manner.

mod memory;
mod perf;
mod traits;

// Re-export the analyzer implementations from their modules
pub use memory::{
    Valgrind as MemoryAnalyzer,
    AddressSanitizer,
    RustMemoryLeakDetector,
    PythonMemoryAnalyzer
};

pub use perf::{
    PerfCpuProfiler as PerformanceAnalyzer
};

// Re-export all types from traits module
pub use traits::{
    Analyzer,
    AnalysisOptions,
    AnalysisResult,
    AnalysisDetails,
    AnalysisIssue,
    IssueSeverity,
    AnalysisLevel,
    AnalysisType,
    MemoryDetails,
    MemoryLeak,
    HotspotFunction,
    PerformanceDetails,
};

use std::path::Path;
use std::time::Duration;
use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Maximum memory usage to analyze (in MB)
    pub max_memory: u64,
    /// Maximum CPU time to analyze (in seconds)
    pub max_cpu_time: u64,
    /// Whether to include detailed analysis
    pub detailed: bool,
    /// Custom analysis rules
    pub custom_rules: Vec<AnalysisRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRule {
    /// Rule identifier
    pub id: String,
    /// Rule description
    pub description: String,
    /// Pattern to match
    pub pattern: String,
    /// Severity level
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

// Custom types for the analyzer
#[derive(Debug, Clone, Serialize)]
pub struct CodeLocation {
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32, 
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeFinding {
    /// Finding identifier
    pub id: String,
    /// Description
    pub description: String,
    /// Location in code
    pub location: CodeLocation,
    /// Severity level
    pub severity: Severity,
    /// Suggested fix
    pub suggestion: Option<String>,
}

// Additional performance metrics from the other implementation
#[derive(Debug, Clone, Serialize)]
pub struct PerfMetrics {
    /// Execution time
    pub execution_time: f64,
    /// CPU usage
    pub cpu_usage: f64,
    /// I/O operations
    pub io_ops: u64,
    /// Performance bottlenecks
    pub bottlenecks: Vec<Bottleneck>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bottleneck {
    /// Operation causing bottleneck
    pub operation: String,
    /// Impact on performance
    pub impact: f64,
    /// Location in code
    pub location: CodeLocation,
    /// Optimization suggestions
    pub suggestions: Vec<String>,
}

/// Main analyzer that coordinates memory and performance analysis
pub struct CodeAnalyzer {
    config: AnalysisConfig,
    memory_analyzer: MemoryAnalyzer,
    perf_analyzer: PerformanceAnalyzer,
    analysis_options: AnalysisOptions,
}

impl CodeAnalyzer {
    /// Create a new analyzer with the given configuration
    pub fn new(config: AnalysisConfig) -> Result<Self> {
        // Create the appropriate analysis options
        let analysis_options = AnalysisOptions {
            verbose: config.detailed,
            output_dir: None,
            analysis_level: if config.detailed { 
                AnalysisLevel::Comprehensive 
            } else { 
                AnalysisLevel::Standard 
            },
            save_results: false,
        };
        
        Ok(Self {
            memory_analyzer: MemoryAnalyzer::new(),
            perf_analyzer: PerformanceAnalyzer::new(),
            config,
            analysis_options,
        })
        })
    }

    /// Analyze a file
    pub fn analyze_file(&self, path: &Path) -> Result<traits::AnalysisResult> {
        // Analyze memory usage
        let memory_result = self.memory_analyzer.analyze(path, &self.analysis_options)?;
        
        // Analyze performance separately (incorporate from other implementation)
        let perf_result = self.perf_analyzer.analyze(path, &self.analysis_options)?;
        
        // Apply custom analysis rules
        let findings = self.apply_rules(path)?;
        
        // Create a combined analysis result
        let code_findings = findings.iter().map(|f| {
            AnalysisIssue {
                title: f.id.clone(),
                description: f.description.clone(),
                severity: match f.severity {
                    Severity::Low => IssueSeverity::Low,
                    Severity::Medium => IssueSeverity::Medium,
                    Severity::High => IssueSeverity::High,
                    Severity::Critical => IssueSeverity::Critical,
                },
                location: Some(f.location.file.clone()),
                suggestion: f.suggestion.clone(),
                analysis_type: AnalysisType::Performance, // Default to performance for custom rules
                locations: vec![f.location.file.clone()],
            }
        }).collect::<Vec<_>>();
        
        // Merge our findings with the existing ones
        let mut all_issues = memory_result.issues;
        all_issues.extend(code_findings);
        
        // Add performance issues
        all_issues.extend(perf_result.issues);
        
        // Return a new analysis result with our combined data
        Ok(traits::AnalysisResult {
            analyzer_name: "CodeAnalyzer".to_string(),
            success: memory_result.success && perf_result.success,
            duration: memory_result.duration + perf_result.duration,
            summary: format!("Code analysis completed with {} findings", findings.len()),
            details: memory_result.details,
            report_files: memory_result.report_files,
            issues: all_issues,
        })
    }

    /// Apply custom analysis rules
    fn apply_rules(&self, path: &Path) -> Result<Vec<CodeFinding>> {
        let mut findings = Vec::new();
        
        for rule in &self.config.custom_rules {
            if let Some(finding) = self.check_rule(path, rule)? {
                findings.push(finding);
            }
        }
        
        Ok(findings)
    }

    /// Check a single analysis rule
    fn check_rule(&self, path: &Path, rule: &AnalysisRule) -> Result<Option<CodeFinding>> {
        // Read file contents
        let contents = std::fs::read_to_string(path)?;
        
        // Check if pattern matches
        if let Some(location) = find_pattern(&contents, &rule.pattern) {
            return Ok(Some(CodeFinding {
                id: rule.id.clone(),
                description: rule.description.clone(),
                location: CodeLocation {
                    file: path.to_string_lossy().into_owned(),
                    line: location.0,
                    column: location.1,
                },
                severity: rule.severity,
                suggestion: None,
            }));
        }
        
        Ok(None)
    }
}

/// Find a pattern in the code and return its location
fn find_pattern(contents: &str, pattern: &str) -> Option<(u32, u32)> {
    // Simple pattern matching for now
    // TODO: Implement more sophisticated pattern matching
    if let Some(pos) = contents.find(pattern) {
        let line = contents[..pos].lines().count() as u32;
        let column = contents[..pos].lines().last()?.len() as u32;
        Some((line, column))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_config() -> AnalysisConfig {
        AnalysisConfig {
            max_memory: 1024,
            max_cpu_time: 30,
            detailed: true,
            custom_rules: vec![
                AnalysisRule {
                    id: "TEST001".to_string(),
                    description: "Test rule".to_string(),
                    pattern: "TODO".to_string(),
                    severity: Severity::Low,
                },
            ],
        }
    }

    #[test]
    fn test_analyzer_creation() {
        let config = create_test_config();
        assert!(CodeAnalyzer::new(config).is_ok());
    }

    #[test]
    fn test_file_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.py");
        
        fs::write(&test_file, "def test():\n    # TODO: implement\n    pass\n").unwrap();
        
        let analyzer = CodeAnalyzer::new(create_test_config()).unwrap();
        let result = analyzer.analyze_file(&test_file).unwrap();
        
        // Check that we have at least one issue
        assert!(!result.issues.is_empty());
        // Check that at least one issue has the expected properties 
        assert!(result.issues.iter().any(|issue| issue.title == "TEST001" && 
                                        matches!(issue.severity, IssueSeverity::Low)));
    }

    #[test]
    fn test_pattern_finding() {
        let content = "line 1\nline 2\nTODO: fix this\nline 4\n";
        let pattern = "TODO";
        
        let location = find_pattern(content, pattern).unwrap();
        assert_eq!(location.0, 3); // Line number
        assert_eq!(location.1, 0); // Column number
    }
}

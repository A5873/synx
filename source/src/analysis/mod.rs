//! Analysis module for code and performance analysis
//! 
//! This module provides tools for analyzing code quality, performance,
//! and memory usage in a secure manner.

mod memory;
mod perf;
mod traits;

pub use memory::MemoryAnalyzer;
pub use perf::PerformanceAnalyzer;

// Re-export types from traits module
pub use traits::{
    Analyzer as AnalyzerTrait,
    AnalysisOptions,
    AnalysisResult as TraitsAnalysisResult,
    AnalysisDetails,
    AnalysisIssue as TraitsAnalysisIssue,
    IssueSeverity,
    AnalysisLevel,
    AnalysisType,
    MemoryDetails,
    PerformanceDetails,
    MemoryLeak as TraitsMemoryLeak,
    HotspotFunction,
};

use std::path::Path;
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

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisResult {
    /// File being analyzed
    pub file: String,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Performance metrics
    pub perf_metrics: PerfMetrics,
    /// Analysis findings
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    /// Peak memory usage
    pub peak_memory: u64,
    /// Average memory usage
    pub avg_memory: u64,
    /// Memory allocations
    pub allocations: u64,
    /// Memory leaks detected
    pub leaks: Vec<MemoryLeak>,
}

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
pub struct Finding {
    /// Finding identifier
    pub id: String,
    /// Description
    pub description: String,
    /// Location in code
    pub location: Location,
    /// Severity level
    pub severity: Severity,
    /// Suggested fix
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Location {
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryLeak {
    /// Allocation size
    pub size: u64,
    /// Allocation location
    pub location: Location,
    /// Leak context
    pub context: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bottleneck {
    /// Operation causing bottleneck
    pub operation: String,
    /// Impact on performance
    pub impact: f64,
    /// Location in code
    pub location: Location,
    /// Optimization suggestions
    pub suggestions: Vec<String>,
}

/// Main analyzer that coordinates memory and performance analysis
pub struct Analyzer {
    config: AnalysisConfig,
    memory_analyzer: MemoryAnalyzer,
    perf_analyzer: PerformanceAnalyzer,
}

impl Analyzer {
    /// Create a new analyzer with the given configuration
    pub fn new(config: AnalysisConfig) -> Result<Self> {
        Ok(Self {
            memory_analyzer: MemoryAnalyzer::new(&config)?,
            perf_analyzer: PerformanceAnalyzer::new(&config)?,
            config,
        })
    }

    /// Analyze a file
    pub fn analyze_file(&self, path: &Path) -> Result<AnalysisResult> {
        // Analyze memory usage
        let memory_stats = self.memory_analyzer.analyze(path)?;
        
        // Analyze performance
        let perf_metrics = self.perf_analyzer.analyze(path)?;
        
        // Apply custom analysis rules
        let findings = self.apply_rules(path)?;
        
        Ok(AnalysisResult {
            file: path.to_string_lossy().into_owned(),
            memory_stats,
            perf_metrics,
            findings,
        })
    }

    /// Apply custom analysis rules
    fn apply_rules(&self, path: &Path) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        
        for rule in &self.config.custom_rules {
            if let Some(finding) = self.check_rule(path, rule)? {
                findings.push(finding);
            }
        }
        
        Ok(findings)
    }

    /// Check a single analysis rule
    fn check_rule(&self, path: &Path, rule: &AnalysisRule) -> Result<Option<Finding>> {
        // Read file contents
        let contents = std::fs::read_to_string(path)?;
        
        // Check if pattern matches
        if let Some(location) = find_pattern(&contents, &rule.pattern) {
            return Ok(Some(Finding {
                id: rule.id.clone(),
                description: rule.description.clone(),
                location: Location {
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
        assert!(Analyzer::new(config).is_ok());
    }

    #[test]
    fn test_file_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.py");
        
        fs::write(&test_file, "def test():\n    # TODO: implement\n    pass\n").unwrap();
        
        let analyzer = Analyzer::new(create_test_config()).unwrap();
        let result = analyzer.analyze_file(&test_file).unwrap();
        
        assert!(!result.findings.is_empty());
        assert_eq!(result.findings[0].severity, Severity::Low);
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

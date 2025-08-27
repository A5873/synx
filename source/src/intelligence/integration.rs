use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::{
    metrics::{MetricsAnalyzer, CodeMetrics},
    quality::{QualityAssessment, QualityAssessor},
    suggestions::{SuggestionEngine, Suggestion},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceReport {
    pub file_path: String,
    pub language: String,
    pub metrics: CodeMetrics,
    pub quality_assessment: QualityAssessment,
    pub suggestions: Vec<Suggestion>,
    pub overall_score: f64,
    pub analysis_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectReport {
    pub project_path: String,
    pub total_files: usize,
    pub files_analyzed: usize,
    pub file_reports: Vec<IntelligenceReport>,
    pub project_summary: ProjectSummary,
    pub analysis_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub overall_quality_score: f64,
    pub total_suggestions: usize,
    pub critical_issues: usize,
    pub high_complexity_files: usize,
    pub language_breakdown: HashMap<String, usize>,
    pub top_issues: Vec<String>,
    pub quality_distribution: QualityDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDistribution {
    pub excellent: usize,  // A grade
    pub good: usize,       // B grade
    pub fair: usize,       // C grade
    pub needs_work: usize, // D grade
    pub poor: usize,       // F grade
}

pub struct IntelligenceEngine {
    metrics_analyzer: MetricsAnalyzer,
    quality_assessor: QualityAssessor,
    suggestion_engine: SuggestionEngine,
}

impl IntelligenceEngine {
    pub fn new() -> Self {
        Self {
            metrics_analyzer: MetricsAnalyzer::new(),
            quality_assessor: QualityAssessor::new(),
            suggestion_engine: SuggestionEngine::new(),
        }
    }

    /// Analyze a single file and generate a complete intelligence report
    pub fn analyze_file<P: AsRef<Path>>(&self, file_path: P) -> Result<IntelligenceReport, Box<dyn std::error::Error>> {
        let file_path = file_path.as_ref();
        let content = std::fs::read_to_string(file_path)?;
        
        // Detect language from file extension
        let language = self.detect_language(file_path);
        
        // Run metrics analysis
        let metrics = self.metrics_analyzer.analyze(&content, &language)?;
        
        // Run quality assessment
        let quality_assessment = self.quality_assessor.assess(&metrics, &content, &language)?;
        
        // Generate suggestions
        let suggestions = self.suggestion_engine.analyze(&content, &language, &metrics)?;
        
        // Calculate overall score (weighted average)
        let overall_score = self.calculate_overall_score(&quality_assessment, &suggestions);
        
        Ok(IntelligenceReport {
            file_path: file_path.to_string_lossy().to_string(),
            language,
            metrics,
            quality_assessment,
            suggestions,
            overall_score,
            analysis_timestamp: chrono::Utc::now(),
        })
    }

    /// Analyze an entire project directory
    pub fn analyze_project<P: AsRef<Path>>(&self, project_path: P, exclude_patterns: &[String]) -> Result<ProjectReport, Box<dyn std::error::Error>> {
        let project_path = project_path.as_ref();
        let mut file_reports = Vec::new();
        let mut total_files = 0;
        
        // Walk through all files in the project
        for entry in walkdir::WalkDir::new(project_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                total_files += 1;
                
                let file_path = entry.path();
                
                // Skip files matching exclude patterns
                if self.should_exclude_file(file_path, exclude_patterns) {
                    continue;
                }
                
                // Only analyze code files
                if !self.is_code_file(file_path) {
                    continue;
                }
                
                match self.analyze_file(file_path) {
                    Ok(report) => file_reports.push(report),
                    Err(e) => {
                        eprintln!("Warning: Failed to analyze {}: {}", file_path.display(), e);
                    }
                }
            }
        }
        
        let files_analyzed = file_reports.len();
        let project_summary = self.generate_project_summary(&file_reports);
        
        Ok(ProjectReport {
            project_path: project_path.to_string_lossy().to_string(),
            total_files,
            files_analyzed,
            file_reports,
            project_summary,
            analysis_timestamp: chrono::Utc::now(),
        })
    }

    /// Generate a summary report from file reports
    fn generate_project_summary(&self, file_reports: &[IntelligenceReport]) -> ProjectSummary {
        if file_reports.is_empty() {
            return ProjectSummary {
                overall_quality_score: 0.0,
                total_suggestions: 0,
                critical_issues: 0,
                high_complexity_files: 0,
                language_breakdown: HashMap::new(),
                top_issues: Vec::new(),
                quality_distribution: QualityDistribution {
                    excellent: 0,
                    good: 0,
                    fair: 0,
                    needs_work: 0,
                    poor: 0,
                },
            };
        }

        // Calculate overall quality score
        let overall_quality_score = file_reports.iter()
            .map(|r| r.overall_score)
            .sum::<f64>() / file_reports.len() as f64;

        // Count total suggestions
        let total_suggestions = file_reports.iter()
            .map(|r| r.suggestions.len())
            .sum();

        // Count critical issues (high impact suggestions)
        let critical_issues = file_reports.iter()
            .map(|r| r.suggestions.iter().filter(|s| s.impact >= 8).count())
            .sum();

        // Count high complexity files
        let high_complexity_files = file_reports.iter()
            .filter(|r| r.metrics.cyclomatic_complexity > 10.0 || r.metrics.cognitive_complexity > 15.0)
            .count();

        // Language breakdown
        let mut language_breakdown = HashMap::new();
        for report in file_reports {
            *language_breakdown.entry(report.language.clone()).or_insert(0) += 1;
        }

        // Top issues (most common suggestion categories)
        let mut issue_counts: HashMap<String, usize> = HashMap::new();
        for report in file_reports {
            for suggestion in &report.suggestions {
                *issue_counts.entry(suggestion.category.clone()).or_insert(0) += 1;
            }
        }
        
        let mut top_issues: Vec<(String, usize)> = issue_counts.into_iter().collect();
        top_issues.sort_by(|a, b| b.1.cmp(&a.1));
        let top_issues = top_issues.into_iter()
            .take(5)
            .map(|(category, count)| format!("{} ({})", category, count))
            .collect();

        // Quality distribution
        let mut quality_distribution = QualityDistribution {
            excellent: 0,
            good: 0,
            fair: 0,
            needs_work: 0,
            poor: 0,
        };

        for report in file_reports {
            match report.quality_assessment.grade.as_str() {
                "A" => quality_distribution.excellent += 1,
                "B" => quality_distribution.good += 1,
                "C" => quality_distribution.fair += 1,
                "D" => quality_distribution.needs_work += 1,
                "F" => quality_distribution.poor += 1,
                _ => {}
            }
        }

        ProjectSummary {
            overall_quality_score,
            total_suggestions,
            critical_issues,
            high_complexity_files,
            language_breakdown,
            top_issues,
            quality_distribution,
        }
    }

    /// Calculate overall score from quality assessment and suggestions
    fn calculate_overall_score(&self, quality: &QualityAssessment, suggestions: &[Suggestion]) -> f64 {
        let base_score = quality.score;
        
        // Penalty for critical suggestions
        let critical_penalty = suggestions.iter()
            .filter(|s| s.impact >= 8)
            .count() as f64 * 5.0;
        
        // Penalty for high-impact suggestions
        let high_impact_penalty = suggestions.iter()
            .filter(|s| s.impact >= 6 && s.impact < 8)
            .count() as f64 * 2.0;
        
        let total_penalty = critical_penalty + high_impact_penalty;
        
        // Apply penalty but don't go below 0
        (base_score - total_penalty).max(0.0)
    }

    /// Detect programming language from file extension
    fn detect_language<P: AsRef<Path>>(&self, file_path: P) -> String {
        let path = file_path.as_ref();
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => "Rust".to_string(),
            Some("py") => "Python".to_string(),
            Some("js") => "JavaScript".to_string(),
            Some("ts") => "TypeScript".to_string(),
            Some("java") => "Java".to_string(),
            Some("cpp") | Some("cc") | Some("cxx") => "C++".to_string(),
            Some("c") => "C".to_string(),
            Some("go") => "Go".to_string(),
            Some("rb") => "Ruby".to_string(),
            Some("php") => "PHP".to_string(),
            Some("cs") => "C#".to_string(),
            Some("swift") => "Swift".to_string(),
            Some("kt") => "Kotlin".to_string(),
            Some("scala") => "Scala".to_string(),
            Some("sh") | Some("bash") => "Shell".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Check if a file should be excluded based on patterns
    fn should_exclude_file<P: AsRef<Path>>(&self, file_path: P, exclude_patterns: &[String]) -> bool {
        let path_str = file_path.as_ref().to_string_lossy();
        
        for pattern in exclude_patterns {
            if glob_match::glob_match(pattern, &path_str) {
                return true;
            }
        }
        
        // Default exclusions
        let default_exclusions = [
            "*/target/*", "*/node_modules/*", "*/.git/*", 
            "*/build/*", "*/dist/*", "*/__pycache__/*",
            "*.min.js", "*.min.css", "*.lock"
        ];
        
        for pattern in &default_exclusions {
            if glob_match::glob_match(pattern, &path_str) {
                return true;
            }
        }
        
        false
    }

    /// Check if a file is a code file we should analyze
    fn is_code_file<P: AsRef<Path>>(&self, file_path: P) -> bool {
        let path = file_path.as_ref();
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => matches!(
                ext,
                "rs" | "py" | "js" | "ts" | "java" | "cpp" | "cc" | "cxx" | 
                "c" | "go" | "rb" | "php" | "cs" | "swift" | "kt" | "scala" | 
                "sh" | "bash" | "jsx" | "tsx" | "vue" | "svelte"
            ),
            None => false,
        }
    }
}

impl Default for IntelligenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Format an intelligence report as human-readable text
pub fn format_intelligence_report(report: &IntelligenceReport, verbose: bool) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("📊 Intelligence Report: {}\n", report.file_path));
    output.push_str(&format!("Language: {} | Overall Score: {:.1}/100\n", report.language, report.overall_score));
    output.push_str(&format!("Quality Grade: {} ({:.1})\n\n", report.quality_assessment.grade, report.quality_assessment.score));
    
    // Metrics summary
    output.push_str("📈 Code Metrics:\n");
    output.push_str(&format!(
        "  Lines of Code: {} | Cyclomatic Complexity: {:.1} | Cognitive Complexity: {:.1}\n",
        report.metrics.lines_of_code,
        report.metrics.cyclomatic_complexity,
        report.metrics.cognitive_complexity
    ));
    
    if verbose {
        output.push_str(&format!(
            "  Functions: {} | Classes: {} | Comments: {:.1}%\n",
            report.metrics.function_count,
            report.metrics.class_count,
            report.metrics.comment_ratio * 100.0
        ));
    }
    
    // Quality highlights
    if !report.quality_assessment.strengths.is_empty() {
        output.push_str("\n✅ Strengths:\n");
        for strength in &report.quality_assessment.strengths {
            output.push_str(&format!("  • {}\n", strength));
        }
    }
    
    if !report.quality_assessment.weaknesses.is_empty() {
        output.push_str("\n⚠️  Areas for Improvement:\n");
        for weakness in &report.quality_assessment.weaknesses {
            output.push_str(&format!("  • {}\n", weakness));
        }
    }
    
    // Top suggestions
    if !report.suggestions.is_empty() {
        output.push_str("\n💡 Top Suggestions:\n");
        let mut suggestions = report.suggestions.clone();
        suggestions.sort_by(|a, b| b.impact.cmp(&a.impact));
        
        let limit = if verbose { 10 } else { 5 };
        for suggestion in suggestions.iter().take(limit) {
            let priority = if suggestion.impact >= 8 { "🔴 Critical" } 
                          else if suggestion.impact >= 6 { "🟡 High" }
                          else { "🟢 Medium" };
            
            output.push_str(&format!(
                "  {} [{}] {}\n",
                priority,
                suggestion.category,
                suggestion.description
            ));
            
            if verbose && !suggestion.code_example.is_empty() {
                output.push_str(&format!("    Example: {}\n", suggestion.code_example));
            }
        }
    }
    
    output.push('\n');
    output
}

/// Format a project report as human-readable text
pub fn format_project_report(report: &ProjectReport, verbose: bool) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("🏗️  Project Intelligence Report: {}\n", report.project_path));
    output.push_str(&format!("Generated: {}\n\n", report.analysis_timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Summary statistics
    output.push_str("📊 Project Summary:\n");
    output.push_str(&format!("  Files Analyzed: {} / {} total files\n", report.files_analyzed, report.total_files));
    output.push_str(&format!("  Overall Quality Score: {:.1}/100\n", report.project_summary.overall_quality_score));
    output.push_str(&format!("  Total Suggestions: {}\n", report.project_summary.total_suggestions));
    output.push_str(&format!("  Critical Issues: {}\n", report.project_summary.critical_issues));
    output.push_str(&format!("  High Complexity Files: {}\n\n", report.project_summary.high_complexity_files));
    
    // Quality distribution
    output.push_str("📈 Quality Distribution:\n");
    let dist = &report.project_summary.quality_distribution;
    output.push_str(&format!("  Excellent (A): {} files\n", dist.excellent));
    output.push_str(&format!("  Good (B): {} files\n", dist.good));
    output.push_str(&format!("  Fair (C): {} files\n", dist.fair));
    output.push_str(&format!("  Needs Work (D): {} files\n", dist.needs_work));
    output.push_str(&format!("  Poor (F): {} files\n\n", dist.poor));
    
    // Language breakdown
    if !report.project_summary.language_breakdown.is_empty() {
        output.push_str("🌐 Language Breakdown:\n");
        let mut langs: Vec<_> = report.project_summary.language_breakdown.iter().collect();
        langs.sort_by(|a, b| b.1.cmp(a.1));
        
        for (lang, count) in langs {
            output.push_str(&format!("  {}: {} files\n", lang, count));
        }
        output.push('\n');
    }
    
    // Top issues
    if !report.project_summary.top_issues.is_empty() {
        output.push_str("🔍 Most Common Issues:\n");
        for (i, issue) in report.project_summary.top_issues.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, issue));
        }
        output.push('\n');
    }
    
    // Individual file reports (if verbose)
    if verbose {
        output.push_str("📁 Individual File Reports:\n");
        output.push_str("=".repeat(50));
        output.push('\n');
        
        for file_report in &report.file_reports {
            output.push_str(&format_intelligence_report(file_report, false));
            output.push_str("-".repeat(30));
            output.push('\n');
        }
    }
    
    output
}

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

pub mod metrics;
pub mod patterns;
pub mod quality;
pub mod suggestions;
pub mod trends;
pub mod learning;
pub mod sentinel;

pub use metrics::CodeMetrics;
pub use patterns::ErrorPattern;
pub use quality::QualityScore;
pub use suggestions::SmartSuggestion;

/// Main intelligence engine that coordinates all ML-inspired features
pub struct IntelligenceEngine {
    pub metrics_analyzer: metrics::MetricsAnalyzer,
    pub pattern_analyzer: patterns::PatternAnalyzer,
    pub quality_assessor: quality::QualityAssessor,
    pub suggestion_engine: suggestions::SuggestionEngine,
    pub trend_tracker: trends::TrendTracker,
    pub learning_engine: learning::LearningEngine,
    pub sentinel_ai: sentinel::SentinelAI,
    
    // Data storage
    pub database: IntelligenceDatabase,
}

/// Central database for storing intelligence data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceDatabase {
    pub file_metrics: HashMap<PathBuf, FileIntelligence>,
    pub project_metrics: ProjectIntelligence,
    pub error_patterns: Vec<ErrorPattern>,
    pub historical_data: Vec<HistoricalSnapshot>,
    pub learning_data: HashMap<String, serde_json::Value>,
    pub last_updated: DateTime<Utc>,
}

/// Intelligence data for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct FileIntelligence {
    pub path: PathBuf,
    pub metrics: CodeMetrics,
    pub quality_score: QualityScore,
    pub error_history: Vec<ErrorRecord>,
    pub complexity_trends: Vec<ComplexityPoint>,
    pub risk_assessment: RiskAssessment,
    pub suggestions: Vec<SmartSuggestion>,
    pub last_analyzed: DateTime<Utc>,
}

/// Project-level intelligence data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ProjectIntelligence {
    pub total_files: usize,
    pub languages: HashMap<String, LanguageStats>,
    pub overall_quality: QualityScore,
    pub technical_debt: TechnicalDebtEstimate,
    pub complexity_distribution: ComplexityDistribution,
    pub error_frequency: HashMap<String, usize>,
    pub team_metrics: TeamMetrics,
    pub trends: TrendSummary,
}

/// Historical snapshot for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalSnapshot {
    pub timestamp: DateTime<Utc>,
    pub project_metrics: ProjectIntelligence,
    pub file_count: usize,
    pub quality_summary: QualitySummary,
}

/// Error record for pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ErrorRecord {
    pub error_type: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub severity: ErrorSeverity,
    pub timestamp: DateTime<Utc>,
    pub fixed: bool,
    pub fix_time: Option<DateTime<Utc>>,
}

/// Risk assessment for files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub complexity_risk: f64,
    pub error_proneness: f64,
    pub maintenance_difficulty: f64,
    pub change_frequency: f64,
    pub coupling_risk: f64,
    pub risk_factors: Vec<RiskFactor>,
}

/// Technical debt estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtEstimate {
    pub total_debt_hours: f64,
    pub debt_ratio: f64,
    pub debt_by_category: HashMap<String, f64>,
    pub high_debt_files: Vec<PathBuf>,
    pub remediation_suggestions: Vec<RemediationSuggestion>,
}

/// Language-specific statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub file_count: usize,
    pub total_lines: usize,
    pub avg_complexity: f64,
    pub error_rate: f64,
    pub quality_score: f64,
    pub common_patterns: Vec<String>,
}

/// Team productivity and quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMetrics {
    pub avg_code_quality: f64,
    pub consistency_score: f64,
    pub error_resolution_time: f64,
    pub productivity_trends: Vec<ProductivityPoint>,
    pub code_review_insights: CodeReviewInsights,
}

/// Various enums and supporting structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    Minimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: String,
    pub description: String,
    pub impact: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityPoint {
    pub timestamp: DateTime<Utc>,
    pub cyclomatic: usize,
    pub cognitive: usize,
    pub lines_of_code: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityDistribution {
    pub low: usize,
    pub medium: usize,
    pub high: usize,
    pub critical: usize,
    pub average: f64,
    pub percentiles: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySummary {
    pub overall_score: f64,
    pub maintainability: f64,
    pub reliability: f64,
    pub security: f64,
    pub performance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendSummary {
    pub quality_trend: TrendDirection,
    pub complexity_trend: TrendDirection,
    pub error_trend: TrendDirection,
    pub productivity_trend: TrendDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityPoint {
    pub timestamp: DateTime<Utc>,
    pub files_analyzed: usize,
    pub errors_found: usize,
    pub errors_fixed: usize,
    pub quality_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewInsights {
    pub avg_review_time: f64,
    pub common_review_comments: Vec<String>,
    pub quality_correlation: f64,
    pub reviewer_consistency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationSuggestion {
    pub file_path: PathBuf,
    pub issue_type: String,
    pub description: String,
    pub estimated_effort_hours: f64,
    pub priority: Priority,
    pub impact: Impact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Impact {
    High,
    Medium,
    Low,
}

impl IntelligenceEngine {
    /// Create a new intelligence engine
    pub fn new() -> Result<Self> {
        Ok(Self {
            metrics_analyzer: metrics::MetricsAnalyzer::new(),
            pattern_analyzer: patterns::PatternAnalyzer::new(),
            quality_assessor: quality::QualityAssessor::new(),
            suggestion_engine: suggestions::SuggestionEngine::new(),
            trend_tracker: trends::TrendTracker::new(),
            learning_engine: learning::LearningEngine::new(),
            sentinel_ai: sentinel::SentinelAI::new()?,
            database: IntelligenceDatabase::new(),
        })
    }
    
    /// Load existing intelligence data
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let mut engine = Self::new()?;
        if path.exists() {
            let data = fs::read_to_string(path)?;
            engine.database = serde_json::from_str(&data)?;
        }
        Ok(engine)
    }
    
    /// Save intelligence data to file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.database)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, data)?;
        Ok(())
    }
    
    /// Perform comprehensive analysis on a file
    pub fn analyze_file(&mut self, file_path: &Path) -> Result<FileIntelligence> {
        let content = fs::read_to_string(file_path)?;
        
        // Calculate metrics
        let metrics = self.metrics_analyzer.analyze_file(file_path, &content)?;
        
        // Assess quality
        let quality_score = self.quality_assessor.assess_file(file_path, &content, &metrics)?;
        
        // Analyze patterns
        self.pattern_analyzer.analyze_file(file_path, &content)?;
        
        // Generate suggestions
        let suggestions = self.suggestion_engine.generate_suggestions(file_path, &metrics, &quality_score)?;
        
        // Assess risk
        let risk_assessment = self.assess_risk(file_path, &metrics, &quality_score)?;
        
        // Get error history
        let error_history = self.database.file_metrics
            .get(file_path)
            .map(|fi| fi.error_history.clone())
            .unwrap_or_default();
        
        // Get complexity trends
        let complexity_trends = self.database.file_metrics
            .get(file_path)
            .map(|fi| fi.complexity_trends.clone())
            .unwrap_or_default();
        
        let file_intelligence = FileIntelligence {
            path: file_path.to_path_buf(),
            metrics,
            quality_score,
            error_history,
            complexity_trends,
            risk_assessment,
            suggestions,
            last_analyzed: Utc::now(),
        };
        
        // Store in database
        self.database.file_metrics.insert(file_path.to_path_buf(), file_intelligence.clone());
        self.database.last_updated = Utc::now();
        
        Ok(file_intelligence)
    }
    
    /// Perform project-wide analysis
    pub fn analyze_project(&mut self, project_path: &Path) -> Result<ProjectIntelligence> {
        let mut languages = HashMap::new();
        let mut total_files = 0;
        let mut overall_quality_sum = 0.0;
        let mut error_frequency = HashMap::new();
        
        // Walk through all files in the project
        for entry in walkdir::WalkDir::new(project_path) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && self.is_source_file(path) {
                total_files += 1;
                
                // Analyze file
                let file_intelligence = self.analyze_file(path)?;
                
                // Update language stats
                if let Some(lang) = self.detect_language(path) {
                    let stats = languages.entry(lang).or_insert(LanguageStats {
                        file_count: 0,
                        total_lines: 0,
                        avg_complexity: 0.0,
                        error_rate: 0.0,
                        quality_score: 0.0,
                        common_patterns: Vec::new(),
                    });
                    
                    stats.file_count += 1;
                    stats.total_lines += file_intelligence.metrics.lines_of_code;
                    stats.quality_score += file_intelligence.quality_score.overall;
                }
                
                overall_quality_sum += file_intelligence.quality_score.overall;
                
                // Update error frequency
                for error in &file_intelligence.error_history {
                    *error_frequency.entry(error.error_type.clone()).or_insert(0) += 1;
                }
            }
        }
        
        // Calculate averages
        for stats in languages.values_mut() {
            if stats.file_count > 0 {
                stats.quality_score /= stats.file_count as f64;
            }
        }
        
        let overall_quality = QualityScore {
            overall: if total_files > 0 { overall_quality_sum / total_files as f64 } else { 0.0 },
            maintainability: 0.0, // Will be calculated by quality assessor
            reliability: 0.0,
            security: 0.0,
            performance: 0.0,
            test_coverage: 0.0,
        };
        
        // Calculate technical debt
        let technical_debt = self.calculate_technical_debt(&self.database.file_metrics)?;
        
        // Calculate complexity distribution
        let complexity_distribution = self.calculate_complexity_distribution(&self.database.file_metrics);
        
        // Generate team metrics
        let team_metrics = self.calculate_team_metrics(&self.database.file_metrics);
        
        // Generate trend summary
        let trends = self.trend_tracker.analyze_trends(&self.database.historical_data);
        
        let project_intelligence = ProjectIntelligence {
            total_files,
            languages,
            overall_quality,
            technical_debt,
            complexity_distribution,
            error_frequency,
            team_metrics,
            trends,
        };
        
        self.database.project_metrics = project_intelligence.clone();
        
        Ok(project_intelligence)
    }
    
    /// Record validation error for pattern learning
    pub fn record_error(&mut self, file_path: &Path, error_type: String, message: String, line: Option<usize>, column: Option<usize>) {
        let error_record = ErrorRecord {
            error_type,
            message,
            line,
            column,
            severity: ErrorSeverity::High, // Default, could be calculated
            timestamp: Utc::now(),
            fixed: false,
            fix_time: None,
        };
        
        // Add to file's error history
        if let Some(file_intel) = self.database.file_metrics.get_mut(file_path) {
            file_intel.error_history.push(error_record.clone());
        }
        
        // Learn from the error pattern
        self.learning_engine.learn_from_error(&error_record);
        
        // Update pattern analyzer
        self.pattern_analyzer.record_error_pattern(file_path, &error_record);
    }
    
    /// Generate comprehensive intelligence report
    pub fn generate_report(&self) -> IntelligenceReport {
        IntelligenceReport {
            timestamp: Utc::now(),
            project_summary: self.database.project_metrics.clone(),
            top_risk_files: self.get_top_risk_files(10),
            quality_insights: self.generate_quality_insights(),
            performance_insights: self.generate_performance_insights(),
            security_insights: self.generate_security_insights(),
            recommendations: self.generate_recommendations(),
            trend_analysis: self.trend_tracker.get_current_trends(),
        }
    }
    
    /// Get statistics about the intelligence engine
    pub fn get_statistics(&self) -> EngineStatistics {
        EngineStatistics {
            supported_languages: vec![
                "Rust".to_string(), "Python".to_string(), "JavaScript".to_string(),
                "TypeScript".to_string(), "Java".to_string(), "C++".to_string(),
                "C".to_string(), "Go".to_string(), "C#".to_string()
            ],
            available_metrics: vec![
                "Cyclomatic Complexity".to_string(),
                "Cognitive Complexity".to_string(),
                "Lines of Code".to_string(),
                "Function Count".to_string(),
                "Class Count".to_string(),
                "Technical Debt".to_string(),
                "Maintainability Index".to_string(),
            ],
            quality_factors: vec![
                "Maintainability".to_string(),
                "Reliability".to_string(),
                "Security".to_string(),
                "Performance".to_string(),
                "Test Coverage".to_string(),
            ],
            suggestion_rules: self.suggestion_engine.get_rule_count(),
        }
    }
    
    // Helper methods
    fn assess_risk(&self, file_path: &Path, metrics: &CodeMetrics, quality: &QualityScore) -> Result<RiskAssessment> {
        let complexity_risk = (metrics.cyclomatic_complexity as f64 / 20.0).min(1.0);
        let error_proneness = 1.0 - quality.reliability;
        let maintenance_difficulty = 1.0 - quality.maintainability;
        
        // Calculate change frequency from history
        let change_frequency = self.database.file_metrics
            .get(file_path)
            .map(|fi| fi.complexity_trends.len() as f64 / 100.0)
            .unwrap_or(0.0)
            .min(1.0);
        
        let coupling_risk = (metrics.dependencies.len() as f64 / 50.0).min(1.0);
        
        let overall_risk_score = (complexity_risk + error_proneness + maintenance_difficulty + change_frequency + coupling_risk) / 5.0;
        
        let overall_risk = if overall_risk_score > 0.8 {
            RiskLevel::Critical
        } else if overall_risk_score > 0.6 {
            RiskLevel::High
        } else if overall_risk_score > 0.4 {
            RiskLevel::Medium
        } else if overall_risk_score > 0.2 {
            RiskLevel::Low
        } else {
            RiskLevel::Minimal
        };
        
        let mut risk_factors = Vec::new();
        
        if complexity_risk > 0.7 {
            risk_factors.push(RiskFactor {
                factor_type: "High Complexity".to_string(),
                description: "Code has high cyclomatic complexity".to_string(),
                impact: complexity_risk,
                confidence: 0.9,
            });
        }
        
        if error_proneness > 0.7 {
            risk_factors.push(RiskFactor {
                factor_type: "Error Prone".to_string(),
                description: "File has history of frequent errors".to_string(),
                impact: error_proneness,
                confidence: 0.8,
            });
        }
        
        Ok(RiskAssessment {
            overall_risk,
            complexity_risk,
            error_proneness,
            maintenance_difficulty,
            change_frequency,
            coupling_risk,
            risk_factors,
        })
    }
    
    fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(), 
                "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "cs" | "go" | 
                "php" | "rb" | "kt" | "swift" | "scala" | "sh" | "pl" | "r"
            )
        } else {
            false
        }
    }
    
    fn detect_language(&self, path: &Path) -> Option<String> {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| ext.to_lowercase())
    }
    
    fn calculate_technical_debt(&self, file_metrics: &HashMap<PathBuf, FileIntelligence>) -> Result<TechnicalDebtEstimate> {
        let mut total_debt_hours = 0.0;
        let mut debt_by_category = HashMap::new();
        let mut high_debt_files = Vec::new();
        
        for (path, intelligence) in file_metrics {
            let complexity_debt = (intelligence.metrics.cyclomatic_complexity as f64 - 10.0).max(0.0) * 0.5;
            let quality_debt = (1.0 - intelligence.quality_score.overall) * 8.0;
            let file_debt = complexity_debt + quality_debt;
            
            total_debt_hours += file_debt;
            
            if file_debt > 4.0 {
                high_debt_files.push(path.clone());
            }
            
            *debt_by_category.entry("Complexity".to_string()).or_insert(0.0) += complexity_debt;
            *debt_by_category.entry("Quality".to_string()).or_insert(0.0) += quality_debt;
        }
        
        let total_lines: usize = file_metrics.values()
            .map(|fi| fi.metrics.lines_of_code)
            .sum();
        
        let debt_ratio = if total_lines > 0 {
            total_debt_hours / (total_lines as f64 / 1000.0)
        } else {
            0.0
        };
        
        Ok(TechnicalDebtEstimate {
            total_debt_hours,
            debt_ratio,
            debt_by_category,
            high_debt_files,
            remediation_suggestions: Vec::new(), // Will be populated by suggestion engine
        })
    }
    
    fn calculate_complexity_distribution(&self, file_metrics: &HashMap<PathBuf, FileIntelligence>) -> ComplexityDistribution {
        let complexities: Vec<usize> = file_metrics.values()
            .map(|fi| fi.metrics.cyclomatic_complexity)
            .collect();
        
        let mut low = 0;
        let mut medium = 0;
        let mut high = 0;
        let mut critical = 0;
        
        for &complexity in &complexities {
            match complexity {
                0..=5 => low += 1,
                6..=10 => medium += 1,
                11..=20 => high += 1,
                _ => critical += 1,
            }
        }
        
        let average = if !complexities.is_empty() {
            complexities.iter().sum::<usize>() as f64 / complexities.len() as f64
        } else {
            0.0
        };
        
        let mut sorted_complexities = complexities.clone();
        sorted_complexities.sort();
        
        let mut percentiles = HashMap::new();
        if !sorted_complexities.is_empty() {
            let len = sorted_complexities.len();
            percentiles.insert("50th".to_string(), sorted_complexities[len / 2] as f64);
            percentiles.insert("75th".to_string(), sorted_complexities[len * 3 / 4] as f64);
            percentiles.insert("90th".to_string(), sorted_complexities[len * 9 / 10] as f64);
            percentiles.insert("95th".to_string(), sorted_complexities[len * 95 / 100] as f64);
        }
        
        ComplexityDistribution {
            low,
            medium,
            high,
            critical,
            average,
            percentiles,
        }
    }
    
    fn calculate_team_metrics(&self, file_metrics: &HashMap<PathBuf, FileIntelligence>) -> TeamMetrics {
        let quality_scores: Vec<f64> = file_metrics.values()
            .map(|fi| fi.quality_score.overall)
            .collect();
        
        let avg_code_quality = if !quality_scores.is_empty() {
            quality_scores.iter().sum::<f64>() / quality_scores.len() as f64
        } else {
            0.0
        };
        
        // Calculate consistency (standard deviation of quality scores)
        let variance = if quality_scores.len() > 1 {
            let mean = avg_code_quality;
            quality_scores.iter()
                .map(|score| (score - mean).powi(2))
                .sum::<f64>() / (quality_scores.len() - 1) as f64
        } else {
            0.0
        };
        
        let consistency_score = 1.0 - (variance.sqrt() / 100.0).min(1.0);
        
        TeamMetrics {
            avg_code_quality,
            consistency_score,
            error_resolution_time: 24.0, // Default - would be calculated from actual data
            productivity_trends: Vec::new(),
            code_review_insights: CodeReviewInsights {
                avg_review_time: 2.5,
                common_review_comments: Vec::new(),
                quality_correlation: 0.8,
                reviewer_consistency: 0.7,
            },
        }
    }
    
    fn get_top_risk_files(&self, limit: usize) -> Vec<(PathBuf, RiskAssessment)> {
        let mut files: Vec<_> = self.database.file_metrics
            .iter()
            .map(|(path, intelligence)| (path.clone(), intelligence.risk_assessment.clone()))
            .collect();
        
        files.sort_by(|a, b| {
            let risk_a = match a.1.overall_risk {
                RiskLevel::Critical => 5,
                RiskLevel::High => 4,
                RiskLevel::Medium => 3,
                RiskLevel::Low => 2,
                RiskLevel::Minimal => 1,
            };
            let risk_b = match b.1.overall_risk {
                RiskLevel::Critical => 5,
                RiskLevel::High => 4,
                RiskLevel::Medium => 3,
                RiskLevel::Low => 2,
                RiskLevel::Minimal => 1,
            };
            risk_b.cmp(&risk_a)
        });
        
        files.into_iter().take(limit).collect()
    }
    
    fn generate_quality_insights(&self) -> Vec<QualityInsight> {
        // This would analyze patterns and generate insights
        Vec::new()
    }
    
    fn generate_performance_insights(&self) -> Vec<PerformanceInsight> {
        // This would analyze performance patterns
        Vec::new()
    }
    
    fn generate_security_insights(&self) -> Vec<SecurityInsight> {
        // This would analyze security patterns
        Vec::new()
    }
    
    fn generate_recommendations(&self) -> Vec<Recommendation> {
        // This would generate actionable recommendations
        Vec::new()
    }
}

impl IntelligenceDatabase {
    pub fn new() -> Self {
        Self {
            file_metrics: HashMap::new(),
            project_metrics: ProjectIntelligence {
                total_files: 0,
                languages: HashMap::new(),
                overall_quality: QualityScore::default(),
                technical_debt: TechnicalDebtEstimate {
                    total_debt_hours: 0.0,
                    debt_ratio: 0.0,
                    debt_by_category: HashMap::new(),
                    high_debt_files: Vec::new(),
                    remediation_suggestions: Vec::new(),
                },
                complexity_distribution: ComplexityDistribution {
                    low: 0,
                    medium: 0,
                    high: 0,
                    critical: 0,
                    average: 0.0,
                    percentiles: HashMap::new(),
                },
                error_frequency: HashMap::new(),
                team_metrics: TeamMetrics {
                    avg_code_quality: 0.0,
                    consistency_score: 0.0,
                    error_resolution_time: 0.0,
                    productivity_trends: Vec::new(),
                    code_review_insights: CodeReviewInsights {
                        avg_review_time: 0.0,
                        common_review_comments: Vec::new(),
                        quality_correlation: 0.0,
                        reviewer_consistency: 0.0,
                    },
                },
                trends: TrendSummary {
                    quality_trend: TrendDirection::Stable,
                    complexity_trend: TrendDirection::Stable,
                    error_trend: TrendDirection::Stable,
                    productivity_trend: TrendDirection::Stable,
                },
            },
            error_patterns: Vec::new(),
            historical_data: Vec::new(),
            learning_data: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Comprehensive intelligence report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct IntelligenceReport {
    pub timestamp: DateTime<Utc>,
    pub project_summary: ProjectIntelligence,
    pub top_risk_files: Vec<(PathBuf, RiskAssessment)>,
    pub quality_insights: Vec<QualityInsight>,
    pub performance_insights: Vec<PerformanceInsight>,
    pub security_insights: Vec<SecurityInsight>,
    pub recommendations: Vec<Recommendation>,
    pub trend_analysis: trends::TrendAnalysis,
}

// Additional insight types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInsight {
    pub insight_type: String,
    pub description: String,
    pub impact: Impact,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInsight {
    pub insight_type: String,
    pub description: String,
    pub bottleneck: String,
    pub improvement_potential: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInsight {
    pub vulnerability_type: String,
    pub description: String,
    pub severity: String,
    pub affected_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_effort: f64,
    pub expected_impact: f64,
}

/// Statistics about the intelligence engine capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct EngineStatistics {
    pub supported_languages: Vec<String>,
    pub available_metrics: Vec<String>,
    pub quality_factors: Vec<String>,
    pub suggestion_rules: usize,
}

/// Format a file intelligence report as human-readable text
pub fn format_file_report(report: &FileIntelligence) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("📊 Intelligence Report: {}\n", report.path.display()));
    output.push_str(&format!("Overall Quality Score: {:.1}/100\n", report.quality_score.overall));
    output.push_str(&format!("Last Analyzed: {}\n\n", report.last_analyzed.format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Metrics summary
    output.push_str("📈 Code Metrics:\n");
    output.push_str(&format!(
        "  Lines of Code: {} | Cyclomatic Complexity: {} | Cognitive Complexity: {}\n",
        report.metrics.lines_of_code,
        report.metrics.cyclomatic_complexity,
        report.metrics.cognitive_complexity
    ));
    output.push_str(&format!(
        "  Functions: {} | Classes: {} | Nesting Depth: {}\n",
        report.metrics.function_count,
        report.metrics.class_count,
        report.metrics.nesting_depth
    ));
    
    // Quality breakdown
    output.push_str("\n🎯 Quality Assessment:\n");
    output.push_str(&format!("  Maintainability: {:.1}%\n", report.quality_score.maintainability));
    output.push_str(&format!("  Reliability: {:.1}%\n", report.quality_score.reliability));
    output.push_str(&format!("  Security: {:.1}%\n", report.quality_score.security));
    output.push_str(&format!("  Performance: {:.1}%\n", report.quality_score.performance));
    output.push_str(&format!("  Test Coverage: {:.1}%\n", report.quality_score.test_coverage));
    
    // Risk assessment
    output.push_str(&format!("\n⚠️ Risk Level: {:?}\n", report.risk_assessment.overall_risk));
    if !report.risk_assessment.risk_factors.is_empty() {
        output.push_str("Risk Factors:\n");
        for factor in &report.risk_assessment.risk_factors {
            output.push_str(&format!("  • {} (Impact: {:.2})\n", factor.description, factor.impact));
        }
    }
    
    // Top suggestions
    if !report.suggestions.is_empty() {
        output.push_str("\n💡 Top Suggestions:\n");
        let mut suggestions = report.suggestions.clone();
        suggestions.sort_by(|a, b| b.priority_score.cmp(&a.priority_score));
        
        for suggestion in suggestions.iter().take(5) {
            let priority = match suggestion.priority_score {
                8..=10 => "🔴 Critical",
                6..=7 => "🟡 High",
                4..=5 => "🔵 Medium",
                _ => "🟢 Low",
            };
            
            output.push_str(&format!(
                "  {} [{:?}] {}\n",
                priority,
                suggestion.category,
                suggestion.title
            ));
        }
    }
    
    output.push('\n');
    output
}

/// Format a project intelligence report as human-readable text
pub fn format_project_report(report: &ProjectIntelligence) -> String {
    let mut output = String::new();
    
    output.push_str("🏗️ Project Intelligence Report\n");
    output.push_str("==============================\n\n");
    
    // Summary statistics
    output.push_str("📊 Project Summary:\n");
    output.push_str(&format!("  Total Files: {}\n", report.total_files));
    output.push_str(&format!("  Overall Quality Score: {:.1}/100\n", report.overall_quality.overall));
    output.push_str(&format!("  Technical Debt: {:.1} hours\n", report.technical_debt.total_debt_hours));
    output.push_str(&format!("  Debt Ratio: {:.2}\n\n", report.technical_debt.debt_ratio));
    
    // Quality breakdown
    output.push_str("🎯 Quality Breakdown:\n");
    output.push_str(&format!("  Maintainability: {:.1}%\n", report.overall_quality.maintainability));
    output.push_str(&format!("  Reliability: {:.1}%\n", report.overall_quality.reliability));
    output.push_str(&format!("  Security: {:.1}%\n", report.overall_quality.security));
    output.push_str(&format!("  Performance: {:.1}%\n", report.overall_quality.performance));
    output.push_str(&format!("  Test Coverage: {:.1}%\n\n", report.overall_quality.test_coverage));
    
    // Complexity distribution
    output.push_str("📈 Complexity Distribution:\n");
    let dist = &report.complexity_distribution;
    output.push_str(&format!("  Low (0-5): {} files\n", dist.low));
    output.push_str(&format!("  Medium (6-10): {} files\n", dist.medium));
    output.push_str(&format!("  High (11-20): {} files\n", dist.high));
    output.push_str(&format!("  Critical (20+): {} files\n", dist.critical));
    output.push_str(&format!("  Average: {:.1}\n\n", dist.average));
    
    // Language breakdown
    if !report.languages.is_empty() {
        output.push_str("🌐 Language Breakdown:\n");
        let mut langs: Vec<_> = report.languages.iter().collect();
        langs.sort_by(|a, b| b.1.file_count.cmp(&a.1.file_count));
        
        for (lang, stats) in langs {
            output.push_str(&format!(
                "  {}: {} files ({} lines, avg quality: {:.1})\n",
                lang, stats.file_count, stats.total_lines, stats.quality_score
            ));
        }
        output.push('\n');
    }
    
    // Team metrics
    output.push_str("👥 Team Metrics:\n");
    output.push_str(&format!("  Average Code Quality: {:.1}%\n", report.team_metrics.avg_code_quality));
    output.push_str(&format!("  Consistency Score: {:.1}%\n", report.team_metrics.consistency_score));
    output.push_str(&format!("  Error Resolution Time: {:.1} hours\n\n", report.team_metrics.error_resolution_time));
    
    // Trends
    output.push_str("📊 Trends:\n");
    output.push_str(&format!("  Quality: {:?}\n", report.trends.quality_trend));
    output.push_str(&format!("  Complexity: {:?}\n", report.trends.complexity_trend));
    output.push_str(&format!("  Errors: {:?}\n", report.trends.error_trend));
    output.push_str(&format!("  Productivity: {:?}\n\n", report.trends.productivity_trend));
    
    // Technical debt
    if !report.technical_debt.high_debt_files.is_empty() {
        output.push_str("💸 High Technical Debt Files:\n");
        for file in report.technical_debt.high_debt_files.iter().take(10) {
            output.push_str(&format!("  • {}\n", file.display()));
        }
        output.push('\n');
    }
    
    output
}

/// Generate a detailed report for a project
pub fn generate_detailed_report(report: &ProjectIntelligence) -> String {
    let mut output = String::new();
    
    output.push_str("# Detailed Project Intelligence Report\n\n");
    output.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Executive Summary
    output.push_str("## Executive Summary\n\n");
    output.push_str(&format!("This project contains {} files with an overall quality score of {:.1}/100. ",
        report.total_files, report.overall_quality.overall));
    
    let quality_level = match report.overall_quality.overall {
        90.0..=100.0 => "excellent",
        80.0..=89.9 => "good",
        70.0..=79.9 => "fair",
        60.0..=69.9 => "poor",
        _ => "critical",
    };
    output.push_str(&format!("The code quality is considered {}.\n\n", quality_level));
    
    // Technical Debt Analysis
    output.push_str("## Technical Debt Analysis\n\n");
    output.push_str(&format!("**Total Technical Debt:** {:.1} hours\n", report.technical_debt.total_debt_hours));
    output.push_str(&format!("**Debt Ratio:** {:.2}\n\n", report.technical_debt.debt_ratio));
    
    if !report.technical_debt.debt_by_category.is_empty() {
        output.push_str("### Debt by Category:\n\n");
        for (category, hours) in &report.technical_debt.debt_by_category {
            output.push_str(&format!("- **{}:** {:.1} hours\n", category, hours));
        }
        output.push_str("\n");
    }
    
    // Language Analysis
    if !report.languages.is_empty() {
        output.push_str("## Language Analysis\n\n");
        let mut langs: Vec<_> = report.languages.iter().collect();
        langs.sort_by(|a, b| b.1.file_count.cmp(&a.1.file_count));
        
        for (lang, stats) in langs {
            output.push_str(&format!("### {}\n\n", lang));
            output.push_str(&format!("- **Files:** {}\n", stats.file_count));
            output.push_str(&format!("- **Total Lines:** {}\n", stats.total_lines));
            output.push_str(&format!("- **Average Quality:** {:.1}%\n", stats.quality_score));
            output.push_str(&format!("- **Average Complexity:** {:.1}\n\n", stats.avg_complexity));
        }
    }
    
    // Quality Metrics Detail
    output.push_str("## Quality Metrics\n\n");
    output.push_str(&format!("- **Maintainability:** {:.1}%\n", report.overall_quality.maintainability));
    output.push_str(&format!("- **Reliability:** {:.1}%\n", report.overall_quality.reliability));
    output.push_str(&format!("- **Security:** {:.1}%\n", report.overall_quality.security));
    output.push_str(&format!("- **Performance:** {:.1}%\n", report.overall_quality.performance));
    output.push_str(&format!("- **Test Coverage:** {:.1}%\n\n", report.overall_quality.test_coverage));
    
    // Complexity Analysis
    output.push_str("## Complexity Analysis\n\n");
    let dist = &report.complexity_distribution;
    output.push_str(&format!("- **Low Complexity (0-5):** {} files ({:.1}%)\n", 
        dist.low, (dist.low as f64 / report.total_files as f64) * 100.0));
    output.push_str(&format!("- **Medium Complexity (6-10):** {} files ({:.1}%)\n", 
        dist.medium, (dist.medium as f64 / report.total_files as f64) * 100.0));
    output.push_str(&format!("- **High Complexity (11-20):** {} files ({:.1}%)\n", 
        dist.high, (dist.high as f64 / report.total_files as f64) * 100.0));
    output.push_str(&format!("- **Critical Complexity (20+):** {} files ({:.1}%)\n", 
        dist.critical, (dist.critical as f64 / report.total_files as f64) * 100.0));
    output.push_str(&format!("- **Average Complexity:** {:.1}\n\n", dist.average));
    
    // Recommendations
    output.push_str("## Recommendations\n\n");
    
    if report.overall_quality.overall < 70.0 {
        output.push_str("### Priority Actions\n\n");
        output.push_str("1. **Improve Code Quality:** Focus on refactoring high-complexity functions\n");
        output.push_str("2. **Increase Test Coverage:** Add comprehensive unit and integration tests\n");
        output.push_str("3. **Address Technical Debt:** Prioritize fixing high-debt files\n\n");
    }
    
    if dist.critical > 0 {
        output.push_str(&format!("### Critical Complexity Files\n\n{} files have critical complexity and should be refactored immediately.\n\n", dist.critical));
    }
    
    if report.technical_debt.debt_ratio > 0.5 {
        output.push_str("### Technical Debt Management\n\nThe technical debt ratio is high. Consider allocating dedicated time for debt reduction.\n\n");
    }
    
    output.push_str("---\n");
    output.push_str(&format!("*Report generated by Synx Intelligence Engine at {}*\n", 
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    output
}

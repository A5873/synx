use std::path::Path;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use super::metrics::{CodeMetrics, ComplexityRating};

/// Comprehensive quality score for code
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityScore {
    pub overall: f64,
    pub maintainability: f64,
    pub reliability: f64,
    pub security: f64,
    pub performance: f64,
    pub test_coverage: f64,
}

/// Detailed quality assessment with insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub score: QualityScore,
    pub grade: QualityGrade,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub recommendations: Vec<QualityRecommendation>,
    pub trend: QualityTrend,
}

/// Quality grade classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityGrade {
    Excellent,  // 90-100%
    Good,       // 80-89%
    Fair,       // 70-79%
    Poor,       // 60-69%
    Critical,   // Below 60%
}

/// Quality improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecommendation {
    pub category: String,
    pub description: String,
    pub impact: ImpactLevel,
    pub effort: EffortLevel,
    pub priority: u8, // 1-10 scale
}

/// Quality trend over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityTrend {
    Improving,
    Stable,
    Declining,
    Volatile,
    Unknown,
}

/// Impact level of recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
}

/// Effort required for improvements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    High,
    Medium,
    Low,
}

/// Quality assessor that evaluates code quality
pub struct QualityAssessor {
    // Quality thresholds and weights
    maintainability_weight: f64,
    reliability_weight: f64,
    security_weight: f64,
    performance_weight: f64,
    test_coverage_weight: f64,
}

impl QualityAssessor {
    pub fn new() -> Self {
        Self {
            maintainability_weight: 0.3,
            reliability_weight: 0.25,
            security_weight: 0.2,
            performance_weight: 0.15,
            test_coverage_weight: 0.1,
        }
    }
    
    /// Assess the quality of a file based on its metrics
    pub fn assess_file(&self, file_path: &Path, content: &str, metrics: &CodeMetrics) -> Result<QualityScore> {
        let maintainability = self.assess_maintainability(metrics);
        let reliability = self.assess_reliability(metrics);
        let security = self.assess_security(content, metrics);
        let performance = self.assess_performance(metrics);
        let test_coverage = self.assess_test_coverage(file_path, content);
        
        let overall = (maintainability * self.maintainability_weight +
                      reliability * self.reliability_weight +
                      security * self.security_weight +
                      performance * self.performance_weight +
                      test_coverage * self.test_coverage_weight) / 
                     (self.maintainability_weight + self.reliability_weight + 
                      self.security_weight + self.performance_weight + 
                      self.test_coverage_weight);
        
        Ok(QualityScore {
            overall,
            maintainability,
            reliability,
            security,
            performance,
            test_coverage,
        })
    }
    
    /// Generate comprehensive quality assessment
    pub fn generate_assessment(&self, file_path: &Path, content: &str, metrics: &CodeMetrics) -> Result<QualityAssessment> {
        let score = self.assess_file(file_path, content, metrics)?;
        let grade = self.calculate_grade(score.overall);
        let strengths = self.identify_strengths(&score, metrics);
        let weaknesses = self.identify_weaknesses(&score, metrics);
        let recommendations = self.generate_recommendations(&score, metrics);
        
        Ok(QualityAssessment {
            score,
            grade,
            strengths,
            weaknesses,
            recommendations,
            trend: QualityTrend::Unknown, // Would be calculated from historical data
        })
    }
    
    /// Assess maintainability based on complexity and structure
    fn assess_maintainability(&self, metrics: &CodeMetrics) -> f64 {
        let mut score = 100.0;
        
        // Penalize high cyclomatic complexity
        if metrics.cyclomatic_complexity > 10 {
            score -= (metrics.cyclomatic_complexity as f64 - 10.0) * 2.0;
        }
        
        // Penalize cognitive complexity
        if metrics.cognitive_complexity > 15 {
            score -= (metrics.cognitive_complexity as f64 - 15.0) * 1.5;
        }
        
        // Penalize deep nesting
        if metrics.nesting_depth > 4 {
            score -= (metrics.nesting_depth as f64 - 4.0) * 3.0;
        }
        
        // Consider function complexity distribution
        let complex_functions = metrics.functions.iter()
            .filter(|f| matches!(f.complexity_rating, ComplexityRating::Complex | ComplexityRating::Critical))
            .count();
        
        if !metrics.functions.is_empty() {
            let complex_ratio = complex_functions as f64 / metrics.functions.len() as f64;
            score -= complex_ratio * 20.0;
        }
        
        // Consider maintainability index if available
        if metrics.maintainability_index > 0.0 {
            score = (score + metrics.maintainability_index) / 2.0;
        }
        
        score.max(0.0).min(100.0)
    }
    
    /// Assess reliability based on error patterns and code structure
    fn assess_reliability(&self, metrics: &CodeMetrics) -> f64 {
        let mut score = 100.0;
        
        // Penalize high complexity (more likely to have bugs)
        if metrics.cyclomatic_complexity > 15 {
            score -= (metrics.cyclomatic_complexity as f64 - 15.0) * 1.5;
        }
        
        // Consider function length (longer functions are more error-prone)
        let avg_function_length = if !metrics.functions.is_empty() {
            metrics.functions.iter().map(|f| f.lines_of_code).sum::<usize>() as f64 / metrics.functions.len() as f64
        } else {
            0.0
        };
        
        if avg_function_length > 50.0 {
            score -= (avg_function_length - 50.0) * 0.5;
        }
        
        // Consider parameter counts (high parameter count increases error risk)
        let avg_params = if !metrics.functions.is_empty() {
            metrics.functions.iter().map(|f| f.parameter_count).sum::<usize>() as f64 / metrics.functions.len() as f64
        } else {
            0.0
        };
        
        if avg_params > 5.0 {
            score -= (avg_params - 5.0) * 2.0;
        }
        
        // Consider coupling (high coupling reduces reliability)
        if metrics.coupling_factor > 0.7 {
            score -= (metrics.coupling_factor - 0.7) * 30.0;
        }
        
        score.max(0.0).min(100.0)
    }
    
    /// Assess security based on common patterns and practices
    fn assess_security(&self, content: &str, metrics: &CodeMetrics) -> f64 {
        let mut score: f64 = 100.0;
        let mut _security_issues = 0;
        
        // Check for common security anti-patterns
        let security_patterns = [
            ("password", 5.0),
            ("hardcoded", 10.0),
            ("eval(", 15.0),
            ("exec(", 15.0),
            ("system(", 10.0),
            ("shell_exec", 10.0),
            ("unsafe", 8.0),
            ("TODO: security", 3.0),
            ("FIXME: security", 3.0),
        ];
        
        for (pattern, penalty) in &security_patterns {
            if content.to_lowercase().contains(pattern) {
                score -= penalty;
                _security_issues += 1;
            }
        }
        
        // Check for SQL injection patterns
        let sql_patterns = [
            "SELECT * FROM",
            "INSERT INTO",
            "UPDATE SET",
            "DELETE FROM",
        ];
        
        for pattern in &sql_patterns {
            if content.contains(pattern) && content.contains("+") {
                score -= 15.0; // Potential SQL injection
                _security_issues += 1;
            }
        }
        
        // Consider language-specific security features
        match metrics.language.as_str() {
            "rust" => {
                // Rust has memory safety, bonus points
                score += 10.0;
                if content.contains("unsafe") {
                    score -= 20.0; // But unsafe blocks are risky
                }
            }
            "c" | "cpp" => {
                // C/C++ are inherently less secure
                score -= 10.0;
                if content.contains("strcpy") || content.contains("gets") {
                    score -= 25.0; // Dangerous functions
                }
            }
            "python" | "javascript" => {
                if content.contains("eval") {
                    score -= 20.0; // eval is dangerous
                }
            }
            _ => {}
        }
        
        score.max(0.0).min(100.0)
    }
    
    /// Assess performance based on algorithmic complexity indicators
    fn assess_performance(&self, metrics: &CodeMetrics) -> f64 {
        let mut score = 100.0;
        
        // Penalize deeply nested loops (potential O(n^k) complexity)
        if metrics.nesting_depth > 3 {
            score -= (metrics.nesting_depth as f64 - 3.0) * 5.0;
        }
        
        // Consider function complexity as performance indicator
        let high_complexity_functions = metrics.functions.iter()
            .filter(|f| f.cyclomatic_complexity > 20)
            .count();
        
        if !metrics.functions.is_empty() {
            let complexity_ratio = high_complexity_functions as f64 / metrics.functions.len() as f64;
            score -= complexity_ratio * 25.0;
        }
        
        // Consider coupling (high coupling can impact performance)
        if metrics.coupling_factor > 0.8 {
            score -= (metrics.coupling_factor - 0.8) * 20.0;
        }
        
        score.max(0.0).min(100.0)
    }
    
    /// Assess test coverage (simplified - would integrate with coverage tools)
    fn assess_test_coverage(&self, file_path: &Path, content: &str) -> f64 {
        let mut score: f64 = 0.0;
        
        // Check if this is a test file
        let path_str = file_path.to_string_lossy().to_lowercase();
        if path_str.contains("test") || path_str.contains("spec") {
            return 100.0; // Test files get full coverage score
        }
        
        // Look for test-related patterns in the code
        let test_patterns = [
            "#[test]",
            "@Test",
            "test_",
            "describe(",
            "it(",
            "assert",
            "expect(",
        ];
        
        for pattern in &test_patterns {
            if content.contains(pattern) {
                score += 20.0;
            }
        }
        
        // Check for corresponding test files (simplified)
        let test_indicators = [
            "assert",
            "test",
            "mock",
            "stub",
        ];
        
        for indicator in &test_indicators {
            if content.to_lowercase().contains(indicator) {
                score += 10.0;
            }
        }
        
        score.min(100.0)
    }
    
    /// Calculate quality grade from overall score
    fn calculate_grade(&self, score: f64) -> QualityGrade {
        match score {
            90.0..=100.0 => QualityGrade::Excellent,
            80.0..=89.9 => QualityGrade::Good,
            70.0..=79.9 => QualityGrade::Fair,
            60.0..=69.9 => QualityGrade::Poor,
            _ => QualityGrade::Critical,
        }
    }
    
    /// Identify strengths in the code quality
    fn identify_strengths(&self, score: &QualityScore, metrics: &CodeMetrics) -> Vec<String> {
        let mut strengths = Vec::new();
        
        if score.maintainability > 80.0 {
            strengths.push("High maintainability - code is well-structured and easy to modify".to_string());
        }
        
        if score.reliability > 85.0 {
            strengths.push("Good reliability indicators - low complexity and good structure".to_string());
        }
        
        if score.security > 90.0 {
            strengths.push("Strong security posture - no obvious security anti-patterns detected".to_string());
        }
        
        if score.performance > 85.0 {
            strengths.push("Good performance characteristics - efficient algorithmic patterns".to_string());
        }
        
        if metrics.functions.iter().all(|f| matches!(f.complexity_rating, ComplexityRating::Simple | ComplexityRating::Moderate)) {
            strengths.push("All functions have manageable complexity".to_string());
        }
        
        if metrics.cohesion_factor > 0.8 {
            strengths.push("High cohesion - related functionality is well-grouped".to_string());
        }
        
        strengths
    }
    
    /// Identify weaknesses in the code quality
    fn identify_weaknesses(&self, score: &QualityScore, metrics: &CodeMetrics) -> Vec<String> {
        let mut weaknesses = Vec::new();
        
        if score.maintainability < 60.0 {
            weaknesses.push("Low maintainability - code structure needs improvement".to_string());
        }
        
        if score.reliability < 70.0 {
            weaknesses.push("Reliability concerns - high complexity increases error risk".to_string());
        }
        
        if score.security < 80.0 {
            weaknesses.push("Security issues detected - review for potential vulnerabilities".to_string());
        }
        
        if score.performance < 70.0 {
            weaknesses.push("Performance concerns - algorithmic complexity may be too high".to_string());
        }
        
        if score.test_coverage < 50.0 {
            weaknesses.push("Low test coverage - needs more comprehensive testing".to_string());
        }
        
        let critical_functions = metrics.functions.iter()
            .filter(|f| matches!(f.complexity_rating, ComplexityRating::Critical))
            .count();
        
        if critical_functions > 0 {
            weaknesses.push(format!("{} functions have critical complexity and need refactoring", critical_functions));
        }
        
        if metrics.coupling_factor > 0.7 {
            weaknesses.push("High coupling detected - components are too interdependent".to_string());
        }
        
        weaknesses
    }
    
    /// Generate specific recommendations for improvement
    fn generate_recommendations(&self, score: &QualityScore, metrics: &CodeMetrics) -> Vec<QualityRecommendation> {
        let mut recommendations = Vec::new();
        
        if score.maintainability < 70.0 {
            recommendations.push(QualityRecommendation {
                category: "Maintainability".to_string(),
                description: "Break down complex functions into smaller, more focused functions".to_string(),
                impact: ImpactLevel::High,
                effort: EffortLevel::Medium,
                priority: 8,
            });
        }
        
        if metrics.cyclomatic_complexity > 15 {
            recommendations.push(QualityRecommendation {
                category: "Complexity".to_string(),
                description: "Reduce cyclomatic complexity by simplifying control flow".to_string(),
                impact: ImpactLevel::High,
                effort: EffortLevel::High,
                priority: 9,
            });
        }
        
        if score.security < 80.0 {
            recommendations.push(QualityRecommendation {
                category: "Security".to_string(),
                description: "Review code for security vulnerabilities and implement best practices".to_string(),
                impact: ImpactLevel::High,
                effort: EffortLevel::Medium,
                priority: 10,
            });
        }
        
        if score.test_coverage < 60.0 {
            recommendations.push(QualityRecommendation {
                category: "Testing".to_string(),
                description: "Increase test coverage with unit and integration tests".to_string(),
                impact: ImpactLevel::Medium,
                effort: EffortLevel::High,
                priority: 6,
            });
        }
        
        let long_functions = metrics.functions.iter()
            .filter(|f| f.lines_of_code > 50)
            .count();
        
        if long_functions > 0 {
            recommendations.push(QualityRecommendation {
                category: "Function Size".to_string(),
                description: format!("Extract logic from {} long functions into smaller functions", long_functions),
                impact: ImpactLevel::Medium,
                effort: EffortLevel::Medium,
                priority: 7,
            });
        }
        
        if metrics.coupling_factor > 0.7 {
            recommendations.push(QualityRecommendation {
                category: "Architecture".to_string(),
                description: "Reduce coupling by introducing abstractions and dependency injection".to_string(),
                impact: ImpactLevel::High,
                effort: EffortLevel::High,
                priority: 8,
            });
        }
        
        // Sort by priority (highest first)
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        recommendations
    }
}

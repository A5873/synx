use std::path::Path;
use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use super::metrics::{CodeMetrics, ComplexityRating, FunctionMetrics};
use super::quality::QualityScore;

/// Smart suggestion for code improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartSuggestion {
    pub category: SuggestionCategory,
    pub title: String,
    pub description: String,
    pub rationale: String,
    pub confidence: f64,
    pub impact: ImpactLevel,
    pub effort_estimate: EffortLevel,
    pub priority_score: u8, // 1-10 scale
    pub code_example: Option<CodeExample>,
    pub related_patterns: Vec<String>,
    pub applicable_files: Vec<String>,
}

/// Categories of suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Complexity,
    Performance,
    Security,
    Maintainability,
    Testing,
    Architecture,
    CodeStyle,
    Documentation,
    ErrorHandling,
    ResourceManagement,
}

/// Impact level of suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Critical,   // Fixes critical issues
    High,       // Significant improvement
    Medium,     // Moderate improvement
    Low,        // Minor improvement
}

/// Effort required to implement suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    High,       // > 4 hours
    Medium,     // 1-4 hours
    Low,        // < 1 hour
}

/// Code example showing before/after
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub before: String,
    pub after: String,
    pub explanation: String,
}

/// Rule-based suggestion with conditions
struct SuggestionRule {
    pub condition: Box<dyn Fn(&CodeMetrics, &QualityScore) -> bool>,
    pub generator: Box<dyn Fn(&CodeMetrics, &QualityScore) -> SmartSuggestion>,
}

/// Engine for generating intelligent suggestions
pub struct SuggestionEngine {
    rules: Vec<SuggestionRule>,
    language_specific_rules: HashMap<String, Vec<SuggestionRule>>,
    pattern_database: PatternDatabase,
}

/// Database of common code patterns and their fixes
#[derive(Debug, Clone)]
struct PatternDatabase {
    complexity_patterns: Vec<ComplexityPattern>,
    security_patterns: Vec<SecurityPattern>,
    performance_patterns: Vec<PerformancePattern>,
}

/// Pattern for complexity issues
#[derive(Debug, Clone)]
struct ComplexityPattern {
    name: String,
    description: String,
    detection_logic: String, // Would be actual detection logic
    suggested_fix: String,
    example: Option<CodeExample>,
}

/// Pattern for security issues
#[derive(Debug, Clone)]
struct SecurityPattern {
    name: String,
    vulnerability_type: String,
    risk_level: String,
    detection_patterns: Vec<String>,
    mitigation: String,
}

/// Pattern for performance issues
#[derive(Debug, Clone)]
struct PerformancePattern {
    name: String,
    performance_impact: String,
    detection_criteria: String,
    optimization_suggestion: String,
}

impl SuggestionEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            rules: Vec::new(),
            language_specific_rules: HashMap::new(),
            pattern_database: PatternDatabase::new(),
        };
        
        engine.initialize_rules();
        engine.initialize_language_rules();
        
        engine
    }
    
    /// Generate comprehensive suggestions based on metrics and quality
    pub fn generate_suggestions(
        &self, 
        file_path: &Path, 
        metrics: &CodeMetrics, 
        quality: &QualityScore
    ) -> Result<Vec<SmartSuggestion>> {
        let mut suggestions = Vec::new();
        
        // Apply general rules
        for rule in &self.rules {
            if (rule.condition)(metrics, quality) {
                suggestions.push((rule.generator)(metrics, quality));
            }
        }
        
        // Apply language-specific rules
        if let Some(lang_rules) = self.language_specific_rules.get(&metrics.language) {
            for rule in lang_rules {
                if (rule.condition)(metrics, quality) {
                    suggestions.push((rule.generator)(metrics, quality));
                }
            }
        }
        
        // Generate function-specific suggestions
        for function in &metrics.functions {
            suggestions.extend(self.generate_function_suggestions(function));
        }
        
        // Generate architecture suggestions
        suggestions.extend(self.generate_architecture_suggestions(metrics, quality));
        
        // Generate security suggestions based on patterns
        suggestions.extend(self.generate_security_suggestions(file_path, metrics));
        
        // Generate performance suggestions
        suggestions.extend(self.generate_performance_suggestions(metrics));
        
        // Sort by priority and confidence
        suggestions.sort_by(|a, b| {
            b.priority_score.cmp(&a.priority_score)
                .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        // Remove duplicates and limit results
        suggestions.dedup_by(|a, b| a.title == b.title);
        suggestions.truncate(10); // Limit to top 10 suggestions
        
        Ok(suggestions)
    }
    
    /// Initialize general suggestion rules
    fn initialize_rules(&mut self) {
        // High complexity rule
        self.rules.push(SuggestionRule {
            condition: Box::new(|metrics, _| metrics.cyclomatic_complexity > 15),
            generator: Box::new(|metrics, _| SmartSuggestion {
                category: SuggestionCategory::Complexity,
                title: "Reduce Cyclomatic Complexity".to_string(),
                description: format!(
                    "This code has a cyclomatic complexity of {}, which is quite high. Consider breaking it down into smaller functions.",
                    metrics.cyclomatic_complexity
                ),
                rationale: "High cyclomatic complexity makes code harder to understand, test, and maintain. It also increases the likelihood of bugs.".to_string(),
                confidence: 0.9,
                impact: ImpactLevel::High,
                effort_estimate: EffortLevel::Medium,
                priority_score: 8,
                code_example: Some(CodeExample {
                    before: "fn complex_function(data: Vec<Item>) -> Result<Output> {\n    // 20+ lines of complex logic with multiple conditions\n}".to_string(),
                    after: "fn complex_function(data: Vec<Item>) -> Result<Output> {\n    let processed = preprocess_data(data)?;\n    let validated = validate_data(processed)?;\n    generate_output(validated)\n}\n\nfn preprocess_data(data: Vec<Item>) -> Result<ProcessedData> { ... }\nfn validate_data(data: ProcessedData) -> Result<ValidatedData> { ... }\nfn generate_output(data: ValidatedData) -> Result<Output> { ... }".to_string(),
                    explanation: "Break complex functions into smaller, focused functions that each handle a specific responsibility.".to_string(),
                }),
                related_patterns: vec!["extract_method".to_string(), "single_responsibility".to_string()],
                applicable_files: vec![],
            }),
        });
        
        // Low maintainability rule
        self.rules.push(SuggestionRule {
            condition: Box::new(|_, quality| quality.maintainability < 60.0),
            generator: Box::new(|_metrics, quality| SmartSuggestion {
                category: SuggestionCategory::Maintainability,
                title: "Improve Code Maintainability".to_string(),
                description: format!(
                    "Maintainability score is {:.1}%, which is below the recommended threshold of 70%.",
                    quality.maintainability
                ),
                rationale: "Low maintainability makes future changes more difficult and error-prone, increasing development costs.".to_string(),
                confidence: 0.85,
                impact: ImpactLevel::High,
                effort_estimate: EffortLevel::High,
                priority_score: 7,
                code_example: None,
                related_patterns: vec!["refactoring".to_string(), "clean_code".to_string()],
                applicable_files: vec![],
            }),
        });
        
        // Security concern rule
        self.rules.push(SuggestionRule {
            condition: Box::new(|_, quality| quality.security < 80.0),
            generator: Box::new(|_, quality| SmartSuggestion {
                category: SuggestionCategory::Security,
                title: "Address Security Concerns".to_string(),
                description: format!(
                    "Security score is {:.1}%. Review code for potential vulnerabilities.",
                    quality.security
                ),
                rationale: "Security vulnerabilities can lead to data breaches, system compromises, and compliance issues.".to_string(),
                confidence: 0.8,
                impact: ImpactLevel::Critical,
                effort_estimate: EffortLevel::Medium,
                priority_score: 10,
                code_example: Some(CodeExample {
                    before: "let query = format!(\"SELECT * FROM users WHERE id = {}\", user_id);".to_string(),
                    after: "let query = \"SELECT * FROM users WHERE id = $1\";\nlet result = db.execute(query, &[&user_id]);".to_string(),
                    explanation: "Use parameterized queries instead of string concatenation to prevent SQL injection attacks.".to_string(),
                }),
                related_patterns: vec!["sql_injection_prevention".to_string(), "input_validation".to_string()],
                applicable_files: vec![],
            }),
        });
        
        // Performance concern rule
        self.rules.push(SuggestionRule {
            condition: Box::new(|_, quality| quality.performance < 70.0),
            generator: Box::new(|_metrics, quality| SmartSuggestion {
                category: SuggestionCategory::Performance,
                title: "Optimize Performance".to_string(),
                description: format!(
                    "Performance score is {:.1}%. Consider optimizing algorithmic complexity.",
                    quality.performance
                ),
                rationale: "Poor performance can lead to slow response times and poor user experience.".to_string(),
                confidence: 0.75,
                impact: ImpactLevel::Medium,
                effort_estimate: EffortLevel::High,
                priority_score: 6,
                code_example: Some(CodeExample {
                    before: "for item in &collection {\n    for other in &other_collection {\n        // O(n²) operation\n    }\n}".to_string(),
                    after: "let lookup: HashMap<_, _> = other_collection.iter().collect();\nfor item in &collection {\n    if let Some(other) = lookup.get(&item.key) {\n        // O(n) operation\n    }\n}".to_string(),
                    explanation: "Use hash maps or other efficient data structures to reduce algorithmic complexity.".to_string(),
                }),
                related_patterns: vec!["algorithm_optimization".to_string(), "data_structure_selection".to_string()],
                applicable_files: vec![],
            }),
        });
        
        // Test coverage rule
        self.rules.push(SuggestionRule {
            condition: Box::new(|_, quality| quality.test_coverage < 60.0),
            generator: Box::new(|_, quality| SmartSuggestion {
                category: SuggestionCategory::Testing,
                title: "Increase Test Coverage".to_string(),
                description: format!(
                    "Test coverage is {:.1}%. Add more unit and integration tests.",
                    quality.test_coverage
                ),
                rationale: "Comprehensive tests reduce bugs, enable refactoring, and improve code reliability.".to_string(),
                confidence: 0.9,
                impact: ImpactLevel::High,
                effort_estimate: EffortLevel::High,
                priority_score: 7,
                code_example: Some(CodeExample {
                    before: "// No tests for this function\nfn calculate_total(items: &[Item]) -> f64 {\n    items.iter().map(|i| i.price).sum()\n}".to_string(),
                    after: "#[cfg(test)]\nmod tests {\n    use super::*;\n    \n    #[test]\n    fn test_calculate_total() {\n        let items = vec![Item { price: 10.0 }, Item { price: 20.0 }];\n        assert_eq!(calculate_total(&items), 30.0);\n    }\n}".to_string(),
                    explanation: "Add comprehensive unit tests to verify function behavior and prevent regressions.".to_string(),
                }),
                related_patterns: vec!["unit_testing".to_string(), "test_driven_development".to_string()],
                applicable_files: vec![],
            }),
        });
    }
    
    /// Initialize language-specific rules
    fn initialize_language_rules(&mut self) {
        // Rust-specific rules
        let mut rust_rules = Vec::new();
        
        rust_rules.push(SuggestionRule {
            condition: Box::new(|metrics, _| {
                metrics.language == "rust" && metrics.functions.iter().any(|f| f.parameter_count > 5)
            }),
            generator: Box::new(|_, _| SmartSuggestion {
                category: SuggestionCategory::CodeStyle,
                title: "Use Struct for Multiple Parameters".to_string(),
                description: "Consider using a struct or builder pattern for functions with many parameters.".to_string(),
                rationale: "Functions with many parameters are hard to use correctly and maintain. Rust's struct pattern provides better type safety.".to_string(),
                confidence: 0.85,
                impact: ImpactLevel::Medium,
                effort_estimate: EffortLevel::Medium,
                priority_score: 6,
                code_example: Some(CodeExample {
                    before: "fn create_user(name: String, email: String, age: u32, active: bool, role: String, department: String) -> User { ... }".to_string(),
                    after: "#[derive(Debug)]\nstruct CreateUserRequest {\n    name: String,\n    email: String,\n    age: u32,\n    active: bool,\n    role: String,\n    department: String,\n}\n\nfn create_user(req: CreateUserRequest) -> User { ... }".to_string(),
                    explanation: "Use a struct to group related parameters, making the function signature cleaner and more maintainable.".to_string(),
                }),
                related_patterns: vec!["parameter_object".to_string(), "builder_pattern".to_string()],
                applicable_files: vec![],
            }),
        });
        
        self.language_specific_rules.insert("rust".to_string(), rust_rules);
        
        // Python-specific rules
        let mut python_rules = Vec::new();
        
        python_rules.push(SuggestionRule {
            condition: Box::new(|metrics, _| {
                metrics.language == "python" && metrics.functions.iter().any(|f| f.lines_of_code > 50)
            }),
            generator: Box::new(|_, _| SmartSuggestion {
                category: SuggestionCategory::Maintainability,
                title: "Follow PEP 8 Function Length Guidelines".to_string(),
                description: "Consider breaking down long Python functions for better readability.".to_string(),
                rationale: "PEP 8 recommends keeping functions focused and reasonably sized for better maintainability.".to_string(),
                confidence: 0.8,
                impact: ImpactLevel::Medium,
                effort_estimate: EffortLevel::Medium,
                priority_score: 5,
                code_example: None,
                related_patterns: vec!["extract_method".to_string(), "pep8".to_string()],
                applicable_files: vec![],
            }),
        });
        
        self.language_specific_rules.insert("python".to_string(), python_rules);
    }
    
    /// Generate function-specific suggestions
    fn generate_function_suggestions(&self, function: &FunctionMetrics) -> Vec<SmartSuggestion> {
        let mut suggestions = Vec::new();
        
        // Too many parameters
        if function.parameter_count > 5 {
            suggestions.push(SmartSuggestion {
                category: SuggestionCategory::CodeStyle,
                title: format!("Reduce parameters in '{}'", function.name),
                description: format!(
                    "Function '{}' has {} parameters. Consider using a parameter object or breaking it down.",
                    function.name, function.parameter_count
                ),
                rationale: "Functions with many parameters are harder to use, test, and maintain.".to_string(),
                confidence: 0.9,
                impact: ImpactLevel::Medium,
                effort_estimate: EffortLevel::Low,
                priority_score: 6,
                code_example: None,
                related_patterns: vec!["parameter_object".to_string()],
                applicable_files: vec![],
            });
        }
        
        // Too many return statements
        if function.return_statements > 5 {
            suggestions.push(SmartSuggestion {
                category: SuggestionCategory::CodeStyle,
                title: format!("Simplify return logic in '{}'", function.name),
                description: format!(
                    "Function '{}' has {} return statements. Consider consolidating the return logic.",
                    function.name, function.return_statements
                ),
                rationale: "Multiple return statements can make code harder to follow and debug.".to_string(),
                confidence: 0.8,
                impact: ImpactLevel::Low,
                effort_estimate: EffortLevel::Low,
                priority_score: 4,
                code_example: None,
                related_patterns: vec!["single_exit_point".to_string()],
                applicable_files: vec![],
            });
        }
        
        // Critical complexity
        if matches!(function.complexity_rating, ComplexityRating::Critical) {
            suggestions.push(SmartSuggestion {
                category: SuggestionCategory::Complexity,
                title: format!("Refactor critical complexity in '{}'", function.name),
                description: format!(
                    "Function '{}' has critical complexity (CC: {}). Immediate refactoring is recommended.",
                    function.name, function.cyclomatic_complexity
                ),
                rationale: "Critical complexity makes code extremely difficult to understand, test, and maintain.".to_string(),
                confidence: 0.95,
                impact: ImpactLevel::Critical,
                effort_estimate: EffortLevel::High,
                priority_score: 10,
                code_example: None,
                related_patterns: vec!["extract_method".to_string(), "strategy_pattern".to_string()],
                applicable_files: vec![],
            });
        }
        
        suggestions
    }
    
    /// Generate architecture-level suggestions
    fn generate_architecture_suggestions(&self, metrics: &CodeMetrics, _quality: &QualityScore) -> Vec<SmartSuggestion> {
        let mut suggestions = Vec::new();
        
        // High coupling
        if metrics.coupling_factor > 0.8 {
            suggestions.push(SmartSuggestion {
                category: SuggestionCategory::Architecture,
                title: "Reduce System Coupling".to_string(),
                description: format!(
                    "Coupling factor is {:.2}, indicating high interdependence between components.",
                    metrics.coupling_factor
                ),
                rationale: "High coupling makes the system rigid, fragile, and difficult to modify or test.".to_string(),
                confidence: 0.85,
                impact: ImpactLevel::High,
                effort_estimate: EffortLevel::High,
                priority_score: 8,
                code_example: None,
                related_patterns: vec!["dependency_injection".to_string(), "interface_segregation".to_string()],
                applicable_files: vec![],
            });
        }
        
        // Low cohesion
        if metrics.cohesion_factor < 0.6 {
            suggestions.push(SmartSuggestion {
                category: SuggestionCategory::Architecture,
                title: "Improve Module Cohesion".to_string(),
                description: format!(
                    "Cohesion factor is {:.2}, suggesting scattered responsibilities.",
                    metrics.cohesion_factor
                ),
                rationale: "Low cohesion indicates that related functionality is spread across the codebase.".to_string(),
                confidence: 0.8,
                impact: ImpactLevel::Medium,
                effort_estimate: EffortLevel::High,
                priority_score: 7,
                code_example: None,
                related_patterns: vec!["single_responsibility".to_string(), "module_organization".to_string()],
                applicable_files: vec![],
            });
        }
        
        suggestions
    }
    
    /// Generate security-specific suggestions
    fn generate_security_suggestions(&self, file_path: &Path, _metrics: &CodeMetrics) -> Vec<SmartSuggestion> {
        let mut suggestions = Vec::new();
        
        // Check file content for security patterns (simplified)
        if let Ok(content) = std::fs::read_to_string(file_path) {
            for pattern in &self.pattern_database.security_patterns {
                for detection_pattern in &pattern.detection_patterns {
                    if content.contains(detection_pattern) {
                        suggestions.push(SmartSuggestion {
                            category: SuggestionCategory::Security,
                            title: format!("Address {}", pattern.name),
                            description: format!("Potential {} detected in code.", pattern.vulnerability_type),
                            rationale: format!("This pattern may lead to security vulnerabilities: {}", pattern.mitigation),
                            confidence: 0.7,
                            impact: ImpactLevel::High,
                            effort_estimate: EffortLevel::Medium,
                            priority_score: 9,
                            code_example: None,
                            related_patterns: vec![pattern.name.clone()],
                            applicable_files: vec![file_path.to_string_lossy().to_string()],
                        });
                    }
                }
            }
        }
        
        suggestions
    }
    
    /// Generate performance-specific suggestions
    fn generate_performance_suggestions(&self, metrics: &CodeMetrics) -> Vec<SmartSuggestion> {
        let mut suggestions = Vec::new();
        
        // Deep nesting suggests potential performance issues
        if metrics.nesting_depth > 4 {
            suggestions.push(SmartSuggestion {
                category: SuggestionCategory::Performance,
                title: "Optimize Nested Loops".to_string(),
                description: format!(
                    "Deep nesting (depth: {}) may indicate inefficient algorithms.",
                    metrics.nesting_depth
                ),
                rationale: "Deeply nested code often has higher time complexity and can benefit from algorithmic improvements.".to_string(),
                confidence: 0.75,
                impact: ImpactLevel::Medium,
                effort_estimate: EffortLevel::Medium,
                priority_score: 6,
                code_example: None,
                related_patterns: vec!["algorithm_optimization".to_string()],
                applicable_files: vec![],
            });
        }
        
        suggestions
    }
    
    /// Get the total number of suggestion rules
    pub fn get_rule_count(&self) -> usize {
        self.rules.len() + self.language_specific_rules.values()
            .map(|rules| rules.len()).sum::<usize>()
    }
}

impl PatternDatabase {
    fn new() -> Self {
        Self {
            complexity_patterns: vec![
                ComplexityPattern {
                    name: "Deeply Nested Conditionals".to_string(),
                    description: "Multiple levels of if-else statements".to_string(),
                    detection_logic: "nesting_depth > 4".to_string(),
                    suggested_fix: "Extract methods or use guard clauses".to_string(),
                    example: None,
                },
                ComplexityPattern {
                    name: "Long Parameter Lists".to_string(),
                    description: "Functions with too many parameters".to_string(),
                    detection_logic: "parameter_count > 5".to_string(),
                    suggested_fix: "Use parameter objects or builder pattern".to_string(),
                    example: None,
                },
            ],
            security_patterns: vec![
                SecurityPattern {
                    name: "SQL Injection Risk".to_string(),
                    vulnerability_type: "Injection".to_string(),
                    risk_level: "High".to_string(),
                    detection_patterns: vec!["SELECT * FROM".to_string(), "INSERT INTO".to_string()],
                    mitigation: "Use parameterized queries".to_string(),
                },
                SecurityPattern {
                    name: "Unsafe Code Evaluation".to_string(),
                    vulnerability_type: "Code Injection".to_string(),
                    risk_level: "Critical".to_string(),
                    detection_patterns: vec!["eval(".to_string(), "exec(".to_string()],
                    mitigation: "Avoid dynamic code execution".to_string(),
                },
            ],
            performance_patterns: vec![
                PerformancePattern {
                    name: "Nested Loop Performance".to_string(),
                    performance_impact: "O(n²) or higher complexity".to_string(),
                    detection_criteria: "nesting_depth > 3".to_string(),
                    optimization_suggestion: "Use hash maps or other efficient data structures".to_string(),
                },
            ],
        }
    }
}

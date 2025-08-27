use std::collections::HashSet;
use std::path::Path;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use regex::Regex;

/// Comprehensive code metrics for complexity analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeMetrics {
    // Basic metrics
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub total_lines: usize,
    
    // Complexity metrics
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub nesting_depth: usize,
    pub halstead_metrics: HalsteadMetrics,
    
    // Structure metrics
    pub function_count: usize,
    pub class_count: usize,
    pub method_count: usize,
    pub variable_count: usize,
    
    // Quality metrics
    pub code_duplication: f64,
    pub maintainability_index: f64,
    pub technical_debt_ratio: f64,
    
    // Dependency metrics
    pub dependencies: Vec<String>,
    pub coupling_factor: f64,
    pub cohesion_factor: f64,
    
    // Language-specific metrics
    pub language: String,
    pub language_features: LanguageFeatures,
    
    // Function-level metrics
    pub functions: Vec<FunctionMetrics>,
    pub classes: Vec<ClassMetrics>,
}

/// Halstead complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HalsteadMetrics {
    pub distinct_operators: usize,
    pub distinct_operands: usize,
    pub total_operators: usize,
    pub total_operands: usize,
    pub vocabulary: usize,
    pub length: usize,
    pub calculated_length: f64,
    pub volume: f64,
    pub difficulty: f64,
    pub effort: f64,
    pub time_to_understand: f64,
    pub bugs_delivered: f64,
}

/// Language-specific features that affect complexity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LanguageFeatures {
    pub has_generics: bool,
    pub has_async: bool,
    pub has_macros: bool,
    pub has_lambdas: bool,
    pub has_exceptions: bool,
    pub has_reflection: bool,
    pub paradigm: ProgrammingParadigm,
}

/// Programming paradigm classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgrammingParadigm {
    Procedural,
    ObjectOriented,
    Functional,
    Mixed,
    Unknown,
}

impl Default for ProgrammingParadigm {
    fn default() -> Self {
        ProgrammingParadigm::Unknown
    }
}

/// Metrics for individual functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetrics {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub lines_of_code: usize,
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub parameter_count: usize,
    pub local_variable_count: usize,
    pub return_statements: usize,
    pub nesting_depth: usize,
    pub fan_in: usize,  // How many functions call this one
    pub fan_out: usize, // How many functions this one calls
    pub complexity_rating: ComplexityRating,
    pub suggestions: Vec<String>,
}

/// Metrics for classes/structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassMetrics {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub lines_of_code: usize,
    pub method_count: usize,
    pub field_count: usize,
    pub public_methods: usize,
    pub private_methods: usize,
    pub inheritance_depth: usize,
    pub coupling: usize,
    pub cohesion: f64,
    pub complexity_rating: ComplexityRating,
}

/// Complexity rating based on thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityRating {
    Simple,     // Green - Easy to understand and maintain
    Moderate,   // Yellow - Some complexity, manageable
    Complex,    // Orange - High complexity, needs attention
    Critical,   // Red - Very complex, refactor recommended
}

/// Main analyzer for code metrics
pub struct MetricsAnalyzer {
    // Language-specific analyzers
    rust_analyzer: RustAnalyzer,
    python_analyzer: PythonAnalyzer,
    javascript_analyzer: JavaScriptAnalyzer,
    java_analyzer: JavaAnalyzer,
    cpp_analyzer: CppAnalyzer,
    go_analyzer: GoAnalyzer,
    
    // Common patterns and keywords
    common_patterns: CommonPatterns,
}

/// Common code patterns across languages
struct CommonPatterns {
    pub control_flow_keywords: HashSet<String>,
    pub loop_keywords: HashSet<String>,
    pub conditional_keywords: HashSet<String>,
    pub exception_keywords: HashSet<String>,
    pub function_keywords: HashSet<String>,
    pub class_keywords: HashSet<String>,
}

impl MetricsAnalyzer {
    pub fn new() -> Self {
        Self {
            rust_analyzer: RustAnalyzer::new(),
            python_analyzer: PythonAnalyzer::new(),
            javascript_analyzer: JavaScriptAnalyzer::new(),
            java_analyzer: JavaAnalyzer::new(),
            cpp_analyzer: CppAnalyzer::new(),
            go_analyzer: GoAnalyzer::new(),
            common_patterns: CommonPatterns::new(),
        }
    }
    
    /// Analyze a file and return comprehensive metrics
    pub fn analyze_file(&self, file_path: &Path, content: &str) -> Result<CodeMetrics> {
        let language = self.detect_language(file_path);
        let mut metrics = CodeMetrics::default();
        metrics.language = language.clone();
        
        // Basic line-based metrics
        self.analyze_basic_metrics(&mut metrics, content);
        
        // Language-specific analysis
        match language.as_str() {
            "rust" | "rs" => self.rust_analyzer.analyze(&mut metrics, content)?,
            "python" | "py" => self.python_analyzer.analyze(&mut metrics, content)?,
            "javascript" | "js" => self.javascript_analyzer.analyze(&mut metrics, content)?,
            "java" => self.java_analyzer.analyze(&mut metrics, content)?,
            "cpp" | "c" | "cc" | "cxx" => self.cpp_analyzer.analyze(&mut metrics, content)?,
            "go" => self.go_analyzer.analyze(&mut metrics, content)?,
            _ => self.analyze_generic(&mut metrics, content)?,
        }
        
        // Calculate derived metrics
        self.calculate_derived_metrics(&mut metrics);
        
        // Rate complexity for functions and classes
        self.rate_complexity(&mut metrics);
        
        Ok(metrics)
    }
    
    /// Analyze basic metrics that apply to all languages
    fn analyze_basic_metrics(&self, metrics: &mut CodeMetrics, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        metrics.total_lines = lines.len();
        
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                metrics.blank_lines += 1;
            } else if self.is_comment_line(trimmed, &metrics.language) {
                metrics.comment_lines += 1;
            } else {
                metrics.lines_of_code += 1;
            }
        }
    }
    
    /// Calculate derived complexity metrics
    fn calculate_derived_metrics(&self, metrics: &mut CodeMetrics) {
        // Maintainability Index calculation
        // MI = 171 - 5.2 * ln(Halstead Volume) - 0.23 * (Cyclomatic Complexity) - 16.2 * ln(Lines of Code)
        if metrics.lines_of_code > 0 && metrics.halstead_metrics.volume > 0.0 {
            let mi = 171.0 
                - 5.2 * metrics.halstead_metrics.volume.ln()
                - 0.23 * (metrics.cyclomatic_complexity as f64)
                - 16.2 * (metrics.lines_of_code as f64).ln();
            metrics.maintainability_index = mi.max(0.0).min(100.0);
        }
        
        // Technical Debt Ratio
        // Based on complexity, duplication, and maintainability
        let complexity_factor = (metrics.cyclomatic_complexity as f64 / metrics.lines_of_code as f64 * 100.0).min(100.0);
        let maintainability_factor = 100.0 - metrics.maintainability_index;
        let duplication_factor = metrics.code_duplication * 100.0;
        
        metrics.technical_debt_ratio = (complexity_factor + maintainability_factor + duplication_factor) / 3.0;
        
        // Coupling and Cohesion factors
        if !metrics.functions.is_empty() {
            let total_fan_out: usize = metrics.functions.iter().map(|f| f.fan_out).sum();
            metrics.coupling_factor = total_fan_out as f64 / metrics.functions.len() as f64;
            
            // Simple cohesion estimate based on variable usage patterns
            metrics.cohesion_factor = self.estimate_cohesion(metrics);
        }
    }
    
    /// Rate the complexity of functions and classes
    fn rate_complexity(&self, metrics: &mut CodeMetrics) {
        for function in &mut metrics.functions {
            function.complexity_rating = self.rate_function_complexity(function);
            function.suggestions = self.generate_function_suggestions(function);
        }
        
        for class in &mut metrics.classes {
            class.complexity_rating = self.rate_class_complexity(class);
        }
    }
    
    /// Rate individual function complexity
    fn rate_function_complexity(&self, function: &FunctionMetrics) -> ComplexityRating {
        let mut score = 0;
        
        // Cyclomatic complexity scoring
        score += match function.cyclomatic_complexity {
            0..=5 => 0,
            6..=10 => 1,
            11..=20 => 2,
            _ => 3,
        };
        
        // Cognitive complexity scoring
        score += match function.cognitive_complexity {
            0..=5 => 0,
            6..=15 => 1,
            16..=25 => 2,
            _ => 3,
        };
        
        // Function length scoring
        score += match function.lines_of_code {
            0..=20 => 0,
            21..=50 => 1,
            51..=100 => 2,
            _ => 3,
        };
        
        // Parameter count scoring
        score += match function.parameter_count {
            0..=3 => 0,
            4..=6 => 1,
            7..=10 => 2,
            _ => 3,
        };
        
        // Nesting depth scoring
        score += match function.nesting_depth {
            0..=3 => 0,
            4..=5 => 1,
            6..=8 => 2,
            _ => 3,
        };
        
        match score {
            0..=3 => ComplexityRating::Simple,
            4..=8 => ComplexityRating::Moderate,
            9..=12 => ComplexityRating::Complex,
            _ => ComplexityRating::Critical,
        }
    }
    
    /// Rate class complexity
    fn rate_class_complexity(&self, class: &ClassMetrics) -> ComplexityRating {
        let mut score = 0;
        
        // Method count scoring
        score += match class.method_count {
            0..=10 => 0,
            11..=20 => 1,
            21..=30 => 2,
            _ => 3,
        };
        
        // Lines of code scoring
        score += match class.lines_of_code {
            0..=200 => 0,
            201..=500 => 1,
            501..=1000 => 2,
            _ => 3,
        };
        
        // Coupling scoring
        score += match class.coupling {
            0..=5 => 0,
            6..=10 => 1,
            11..=20 => 2,
            _ => 3,
        };
        
        // Cohesion scoring (inverted - lower cohesion = higher score)
        score += match class.cohesion {
            0.8..=1.0 => 0,
            0.6..=0.79 => 1,
            0.4..=0.59 => 2,
            _ => 3,
        };
        
        match score {
            0..=3 => ComplexityRating::Simple,
            4..=8 => ComplexityRating::Moderate,
            9..=12 => ComplexityRating::Complex,
            _ => ComplexityRating::Critical,
        }
    }
    
    /// Generate suggestions for improving function complexity
    fn generate_function_suggestions(&self, function: &FunctionMetrics) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if function.cyclomatic_complexity > 10 {
            suggestions.push("Consider breaking this function into smaller functions to reduce cyclomatic complexity".to_string());
        }
        
        if function.cognitive_complexity > 15 {
            suggestions.push("This function has high cognitive complexity. Consider simplifying the logic flow".to_string());
        }
        
        if function.lines_of_code > 50 {
            suggestions.push("This function is quite long. Consider extracting some logic into separate functions".to_string());
        }
        
        if function.parameter_count > 5 {
            suggestions.push("This function has many parameters. Consider using a parameter object or builder pattern".to_string());
        }
        
        if function.nesting_depth > 4 {
            suggestions.push("Deep nesting detected. Consider using early returns or extracting nested logic".to_string());
        }
        
        if function.return_statements > 5 {
            suggestions.push("Multiple return statements found. Consider consolidating return logic".to_string());
        }
        
        suggestions
    }
    
    /// Detect programming language from file extension
    fn detect_language(&self, file_path: &Path) -> String {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "rs" => "rust".to_string(),
                "py" => "python".to_string(),
                "js" => "javascript".to_string(),
                "ts" => "typescript".to_string(),
                "java" => "java".to_string(),
                "cpp" | "cc" | "cxx" => "cpp".to_string(),
                "c" => "c".to_string(),
                "cs" => "csharp".to_string(),
                "go" => "go".to_string(),
                "php" => "php".to_string(),
                "rb" => "ruby".to_string(),
                _ => "unknown".to_string(),
            }
        } else {
            "unknown".to_string()
        }
    }
    
    /// Check if a line is a comment
    fn is_comment_line(&self, line: &str, language: &str) -> bool {
        match language {
            "rust" | "javascript" | "typescript" | "java" | "cpp" | "c" | "csharp" | "go" => {
                line.starts_with("//") || line.starts_with("/*") || line.starts_with("*/") || line.starts_with("*")
            }
            "python" | "ruby" => {
                line.starts_with("#")
            }
            _ => false,
        }
    }
    
    /// Estimate cohesion based on variable usage patterns
    fn estimate_cohesion(&self, metrics: &CodeMetrics) -> f64 {
        if metrics.functions.is_empty() {
            return 1.0;
        }
        
        // Simple cohesion estimate based on shared variable usage
        // In a real implementation, this would analyze actual variable dependencies
        let avg_local_vars: f64 = metrics.functions.iter()
            .map(|f| f.local_variable_count as f64)
            .sum::<f64>() / metrics.functions.len() as f64;
        
        let total_vars = metrics.variable_count as f64;
        
        if total_vars > 0.0 {
            (avg_local_vars / total_vars).min(1.0)
        } else {
            1.0
        }
    }
    
    /// Generic analysis for unknown languages
    fn analyze_generic(&self, metrics: &mut CodeMetrics, content: &str) -> Result<()> {
        // Basic pattern matching for common constructs
        let function_patterns = [
            r"function\s+\w+",
            r"def\s+\w+",
            r"fn\s+\w+",
            r"\w+\s*\([^)]*\)\s*\{",
        ];
        
        let class_patterns = [
            r"class\s+\w+",
            r"struct\s+\w+",
            r"interface\s+\w+",
        ];
        
        for pattern in &function_patterns {
            if let Ok(re) = Regex::new(pattern) {
                metrics.function_count += re.find_iter(content).count();
            }
        }
        
        for pattern in &class_patterns {
            if let Ok(re) = Regex::new(pattern) {
                metrics.class_count += re.find_iter(content).count();
            }
        }
        
        // Estimate cyclomatic complexity based on control flow keywords
        let control_keywords = ["if", "else", "while", "for", "switch", "case", "catch", "&&", "||"];
        for keyword in &control_keywords {
            metrics.cyclomatic_complexity += content.matches(keyword).count();
        }
        
        metrics.cyclomatic_complexity = metrics.cyclomatic_complexity.max(1);
        
        Ok(())
    }
}

impl CommonPatterns {
    fn new() -> Self {
        let mut control_flow = HashSet::new();
        control_flow.insert("if".to_string());
        control_flow.insert("else".to_string());
        control_flow.insert("elif".to_string());
        control_flow.insert("switch".to_string());
        control_flow.insert("case".to_string());
        control_flow.insert("match".to_string());
        
        let mut loops = HashSet::new();
        loops.insert("for".to_string());
        loops.insert("while".to_string());
        loops.insert("loop".to_string());
        loops.insert("do".to_string());
        
        let mut conditionals = HashSet::new();
        conditionals.insert("&&".to_string());
        conditionals.insert("||".to_string());
        conditionals.insert("and".to_string());
        conditionals.insert("or".to_string());
        
        let mut exceptions = HashSet::new();
        exceptions.insert("try".to_string());
        exceptions.insert("catch".to_string());
        exceptions.insert("except".to_string());
        exceptions.insert("finally".to_string());
        exceptions.insert("raise".to_string());
        exceptions.insert("throw".to_string());
        
        let mut functions = HashSet::new();
        functions.insert("function".to_string());
        functions.insert("def".to_string());
        functions.insert("fn".to_string());
        functions.insert("func".to_string());
        
        let mut classes = HashSet::new();
        classes.insert("class".to_string());
        classes.insert("struct".to_string());
        classes.insert("interface".to_string());
        classes.insert("trait".to_string());
        
        Self {
            control_flow_keywords: control_flow,
            loop_keywords: loops,
            conditional_keywords: conditionals,
            exception_keywords: exceptions,
            function_keywords: functions,
            class_keywords: classes,
        }
    }
}

// Language-specific analyzers (we'll implement these next)
struct RustAnalyzer;
struct PythonAnalyzer;
struct JavaScriptAnalyzer;
struct JavaAnalyzer;
struct CppAnalyzer;
struct GoAnalyzer;

impl RustAnalyzer {
    fn new() -> Self { Self }
    
    fn analyze(&self, metrics: &mut CodeMetrics, content: &str) -> Result<()> {
        // Rust-specific analysis
        metrics.language_features.has_generics = content.contains("<") && content.contains(">");
        metrics.language_features.has_async = content.contains("async") || content.contains("await");
        metrics.language_features.has_macros = content.contains("!");
        metrics.language_features.paradigm = ProgrammingParadigm::Mixed;
        
        // Count functions
        let fn_regex = Regex::new(r"fn\s+(\w+)")?;
        for cap in fn_regex.captures_iter(content) {
            let function_name = cap[1].to_string();
            let function_metrics = self.analyze_rust_function(content, &function_name)?;
            metrics.functions.push(function_metrics);
        }
        metrics.function_count = metrics.functions.len();
        
        // Count structs and enums
        let struct_regex = Regex::new(r"(?:struct|enum)\s+(\w+)")?;
        for cap in struct_regex.captures_iter(content) {
            let class_name = cap[1].to_string();
            let class_metrics = self.analyze_rust_struct(content, &class_name)?;
            metrics.classes.push(class_metrics);
        }
        metrics.class_count = metrics.classes.len();
        
        // Calculate cyclomatic complexity
        self.calculate_rust_complexity(metrics, content)?;
        
        // Calculate Halstead metrics
        self.calculate_halstead_metrics(metrics, content)?;
        
        Ok(())
    }
    
    fn analyze_rust_function(&self, _content: &str, name: &str) -> Result<FunctionMetrics> {
        // Simplified function analysis - in a real implementation,
        // this would parse the AST and analyze the function body
        Ok(FunctionMetrics {
            name: name.to_string(),
            start_line: 0,
            end_line: 0,
            lines_of_code: 10, // Placeholder
            cyclomatic_complexity: 1,
            cognitive_complexity: 1,
            parameter_count: 0,
            local_variable_count: 0,
            return_statements: 1,
            nesting_depth: 1,
            fan_in: 0,
            fan_out: 0,
            complexity_rating: ComplexityRating::Simple,
            suggestions: Vec::new(),
        })
    }
    
    fn analyze_rust_struct(&self, _content: &str, name: &str) -> Result<ClassMetrics> {
        Ok(ClassMetrics {
            name: name.to_string(),
            start_line: 0,
            end_line: 0,
            lines_of_code: 5, // Placeholder
            method_count: 0,
            field_count: 0,
            public_methods: 0,
            private_methods: 0,
            inheritance_depth: 0,
            coupling: 0,
            cohesion: 1.0,
            complexity_rating: ComplexityRating::Simple,
        })
    }
    
    fn calculate_rust_complexity(&self, metrics: &mut CodeMetrics, content: &str) -> Result<()> {
        // Control flow keywords that add to complexity
        let keywords = ["if", "else if", "match", "while", "for", "loop"];
        
        for keyword in &keywords {
            metrics.cyclomatic_complexity += content.matches(keyword).count();
        }
        
        // Pattern matching arms add complexity
        let match_arms = Regex::new(r"=>\s*[^,}]+")?;
        metrics.cyclomatic_complexity += match_arms.find_iter(content).count();
        
        // Ensure minimum complexity of 1
        metrics.cyclomatic_complexity = metrics.cyclomatic_complexity.max(1);
        
        Ok(())
    }
    
    fn calculate_halstead_metrics(&self, metrics: &mut CodeMetrics, content: &str) -> Result<()> {
        // Simplified Halstead calculation
        // In a real implementation, this would use proper tokenization
        
        let operators = ["=", "+", "-", "*", "/", "%", "&&", "||", "!", "==", "!=", "<", ">", "<=", ">="];
        let mut operator_count = 0;
        let mut distinct_operators = HashSet::new();
        
        for op in &operators {
            let count = content.matches(op).count();
            if count > 0 {
                distinct_operators.insert(op.to_string());
                operator_count += count;
            }
        }
        
        // Count identifiers (simplified)
        let identifier_regex = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b")?;
        let identifiers: Vec<&str> = identifier_regex.find_iter(content)
            .map(|m| m.as_str())
            .collect();
        
        let distinct_operands: HashSet<String> = identifiers.iter()
            .map(|s| s.to_string())
            .collect();
        
        metrics.halstead_metrics = HalsteadMetrics {
            distinct_operators: distinct_operators.len(),
            distinct_operands: distinct_operands.len(),
            total_operators: operator_count,
            total_operands: identifiers.len(),
            vocabulary: distinct_operators.len() + distinct_operands.len(),
            length: operator_count + identifiers.len(),
            calculated_length: 0.0, // Will be calculated
            volume: 0.0,
            difficulty: 0.0,
            effort: 0.0,
            time_to_understand: 0.0,
            bugs_delivered: 0.0,
        };
        
        // Calculate derived Halstead metrics
        let vocab = metrics.halstead_metrics.vocabulary as f64;
        let length = metrics.halstead_metrics.length as f64;
        
        if vocab > 0.0 {
            metrics.halstead_metrics.volume = length * vocab.log2();
            
            if metrics.halstead_metrics.distinct_operands > 0 {
                let difficulty = (metrics.halstead_metrics.distinct_operators as f64 / 2.0) * 
                    (metrics.halstead_metrics.total_operands as f64 / metrics.halstead_metrics.distinct_operands as f64);
                metrics.halstead_metrics.difficulty = difficulty;
                metrics.halstead_metrics.effort = difficulty * metrics.halstead_metrics.volume;
                metrics.halstead_metrics.time_to_understand = metrics.halstead_metrics.effort / 18.0;
                metrics.halstead_metrics.bugs_delivered = metrics.halstead_metrics.volume / 3000.0;
            }
        }
        
        Ok(())
    }
}

// Placeholder implementations for other language analyzers
impl PythonAnalyzer {
    fn new() -> Self { Self }
    fn analyze(&self, metrics: &mut CodeMetrics, content: &str) -> Result<()> {
        metrics.language_features.paradigm = ProgrammingParadigm::Mixed;
        metrics.language_features.has_lambdas = content.contains("lambda");
        metrics.language_features.has_async = content.contains("async") || content.contains("await");
        
        // Count functions and classes using Python syntax
        let function_regex = Regex::new(r"def\s+(\w+)")?;
        metrics.function_count = function_regex.find_iter(content).count();
        
        let class_regex = Regex::new(r"class\s+(\w+)")?;
        metrics.class_count = class_regex.find_iter(content).count();
        
        // Basic complexity calculation
        let keywords = ["if", "elif", "else", "while", "for", "try", "except", "and", "or"];
        for keyword in &keywords {
            metrics.cyclomatic_complexity += content.matches(keyword).count();
        }
        metrics.cyclomatic_complexity = metrics.cyclomatic_complexity.max(1);
        
        Ok(())
    }
}

impl JavaScriptAnalyzer {
    fn new() -> Self { Self }
    fn analyze(&self, metrics: &mut CodeMetrics, content: &str) -> Result<()> {
        metrics.language_features.paradigm = ProgrammingParadigm::Mixed;
        metrics.language_features.has_lambdas = content.contains("=>") || content.contains("function");
        metrics.language_features.has_async = content.contains("async") || content.contains("await");
        
        let function_regex = Regex::new(r"(?:function\s+(\w+)|(\w+)\s*=\s*(?:async\s+)?(?:function|\([^)]*\)\s*=>))")?;
        metrics.function_count = function_regex.find_iter(content).count();
        
        let class_regex = Regex::new(r"class\s+(\w+)")?;
        metrics.class_count = class_regex.find_iter(content).count();
        
        let keywords = ["if", "else", "while", "for", "switch", "case", "try", "catch", "&&", "||"];
        for keyword in &keywords {
            metrics.cyclomatic_complexity += content.matches(keyword).count();
        }
        metrics.cyclomatic_complexity = metrics.cyclomatic_complexity.max(1);
        
        Ok(())
    }
}

impl JavaAnalyzer {
    fn new() -> Self { Self }
    fn analyze(&self, metrics: &mut CodeMetrics, content: &str) -> Result<()> {
        metrics.language_features.paradigm = ProgrammingParadigm::ObjectOriented;
        metrics.language_features.has_generics = content.contains("<") && content.contains(">");
        metrics.language_features.has_exceptions = content.contains("try") || content.contains("catch");
        
        // Similar implementation to other analyzers
        Ok(())
    }
}

impl CppAnalyzer {
    fn new() -> Self { Self }
    fn analyze(&self, metrics: &mut CodeMetrics, content: &str) -> Result<()> {
        metrics.language_features.paradigm = ProgrammingParadigm::Mixed;
        metrics.language_features.has_generics = content.contains("template");
        
        // C++ specific analysis
        Ok(())
    }
}

impl GoAnalyzer {
    fn new() -> Self { Self }
    fn analyze(&self, metrics: &mut CodeMetrics, _content: &str) -> Result<()> {
        metrics.language_features.paradigm = ProgrammingParadigm::Procedural;
        
        // Go specific analysis
        Ok(())
    }
}

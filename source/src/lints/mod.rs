//! Lint rules and explanations
//!
//! This module provides definitions and explanations for lint rules.

pub mod explanations;

use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a collection of lint rules by language
pub struct LintRules {
    /// Mapping of language identifiers to rules
    pub rules_by_language: HashMap<String, Vec<LintRule>>,
}

impl LintRules {
    /// Create a new collection of lint rules
    pub fn new() -> Self {
        let mut rules = Self {
            rules_by_language: HashMap::new(),
        };
        
        // Initialize with standard rules
        rules.initialize_standard_rules();
        
        rules
    }
    
    /// Initialize standard rules for all supported languages
    fn initialize_standard_rules(&mut self) {
        // Add rust rules
        self.rules_by_language.insert("rust".to_string(), vec![
            explanations::RUST_UNUSED_VARIABLE.clone(),
            explanations::RUST_UNUSED_IMPORT.clone(),
            explanations::RUST_UNUSED_MUST_USE.clone(),
            explanations::RUST_DEAD_CODE.clone(),
        ]);
        
        // Add javascript rules
        self.rules_by_language.insert("javascript".to_string(), vec![
            explanations::JS_UNUSED_VARIABLE.clone(),
            explanations::JS_NO_VAR.clone(),
            explanations::JS_PREFER_CONST.clone(),
            explanations::JS_EQEQEQ.clone(),
        ]);
        
        // Add python rules
        self.rules_by_language.insert("python".to_string(), vec![
            explanations::PY_UNUSED_VARIABLE.clone(),
            explanations::PY_UNDEFINED_NAME.clone(),
            explanations::PY_MISSING_DOCSTRING.clone(),
            explanations::PY_LINE_TOO_LONG.clone(),
        ]);
    }
    
    /// Get rules for a specific language
    pub fn get_rules_for_language(&self, language: &str) -> Option<&Vec<LintRule>> {
        self.rules_by_language.get(language)
    }
    
    /// Find rule by code
    pub fn find_rule_by_code(&self, code: &str) -> Option<&LintRule> {
        for rules in self.rules_by_language.values() {
            if let Some(rule) = rules.iter().find(|r| r.code == code) {
                return Some(rule);
            }
        }
        None
    }
}

/// Lint rule with explanation
#[derive(Debug, Clone)]
pub struct LintRule {
    /// Unique rule code
    pub code: String,
    
    /// Rule name
    pub name: String,
    
    /// Brief description
    pub description: String,
    
    /// Severity level
    pub severity: LintSeverity,
    
    /// Full explanation
    pub explanation: String,
    
    /// Example of incorrect code
    pub incorrect_example: String,
    
    /// Example of correct code
    pub correct_example: String,
    
    /// Link to documentation
    pub doc_link: Option<String>,
    
    /// Common fix patterns
    pub common_fixes: Vec<String>,
    
    /// Severity rationale
    pub severity_rationale: String,
}

/// Lint severity level
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintSeverity {
    /// Informational message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
}


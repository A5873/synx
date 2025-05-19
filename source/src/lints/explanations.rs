//! Explanations for lint rules
//!
//! This module provides detailed explanations for all lint rules supported by the system.

use super::LintRule;
use super::LintSeverity;
use lazy_static::lazy_static;

/// Create a formatted explanation string with sections
fn format_explanation(
    description: &str,
    details: &str,
    why_important: &str,
    how_to_fix: &str,
) -> String {
    format!(
        "## Description\n{}\n\n## Details\n{}\n\n## Why is this important?\n{}\n\n## How to fix\n{}",
        description, details, why_important, how_to_fix
    )
}

// Rust lint rules
lazy_static! {
    /// Unused variable in Rust
    pub static ref RUST_UNUSED_VARIABLE: LintRule = LintRule {
        code: "R0001".to_string(),
        name: "unused_variable".to_string(),
        description: "Variable is defined but never used".to_string(),
        severity: LintSeverity::Warning,
        explanation: format_explanation(
            "This lint is triggered when a variable is declared but never used in the code.",
            "In Rust, unused variables are generally considered a code smell and might indicate redundant code or a potential bug where you intended to use the variable but didn't.",
            "Unused variables increase code complexity without adding value. They can make code harder to understand and maintain, as the reader may wonder about the intended purpose of these variables.",
            "You can fix this by:\n- Removing the unused variable\n- Using the variable where needed\n- Prefixing the variable name with underscore (_) to explicitly mark it as unused\n- Using the variable in debug assertions or logging",
        ),
        incorrect_example: r#"
fn process_data(data: &str) {
    let processed = data.trim();  // 'processed' is never used
    println!("Processing completed");
}
"#.to_string(),
        correct_example: r#"
fn process_data(data: &str) {
    let processed = data.trim();
    println!("Processing completed: {}", processed);
}

// OR, if truly unused
fn process_data(data: &str) {
    let _processed = data.trim();  // Underscore prefix indicates intentionally unused
    println!("Processing completed");
}

// OR, simply remove it
fn process_data(data: &str) {
    println!("Processing completed");
}
"#.to_string(),
        doc_link: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-variables".to_string()),
        common_fixes: vec![
            "Remove the variable declaration".to_string(),
            "Use the variable in your code".to_string(),
            "Prefix the variable name with an underscore (_)".to_string(),
        ],
        severity_rationale: "This is a warning rather than an error because unused variables don't prevent code from running correctly, but they do indicate potential issues or inefficient code design.".to_string(),
    };

    /// Unused import in Rust
    pub static ref RUST_UNUSED_IMPORT: LintRule = LintRule {
        code: "R0002".to_string(),
        name: "unused_import".to_string(),
        description: "Imported item is never used".to_string(),
        severity: LintSeverity::Warning,
        explanation: format_explanation(
            "This lint triggers when an imported item (module, function, type, etc.) is not used in the code.",
            "Rust's module system allows importing specific items from other modules or crates. When an import is never used, it's considered unnecessary and should be removed.",
            "Unused imports clutter your code and can potentially slow down compilation. They can also mislead other developers about the actual dependencies of your code.",
            "You can fix this by removing the unused import, or if you plan to use it in the future, consider commenting it out with a TODO note.",
        ),
        incorrect_example: r#"
use std::collections::HashMap;  // This is never used
use std::fs::File;

fn main() {
    let file = File::open("data.txt").unwrap();
    // No HashMap is used
}
"#.to_string(),
        correct_example: r#"
use std::fs::File;

fn main() {
    let file = File::open("data.txt").unwrap();
}
"#.to_string(),
        doc_link: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-imports".to_string()),
        common_fixes: vec![
            "Remove the unused import".to_string(),
            "Comment out the import if you plan to use it soon".to_string(),
        ],
        severity_rationale: "This is a warning because unused imports don't affect runtime behavior, but they do make code less maintainable and can slow down compilation.".to_string(),
    };

    /// Unused #[must_use] result in Rust
    pub static ref RUST_UNUSED_MUST_USE: LintRule = LintRule {
        code: "R0003".to_string(),
        name: "unused_must_use".to_string(),
        description: "Return value of a #[must_use] function is discarded".to_string(),
        severity: LintSeverity::Warning,
        explanation: format_explanation(
            "This lint is triggered when you call a function marked with #[must_use] but ignore its return value.",
            "The #[must_use] attribute indicates that the function's return value is important and should not be ignored. This is often used for functions where discarding the return value is almost certainly a mistake.",
            "Functions marked as #[must_use] are designed this way because ignoring their return values typically leads to bugs. For example, ignoring the Result from a file operation means you won't detect or handle errors.",
            "You can fix this by:\n- Assigning the return value to a variable\n- Using the return value in an expression\n- Explicitly calling .unwrap() or similar to handle Results\n- Using let _ = ... if you're intentionally discarding the value",
        ),
        incorrect_example: r#"
fn main() {
    // Result is a #[must_use] type
    std::fs::File::open("file.txt");  // Error is ignored!
}
"#.to_string(),
        correct_example: r#"
fn main() {
    // Handling the result properly
    match std::fs::File::open("file.txt") {
        Ok(file) => { /* use file */ },
        Err(e) => println!("Error opening file: {}", e),
    }
    
    // Or if you really want to ignore it:
    let _ = std::fs::File::open("file.txt");  // Explicitly discarding
}
"#.to_string(),
        doc_link: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-must-use".to_string()),
        common_fixes: vec![
            "Handle the return value with a match or if let".to_string(),
            "Use .unwrap() or .expect() (where appropriate)".to_string(),
            "Explicitly discard with let _ = ...".to_string(),
        ],
        severity_rationale: "This is a warning because ignoring #[must_use] values often leads to bugs, but sometimes deliberate ignoring is acceptable when explicitly acknowledged.".to_string(),
    };

    /// Dead code in Rust
    pub static ref RUST_DEAD_CODE: LintRule = LintRule {
        code: "R0004".to_string(),
        name: "dead_code".to_string(),
        description: "Code is never used or reachable".to_string(),
        severity: LintSeverity::Warning,
        explanation: format_explanation(
            "This lint is triggered when there's code in your program that can never be executed - functions that are never called, unreachable branches, etc.",
            "Dead code can take various forms: unused functions, methods, or entire modules; code after a return statement; unreachable branches due to constant conditionals; etc.",
            "Dead code increases the size of your codebase without providing any benefit. It requires maintenance, can confuse readers, and indicates potential logical errors in your program design.",
            "You can fix this by removing the dead code, refactoring to make it reachable, or using #[allow(dead_code)] to mark intentionally unused code (for example, code that will be used in future development).",
        ),
        incorrect_example: r#"
fn main() {
    println!("Hello, world!");
    return;
    println!("This will never be printed");  // Dead code!
}

fn unused_function() {  // Dead code!
    println!("This function is never called");
}
"#.to_string(),
        correct_example: r#"
fn main() {
    println!("Hello, world!");
}

// If needed for future development:
#[allow(dead_code)]
fn unused_function() {
    println!("This function is not used yet but will be later");
}
"#.to_string(),
        doc_link: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#dead-code".to_string()),
        common_fixes: vec![
            "Remove the unused code".to_string(),
            "Make the code reachable".to_string(),
            "Add #[allow(dead_code)] for intentional cases".to_string(),
        ],
        severity_rationale: "This is a warning rather than an error because while dead code doesn't cause runtime issues, it indicates potential design problems and increases maintenance burden.".to_string(),
    };
}

// JavaScript lint rules
lazy_static! {
    /// Unused variable in JavaScript
    pub static ref JS_UNUSED_VARIABLE: LintRule = LintRule {
        code: "J0001".to_string(),
        name: "no-unused-vars".to_string(),
        description: "Variable is defined but never used".to_string(),
        severity: LintSeverity::Warning,
        explanation: format_explanation(
            "This lint triggers when a variable, function, or function parameter is declared but not used anywhere in the code.",
            "In JavaScript, unused variables are often a sign of incomplete refactoring, copy-paste errors, or simply forgotten code. They can also indicate potential bugs where you intended to use a variable but didn't.",
            "Unused variables pollute the global namespace, make code harder to understand, and can lead to memory usage issues in some environments. They make maintenance more difficult by creating noise in the codebase.",
            "You can fix this by removing the unused variable, using it where needed, or prefixing it with an underscore to indicate it's intentionally unused (if your lint configuration allows this convention).",
        ),
        incorrect_example: r#"
function process(data, config) {
    const processed = data.trim();  // 'processed' is never used
    return data.length;  // 'config' parameter is never used
}
"#.to_string(),
        correct_example: r#"
function process(data) {  // Removed unused parameter
    return data.length;
}

// If you need the parameter for API compatibility:
function process(data, _config) {  // Underscore prefix
    return data.length;
}
"#.to_string(),
        doc_link: Some("https://eslint.org/docs/rules/no-unused-vars".to_string()),
        common_fixes: vec![
            "Remove the unused variable or parameter".to_string(),
            "Use the variable in your code".to_string(),
            "Prefix parameter names with underscore".to_string(),
        ],
        severity_rationale: "This is a warning because unused variables don't break functionality but do indicate poor code quality and potential bugs.".to_string(),
    };

    /// Using var in JavaScript
    pub static ref JS_NO_VAR: LintRule = LintRule {
        code: "J0002".to_string(),
        name: "no-var".to_string(),
        description: "Unexpected var, use let or const instead".to_string(),
        severity:


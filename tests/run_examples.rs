//! Test script to validate all example files
//! 
//! This script tests:
//! 1. All valid examples should pass validation
//! 2. All invalid examples should fail validation

use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

const EXAMPLES_DIR: &str = "examples";

/// Test case structure
struct TestCase {
    file_path: PathBuf,
    should_pass: bool,
    description: String,
}

/// Run all tests and report results
fn main() {
    println!("Running validation tests on example files...\n");
    
    let test_cases = collect_test_cases();
    let mut pass_count = 0;
    let mut fail_count = 0;
    
    for (i, test_case) in test_cases.iter().enumerate() {
        println!("Test #{}: {}", i + 1, test_case.description);
        println!("  File: {:?}", test_case.file_path);
        println!("  Expected: {}", if test_case.should_pass { "PASS" } else { "FAIL" });
        
        // Run synx on the file
        let result = run_validation(&test_case.file_path);
        let passed = (result.success && test_case.should_pass) || 
                    (!result.success && !test_case.should_pass);
        
        if passed {
            println!("  Result: ✅ TEST PASSED");
            pass_count += 1;
        } else {
            println!("  Result: ❌ TEST FAILED");
            println!("  Output: {}", result.output);
            fail_count += 1;
        }
        println!("------------------------");
    }
    
    // Print summary
    println!("\nTest Summary:");
    println!("  Total tests: {}", test_cases.len());
    println!("  Passed: {}", pass_count);
    println!("  Failed: {}", fail_count);
    
    if fail_count > 0 {
        println!("\n❌ Some tests failed!");
        std::process::exit(1);
    } else {
        println!("\n✅ All tests passed!");
    }
}

/// Validation result
struct ValidationResult {
    success: bool,
    output: String,
}

/// Run validation on a file
fn run_validation(file_path: &Path) -> ValidationResult {
    let output = Command::new("cargo")
        .args(["run", "--", "--verbose", file_path.to_str().unwrap()])
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined_output = format!("{}\n{}", stdout, stderr);
            
            ValidationResult {
                success: output.status.success(),
                output: combined_output,
            }
        },
        Err(e) => ValidationResult {
            success: false,
            output: format!("Failed to run synx: {}", e),
        },
    }
}

/// Collect all test cases from the examples directory
fn collect_test_cases() -> Vec<TestCase> {
    let mut test_cases = Vec::new();
    
    // Check if examples directory exists
    let examples_dir = PathBuf::from(EXAMPLES_DIR);
    if !examples_dir.exists() || !examples_dir.is_dir() {
        eprintln!("Examples directory not found: {}", EXAMPLES_DIR);
        return test_cases;
    }
    
    // Walk through subdirectories
    for entry in fs::read_dir(&examples_dir).unwrap() {
        if let Ok(entry) = entry {
            let type_dir = entry.path();
            if type_dir.is_dir() {
                let file_type = type_dir.file_name().unwrap().to_string_lossy().to_string();
                
                // Look for valid and invalid examples
                for file_name in ["valid", "invalid"] {
                    let base_name = format!("{}.{}", file_name, get_extension(&file_type));
                    let file_path = type_dir.join(&base_name);
                    
                    // Special case for Dockerfile
                    if file_type == "docker" {
                        let docker_path = type_dir.join(format!("{}.Dockerfile", file_name));
                        if docker_path.exists() {
                            test_cases.push(TestCase {
                                file_path: docker_path,
                                should_pass: file_name == "valid",
                                description: format!("{} {} example", file_type, file_name),
                            });
                            continue;
                        }
                    }
                    
                    // Add test case if file exists
                    if file_path.exists() {
                        test_cases.push(TestCase {
                            file_path,
                            should_pass: file_name == "valid",
                            description: format!("{} {} example", file_type, file_name),
                        });
                    }
                }
            }
        }
    }
    
    test_cases
}

/// Get file extension for file type
fn get_extension(file_type: &str) -> String {
    match file_type {
        "python" => "py",
        "javascript" => "js",
        "html" => "html",
        "css" => "css",
        "json" => "json",
        "yaml" => "yaml",
        "toml" => "toml",
        "shell" => "sh",
        "markdown" => "md",
        "rust" => "rs",
        "docker" => "Dockerfile",
        _ => file_type,
    }.to_string()
}


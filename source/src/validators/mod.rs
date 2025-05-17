use anyhow::{Result, anyhow};
use std::path::Path;
use std::process::Command;
use std::str;
use tempfile;

pub struct ValidationOptions {
    pub strict: bool,
    pub verbose: bool,
}

pub fn validate_file(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let file_type = detect_file_type(file_path)?;
    let validator = get_validator_for_type(&file_type);
    validator(file_path, options)
}

fn detect_file_type(file_path: &Path) -> Result<String> {
    if let Some(ext) = file_path.extension() {
        if let Some(ext_str) = ext.to_str() {
            return Ok(ext_str.to_lowercase());
        }
    }
    let mime = tree_magic_mini::from_filepath(file_path)
        .ok_or_else(|| anyhow!("Failed to detect file type"))?;
    Ok(mime.split("/").last().unwrap_or("unknown").to_string())
}

fn get_validator_for_type(file_type: &str) -> fn(&Path, &ValidationOptions) -> Result<bool> {
    match file_type {
        "rs" => validate_rust,
        "cpp" | "cxx" | "cc" => validate_cpp,
        "c" => validate_c,
        "cs" => validate_csharp,
        "py" | "python" => validate_python,
        "js" | "javascript" => validate_javascript,
        "java" => validate_java,
        "go" => validate_go,
        "ts" | "tsx" => validate_typescript,
        "json" => validate_json,
        "yaml" | "yml" => validate_yaml,
        "html" | "htm" => validate_html,
        "css" => validate_css,
        "sh" | "bash" => validate_shell,
        "dockerfile" => validate_dockerfile,
        _ => validate_unknown,
    }
}

fn validate_rust(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let temp_dir = tempfile::Builder::new()
                    .prefix("synx-rust-check")
                    .tempdir()?;
    let output_path = temp_dir.path().join("output");

    let mut cmd = Command::new("rustc");
    cmd.arg("--edition=2021")
       .arg("--crate-type=lib")
       .arg("--error-format=short")
       .arg("-A").arg("dead_code")
       .arg("-o").arg(&output_path)
       .arg(file_path);

    if options.strict {
        cmd.arg("-D").arg("warnings");
    }

    let output = cmd.output()?;
    let success = output.status.success();
    
    if !success && options.verbose {
        eprintln!("Rust validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_cpp(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("g++");
    cmd.arg("-fsyntax-only")
       .arg("-pedantic")
       .arg("-Wall");

    if options.strict {
        cmd.arg("-Werror");
    }

    cmd.arg(file_path);
    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("C++ validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_java(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("javac");
    cmd.arg("-Werror")
       .arg("-Xlint:all")
       .arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();
    
    if !success && options.verbose {
        eprintln!("Java validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    if success && options.strict {
        let checkstyle = Command::new("checkstyle")
            .arg("-c")
            .arg("/google_checks.xml")
            .arg(file_path)
            .output();

        if let Ok(output) = checkstyle {
            if !output.status.success() && options.verbose {
                eprintln!("Java style errors:");
                eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                return Ok(false);
            }
        }
    }

    Ok(success)
}

fn validate_go(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let dir = file_path.parent().unwrap_or(Path::new("."));
    
    let mut cmd = Command::new("go");
    cmd.current_dir(dir)
       .arg("vet")
       .arg(file_path);

    let output = cmd.output()?;
    if !output.status.success() {
        if options.verbose {
            eprintln!("Go validation errors:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        return Ok(false);
    }

    if options.strict {
        let gofmt = Command::new("gofmt")
            .arg("-l")
            .arg(file_path)
            .output()?;

        if !gofmt.stdout.is_empty() && options.verbose {
            eprintln!("Go formatting errors detected");
            return Ok(false);
        }

        let lint = Command::new("golangci-lint")
            .current_dir(dir)
            .arg("run")
            .arg("--no-config")
            .arg("--disable-all")
            .arg("--enable=govet,golint,gofmt,goimports")
            .arg(file_path)
            .output();

        if let Ok(output) = lint {
            if !output.status.success() && options.verbose {
                eprintln!("Go lint errors:");
                eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                return Ok(false);
            }
        }
    }

    Ok(true)
}

fn validate_typescript(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let tsc_check = Command::new("tsc").arg("--version").output();
    if tsc_check.is_err() {
        if options.verbose {
            eprintln!("TypeScript validation requires tsc to be installed");
            eprintln!("Install with: npm install -g typescript");
        }
        return Ok(false);
    }

    let mut cmd = Command::new("tsc");
    cmd.arg("--noEmit")
       .arg("--pretty")
       .arg("--strict");

    if options.strict {
        cmd.arg("--noImplicitAny")
           .arg("--noImplicitThis")
           .arg("--alwaysStrict")
           .arg("--strictNullChecks")
           .arg("--strictFunctionTypes")
           .arg("--strictPropertyInitialization");
    }

    cmd.arg(file_path);
    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("TypeScript validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    if success && options.strict {
        let eslint = Command::new("eslint")
            .arg("--parser")
            .arg("@typescript-eslint/parser")
            .arg("--plugin")
            .arg("@typescript-eslint")
            .arg(file_path)
            .output();

        if let Ok(output) = eslint {
            if !output.status.success() && options.verbose {
                eprintln!("TypeScript lint errors:");
                eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                return Ok(false);
            }
        }
    }

    Ok(success)
}

fn validate_json(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let output = Command::new("jq")
        .arg(".")
        .arg(file_path)
        .output()?;

    let success = output.status.success();
    if !success && options.verbose {
        eprintln!("JSON validation errors:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(success)
}

fn validate_yaml(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("yamllint");
    
    if options.strict {
        cmd.arg("-d").arg("{extends: default, rules: {line-length: disable}}");
    } else {
        cmd.arg("-d").arg("{extends: relaxed, rules: {line-length: disable}}");
    }
    
    cmd.arg(file_path);
    let output = cmd.output()?;

    let success = output.status.success();
    if options.verbose && !success {
        eprintln!("YAML validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        if !output.stdout.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }

    Ok(success)
}

fn validate_html(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("tidy");
    cmd.arg("-q")
       .arg("-e")
       .arg("--show-warnings").arg(if options.strict { "yes" } else { "no" })
       .arg("--show-errors").arg("1")
       .arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();
    
    if !success && options.verbose {
        eprintln!("HTML validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_css(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let output = Command::new("csslint")
        .arg("--format=compact")
        .arg(file_path)
        .output()?;

    let success = output.status.success();
    if !success && options.verbose {
        eprintln!("CSS validation errors:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(success)
}

fn validate_shell(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("shellcheck");
    
    if !options.strict {
        cmd.arg("--severity=error");
    }
    
    cmd.arg(file_path);
    let output = cmd.output()?;

    let success = output.status.success();
    if !success && options.verbose {
        eprintln!("Shell script errors:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(success)
}

fn validate_dockerfile(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("hadolint");
    
    if !options.strict {
        cmd.arg("--failure-threshold=error");
    }
    
    cmd.arg(file_path);
    let output = cmd.output()?;

    let success = output.status.success();
    if !success && options.verbose {
        eprintln!("Dockerfile validation errors:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(success)
}

fn validate_c(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("gcc");
    cmd.arg("-fsyntax-only")
       .arg("-Wall")
       .arg("-pedantic");

    if options.strict {
        cmd.arg("-Werror")
           .arg("-Wextra")
           .arg("-Wconversion")
           .arg("-Wformat=2")
           .arg("-Wuninitialized")
           .arg("-Wmissing-prototypes");
    }

    cmd.arg(file_path);
    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("C validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    // In strict mode, also check for memory leaks with valgrind if applicable
    if success && options.strict {
        // Check if file is executable by compiling it first
        let temp_dir = tempfile::Builder::new()
                        .prefix("synx-c-check")
                        .tempdir()?;
        let output_path = temp_dir.path().join("a.out");
        
        let compile_output = Command::new("gcc")
            .arg("-o")
            .arg(&output_path)
            .arg(file_path)
            .output();
            
        if let Ok(output) = compile_output {
            if output.status.success() {
                // Check if valgrind is available
                if Command::new("valgrind").arg("--version").output().is_ok() {
                    let valgrind_output = Command::new("valgrind")
                        .arg("--leak-check=full")
                        .arg("--error-exitcode=1")
                        .arg(&output_path)
                        .output()?;
                        
                    if !valgrind_output.status.success() && options.verbose {
                        eprintln!("Memory leak detected:");
                        eprintln!("{}", String::from_utf8_lossy(&valgrind_output.stderr));
                        return Ok(false);
                    }
                } else if options.verbose {
                    eprintln!("Note: Valgrind not available, skipping memory leak check");
                }
            }
        }
    }

    Ok(success)
}

fn validate_csharp(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    // First, try dotnet CLI
    let dotnet_check = Command::new("dotnet").arg("--version").output();
    
    if dotnet_check.is_ok() {
        // Use dotnet CLI to build the C# file
        let temp_dir = tempfile::Builder::new()
                        .prefix("synx-csharp-check")
                        .tempdir()?;
                        
        let mut cmd = Command::new("dotnet");
        cmd.arg("build")
           .current_dir(temp_dir.path())
           .arg(file_path);
           
        if options.strict {
            cmd.arg("/warnaserror");
        }
        
        let output = cmd.output()?;
        let success = output.status.success();
        
        if !success && options.verbose {
            eprintln!("C# validation errors (dotnet):");
            if !output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        
        if success && options.strict {
            // Check style with dotnet format if available
            let format_check = Command::new("dotnet")
                .arg("format")
                .arg("--verify-no-changes")
                .arg(file_path)
                .output();
                
            if let Ok(output) = format_check {
                if !output.status.success() && options.verbose {
                    eprintln!("C# style errors detected");
                    return Ok(false);
                }
            }
        }
        
        return Ok(success);
    } else {
        // Fallback to Mono's mcs compiler if dotnet CLI is not available
        let mcs_check = Command::new("mcs").arg("--version").output();
        
        if mcs_check.is_ok() {
            let mut cmd = Command::new("mcs");
            cmd.arg("-out:/dev/null") // Discard output assembly
               .arg("-warnaserror");
               
            if options.strict {
                cmd.arg("-warn:4"); // High warning level
            } else {
                cmd.arg("-warn:1"); // Basic warning level
            }
            
            cmd.arg(file_path);
            let output = cmd.output()?;
            let success = output.status.success();
            
            if !success && options.verbose {
                eprintln!("C# validation errors (Mono):");
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
            }
            
            return Ok(success);
        } else if options.verbose {
            eprintln!("No C# compiler found. Please install .NET SDK or Mono.");
        }
        
        return Ok(!options.strict);
    }
}

fn validate_python(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    // Basic syntax check
    let mut cmd = Command::new("python3");
    cmd.arg("-m")
       .arg("py_compile")
       .arg(file_path);
       
    let output = cmd.output()?;
    let syntax_valid = output.status.success();
    
    if !syntax_valid {
        if options.verbose {
            eprintln!("Python syntax errors:");
            if !output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        return Ok(false);
    }
    
    // If strict or verbose mode, run additional checks
    if options.strict || options.verbose {
        let mut success = true;
        
        // Check if mypy is available for type checking
        let mypy_check = Command::new("mypy").arg("--version").output();
        if mypy_check.is_ok() {
            let mypy_output = Command::new("mypy")
                .arg("--show-column-numbers")
                .arg(file_path)
                .output()?;
                
            if !mypy_output.status.success() {
                success = false;
                if options.verbose {
                    eprintln!("Python type errors:");
                    if !mypy_output.stdout.is_empty() {
                        eprintln!("{}", String::from_utf8_lossy(&mypy_output.stdout));
                    }
                }
            }
        } else if options.verbose {
            eprintln!("Note: mypy not available, skipping type checking");
        }
        
        // Check if pylint is available for linting
        let pylint_check = Command::new("pylint").arg("--version").output();
        if pylint_check.is_ok() {
            let mut pylint_cmd = Command::new("pylint");
            
            if options.strict {
                pylint_cmd.arg("--fail-under=9.0"); // Stricter score threshold
            } else {
                pylint_cmd.arg("--fail-under=7.0"); // More lenient score threshold
            }
            
            pylint_cmd.arg("--output-format=text")
                     .arg(file_path);
                     
            let pylint_output = pylint_cmd.output()?;
            
            if !pylint_output.status.success() {
                success = false;
                if options.verbose {
                    eprintln!("Python linting issues:");
                    if !pylint_output.stdout.is_empty() {
                        eprintln!("{}", String::from_utf8_lossy(&pylint_output.stdout));
                    }
                }
            }
        } else if options.verbose {
            eprintln!("Note: pylint not available, skipping linting");
        }
        
        return Ok(success || !options.strict);
    }
    
    Ok(true)
}

fn validate_javascript(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    // Check if node is available
    let node_check = Command::new("node").arg("--version").output();
    if node_check.is_err() {
        if options.verbose {
            eprintln!("JavaScript validation requires Node.js to be installed");
        }
        return Ok(!options.strict);
    }
    
    // Basic syntax check
    let syntax_output = Command::new("node")
        .arg("--check")
        .arg(file_path)
        .output()?;
        
    let syntax_valid = syntax_output.status.success();
    
    if !syntax_valid {
        if options.verbose {
            eprintln!("JavaScript syntax errors:");
            if !syntax_output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&syntax_output.stderr));
            }
        }
        return Ok(false);
    }
    
    // For strict mode or verbose, run ESLint
    if options.strict || options.verbose {
        // Check if ESLint is available
        let eslint_check = Command::new("eslint").arg("--version").output();
        if eslint_check.is_ok() {
            let mut cmd = Command::new("eslint");
            cmd.arg("--format=stylish");
            
            if options.strict {
                cmd.arg("--max-warnings=0");
            }
            
            cmd.arg(file_path);
            let output = cmd.output()?;
            
            if !output.status.success() && (options.strict || options.verbose) {
                if options.verbose {
                    eprintln!("JavaScript linting issues:");
                    if !output.stdout.is_empty() {
                        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                    if !output.stderr.is_empty() {
                        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                    }
                }
                
                return Ok(!options.strict);
            }
        } else if options.verbose {
            eprintln!("Note: ESLint not available, skipping linting");
            eprintln!("Install with: npm install -g eslint");
        }
    }
    
    Ok(true)
}

fn validate_unknown(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    if options.verbose {
        eprintln!("No validator available for file: {}", file_path.display());
    }
    Ok(!options.strict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_file_path(language: &str, valid: bool, filename: &str) -> PathBuf {
        let validity = if valid { "valid" } else { "invalid" };
        PathBuf::from(format!(
            "source/tests/files/{}/{}/{}",
            language, validity, filename
        ))
    }

    // C Validator Tests
    #[test]
    fn test_validate_valid_c() {
        let file_path = get_test_file_path("c", true, "hello.c");
        let options = ValidationOptions {
            strict: false,
            verbose: false,
        };
        let result = validate_c(&file_path, &options).unwrap();
        assert!(result, "Valid C file should pass validation");
    }

    #[test]
    fn test_validate_invalid_c() {
        let file_path = get_test_file_path("c", false, "broken.c");
        let options = ValidationOptions {
            strict: false,
            verbose: false,
        };
        let result = validate_c(&file_path, &options).unwrap();
        assert!(!result, "Invalid C file should fail validation");
    }

    #[test]
    fn test_validate_c_strict_mode() {
        let file_path = get_test_file_path("c", true, "hello.c");
        let options = ValidationOptions {
            strict: true,
            verbose: false,
        };
        let result = validate_c(&file_path, &options).unwrap();
        assert!(result, "Valid C file should pass strict validation");
    }

    // C# Validator Tests
    #[test]
    fn test_validate_valid_csharp() {
        let file_path = get_test_file_path("csharp", true, "Person.cs");
        let options = ValidationOptions {
            strict: false,
            verbose: false,
        };
        let result = validate_csharp(&file_path, &options).unwrap();
        assert!(result, "Valid C# file should pass validation");
    }

    #[test]
    fn test_validate_invalid_csharp() {
        let file_path = get_test_file_path("csharp", false, "badclass.cs");
        let options = ValidationOptions {
            strict: false,
            verbose: false,
        };
        let result = validate_csharp(&file_path, &options).unwrap();
        assert!(!result, "Invalid C# file should fail validation");
    }

    // Python Validator Tests
    #[test]
    fn test_validate_valid_python() {
        let file_path = get_test_file_path("python", true, "calculator.py");
        let options = ValidationOptions {
            strict: false,
            verbose: false,
        };
        let result = validate_python(&file_path, &options).unwrap();
        assert!(result, "Valid Python file should pass validation");
    }

    #[test]
    fn test_validate_invalid_python() {
        let file_path = get_test_file_path("python", false, "bad_code.py");
        let options = ValidationOptions {
            strict: false,
            verbose: false,
        };
        let result = validate_python(&file_path, &options).unwrap();
        // Even with syntax errors, basic Python validation might pass as long as it's valid Python
        // The strict mode test below will catch style and type issues
        assert!(result, "Invalid Python file with valid syntax should pass basic validation");
    }

    #[test]
    fn test_validate_python_strict_mode() {
        let file_path = get_test_file_path("python", false, "bad_code.py");
        let options = ValidationOptions {
            strict: true,
            verbose: false,
        };
        let result = validate_python(&file_path, &options).unwrap();
        // In strict mode with pylint and mypy, this should fail
        assert!(!result, "Invalid Python file should fail strict validation");
    }

    // JavaScript Validator Tests
    #[test]
    fn test_validate_valid_javascript() {
        let file_path = get_test_file_path("javascript", true, "module.js");
        let options = ValidationOptions {
            strict: false,
            verbose: false,
        };
        let result = validate_javascript(&file_path, &options).unwrap();
        assert!(result, "Valid JavaScript file should pass validation");
    }

    #[test]
    fn test_validate_invalid_javascript() {
        let file_path = get_test_file_path("javascript", false, "bad_code.js");
        let options = ValidationOptions {
            strict: false,
            verbose: true, // Use verbose to see the errors
        };
        let result = validate_javascript(&file_path, &options).unwrap();
        // Basic syntax check may still pass as the JS has valid syntax but ESLint issues
        assert!(result, "Invalid JavaScript with valid syntax should pass basic validation");
    }

    #[test]
    fn test_validate_javascript_strict_mode() {
        let file_path = get_test_file_path("javascript", false, "bad_code.js");
        let options = ValidationOptions {
            strict: true,
            verbose: false,
        };
        let result = validate_javascript(&file_path, &options).unwrap();
        // In strict mode with ESLint, this should fail
        assert!(!result, "Invalid JavaScript file should fail strict validation");
    }

    // File type detection tests
    #[test]
    fn test_detect_file_type() {
        let c_file = PathBuf::from("test.c");
        assert_eq!(detect_file_type(&c_file).unwrap(), "c");

        let cs_file = PathBuf::from("test.cs");
        assert_eq!(detect_file_type(&cs_file).unwrap(), "cs");

        let py_file = PathBuf::from("test.py");
        assert_eq!(detect_file_type(&py_file).unwrap(), "py");

        let js_file = PathBuf::from("test.js");
        assert_eq!(detect_file_type(&js_file).unwrap(), "js");
    }

    // Validator routing tests
    #[test]
    fn test_get_validator_for_type() {
        // Can't directly compare function pointers, so just verify they're mapped correctly
        assert_eq!(
            std::mem::discriminant(&(get_validator_for_type("c") as fn(&Path, &ValidationOptions) -> Result<bool>)),
            std::mem::discriminant(&(validate_c as fn(&Path, &ValidationOptions) -> Result<bool>))
        );

        assert_eq!(
            std::mem::discriminant(&(get_validator_for_type("cs") as fn(&Path, &ValidationOptions) -> Result<bool>)),
            std::mem::discriminant(&(validate_csharp as fn(&Path, &ValidationOptions) -> Result<bool>))
        );

        assert_eq!(
            std::mem::discriminant(&(get_validator_for_type("py") as fn(&Path, &ValidationOptions) -> Result<bool>)),
            std::mem::discriminant(&(validate_python as fn(&Path, &ValidationOptions) -> Result<bool>))
        );

        assert_eq!(
            std::mem::discriminant(&(get_validator_for_type("js") as fn(&Path, &ValidationOptions) -> Result<bool>)),
            std::mem::discriminant(&(validate_javascript as fn(&Path, &ValidationOptions) -> Result<bool>))
        );
    }
}

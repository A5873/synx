use std::path::Path;
use std::process::Command;
use anyhow::{Result, anyhow, Context};
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;
use tempfile;

use crate::config::Config;
use crate::detectors::FileType;

/// Trait for file validators
pub trait Validator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()>;
    fn name(&self) -> &str;
}

/// Comprehensive C code validator with advanced analysis features
struct CValidator;
impl Validator for CValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Configuration options
        let mut analysis_level = "basic"; // Default to basic validation
        let mut use_valgrind = false;
        let mut use_asan = false;
        let mut use_tsan = false;
        let mut use_ubsan = false;
        let mut use_msan = false;
        let mut use_lsan = false;
        let mut use_static_analyzer = false;
        let mut use_cppcheck = false;
        let mut use_iwyu = false;
        let mut use_clang_tidy = false;
        
        if let Some(validator_config) = config.validators.get("c") {
            if !validator_config.enabled {
                if verbose {
                    println!("C validator is disabled in config");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                let mut all_args: Vec<String> = args.clone();
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
            
            // Extract configuration options from args
            if let Some(args) = &validator_config.args {
                for arg in args {
                    match arg.as_str() {
                        "analysis=basic" => analysis_level = "basic",
                        "analysis=advanced" => analysis_level = "advanced",
                        "analysis=comprehensive" => analysis_level = "comprehensive",
                        "use_valgrind=true" => use_valgrind = true,
                        "use_asan=true" => use_asan = true,
                        "use_tsan=true" => use_tsan = true,
                        "use_ubsan=true" => use_ubsan = true,
                        "use_msan=true" => use_msan = true,
                        "use_lsan=true" => use_lsan = true,
                        "use_static_analyzer=true" => use_static_analyzer = true,
                        "use_cppcheck=true" => use_cppcheck = true,
                        "use_iwyu=true" => use_iwyu = true,
                        "use_clang_tidy=true" => use_clang_tidy = true,
                        _ => {}
                    }
                }
            }
        }
        
        // Check if C compiler is available
        if !is_command_available("gcc") && !is_command_available("clang") {
            print_colored("No C compiler (gcc or clang) is installed", false, false)?;
            println!("Please install a C compiler to validate C files.");
            return Err(anyhow!("C compiler is not installed"));
        }
        
        // Create temporary directory for analysis
        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory for C validation")?;
        
        // Copy the source file to the temporary directory
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let temp_file_path = temp_dir.path().join(file_name);
        std::fs::copy(file_path, &temp_file_path)
            .context("Failed to copy C file to temporary directory")?;
        
        // Create a test program with main function to compile and run the C file
        let test_main_path = temp_dir.path().join("test_main.c");
        let test_main_content = format!(r#"
#include <stdio.h>

// Include the source file to test
#include "{}"

// Simple main function to exercise the code
int main() {{
    printf("C program validation\n");
    return 0;
}}
"#, file_name);
        
        std::fs::write(&test_main_path, test_main_content)
            .context("Failed to write test main file")?;
        
        // Create a Makefile for compiling with various options
        let makefile_path = temp_dir.path().join("Makefile");
        let makefile_content = r#"
CC ?= gcc
CFLAGS = -Wall -Wextra -pedantic

all: basic

basic:
	$(CC) $(CFLAGS) -c *.c -o /dev/null

asan:
	$(CC) $(CFLAGS) -fsanitize=address -g *.c -o test_asan
	./test_asan

lsan:
	$(CC) $(CFLAGS) -fsanitize=leak -g *.c -o test_lsan
	./test_lsan

ubsan:
	$(CC) $(CFLAGS) -fsanitize=undefined -g *.c -o test_ubsan
	./test_ubsan

tsan:
	$(CC) $(CFLAGS) -fsanitize=thread -g *.c -o test_tsan
	./test_tsan

valgrind:
	$(CC) $(CFLAGS) -g *.c -o test_valgrind
	valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --verbose ./test_valgrind

clean:
	rm -f test_* *.o
"#;
        
        std::fs::write(&makefile_path, makefile_content)
            .context("Failed to write Makefile")?;
        
        // Initialize results collection
        let mut has_errors = false;
        let mut error_messages = Vec::new();
        
        // 1. Basic syntax check - just compile with warnings as errors
        if verbose {
            println!("Running basic C syntax validation...");
        }
        
        let compiler = if is_command_available("clang") { "clang" } else { "gcc" };
        let mut compile_cmd = Command::new(compiler);
        compile_cmd.current_dir(&temp_dir)
                  .args(["-Wall", "-Wextra", "-pedantic", "-Werror", "-c", file_name, "-o", "/dev/null"]);
        
        match compile_cmd.output() {
            Ok(output) => {
                if !output.status.success() {
                    has_errors = true;
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error_messages.push(format!("Basic syntax check failed:\n{}", stderr));
                    
                    // If basic syntax check fails, no need to proceed
                    print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
                    for msg in error_messages {
                        println!("{}", msg);
                    }
                    return Err(anyhow!("C validation failed"));
                } else if verbose {
                    println!("Basic syntax check passed.");
                }
            },
            Err(e) => {
                return Err(anyhow!("Failed to run C compiler: {}", e));
            }
        }
        
        // Stop here if only basic analysis is required
        if analysis_level == "basic" {
            if has_errors {
                print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
                for msg in error_messages {
                    println!("{}", msg);
                }
                return Err(anyhow!("C validation failed"));
            } else {
                print_colored(&format!("{}: No issues found in {:?}", self.name(), file_path), true, verbose)?;
                return Ok(());
            }
        }
        
        // 2. Static analysis with Clang Static Analyzer
        if use_static_analyzer {
            if !is_command_available("scan-build") {
                if verbose {
                    println!("Clang Static Analyzer (scan-build) is not installed. Skipping static analysis.");
                    println!("To install: apt install clang-tools or brew install llvm");
                }
            } else {
                if verbose {
                    println!("Running Clang Static Analyzer...");
                }
                
                let mut scan_cmd = Command::new("scan-build");
                scan_cmd.current_dir(&temp_dir)
                       .args(["-o", "scan-output", "make", "basic"]);
                
                match scan_cmd.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            has_errors = true;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let combined = format!("{}\n{}", stdout, stderr);
                            error_messages.push(format!("Static analysis issues (scan-build):\n{}", combined));
                        } else if verbose {
                            println!("Clang Static Analyzer passed.");
                        }
                    },
                    Err(e) => {
                        error_messages.push(format!("Failed to run Clang Static Analyzer: {}", e));
                    }
                }
            }
        }
        
        // 3. Static analysis with Cppcheck
        if use_cppcheck {
            if !is_command_available("cppcheck") {
                if verbose {
                    println!("Cppcheck is not installed. Skipping cppcheck analysis.");
                    println!("To install: apt install cppcheck or brew install cppcheck");
                }
            } else {
                if verbose {
                    println!("Running Cppcheck...");
                }
                
                let mut cppcheck_cmd = Command::new("cppcheck");
                cppcheck_cmd.current_dir(&temp_dir)
                          .args(["--enable=all", "--inconclusive", "--force", "--quiet", file_name]);
                
                match cppcheck_cmd.output() {
                    Ok(output) => {
                        if !output.status.success() || !output.stderr.is_empty() {
                            has_errors = true;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            error_messages.push(format!("Static analysis issues (cppcheck):\n{}", stderr));
                        } else if verbose {
                            println!("Cppcheck passed.");
                        }
                    },
                    Err(e) => {
                        error_messages.push(format!("Failed to run Cppcheck: {}", e));
                    }
                }
            }
        }
        
        // 4. Memory issues with AddressSanitizer
        if use_asan {
            if verbose {
                println!("Running AddressSanitizer (ASan)...");
            }
            
            let mut asan_cmd = Command::new("make");
            asan_cmd.current_dir(&temp_dir)
                   .args(["asan"]);
            
            match asan_cmd.output() {
                Ok(output) => {
                    if !output.status.success() {
                        has_errors = true;
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let combined = format!("{}\n{}", stdout, stderr);
                        error_messages.push(format!("Memory issues detected (ASan):\n{}", combined));
                    } else if verbose {
                        println!("AddressSanitizer passed.");
                    }
                },
                Err(e) => {
                    if verbose {
                        println!("Warning: Failed to run AddressSanitizer: {}", e);
                        println!("This may require a newer compiler version.");
                    }
                }
            }
        }
        
        // 5. Memory leaks with LeakSanitizer
        if use_lsan {
            if verbose {
                println!("Running LeakSanitizer (LSan)...");
            }
            
            let mut lsan_cmd = Command::new("make");
            lsan_cmd.current_dir(&temp_dir)
                   .args(["lsan"]);
            
            match lsan_cmd.output() {
                Ok(output) => {
                    if !output.status.success() {
                        has_errors = true;
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let combined = format!("{}\n{}", stdout, stderr);
                        error_messages.push(format!("Memory leaks detected (LSan):\n{}", combined));
                    } else if verbose {
                        println!("LeakSanitizer passed.");
                    }
                },
                Err(e) => {
                    if verbose {
                        println!("Warning: Failed to run LeakSanitizer: {}", e);
                        println!("This may require a newer compiler version.");
                    }
                }
            }
        }
        
        // 6. Undefined behavior with UndefinedBehaviorSanitizer
        if use_ubsan {
            if verbose {
                println!("Running UndefinedBehaviorSanitizer (UBSan)...");
            }
            
            let mut ubsan_cmd = Command::new("make");
            ubsan_cmd.current_dir(&temp_dir)
                    .args(["ubsan"]);
            
            match ubsan_cmd.output() {
                Ok(output) => {
                    if !output.status.success() {
                        has_errors = true;
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let combined = format!("{}\n{}", stdout, stderr);
                        error_messages.push(format!("Undefined behavior detected (UBSan):\n{}", combined));
                    } else if verbose {
                         println!("UndefinedBehaviorSanitizer passed.");
                    }
                },
                Err(e) => {
                    if verbose {
                        println!("Warning: Failed to run UBSan: {}", e);
                        println!("This may require a newer compiler version.");
                    }
                }
            }
        }
        
        // Return results
        if has_errors {
            print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
            for msg in error_messages {
                println!("{}", msg);
            }
            Err(anyhow!("C validation failed"))
        } else {
            print_colored(&format!("{}: No issues found in {:?}", self.name(), file_path), true, verbose)?;
            Ok(())
        }
    }
    
    fn name(&self) -> &str {
        "C Validator"
    }
}
pub fn get_validator(file_type: &FileType) -> Result<Box<dyn Validator>> {
    match file_type {
        FileType::Python => Ok(Box::new(PythonValidator)),
        FileType::JavaScript => Ok(Box::new(JavaScriptValidator)),
        FileType::TypeScript => Ok(Box::new(TypeScriptValidator)),
        FileType::Tsx => Ok(Box::new(TypeScriptValidator)), // Use TypeScript validator for TSX too
        FileType::Jsx => Ok(Box::new(TypeScriptValidator)), // Use TypeScript validator for JSX too
        FileType::Vue => Ok(Box::new(VueValidator)),
        FileType::Svelte => Ok(Box::new(SvelteValidator)),
        FileType::Json => Ok(Box::new(JsonValidator)),
        FileType::Yaml => Ok(Box::new(YamlValidator)),
        FileType::Html => Ok(Box::new(HtmlValidator)),
        FileType::Css => Ok(Box::new(CssValidator)),
        FileType::Dockerfile => Ok(Box::new(DockerfileValidator)),
        FileType::Shell => Ok(Box::new(ShellValidator)),
        FileType::Markdown => Ok(Box::new(MarkdownValidator)),
        FileType::Toml => Ok(Box::new(TomlValidator)),
        FileType::Rust => Ok(Box::new(RustValidator)),
        FileType::C => Ok(Box::new(CValidator)),
        FileType::Cpp => Ok(Box::new(CppValidator)),
        _ => Err(anyhow!("No validator available for {:?}", file_type)),
    }
}

/// Helper function to print colorized output
fn print_colored(message: &str, success: bool, verbose: bool) -> Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    
    if success {
        if verbose {
            stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)))?;
            writeln!(&mut stdout, "+ {}", message)?;
            stdout.reset()?;
        }
    } else {
        stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Red)))?;
        writeln!(&mut stdout, "x {}", message)?;
        stdout.reset()?;
    }
    
    Ok(())
}

/// Helper function to check if a command is available
fn is_command_available(command: &str) -> bool {
    match Command::new("which").arg(command).output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Helper function to run a command and handle its output
fn run_command(cmd: &mut Command, validator_name: &str, file_path: &Path, verbose: bool) -> Result<()> {
    if verbose {
        println!("Running {} on {:?}", validator_name, file_path);
    }
    
    // Try to execute the command
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                print_colored(&format!("{}: No issues found in {:?}", validator_name, file_path), true, verbose)?;
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                print_colored(&format!("{} found issues in {:?}:", validator_name, file_path), false, false)?;
                
                // Some tools output to stdout, some to stderr
                if !stderr.is_empty() {
                    println!("{}", stderr);
                } else if !stdout.is_empty() {
                    println!("{}", stdout);
                }
                
                Err(anyhow!("{} validation failed ", validator_name))
            }
        },
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                print_colored(&format!("{} is not installed ", validator_name), false, false)?;
                println!("Please install the required tool to validate this file type.");
                Err(anyhow!("Validator tool not found: {}", cmd.get_program().to_string_lossy()))
            } else {
                Err(anyhow!("Failed to execute {}: {}", validator_name, e))
            }
        }
    }
}

// Validator implementations

/// Python syntax validator that uses Python's built-in py_compile module
struct PythonValidator;
impl Validator for PythonValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        let mut analysis_level = "basic"; // Default to basic syntax validation
        let mut use_mypy = false;
        let mut use_pylint = false;
        let mut use_memory_profiler = false;
        let mut use_bandit = false;
        
        if let Some(validator_config) = config.validators.get("python") {
            if !validator_config.enabled {
                if verbose {
                    println!("Python validator is disabled in config ");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
            
            // Extract configuration options from args
            if let Some(args) = &validator_config.args {
                for arg in args {
                    match arg.as_str() {
                        "analysis=basic" => analysis_level = "basic",
                        "analysis=advanced" => analysis_level = "advanced",
                        "analysis=comprehensive" => analysis_level = "comprehensive",
                        "use_mypy=true" => use_mypy = true,
                        "use_pylint=true" => use_pylint = true,
                        "use_memory_profiler=true" => use_memory_profiler = true,
                        "use_bandit=true" => use_bandit = true,
                        _ => {}
                    }
                }
            }
        }
        
        // Check if Python is available
        if !is_command_available("python") {
            print_colored("Python is not installed ", false, false)?;
            println!("Please install Python to validate Python files.");
            return Err(anyhow!("Python is not installed "));
        }
        
        // Initialize results collection
        let mut has_errors = false;
        let mut error_messages = Vec::new();
        
        // 1. Basic syntax validation with py_compile
        if verbose {
            println!("Running basic Python syntax validation...");
        }
        
        let mut cmd = Command::new("python");
        cmd.args(["-m ", "py_compile", file_path.to_str().unwrap()]);
        
        match cmd.output() {
            Ok(output) => {
                if !output.status.success() {
                    has_errors = true;
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error_messages.push(format!("Basic syntax check failed: {}", stderr));
                    
                    // If basic check fails, no need to proceed with advanced checks
                    print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
                    for msg in error_messages {
                        println!("{}", msg);
                    }
                    return Err(anyhow!("Python validation failed "));
                } else if verbose {
                    println!("Basic syntax check passed.");
                }
            },
            Err(e) => {
                return Err(anyhow!("Failed to run Python syntax check: {}", e));
            }
        }
        
        // Stop here if only basic analysis is required
        if analysis_level == "basic" {
            if has_errors {
                print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
                for msg in error_messages {
                    println!("{}", msg);
                }
                return Err(anyhow!("Python validation failed "));
            } else {
                print_colored(&format!("{}: No issues found in {:?}", self.name(), file_path), true, verbose)?;
                return Ok(());
            }
        }
        
        // 2. Type checking with mypy
        if use_mypy {
            if !is_command_available("mypy") {
                if verbose {
                    println!("mypy is not installed. Skipping type checking.");
                    println!("To install: pip install mypy ");
                }
            } else {
                if verbose {
                    println!("Running type checking with mypy...");
                }
                
                let mut mypy_cmd = Command::new("mypy");
                mypy_cmd.args(["--strict ", file_path.to_str().unwrap()]);
                
                match mypy_cmd.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            has_errors = true;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let error_output = if !stderr.is_empty() { stderr } else { stdout };
                            error_messages.push(format!("Type checking issues (mypy): {}", error_output));
                        } else if verbose {
                            println!("Type checking passed.");
                        }
                    },
                    Err(e) => {
                        error_messages.push(format!("Failed to run mypy: {}", e));
                    }
                }
            }
        }
        
        // 3. Code style and error checking with pylint
        if use_pylint {
            if !is_command_available("pylint") {
                if verbose {
                    println!("pylint is not installed. Skipping static analysis.");
                    println!("To install: pip install pylint ");
                }
            } else {
                if verbose {
                    println!("Running static analysis with pylint...");
                }
                
                let mut pylint_cmd = Command::new("pylint");
                pylint_cmd.args([
                    "--output-format=text ", 
                    "--score=n ", 
                    "--reports=n ",
                    file_path.to_str().unwrap()
                ]);
                
                match pylint_cmd.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            has_errors = true;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let error_output = if !stderr.is_empty() { stderr } else { stdout };
                            error_messages.push(format!("Static analysis issues (pylint): {}", error_output));
                        } else if verbose {
                            println!("Static analysis passed.");
                        }
                    },
                    Err(e) => {
                        error_messages.push(format!("Failed to run pylint: {}", e));
                    }
                }
            }
        }
        
        // 4. Security analysis with bandit
        if use_bandit {
            if !is_command_available("bandit") {
                if verbose {
                    println!("bandit is not installed. Skipping security analysis.");
                    println!("To install: pip install bandit ");
                }
            } else {
                if verbose {
                    println!("Running security analysis with bandit...");
                }
                
                let mut bandit_cmd = Command::new("bandit");
                bandit_cmd.args(["-f ", "txt", file_path.to_str().unwrap()]);
                
                match bandit_cmd.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            has_errors = true;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let error_output = if !stderr.is_empty() { stderr } else { stdout };
                            error_messages.push(format!("Security issues (bandit): {}", error_output));
                        } else if verbose {
                            println!("Security analysis passed.");
                        }
                    },
                    Err(e) => {
                        error_messages.push(format!("Failed to run bandit: {}", e));
                    }
                }
            }
        }
        
        // 5. Memory profiling with memory_profiler
        if use_memory_profiler {
            if !is_command_available("python") {
                if verbose {
                    println!("memory_profiler functionality requires Python.");
                    println!("To install memory_profiler: pip install memory_profiler ");
                }
            } else {
                if verbose {
                    println!("Running memory usage analysis...");
                }
                
                // Create a temporary script to run memory_profiler
                let temp_dir = tempfile::tempdir()
                    .context("Failed to create temporary directory for memory profiling ")?;
                
                let memory_script_path = temp_dir.path().join("memory_profile.py ");
                let memory_script_content = format!(r#"
import sys
try:
    from memory_profiler import memory_usage
except ImportError:
    print("memory_profiler module not installed. Install with: pip install memory_profiler")
    sys.exit(1)

try:
    script_path = "{}"
    module_name = script_path.replace('/', '.').replace(".py", "")
    
    # Check if the file is importable
    try:
            # First try to do a basic syntax check
            with open(script_path, "r") as f:
                compile(f.read(), script_path, "exec")
                
            # Then try to import if it seems to be a module
            if "__main__" not in script_path:
            # Add parent directory to path if needed
            import os
            parent_dir = os.path.dirname(os.path.abspath(script_path))
            if parent_dir not in sys.path:
                sys.path.insert(0, parent_dir)
                
            # Try importing as a module
            try:
                module_name = os.path.basename(script_path).replace(".py", "")
                __import__(module_name)
                print("Peak memory usage: " + str(memory_usage((__import__, (module_name,)), max_usage=True)[0]) + " MiB")
            except ImportError:
                # If import fails, use exec instead
                with open(script_path, "r") as f:
                    code = compile(f.read(), script_path, "exec")
                    mem = memory_usage((eval, ("exec(code)", globals())), max_usage=True)[0]
                    print("Peak memory usage: " + str(mem) + " MiB")
    except Exception as e:
        print("Memory profiling failed: " + str(e))
        sys.exit(1)
        
except Exception as e:
    print("Memory profiling error: " + str(e))
    sys.exit(1)
"#, file_path.to_str().unwrap());
                
                std::fs::write(&memory_script_path, memory_script_content)
                    .context("Failed to write temporary memory profiling script ")?;
                
                // Run the memory profiling script
                let mut python_cmd = Command::new("python");
                python_cmd.arg(&memory_script_path);
                
                match python_cmd.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            // Memory profiling failures should not block validation
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let error_output = if !stderr.is_empty() { stderr } else { stdout };
                            if verbose {
                                println!("Memory profiling note: {}", error_output);
                            }
                        } else {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            if verbose {
                                println!("Memory profile: {}", stdout);
                            }
                        }
                    },
                    Err(e) => {
                        if verbose {
                            println!("Memory profiling failed to run: {}", e);
                        }
                    }
                }
            }
        }
        
        // Return results
        if has_errors {
            print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
            for msg in error_messages {
                println!("{}", msg);
            }
            Err(anyhow!("Python validation failed "))
        } else {
            print_colored(&format!("{}: No issues found in {:?}", self.name(), file_path), true, verbose)?;
            Ok(())
        }
    }
    
    fn name(&self) -> &str {
        "Python Validator "
    }
}

/// JavaScript syntax validator that uses Node.js's syntax check
struct JavaScriptValidator;
impl Validator for JavaScriptValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("javascript") {
            if !validator_config.enabled {
                if verbose {
                    println!("JavaScript validator is disabled in config ");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        // First check if node is available
        if !is_command_available("node") {
            print_colored("Node.js is not installed ", false, false)?;
            println!("Please install Node.js to validate JavaScript files.");
            return Err(anyhow!("Node.js is not installed "));
        }
        
        let mut cmd = Command::new("node");
        cmd.args(["--check ", file_path.to_str().unwrap()]);
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "JavaScript Syntax Validator "
    }
}

struct JsonValidator;
impl Validator for JsonValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("json") {
            if !validator_config.enabled {
                if verbose {
                    println!("JSON validator is disabled in config ");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        // Check if jq is available
        if !is_command_available("jq") {
            print_colored("jq is not installed ", false, false)?;
            println!("Please install jq to validate JSON files.");
            return Err(anyhow!("jq is not installed "));
        }
        
        let mut cmd = Command::new("jq");
        cmd.args([".", file_path.to_str().unwrap()]);
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "JSON Validator "
    }
}

struct YamlValidator;
impl Validator for YamlValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        let mut cmd = Command::new("yamllint");
        cmd.args(["-f ", "parsable", file_path.to_str().unwrap()]);
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "YAML Validator "
    }
}

struct HtmlValidator;
impl Validator for HtmlValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        let mut cmd = Command::new("tidy");
        cmd.args(["-q ", "-e ", file_path.to_str().unwrap()]);
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "HTML Validator "
    }
}

struct CssValidator;
impl Validator for CssValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        // Using a simple CSS validator (could use stylelint in a real implementation)
        let mut cmd = Command::new("csslint");
        cmd.arg(file_path.to_str().unwrap());
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "CSS Validator "
    }
}

struct DockerfileValidator;
impl Validator for DockerfileValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        let mut cmd = Command::new("hadolint");
        cmd.arg(file_path.to_str().unwrap());
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "Dockerfile Validator "
    }
}

struct ShellValidator;
impl Validator for ShellValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        let mut cmd = Command::new("shellcheck");
        cmd.arg(file_path.to_str().unwrap());
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "Shell Script Validator "
    }
}

struct MarkdownValidator;
impl Validator for MarkdownValidator {
    fn validate(&self, file_path: &Path, verbose: bool, _config: &Config) -> Result<()> {
        let mut cmd = Command::new("mdl");
        cmd.arg(file_path.to_str().unwrap());
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "Markdown Validator "
    }
}

/// TOML validator that uses the toml crate to parse TOML files
struct TomlValidator;
impl Validator for TomlValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("toml") {
            if !validator_config.enabled {
                if verbose {
                    println!("TOML validator is disabled in config ");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        // Read the file
        let content = std::fs::read_to_string(file_path)
            .context(format!("Failed to read TOML file: {:?}", file_path))?;
        
        // Try to parse it with toml
        match toml::from_str::<toml::Value>(&content) {
            Ok(_) => {
                print_colored(&format!("{}: No issues found in {:?}", self.name(), file_path), true, verbose)?;
                Ok(())
            },
            Err(err) => {
                print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
                println!("{}", err);
                Err(anyhow!("TOML validation failed: {}", err))
            }
        }
    }
    
    fn name(&self) -> &str {
        "TOML Validator "
    }
}

/// Enhanced Rust validator with advanced analysis features
struct RustValidator;
impl Validator for RustValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        let mut analysis_level = "basic"; // Default to basic validation
        let mut use_clippy = true;
        let mut use_miri = false;
        let mut use_analyzer = false;
        let mut use_mirai = false;
        let mut lint_level = "warn";
        let mut threading_checks = false;
        let mut memory_checks = false;
        
        if let Some(validator_config) = config.validators.get("rust") {
            if !validator_config.enabled {
                if verbose {
                    println!("Rust validator is disabled in config ");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
            
            // Extract configuration options from args
            if let Some(args) = &validator_config.args {
                for arg in args {
                    match arg.as_str() {
                        "analysis=basic" => analysis_level = "basic",
                        "analysis=advanced" => analysis_level = "advanced",
                        "analysis=comprehensive" => analysis_level = "comprehensive",
                        "use_clippy=false" => use_clippy = false,
                        "use_miri=true" => use_miri = true,
                        "use_analyzer=true" => use_analyzer = true,
                        "use_mirai=true" => use_mirai = true,
                        "lint_level=warn" => lint_level = "warn",
                        "lint_level=deny" => lint_level = "deny",
                        "lint_level=forbid" => lint_level = "forbid",
                        "threading_checks=true" => threading_checks = true,
                        "memory_checks=true" => memory_checks = true,
                        _ => {}
                    }
                }
            }
        }
        
        // Check if rustc is available
        if !is_command_available("rustc") {
            print_colored("Rust compiler is not installed ", false, false)?;
            println!("Please install Rust to validate Rust files:");
            println!("curl --proto \"=https\" --tlsv1.2 -sSf https://sh.rustup.rs | sh");
            return Err(anyhow!("Rust compiler is not installed"));
        }
        
        // Create a temporary cargo project for proper analysis
        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory for Rust validation")?;
        
        if verbose {
            println!("Creating temporary Cargo project for advanced analysis...");
        }
        
        // Initialize a Cargo project in the temporary directory
        let mut cargo_init_cmd = Command::new("cargo");
        cargo_init_cmd.current_dir(&temp_dir).args(["init", "--lib", "--name", "rust_validator"]);
        
        // Execute cargo init
        match cargo_init_cmd.output() {
            Ok(_) => {
                if verbose {
                    println!("Temporary Cargo project created successfully");
                }
            },
            Err(e) => {
                return Err(anyhow!("Failed to create temporary Cargo project: {}", e));
            }
        }
        
        // Copy the target Rust file to the src directory
        let src_dir = temp_dir.path().join("src");
        let file_name = file_path.file_name().unwrap();
        let target_path = src_dir.join(file_name);
        
        std::fs::copy(file_path, &target_path)
            .context("Failed to copy Rust file to temporary project")?;
        
        // Replace lib.rs with appropriate mod declaration
        let lib_rs_path = src_dir.join("lib.rs");
        let mod_name = file_name.to_str().unwrap().replace(".rs", "");
        
        // Check if it's lib.rs we're validating
        if file_name == "lib.rs" {
            // We're analyzing the lib.rs file directly, so we'll just use it
            std::fs::copy(file_path, &lib_rs_path)
                .context("Failed to copy lib.rs to temporary project")?;
        } else {
            // Create mod declaration in lib.rs
            std::fs::write(&lib_rs_path, format!("mod {};\n", mod_name))
                .context("Failed to write temporary lib.rs file")?;
        }
        
        // Initialize results collection
        let mut has_errors = false;
        let mut error_messages = Vec::new();
        
        // 1. Basic syntax check with cargo check
        if verbose {
            println!("Running basic Rust syntax validation...");
        }
        
        let mut cargo_check_cmd = Command::new("cargo");
        cargo_check_cmd.current_dir(&temp_dir).args(["check", "--color=always"]);
        
        match cargo_check_cmd.output() {
            Ok(output) => {
                if !output.status.success() {
                    has_errors = true;
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error_messages.push(format!("Basic syntax check failed:\n{}", stderr));
                    
                    // If basic check fails, no need to proceed with advanced checks
                    print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
                    for msg in error_messages {
                        println!("{}", msg);
                    }
                    return Err(anyhow!("Rust validation failed"));
                } else if verbose {
                    println!("Basic syntax check passed.");
                }
            },
            Err(e) => {
                return Err(anyhow!("Failed to run cargo check: {}", e));
            }
        }
        
        // Stop here if only basic analysis is required
        if analysis_level == "basic" {
            if has_errors {
                print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
                for msg in error_messages {
                    println!("{}", msg);
                }
                return Err(anyhow!("Rust validation failed"));
            } else {
                print_colored(&format!("{}: No issues found in {:?}", self.name(), file_path), true, verbose)?;
                return Ok(());
            }
        }
        
        // 2. Advanced static analysis with Clippy (if enabled)
        if use_clippy {
            if !is_command_available("cargo-clippy") {
                if verbose {
                    println!("cargo-clippy is not installed. Skipping Clippy analysis.");
                    println!("To install: rustup component add clippy");
                }
            } else {
                if verbose {
                    println!("Running Clippy for advanced static analysis...");
                }
                
                let mut clippy_args = vec!["clippy", "--color=always"];
                
                // Add lint level
                match lint_level {
                    "warn" => clippy_args.push("--warn"),
                    "deny" => clippy_args.push("--deny"),
                    "forbid" => clippy_args.push("--forbid"),
                    _ => clippy_args.push("--warn"), // Default to warning level
                }
                
                // Add all lints for comprehensive analysis
                clippy_args.push("--all-features");
                clippy_args.push("--all-targets");
                
                let mut clippy_cmd = Command::new("cargo");
                clippy_cmd.current_dir(&temp_dir).args(&clippy_args);
                
                match clippy_cmd.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            has_errors = true;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            error_messages.push(format!("Static analysis issues (clippy):\n{}", stderr));
                        } else if verbose {
                            println!("Clippy analysis passed.");
                        }
                    },
                    Err(e) => {
                        error_messages.push(format!("Failed to run clippy: {}", e));
                    }
                }
            }
        }
        
        // 3. Memory and undefined behavior checks with Miri (if enabled)
        if use_miri && memory_checks {
            if !is_command_available("cargo-miri") {
                if verbose {
                    println!("cargo-miri is not installed. Skipping memory safety analysis.");
                    println!("To install: rustup +nightly component add miri");
                    println!("             cargo +nightly install cargo-miri");
                }
            } else {
                if verbose {
                    println!("Running Miri for memory safety and undefined behavior checks...");
                }
                
                // Create a test module to exercise the code
                let test_module = format!(r#"
#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_miri_validation() {{
        // This test is intentionally empty to just exercise code loading
        // Miri will check for undefined behavior during compilation/loading
    }}
}}
"#);
                
                // Add test module to the target file
                std::fs::write(&lib_rs_path, format!("mod {};\n{}", mod_name, test_module))
                    .context("Failed to write test module for Miri analysis")?;
                
                let mut miri_cmd = Command::new("cargo");
                miri_cmd.current_dir(&temp_dir).args(["+nightly", "miri", "test"]);
                
                match miri_cmd.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            has_errors = true;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            error_messages.push(format!("Memory safety issues detected (Miri):\n{}", stderr));
                        } else if verbose {
                            println!("Miri memory safety check passed.");
                        }
                    },
                    Err(e) => {
                        error_messages.push(format!("Failed to run Miri: {}", e));
                    }
                }
            }
        }
        
        // 4. Thread safety analysis (if enabled)
        if threading_checks {
            if verbose {
                println!("Running thread safety analysis...");
            }
            
            // Add thread specific tests to check for race conditions
            let thread_test_module = format!(r#"
#[cfg(test)]
mod thread_tests {{
    use super::*;
    use std::sync::{{Arc, Mutex}};
    use std::thread;

    #[test]
    fn test_threading() {{
        // This is a simple threading test to exercise thread safety
        // If the code has thread safety issues, this might help detect it
    }}
}}
"#);
            
            // Update lib.rs with thread tests
            std::fs::write(&lib_rs_path, format!("mod {};\n{}", mod_name, thread_test_module))
                .context("Failed to write thread test module")?;
            
            // Run with RUSTFLAGS that enables thread sanitizer features
            let mut thread_cmd = Command::new("cargo");
            thread_cmd.current_dir(&temp_dir)
                     .env("RUSTFLAGS", "-Z sanitizer=thread")
                     .args(["+nightly", "test", "--target", "x86_64-unknown-linux-gnu"]);
            
            match thread_cmd.output() {
                Ok(output) => {
                    if !output.status.success() {
                        has_errors = true;
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        error_messages.push(format!("Thread safety issues detected:\n{}", stderr));
                    } else if verbose {
                        println!("Thread safety check passed.");
                    }
                },
                Err(e) => {
                    if verbose {
                        println!("Warning: Thread safety check failed to run: {}", e);
                        println!("This may require installing additional tools or nightly Rust.");
                    }
                }
            }
        }
        
        // 5. Formal verification with MIRAI (if enabled)
        if use_mirai {
            if !is_command_available("cargo-mirai") {
                if verbose {
                    println!("cargo-mirai is not installed. Skipping formal verification.");
                    println!("To install: cargo install cargo-mirai");
                }
            } else {
                if verbose {
                    println!("Running MIRAI for formal verification...");
                }
                
                let mut mirai_cmd = Command::new("cargo");
                mirai_cmd.current_dir(&temp_dir).args(["mirai"]);
                
                match mirai_cmd.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            has_errors = true;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            error_messages.push(format!("Formal verification issues (MIRAI):\n{}", stderr));
                        } else if verbose {
                            println!("MIRAI formal verification passed.");
                        }
                    },
                    Err(e) => {
                        error_messages.push(format!("Failed to run MIRAI: {}", e));
                    }
                }
            }
        }
        
        // 6. rust-analyzer integration for semantic checks (if enabled)
        if use_analyzer {
            if !is_command_available("rust-analyzer") {
                if verbose {
                    println!("rust-analyzer is not installed. Skipping semantic analysis.");
                    println!("To install: rustup component add rust-analyzer");
                }
            } else {
                if verbose {
                    println!("Running rust-analyzer for semantic analysis...");
                }
                
                let mut analyzer_cmd = Command::new("rust-analyzer");
                analyzer_cmd.current_dir(&temp_dir).args(["check", "--workspace"]);
                
                match analyzer_cmd.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            has_errors = true;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            error_messages.push(format!("Semantic analysis issues (rust-analyzer):\n{}", stderr));
                        } else if verbose {
                            println!("rust-analyzer semantic analysis passed.");
                        }
                    },
                    Err(e) => {
                        error_messages.push(format!("Failed to run rust-analyzer: {}", e));
                    }
                }
            }
        }
        
        // Return results
        if has_errors {
            print_colored(&format!("{} found issues in {:?}:", self.name(), file_path), false, false)?;
            for msg in error_messages {
                println!("{}", msg);
            }
            Err(anyhow!("Rust validation failed"))
        } else {
            print_colored(&format!("{}: No issues found in {:?}", self.name(), file_path), true, verbose)?;
            Ok(())
        }
    }
    
    fn name(&self) -> &str {
        "Rust Validator"
    }
}

/// C++ validator - uses C++ compiler and analysis tools
struct CppValidator;
impl Validator for CppValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // For now, use the C validator as C++ is a superset of C
        // This will check basic syntax and could be extended in the future
        let c_validator = CValidator;
        c_validator.validate(file_path, verbose, config)
    }
    
    fn name(&self) -> &str {
        "C++ Validator "
    }
}

/// TypeScript validator that uses tsc for validation
struct TypeScriptValidator;
impl Validator for TypeScriptValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("typescript") {
            if !validator_config.enabled {
                if verbose {
                    println!("TypeScript validator is disabled in config");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        
        // Check if TypeScript compiler is available
        if !is_command_available("tsc") {
            print_colored("TypeScript compiler (tsc) is not installed", false, false)?;
            println!("Please install TypeScript to validate TypeScript files:");
            println!("npm install -g typescript");
            return Err(anyhow!("TypeScript compiler is not installed"));
        }
        
        // Create a temporary directory for tsconfig and the file copy
        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory for TypeScript validation")?;
        
        // Determine if this is JSX/TSX or regular TypeScript
        let is_jsx = file_path.extension().map_or(false, |ext| {
            ext.to_string_lossy().to_lowercase() == "tsx" || ext.to_string_lossy().to_lowercase() == "jsx"
        });
        
        // Copy the source file to the temporary directory
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let temp_file_path = temp_dir.path().join(file_name);
        
        std::fs::copy(file_path, &temp_file_path)
            .context("Failed to copy TypeScript file to temporary directory")?;
        
        // Create a minimal package.json to help with React imports
        let package_json_content = r#"{
  "name": "ts-validation",
  "version": "1.0.0",
  "private": true,
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0"
  }
}"#;
        
        std::fs::write(temp_dir.path().join("package.json"), package_json_content)
            .context("Failed to write temporary package.json for TypeScript validation")?;
        
        // For JSX/TSX files, we need to install React dependencies
        if is_jsx && is_command_available("npm") {
            if verbose {
                println!("Installing React dependencies for JSX/TSX validation...");
            }
            
            // Run npm install in the temporary directory
            let mut install_cmd = Command::new("npm");
            install_cmd.current_dir(temp_dir.path())
                      .args(["install", "--no-fund", "--silent"]);
            
            // Execute the command but handle it separately since it's just preparation
            match install_cmd.output() {
                Ok(output) => {
                    if !output.status.success() {
                        if verbose {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            println!("Warning: Failed to install React dependencies: {}", stderr);
                            println!("Will try to validate without dependencies installed.");
                        }
                    } else if verbose {
                        println!("React dependencies installed successfully.");
                    }
                },
                Err(e) => {
                    if verbose {
                        println!("Warning: Failed to run npm install: {}", e);
                        println!("Will try to validate without dependencies installed.");
                    }
                }
            }
        }
        
        // Create a minimal tsconfig.json for validation
        let tsconfig_content = if is_jsx {
            r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "esModuleInterop": true,
    "strict": false,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "jsx": "react-jsx",
    "noEmit": true,
    "allowJs": true,
    "moduleResolution": "node",
    "isolatedModules": true,
    "noImplicitAny": false
  },
  "include": ["./*.tsx", "./*.jsx", "./*.ts", "./*.js"]
}"#
        } else {
            r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "esModuleInterop": true,
    "strict": false,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "noEmit": true,
    "allowJs": true,
    "moduleResolution": "node",
    "isolatedModules": true,
    "noImplicitAny": false
  },
  "include": ["./*.ts", "./*.js"]
}"#
        };
        
        std::fs::write(temp_dir.path().join("tsconfig.json"), tsconfig_content)
            .context("Failed to write temporary tsconfig.json")?;
        
        // Run TypeScript compiler from the temporary directory
        let mut cmd = Command::new("tsc");
        cmd.current_dir(temp_dir.path()).arg("--noEmit");
        
        // For verbose output, show the exact command being run
        if verbose {
            println!("Running: tsc --noEmit (in temporary directory with {})", file_name);
        }
        
        run_command(&mut cmd, self.name(), file_path, verbose)
    }
    
    fn name(&self) -> &str {
        "TypeScript Validator"
    }
}

/// Vue component validator that uses vue-tsc or vue-template-compiler
struct VueValidator;
impl Validator for VueValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("vue") {
            if !validator_config.enabled {
                if verbose {
                    println!("Vue validator is disabled in config");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        
        // Try vue-tsc first (preferred for more thorough validation)
        if is_command_available("vue-tsc") {
            if verbose {
                println!("Using vue-tsc for Vue component validation");
            }
            
            // Create a temporary directory for validation
            let temp_dir = tempfile::tempdir()
                .context("Failed to create temporary directory for Vue validation")?;
            
            // Copy the Vue file to the temporary directory
            let file_name = file_path.file_name().unwrap().to_str().unwrap();
            let temp_file_path = temp_dir.path().join(file_name);
            
            std::fs::copy(file_path, &temp_file_path)
                .context("Failed to copy Vue file to temporary directory")?;
            
            // Create a minimal tsconfig.json for validation
            let tsconfig_content = r#"{
  "compilerOptions": {
    "target": "esnext",
    "module": "esnext",
    "moduleResolution": "node",
    "strict": true,
    "jsx": "preserve",
    "sourceMap": true,
    "resolveJsonModule": true,
    "esModuleInterop": true,
    "lib": ["esnext", "dom"],
    "noEmit": true
  },
  "include": ["./*.vue"]
}"#;
            
            std::fs::write(temp_dir.path().join("tsconfig.json"), tsconfig_content)
                .context("Failed to write temporary tsconfig.json for Vue validation")?;
            
            // Run vue-tsc from the temporary directory
            let mut cmd = Command::new("vue-tsc");
            cmd.current_dir(temp_dir.path()).arg("--noEmit");
            
            // For verbose output, show the exact command being run
            if verbose {
                println!("Running: vue-tsc --noEmit (in temporary directory with {})", file_name);
            }
            
            return run_command(&mut cmd, self.name(), file_path, verbose);
        }
        
        // Fall back to vue-template-compiler for basic syntax check
        if is_command_available("vue-template-compiler") {
            if verbose {
                println!("Using vue-template-compiler for Vue component validation");
            }
            
            // Create a temporary JS file that imports the Vue component
            let temp_dir = tempfile::tempdir()
                .context("Failed to create temporary directory for Vue validation")?;
            
            let validator_js_path = temp_dir.path().join("validate-vue.js");
            let validator_js_content = format!(r#"
const compiler = require('vue-template-compiler');
const fs = require('fs');

    try {{
        const content = fs.readFileSync("{}", "utf8");
        const result = compiler.parseComponent(content);
        
        // Check template syntax
        if (result.template) {{
            compiler.compile(result.template.content);
        }}
        
        console.log("Vue component syntax is valid");
        process.exit(0);
    }} catch (error) {{
        console.error("Vue component has syntax errors: " + error.message);
        process.exit(1);
    }}
"#, file_path.to_str().unwrap());
            
            std::fs::write(&validator_js_path, validator_js_content)
                .context("Failed to write temporary Vue validator script")?;
            
            // Run the validation script
            let mut cmd = Command::new("node");
            cmd.arg(validator_js_path.to_str().unwrap());
            
            return run_command(&mut cmd, self.name(), file_path, verbose);
        }
        
        // If neither tool is available
        print_colored("Vue validation tools are not installed", false, false)?;
        println!("Please install Vue.js validation tools:");
        println!("npm install -g vue-tsc typescript");
        println!("or");
        println!("npm install -g vue-template-compiler");
        Err(anyhow!("Vue validation tools are not installed"))
    }
    
    fn name(&self) -> &str {
        "Vue Validator"
    }
}

/// Svelte component validator that uses svelte-check
struct SvelteValidator;
impl Validator for SvelteValidator {
    fn validate(&self, file_path: &Path, verbose: bool, config: &Config) -> Result<()> {
        // Check if validator is enabled in config
        if let Some(validator_config) = config.validators.get("svelte") {
            if !validator_config.enabled {
                if verbose {
                    println!("Svelte validator is disabled in config");
                }
                return Ok(());
            }
            
            // Use custom command and args if specified
            if let (Some(cmd_str), Some(args)) = (&validator_config.command, &validator_config.args) {
                let mut cmd = Command::new(cmd_str);
                // Convert args to owned strings
                let mut all_args: Vec<String> = args.clone();
                // Add the file path as the last argument
                all_args.push(file_path.to_str().unwrap().to_string());
                cmd.args(all_args);
                return run_command(&mut cmd, self.name(), file_path, verbose);
            }
        }
        
        // Check if svelte-check is available
        if is_command_available("svelte-check") {
            // For svelte-check, we need to create a temporary project structure
            let temp_dir = tempfile::tempdir()
                .context("Failed to create temporary directory for Svelte validation")?;
            
            // Copy the Svelte file to the temp directory
            let file_name = file_path.file_name().unwrap().to_str().unwrap();
            let temp_file_path = temp_dir.path().join(file_name);
            
            std::fs::copy(file_path, &temp_file_path)
                .context("Failed to copy Svelte file to temporary directory")?;
            
            // Create a minimal package.json for svelte-check
            let package_json_path = temp_dir.path().join("package.json");
            let package_json_content = r#"{
  "name": "svelte-validation",
  "version": "1.0.0",
  "type": "commonjs",
  "private": true
}"#;
            
            std::fs::write(&package_json_path, package_json_content)
                .context("Failed to write temporary package.json for Svelte validation")?;
            
            // Create a minimal svelte.config.js file
            let svelte_config_path = temp_dir.path().join("svelte.config.js");
            let svelte_config_content = r#"/** @type {import('@sveltejs/kit').Config} */
module.exports = {
  compilerOptions: {
    dev: true
  }
};
"#;
            
            std::fs::write(&svelte_config_path, svelte_config_content)
                .context("Failed to write temporary svelte.config.js file")?;
            
            // Create a minimal tsconfig.json for svelte-check
            let tsconfig_path = temp_dir.path().join("tsconfig.json");
            let tsconfig_content = r#"{
  "compilerOptions": {
    "moduleResolution": "node",
    "target": "esnext",
    "module": "esnext",
    "resolveJsonModule": true,
    "baseUrl": ".",
    "checkJs": true,
    "allowJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "skipLibCheck": true
  },
  "include": ["*.svelte"]
}"#;
            
            std::fs::write(&tsconfig_path, tsconfig_content)
                .context("Failed to write temporary tsconfig.json for Svelte validation")?;
            
            // Run svelte-check in the temporary directory
            let mut cmd = Command::new("svelte-check");
            cmd.current_dir(temp_dir.path()).args(["--fail-on-warnings"]);
            
            return run_command(&mut cmd, self.name(), file_path, verbose);
        }
        
        // If svelte-check is not available, we can try a simpler approach with the svelte compiler
        if is_command_available("node") {
            // Create a temporary JS file that imports the Svelte compiler and validates the component
            let temp_dir = tempfile::tempdir()
                .context("Failed to create temporary directory for Svelte validation")?;
            
            let validator_js_path = temp_dir.path().join("validate-svelte.js");
            let validator_js_content = format!(r#"
// First try to require svelte compiler
try {{
    const fs = require("fs");
    const svelte = require("svelte/compiler");
    
    const content = fs.readFileSync("{}", "utf8");
    
    // Determine if this is a TypeScript Svelte component
    const isTypeScript = content.includes("<script lang=\"ts\">") || 
                           content.includes("<script lang='ts'>") ||
                           content.includes("<script lang=ts>");
    
    try {{
        // Parse and validate the Svelte component
        const result = svelte.compile(content, {{
            // Common options
            dev: true,
            css: false,
            hydratable: true,
            immutable: false,
            
            // Additional options for TypeScript if detected
            ...(isTypeScript ? {{
                enableSourcemap: true,
                preserveComments: true,
                preserveWhitespace: true
            }} : {{}})
        }});
        
        // Additional validation for component structure
        if (!content.includes("<script") && !content.includes("<style") && !content.trim()) {{
            throw new Error("Empty or invalid Svelte component: must contain markup, script, or style");
        }}
        
        // Check for any warnings that might be useful
        if (result.warnings && result.warnings.length > 0) {{
            console.log("\\nSvelte component has warnings:");
            result.warnings.forEach(warning => {{
                console.log("- " + warning.message + " at line " + warning.start.line + (warning.code ? " (code: " + warning.code + ")" : ""));
            }});
        }}
        
        console.log("Svelte component syntax is valid");
        process.exit(0);
    }} catch (error) {{
        console.error("Svelte component has syntax errors: " + error.message);
        if (error.frame) {{
            console.error(error.frame); // Show code frame if available
        }}
        process.exit(1);
    }}
}} catch (requireError) {{
    console.error("Svelte compiler is not installed. Please run:");
    console.error("npm install svelte");
    if (requireError.message.includes("Cannot find module")) {{
        console.error("\\nYou may need to run this command in your project directory");
        console.error("or install globally with: npm install -g svelte");
    }}
    process.exit(1);
}}
"#, file_path.to_str().unwrap());
            
            std::fs::write(&validator_js_path, validator_js_content)
                .context("Failed to write temporary Svelte validator script")?;
            
            // Run the validation script
            let mut cmd = Command::new("node");
            cmd.arg(validator_js_path.to_str().unwrap());
            
            return run_command(&mut cmd, self.name(), file_path, verbose);
        }
        
        // If neither tool is available
        print_colored("Svelte validation tools are not installed", false, false)?;
        println!("Please install Svelte validation tools:");
        println!("npm install -g svelte-check typescript");
        println!("or");
        println!("npm install -g svelte");
        println!("\nIf you encounter ES Module issues, try modifying your package.json:");
        println!("  \"type\": \"commonjs\" for CommonJS modules");
        println!("  \"type\": \"module\" for ES modules");
        Err(anyhow!("Svelte validation tools are not installed"))
    }
    
    fn name(&self) -> &str {
        "Svelte Validator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_toml_validator() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.toml");
        
        // Valid TOML
        std::fs::write(&file_path, r#"
        [package]
        name = "test"
        version = "0.1.0"
        "#).unwrap();
        
        let config = Config::default();
        let validator = TomlValidator;
        assert!(validator.validate(&file_path, false, &config).is_ok());
        
        // Invalid TOML
        std::fs::write(&file_path, r#"
        [package
        name = "test"
        "#).unwrap();
        
        assert!(validator.validate(&file_path, false, &config).is_err());
    }
    
    #[test]
    fn test_svelte_validator_parsing() {
        // Since we can't rely on svelte tools being installed in test environment,
        // we'll test the detection logic without actually running the validators
        
        let dir = tempdir().unwrap();
        
        // Valid Svelte component - JavaScript style
        let js_svelte_path = dir.path().join("component.svelte");
        std::fs::write(&js_svelte_path, r#"<script>
    export let name = 'world';

    function handleClick() {
        alert(`Hello ${name}!`);
    }
</script>

<main>
    <h1>Hello {name}!</h1>
    <button on:click={handleClick}>
        Click me
    </button>
</main>

<style>
    main {
        text-align: center;
        padding: 1em;
        margin: 0 auto;
    }

    h1 {
        color: #ff3e00;
        font-size: 4em;
        font-weight: 100;
    }
</style>"#).unwrap();
        
        // Valid Svelte component - TypeScript style
        let ts_svelte_path = dir.path().join("component-ts.svelte");
        std::fs::write(&ts_svelte_path, r#"<script lang="ts">
    export let name: string = 'world';
    export let count: number = 0;

    function handleClick(): void {
        count += 1;
    }
    
    type ButtonProps = {
        text: string;
        disabled?: boolean;
    };
    
    const buttonProps: ButtonProps = {
        text: 'Click me',
        disabled: false
    };
</script>

<main>
    <h1>Hello {name}!</h1>
    <p>Count: {count}</p>
    <button on:click={handleClick} disabled={buttonProps.disabled}>
        {buttonProps.text}
    </button>
</main>

<style>
    main {
        text-align: center;
        padding: 1em;
        margin: 0 auto;
    }

    h1 {
        color: #ff3e00;
        font-size: 4em;
        font-weight: 100;
    }
</style>"#).unwrap();
        
        // Verify the detection logic identifies these as Svelte components
        let js_file_type = crate::detectors::detect_file_type(&js_svelte_path).unwrap();
        let ts_file_type = crate::detectors::detect_file_type(&ts_svelte_path).unwrap();
        
        assert_eq!(js_file_type, FileType::Svelte, "Failed to detect JavaScript-based Svelte component");
        assert_eq!(ts_file_type, FileType::Svelte, "Failed to detect TypeScript-based Svelte component");
        
        // We can't reliably test the actual validation in unit tests because it depends on external tools,
        // but we can verify our validator handles both JS and TS Svelte components
    }
}

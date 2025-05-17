# Core Features and Technical Specifications

This document provides detailed information about Synx's core systems, their current implementation, and planned enhancements. It serves as both documentation for developers and a roadmap for implementation.

## Current Core Systems

### 1. File Validation System

#### Architecture
The validation system follows a modular pattern consisting of:
- A central dispatcher (`validate_files` in `lib.rs`) that processes file validation requests
- A file type detector (`detect_file_type` in `validators/mod.rs`) that determines the appropriate validator
- A validator resolver (`get_validator_for_type` in `validators/mod.rs`) that maps file types to validation functions
- Individual validator implementations for each supported language/format

#### Technical Specifications
- **Input**: File path(s), validation options (strict mode, verbose output)
- **Output**: Boolean result indicating validation success
- **Error Handling**: Uses Rust's `anyhow` for flexible error context
- **Extensibility**: New validators can be added by implementing a validation function and updating the type resolver

#### Current Implementation

```rust
// Core validation function (simplified)
pub fn validate_file(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let file_type = detect_file_type(file_path)?;
    let validator = get_validator_for_type(&file_type);
    validator(file_path, options)
}
```

#### File Type Detection
- Uses file extension as primary detection method
- Falls back to MIME type detection using `tree_magic_mini` for ambiguous cases
- Returns standardized type identifiers (e.g., "rs", "js", "cpp")

#### Validator Resolver
- Maps file type strings to validation functions
- Supports multiple extensions for the same validator (e.g., ".js", ".javascript" → `validate_javascript`)

### 2. Watch Mode

#### Architecture
The watch mode system provides real-time validation by:
- Setting up a file system watcher for each target file
- Using a channel-based approach for event notification
- Periodically checking for changes based on configurable interval
- Re-running validation when changes are detected

#### Technical Specifications
- **Watcher**: Uses the `notify` crate for cross-platform file system events
- **Mode**: Non-recursive watching (watches only specified files, not directories)
- **Interval**: Configurable check interval (default: 2 seconds)
- **Event Handling**: Debounced change detection to prevent multiple validations

#### Current Implementation

```rust
// Watch mode implementation (simplified)
fn watch_files(files: &[String], config: &Config) -> Result<bool> {
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        tx.send(res).unwrap();
    })?;

    for file in files {
        watcher.watch(Path::new(file), RecursiveMode::NonRecursive)?;
    }

    loop {
        match rx.recv_timeout(Duration::from_secs(config.watch_interval)) {
            Ok(_) => {
                println!("\nChange detected, revalidating...");
                validate_files(files, config)?;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(e) => return Err(anyhow!("Watch error: {}", e)),
        }
    }
}
```

### 3. Configuration Management

#### Architecture
The configuration system provides flexibility through:
- A central `Config` struct that holds all user options
- Command-line argument parsing for immediate options
- Support for configuration files (planned enhancement)
- Default values for all configuration options

#### Technical Specifications
- **Config Fields**:
  - `strict`: Boolean flag for strict validation mode
  - `verbose`: Boolean flag for detailed output
  - `watch`: Boolean flag for watch mode
  - `watch_interval`: Duration in seconds between checks
- **Defaults**: Sensible defaults provided through `Default` implementation

#### Current Implementation

```rust
pub struct Config {
    pub strict: bool,
    pub verbose: bool,
    pub watch: bool,
    pub watch_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            strict: false,
            verbose: false,
            watch: false,
            watch_interval: 2,
        }
    }
}
```

### 4. Language-Specific Validators

#### Architecture
Each language validator:
- Implements a common function signature
- Utilizes language-specific tools via command-line interfaces
- Handles both basic and strict validation modes
- Provides detailed error output when verbose mode is enabled

#### Technical Specifications
- **Function Signature**: `fn(file_path: &Path, options: &ValidationOptions) -> Result<bool>`
- **Tool Integration**: Uses `std::process::Command` to execute external tooling
- **Output Handling**: Parses tool-specific output for error reporting
- **Strict Mode**: Enables additional checks and stricter validation rules

#### Currently Implemented Validators
- Rust: Uses `rustc` with optional strict warnings
- C++: Uses `g++` with syntax-only checking
- Java: Uses `javac` with optional style checking
- Go: Uses `go vet` with additional `gofmt` and `golangci-lint` in strict mode
- TypeScript: Uses `tsc` with optional ESLint integration
- JSON: Uses `jq` for validation
- YAML: Uses `yamllint` with configurable strictness
- HTML: Uses `tidy` with error-only or warning+error modes
- CSS: Uses `csslint` for validation
- Shell: Uses `shellcheck` with configurable severity
- Dockerfile: Uses `hadolint` with configurable thresholds

#### Implementation Pattern

```rust
// Example validator implementation (simplified)
fn validate_language(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    // Set up the tool command with appropriate flags
    let mut cmd = Command::new("language-tool");
    cmd.arg("--syntax-check");
    
    // Add strict mode flags if enabled
    if options.strict {
        cmd.arg("--strict");
    }
    
    // Run the validation
    let output = cmd.output()?;
    let success = output.status.success();
    
    // Report detailed errors in verbose mode
    if !success && options.verbose {
        eprintln!("Language validation errors:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(success)
}
```

### 5. Error Handling and Reporting

#### Architecture
Error handling follows a consistent pattern:
- Using `anyhow` for flexible error context and propagation
- Consistent error formatting for user-facing messages
- Detailed output in verbose mode
- Status indicators for each validation result

#### Technical Specifications
- **Error Types**:
  - System errors (file not found, permission issues)
  - Tool execution errors (missing validators, execution failures)
  - Validation errors (syntax errors, style violations)
- **Output Modes**:
  - Regular: File validation status only
  - Verbose: Detailed error messages and warnings

#### Current Implementation

```rust
// Error handling pattern (simplified)
match validate_file(path, &options) {
    Ok(valid) => {
        if !valid {
            success = false;  // Track overall success
        } else if config.verbose {
            println!("✓ {}", file);  // Success indicator in verbose mode
        }
    }
    Err(e) => {
        eprintln!("Error validating {}: {}", file, e);
        success = false;
    }
}
```

## Planned Enhancements

### 1. Validator Improvements

#### C Validator
- **Status**: In progress (PR #2)
- **Compiler**: GCC with syntax-only checking
- **Strict Mode**: Enable additional warnings and treat them as errors
- **Advanced**: Memory leak detection using Valgrind integration
- **Implementation**:
  ```rust
  // GCC for basic syntax and warning checks
  cmd.arg("-fsyntax-only").arg("-Wall").arg("-pedantic");
  
  // In strict mode, additional checks
  if options.strict {
      cmd.arg("-Werror").arg("-Wextra")
        .arg("-Wconversion").arg("-Wformat=2");
  }
  
  // Optional Valgrind integration
  if strict && output.status.success() && Command::new("valgrind").arg("--version").output().is_ok() {
      // Compile and check with valgrind
  }
  ```

#### C# Validator
- **Status**: In progress (PR #2)
- **Compiler**: Support for both .NET SDK and Mono
- **Strict Mode**: Enable additional warnings and style checks
- **Implementation**:
  ```rust
  // Try dotnet CLI first
  let dotnet_check = Command::new("dotnet").arg("--version").output();
  if dotnet_check.is_ok() {
      // Use dotnet CLI
  } else {
      // Fall back to Mono's mcs
  }
  ```

#### Python Validator
- **Status**: In progress (PR #2)
- **Tools**: Python's built-in compiler, mypy for types, pylint for style
- **Strict Mode**: Higher score threshold in pylint, stricter mypy checks
- **Implementation**:
  ```rust
  // Basic syntax check with py_compile
  cmd.arg("-m").arg("py_compile").arg(file_path);
  
  // Additional tools in strict mode
  if options.strict {
      // Run mypy and pylint with appropriate settings
  }
  ```

#### JavaScript Validator
- **Status**: In progress (PR #2)
- **Tools**: Node.js for syntax check, ESLint for style
- **Strict Mode**: No warnings allowed in ESLint
- **Implementation**:
  ```rust
  // Basic syntax check
  cmd.arg("--check").arg(file_path);
  
  // ESLint in strict mode
  if options.strict {
      // Run ESLint with appropriate configuration
  }
  ```

### 2. Configuration System Expansion

#### Configuration File Support
- **Status**: Planned (Phase 1)
- **Format**: TOML configuration files
- **Locations**:
  - Project-level: `.synx.toml` in project root
  - User-level: `~/.config/synx/config.toml`
  - System-level: `/etc/synx/config.toml`
- **Precedence**: Command-line > Project > User > System > Defaults

#### Sample Configuration

```toml
# .synx.toml
[general]
strict = false
verbose = true
watch_interval = 5

[validators]
# Language-specific configurations
[validators.rust]
edition = "2021"
clippy = true

[validators.python]
mypy_strict = true
pylint_threshold = 9.0

[validators.javascript]
eslint_config = "./custom_eslint.json"
```

#### Implementation Plan
1. Add TOML parsing using `toml` crate
2. Implement configuration file loading with proper precedence
3. Extend `Config` struct with language-specific options
4. Modify validators to use language-specific configs

### 3. Performance Optimizations

#### Parallel Validation
- **Status**: Planned (Phase 1)
- **Implementation**: Use Rayon for parallel file processing
- **Benefits**: Significant speedup for multi-file validation
- **Code Sketch**:
  ```rust
  use rayon::prelude::*;
  
  fn validate_files_parallel(files: &[String], config: &Config) -> Result<bool> {
      let options = ValidationOptions { /*...*/ };
      
      let results: Vec<Result<bool>> = files.par_iter()
          .map(|file| validate_single_file(file, &options))
          .collect();
          
      let mut success = true;
      for result in results {
          match result {
              // Process results
          }
      }
      
      Ok(success)
  }
  ```

#### Validation Caching
- **Status**: Planned (Phase 1)
- **Implementation**: File hash-based caching system
- **Cache Storage**: Local `.synx_cache` directory
- **Invalidation**: Based on file modification time and content hash
- **Benefits**: Faster repeated validation, especially in watch mode

#### Tool Availability Detection
- **Status**: Planned (Phase 1)
- **Implementation**: Pre-check for all required tools at startup
- **Benefits**: Avoid runtime errors, provide clear setup instructions
- **Advanced**: Optional automatic installation of missing tools

### 4. Testing Framework

#### Unit Testing
- **Status**: Partially implemented, expansion planned
- **Coverage**: Every validator needs dedicated tests
- **Test Files**: Example files for each language (valid/invalid)
- **Implementation**: Standard Rust testing with real file validation

#### Integration Testing
- **Status**: Planned (Phase 1)
- **Approach**: End-to-end testing with complete validation flows
- **Test Matrix**: Coverage across all supported languages and options
- **Snapshot Testing**: Validate output against known-good examples

#### CI Integration
- **Status**: Basic implementation complete, enhancement planned
- **GitHub Actions**: Test on multiple platforms (Linux, macOS, Windows)
- **Tool Installation**: Ensure all required validation tools are available in CI
- **Environment**: Matrix of configurations and dependency versions

#### Benchmarking
- **Status**: Planned (Phase 1)
- **Implementation**: Using Criterion.rs for reliable benchmarks
- **Metrics**: Validation time per file type, memory usage
- **Baseline**: Establish and track performance over time

## Implementation Guidelines

### Adding New Validators
1. Create test files (valid and invalid examples)
2. Implement the validator function following the standard pattern
3. Add the file type mapping in `get_validator_for_type`
4. Write unit tests for the validator
5. Update documentation and help text

### Code Style
- Follow Rust idioms and standard patterns
- Use descriptive error messages with context
- Consistently handle verbosity in validators
- Document public APIs with rustdoc comments

### External Tool Integration
- Always check for tool availability before using
- Provide helpful messages when tools are missing
- Use appropriate flags for basic vs. strict validation
- Handle tool-specific output formats correctly

### Error Handling
- Use structured errors with context where possible
- Distinguish between system errors and validation failures
- Provide actionable messages that help users fix issues
- Use colored output for better readability when appropriate

### Configuration
- All behaviors should be configurable
- Provide sensible defaults that work for most cases
- Document all configuration options clearly
- Prioritize command-line options over configuration files

## Release Criteria for v1.0

Before releasing version 1.0, all the following must be completed:

1. All planned validator implementations (C, C#, Python, JavaScript)
2. Configuration file support with language-specific options
3. Parallel validation and basic caching system
4. Comprehensive test coverage (>90% for core systems)
5. Complete documentation including examples and troubleshooting


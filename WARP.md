# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

**Synx** is a high-performance, enterprise-grade CLI tool for validating syntax across 15+ programming languages with advanced security features and daemon mode. It's built in Rust and provides:

- Universal syntax validation for multiple languages (Rust, Python, JavaScript, TypeScript, Java, Go, C/C++, C#, HTML, CSS, JSON, YAML, Shell scripts, Dockerfiles)
- Parallel processing with Rayon for handling 1000+ files/minute
- Always-on daemon mode with real-time file watching
- Enterprise security features with sandboxed execution and audit logging
- Rich terminal interface with TUI monitoring
- Comprehensive configuration system with TOML files

## Development Commands

### Core Development Workflow

```bash
# Navigate to source directory (main development happens here)
cd source/

# Build the project
cargo build

# Build with release optimizations
cargo build --release

# Run the application directly (shows help by default)
cargo run

# Run with arguments
cargo run -- --help
cargo run -- validate file.py
cargo run -- scan ./src --verbose

# Run tests
cargo test

# Run specific test module
cargo test config_test

# Run tests with verbose output
cargo test -- --nocapture

# Check for compilation issues without building
cargo check

# Format code
cargo fmt

# Run clippy for additional linting
cargo clippy

# Fix common issues automatically
cargo fix --lib -p synx
```

### Advanced Commands

```bash
# Build with specific features
cargo build --features enhanced-security

# Build with minimal features for lighter builds  
cargo build --features minimal

# Run performance benchmarks
cargo run -- performance benchmark ./test_files --iterations 5

# Test daemon functionality
cargo run -- daemon init-config
cargo run -- daemon start --watch-paths ./test_files --foreground

# Test TUI monitoring
cargo run -- monitor ./test_files --auto-validate
```

## Architecture Overview

### High-Level Structure

The codebase is organized into two main directories:
- **`source/`**: Core Rust implementation with all business logic
- **`packaging/`**: Distribution packages (AUR, Debian, RPM, Homebrew)

### Core Module Architecture

```
source/src/
├── main.rs                 # CLI entry point with comprehensive subcommands
├── lib.rs                  # Library interface and main validation orchestration
├── config/                 # Hierarchical TOML configuration system
│   └── mod.rs             # Supports system/user/project configs with precedence
├── validators/             # Language-specific validation implementations
│   ├── mod.rs             # Central dispatch with 15+ language validators
│   ├── scan.rs            # Directory scanning with parallel processing
│   ├── display.rs         # Rich output formatting with progress bars
│   └── error_display.rs   # Structured error parsing and display
├── tools/                  # Security and utility framework
│   ├── mod.rs             # Secure command execution and policy enforcement
│   ├── audit.rs           # Enterprise audit logging
│   ├── policy.rs          # Security policy definitions
│   └── secure.rs          # Sandboxed execution primitives
├── daemon/                 # Always-on background service
│   ├── mod.rs             # File watching and automatic validation
│   ├── config.rs          # Daemon-specific configuration
│   └── service.rs         # System service installation/management
├── performance/            # Performance monitoring and optimization
│   ├── mod.rs             # Thread pool and resource management
│   ├── cache.rs           # File hash-based validation caching
│   ├── parallel.rs        # Rayon-based parallel processing
│   └── metrics.rs         # Performance statistics collection
├── intelligence/           # Code analysis and machine learning
│   ├── mod.rs             # Pattern recognition and suggestions
│   ├── metrics.rs         # Code complexity analysis
│   ├── quality.rs         # Quality scoring algorithms
│   └── suggestions.rs     # Automated fix recommendations
├── tui/                    # Terminal user interface
│   ├── mod.rs             # Interactive monitoring dashboard
│   ├── widgets.rs         # Custom TUI components
│   ├── syntax.rs          # Syntax highlighting with tree-sitter
│   └── issue_state.rs     # Issue tracking and resolution state
├── analysis/               # Static code analysis
│   ├── mod.rs             # Analysis orchestration
│   ├── traits.rs          # Analysis trait definitions  
│   └── memory.rs          # Memory usage analysis
└── detectors/              # File type detection
    └── mod.rs             # MIME type and extension-based detection
```

### Key Architectural Patterns

1. **Modular Validators**: Each language has its own validator function with consistent signature
2. **Security-First Design**: All external tool execution goes through sandboxed security framework
3. **Configuration Hierarchy**: System → User → Project → CLI with proper precedence
4. **Parallel Processing**: Uses Rayon for concurrent file validation
5. **Enterprise Features**: Audit logging, policy enforcement, resource limits
6. **Extensibility**: Plugin architecture for custom validators via TOML config

### Validation Flow

```rust
// High-level validation flow
validate_file(path, options) 
  → detect_file_type(path)              // Auto-detect based on extension/MIME
  → get_validator_for_type(file_type)   // Get language-specific validator
  → validator(path, options)            // Execute with security constraints
    → SecureCommand::new()              // Sandboxed execution
    → tool_execution                    // Run external tools (rustc, python3, etc.)
    → parse_validation_output()         // Structure error messages
    → display_validation_errors()       // Rich terminal output
```

## Configuration System

### Configuration File Locations (in precedence order)
1. **System**: `/etc/synx/config.toml`
2. **User**: `~/.config/synx/config.toml`  
3. **Project**: `.synx.toml` (in current directory)
4. **Explicit**: Via `--config` flag
5. **CLI**: Command-line arguments (highest precedence)

### Example Configuration
```toml
[general]
strict = false
verbose = true
watch_interval = 5
timeout = 30

[validators.rust]
edition = "2021"
clippy = true
clippy_flags = ["--deny=warnings"]

[validators.python] 
mypy_strict = true
pylint_threshold = 9.0
ignore_rules = ["C0111"]

[validators.custom.xml]
command = "xmllint"
args = ["--noout"]
strict_args = ["--dtdvalid", "schema.dtd"]
success_pattern = "validates$"
```

## Testing Approach

### Test Structure
- **Unit Tests**: Integrated within each module (`#[cfg(test)]`)
- **Integration Tests**: `source/tests/` directory
- **Example Files**: `test_files/` with valid/invalid samples for each language
- **Configuration Tests**: Comprehensive config loading and merging tests

### Key Test Files
- `source/tests/config_test.rs`: Configuration system validation
- `test_files/`: Sample files for validation testing (valid.py, invalid.rs, etc.)
- `examples/config/synx.toml`: Reference configuration examples

### Running Tests
```bash
# All tests
cargo test

# Specific test module  
cargo test config_test

# Integration tests only
cargo test --test '*'

# With output (to see println! statements)
cargo test -- --nocapture
```

## Dependencies and Features

### Core Dependencies
- **clap**: CLI argument parsing with subcommands
- **tokio**: Async runtime for daemon mode
- **rayon**: Parallel processing
- **anyhow**: Error handling
- **serde + toml**: Configuration serialization
- **notify**: File system watching
- **indicatif**: Progress bars and rich terminal output
- **tree-sitter**: Syntax parsing for TUI
- **tui + crossterm**: Terminal user interface

### Feature Flags
```toml
default = ["all-validators", "basic-security"]
enhanced-security = ["platform-security"]  
linux-security = ["nix", "seccompiler", "rlimit"]
macos-security = ["nix", "objc", "core-foundation"]
windows-security = ["windows-sys"]
minimal = []  # Lightweight build
```

## Language Support

### Implemented Validators
- **Rust**: `rustc` with cargo project detection, clippy integration
- **C/C++**: `gcc`/`g++` with syntax-only checking, memory check options
- **Python**: `python3 -m py_compile`, mypy, pylint (strict mode)
- **JavaScript**: `node --check`, ESLint integration  
- **TypeScript**: `tsc --noEmit`, ESLint support
- **Java**: `javac` with error reporting
- **Go**: `go vet`, gofmt, golangci-lint (strict mode)
- **JSON**: `jq` validation
- **YAML**: `yamllint` with configurable rules
- **HTML**: `tidy` with error/warning modes
- **CSS**: `stylelint` validation
- **Shell**: `shellcheck` with severity controls
- **Dockerfile**: `hadolint` with rule configuration

### Adding New Validators
1. Add file type mapping in `get_validator_for_type()` 
2. Implement validator function with signature: `fn(&Path, &ValidationOptions) -> Result<bool>`
3. Add tool availability check and secure command execution
4. Create test files in `test_files/`
5. Add configuration options in `config/mod.rs`

## Development Considerations

### Security Framework
- All external tool execution is sandboxed through `tools::SecureCommand`
- Resource limits enforced (CPU, memory, execution time)
- Audit logging for all operations in enterprise mode
- Path security validation to prevent directory traversal

### Performance Optimizations
- File hash-based caching to skip unchanged files
- Parallel validation using Rayon thread pools
- Progress indicators for large directory scans
- Memory-efficient streaming for large files

### Error Handling
- Structured errors with `anyhow` for rich context
- Language-specific error parsing and formatting
- Colored terminal output with detailed code context
- Graceful degradation when external tools are missing

### Enterprise Features
- Always-on daemon mode with systemd/launchd integration
- TUI monitoring dashboard for real-time insights
- Intelligence engine for code quality analysis
- Performance metrics and benchmark tracking
- Extensive configuration options for enterprise deployment

## Common Development Tasks

### Adding a New Language Validator
```rust
// 1. Add to file type detection
match file_type {
    "new_ext" => validate_new_language,
    // ...existing cases
}

// 2. Implement validator
fn validate_new_language(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = SecureCommand::new("tool_name");
    cmd.arg("--syntax-check").arg(file_path);
    
    if options.strict {
        cmd.arg("--strict-flags");
    }
    
    let output = cmd.output()?;
    Ok(output.status.success())
}

// 3. Add configuration struct in config/mod.rs
// 4. Add test files and test cases
```

### Debugging Issues
```bash
# Run with verbose output to see detailed execution
cargo run -- validate file.py --verbose

# Check configuration loading
cargo run -- config show

# Test specific validator
cargo run -- scan ./test_files --format json

# Monitor file changes
cargo run -- daemon start --foreground --watch-paths ./test_files
```

This architecture provides a robust, scalable foundation for universal syntax validation with enterprise-grade security and performance features.

# Synx

[![CI](https://github.com/A5873/synx/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/A5873/synx/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/A5873/synx)](https://github.com/A5873/synx/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Platform Support](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-brightgreen.svg)](https://github.com/A5873/synx/releases)

```
  _____                     
 / ____|                    
| (___  _   _ _ __ __  __  
 \___ \| | | | '_ \\ \/ /  
 ____) | |_| | | | |>  <   
|_____/ \__, |_| |_/_/\_\  
         __/ |             
        |___/              

 Universal Syntax Validator
 --------------------------
   "Code with confidence"
```

A CLI-first universal syntax validator and linter dispatcher.

## Description

Synx is a command-line tool that inspects any file and attempts to validate its syntax or structure by automatically detecting the filetype and dispatching the appropriate validator or linter. Whether you're checking Python scripts, YAML configs, JSON data, Shell scripts, Dockerfiles, or HTML—Synx streamlines the process into a single command.

Think of it as a universal "linter dispatcher" for developers, sysadmins, and hackers.

### Why Synx?

- **No more guessing which linter to use**: Synx automatically detects file types and runs the appropriate validator
- **One command for everything**: Whether it's Python, JSON, YAML, or HTML, use the same command
- **Fast and efficient**: Built in Rust for performance
- **Configurable**: Easily customize validators and settings
- **Ideal for CI/CD pipelines**: Batch validate multiple files with a single command
- **Watch mode**: Automatically re-validate files when they change
- **Extensible**: Add support for new file types through configuration

## Quick Start

### Installation

#### Pre-built Binaries

Download the pre-built binary for your platform from the [latest release](https://github.com/A5873/synx/releases/latest):

```bash
# Linux
curl -L https://github.com/A5873/synx/releases/latest/download/synx-linux-amd64.tar.gz | tar xz
chmod +x synx
sudo mv synx /usr/local/bin/

# macOS
curl -L https://github.com/A5873/synx/releases/latest/download/synx-macos-amd64.tar.gz | tar xz
chmod +x synx
sudo mv synx /usr/local/bin/

# Windows (using PowerShell)
Invoke-WebRequest -Uri https://github.com/A5873/synx/releases/latest/download/synx-windows-amd64.exe -OutFile synx.exe
# Add to your PATH or move to a directory in your PATH
```

#### From Source

Ensure you have Rust and Cargo installed. Then:

```bash
git clone https://github.com/A5873/synx.git
cd synx
cargo build --release
# Optional: copy to a directory in your PATH
cp target/release/synx ~/.local/bin/
```

### Quick Usage

```bash
# Validate a single file
synx script.py

# Validate multiple files
synx config.yaml Dockerfile script.js

# Use watch mode to automatically revalidate when files change
synx --watch script.py

# Show more detailed output
synx --verbose config.json
```

## Platform Support

Synx is designed to work on multiple platforms:

| Platform | Support Level | Notes |
|----------|---------------|-------|
| Linux    | Full          | Primary development platform |
| macOS    | Full          | Tested on Intel and Apple Silicon |
| Windows  | Full          | Works in PowerShell or Command Prompt |

## Dependencies

Synx relies on external validators being installed on your system. Here's what you need for each file type:

| File Type  | Required Tool          | Installation |
|------------|-----------------------|--------------|
| Python     | Python interpreter    | [python.org](https://www.python.org/downloads/) |
| JavaScript | Node.js               | [nodejs.org](https://nodejs.org/) |
| JSON       | jq                    | `apt install jq` or [stedolan.github.io/jq](https://stedolan.github.io/jq/) |
| YAML       | yamllint              | `pip install yamllint` |
| HTML       | html-tidy             | `apt install tidy` |
| CSS        | csslint               | `npm install -g csslint` |
| Dockerfile | hadolint              | [hadolint.github.io](https://hadolint.github.io/) |
| Shell      | shellcheck            | `apt install shellcheck` |
| Markdown   | markdownlint          | `npm install -g markdownlint-cli` |
| TOML       | Built-in              | No external dependency |
| Rust       | rustc                 | [rust-lang.org](https://www.rust-lang.org/tools/install) |

**Note**: If a tool is not installed, Synx will let you know what you need to install.

## Usage

### Basic Usage

```bash
# Check a single file
synx script.py

# Check multiple files
synx config.yaml script.py Dockerfile

# Check all files of a specific type
synx *.json
```

### Advanced Options

```bash
# Show verbose output
synx --verbose script.py

# Watch for file changes and revalidate
synx --watch script.py

# Use a custom config file
synx --config ~/.config/synx/custom-config.toml script.py

# Initialize a default config file
synx --init-config

# Use strict mode (treat warnings as errors)
synx --strict config.yaml
```

## Supported File Types

Synx can validate the following file types:

- **Python** (`.py`)
- **JavaScript** (`.js`)
- **TypeScript** (`.ts`)
- **HTML** (`.html`, `.htm`)
- **CSS** (`.css`)
- **JSON** (`.json`)
- **YAML** (`.yaml`, `.yml`)
- **TOML** (`.toml`)
- **Dockerfile** (`Dockerfile`)
- **Shell** (`.sh`, `.bash`, `.zsh`)
- **Markdown** (`.md`, `.markdown`)
- **C** (`.c`)
- **C++** (`.cpp`, `.cc`, `.cxx`)
- **Rust** (`.rs`)

Support for additional file types can be added through configuration.

## Configuration

Synx uses a TOML configuration file located at `~/.config/synx/config.toml`. You can customize validators, add file type mappings, and configure default behavior.

### Default Configuration

```toml
[general]
verbose = false
strict = false
timeout = 30

[validators.python]
command = "python"
args = ["-m", "py_compile"]
strict = false
timeout = 10
enabled = true

[validators.json]
command = "jq"
args = ["."]
strict = true
timeout = 5
enabled = true

# File name to validator mappings
[file_mappings]
"Dockerfile" = "dockerfile"
"Jenkinsfile" = "groovy"
"Makefile" = "makefile"
```

### Custom Validators

You can add or modify validators in your config file:

```toml
[validators.custom_lang]
command = "custom-validator"
args = ["--option1", "--option2"]
strict = true
timeout = 15
enabled = true
```

## Development Status

Synx is currently in active development. Here's the current status:

- **Core Functionality**: ✅ Complete
- **File Type Detection**: ✅ Complete
- **Validator Dispatcher**: ✅ Complete
- **Watch Mode**: ✅ Complete
- **Config System**: ✅ Complete
- **CI/CD Integration**: 🔄 In Progress
- **Plugin System**: 🔄 Planned
- **LSP Integration**: 🔄 Planned
- **Web UI**: 🔄 Planned

## Roadmap

Here are the planned features and improvements for Synx:

### Short Term (0-3 months)

- Add support for more file types
- Improve error reporting with suggestions
- Add automatic fixing capabilities
- Better integration with text editors
- Performance optimizations

### Medium Term (3-6 months)

- **LSP Integration**: Provide Language Server Protocol support for integration with editors like VS Code, Vim, etc.
- **Plugin System**: Allow easy extension with custom validators
- **Web UI**: Simple web interface for viewing validation results
- **Daemon Mode**: Run as a background service for continuous validation

### Long Term (6+ months)

- **Static Analysis**: Extend beyond syntax checking to semantic analysis
- **Cross-File Validation**: Validate relationships between files
- **Project-Wide Linting**: Analyze entire projects with a single command
- **Custom Rules Engine**: Allow users to define custom validation rules

## Integration with CI/CD Systems

Synx can be easily integrated into CI/CD pipelines:

### GitHub Actions

```yaml
- name: Validate Syntax
  run: |
    curl -L https://github.com/A5873/synx/releases/latest/download/synx-linux-amd64.tar.gz | tar xz
    chmod +x synx
    ./synx --strict path/to/your/files/*
```

### GitLab CI

```yaml
validate_syntax:
  stage: test
  script:
    - curl -L https://github.com/A5873/synx/releases/latest/download/synx-linux-amd64.tar.gz | tar xz
    - chmod +x synx
    - ./synx --strict path/to/your/files/*
```

### Jenkins

```groovy
stage('Validate Syntax') {
    steps {
        sh '''
            curl -L https://github.com/A5873/synx/releases/latest/download/synx-linux-amd64.tar.gz | tar xz
            chmod +x synx
            ./synx --strict path/to/your/files/*
        '''
    }
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.


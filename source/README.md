<div align="center">

# ✨ Synx ✨

[![CI](https://img.shields.io/github/actions/workflow/status/A5873/synx/ci.yml?branch=main&style=for-the-badge&logo=github&label=CI)](https://github.com/A5873/synx/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/A5873/synx?style=for-the-badge&logo=github&label=Release)](https://github.com/A5873/synx/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Platform Support](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-brightgreen.svg?style=for-the-badge&logo=windowsterminal)](https://github.com/A5873/synx/releases)

```
                                                                                         
      ███████╗██╗   ██╗███╗   ██╗██╗  ██╗
      ██╔════╝╚██╗ ██╔╝████╗  ██║╚██╗██╔╝
      ███████╗ ╚████╔╝ ██╔██╗ ██║ ╚███╔╝ 
      ╚════██║  ╚██╔╝  ██║╚██╗██║ ██╔██╗ 
      ███████║   ██║   ██║ ╚████║██╔╝ ██╗
      ╚══════╝   ╚═╝   ╚═╝  ╚═══╝╚═╝  ╚═╝
                                           
      🔍 Universal Syntax Validator 🔍
      ===============================
         "Code with confidence"
```

<h3>A CLI-first universal syntax validator and linter dispatcher built with ❤️ in Rust.</h3>

</div>

<p align="center">
  <img src="https://img.shields.io/badge/javascript-%23F7DF1E.svg?style=flat-square&logo=javascript&logoColor=black" alt="JavaScript" />
  <img src="https://img.shields.io/badge/python-%233776AB.svg?style=flat-square&logo=python&logoColor=white" alt="Python" />
  <img src="https://img.shields.io/badge/html-%23E34F26.svg?style=flat-square&logo=html5&logoColor=white" alt="HTML" />
  <img src="https://img.shields.io/badge/css-%231572B6.svg?style=flat-square&logo=css3&logoColor=white" alt="CSS" />
  <img src="https://img.shields.io/badge/json-%23000000.svg?style=flat-square&logo=json&logoColor=white" alt="JSON" />
  <img src="https://img.shields.io/badge/yaml-%23CB171E.svg?style=flat-square&logo=yaml&logoColor=white" alt="YAML" />
  <img src="https://img.shields.io/badge/typescript-%233178C6.svg?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/docker-%232496ED.svg?style=flat-square&logo=docker&logoColor=white" alt="Docker" />
</p>

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
## Supported File Types

Synx can validate the following file types:

| File Type | Extensions | Validator Tool | Installation |
|-----------|------------|----------------|-------------|
| Python | `.py` | `python -m py_compile` | [python.org](https://www.python.org/downloads/) |
| JavaScript | `.js` | `node --check` | [nodejs.org](https://nodejs.org/) |
| TypeScript | `.ts` | `tsc --noEmit` | `npm install -g typescript` |
| HTML | `.html`, `.htm` | `tidy` | `apt install tidy` |
| CSS | `.css` | `csslint` | `npm install -g csslint` |
| JSON | `.json` | `jq` | `apt install jq` |
| YAML | `.yaml`, `.yml` | `yamllint` | `pip install yamllint` |
| TOML | `.toml` | Built-in | No external dependency |
| Dockerfile | `Dockerfile` | `hadolint` | [hadolint.github.io](https://hadolint.github.io/) |
| Shell | `.sh`, `.bash`, `.zsh` | `shellcheck` | `apt install shellcheck` |
| Markdown | `.md`, `.markdown` | `mdl` | `gem install mdl` |
| C | `.c` | `gcc -fsyntax-only` | `apt install gcc` |
| C++ | `.cpp`, `.cc`, `.cxx` | `g++ -fsyntax-only` | `apt install g++` |
| Rust | `.rs` | `rustc --emit=check` | [rust-lang.org](https://www.rust-lang.org/tools/install) |

Support for additional file types can be added through configuration.

### Example Files

The repository includes example files for each supported file type in the `examples` directory:

```
examples/
├── python/
│   ├── valid.py        # A valid Python example
│   └── invalid.py      # A Python file with syntax errors
├── javascript/
├── json/
├── yaml/
├── html/
├── css/
├── docker/
└── shell/
```

These examples serve as:

1. **Test cases** for the validator
2. **Documentation** showing proper and improper syntax
3. **Learning resources** for best practices in different languages

#### Running the Example Tests

You can run the examples test script to validate all example files:

```bash
cargo run --bin run_examples
```

This will verify that valid examples pass validation and invalid examples fail validation, confirming that Synx is working correctly.

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
- **Package Distribution**: ✅ In Progress
- **CI/CD Integration**: 🔄 In Progress
- **Plugin System**: 🔄 Planned
- **LSP Integration**: 🔄 Planned
- **Web UI**: 🔄 Planned

## Repository Structure

The Synx project is organized into several key directories:

- `source/` - Main source code repository
  - Contains the core Rust implementation
  - Documentation and examples
  - Build configuration
  - Tests

- `packaging/` - Distribution and packaging files
  - `aur/` - Arch Linux User Repository package
  - `deb/` - Debian packaging files
  - `rpm/` - RPM packaging files
  - `brew/` - Homebrew formula
  - Installation guides and documentation

## Current Development Status

As of May 2025:

- ✅ Core Functionality: Complete and stable
- ✅ Multiple Package Formats: 
  - AUR package available
  - Debian packaging in progress
  - RPM spec ready for testing
  - Homebrew formula under review
- 🔄 Current Focus:
  - Improving package distribution across different platforms
  - Enhancing CI/CD integration
  - Implementing remaining planned features from roadmap

## 🚀 Real-World Scenarios

Let's explore how Synx can improve your development workflow in different scenarios:

### 🌐 Frontend Development Workflow

As a frontend developer working with React, you're juggling multiple file types (JS, JSX, CSS, HTML). Here's how Synx helps:

```bash
# Start a watch session for your component and its styles
synx --watch src/components/UserProfile.jsx src/styles/profile.css

# Quick check before committing
synx $(git diff --name-only --staged)

# Validate entire project
find src -type f \( -name "*.js" -o -name "*.jsx" -o -name "*.css" -o -name "*.html" \) | xargs synx
```

**Benefit**: Catch syntax errors across different file types with a single tool, without switching between different linters or validators.

### 🔧 Backend API Development

You're building a Python FastAPI backend with configuration in YAML and JSON:

```bash
# Validate your API implementation
synx app/routes/users.py app/models/user.py

# Check configuration files
synx config/settings.yaml config/database.json

# Run pre-commit check on all changed files
synx $(git diff --name-only origin/main...HEAD)
```

**Benefit**: Ensure both your code and configuration files are valid, reducing deployment issues caused by malformed configs.

### 🐳 DevOps Configuration Management

As a DevOps engineer managing infrastructure as code:

```bash
# Validate Dockerfiles, docker-compose, and CI config
synx Dockerfile docker-compose.yml .github/workflows/*.yml

# Check shell scripts
synx scripts/*.sh

# Inspect a complex YAML file thoroughly
synx --verbose kubernetes/deployment.yaml
```

**Benefit**: Catch configuration errors before they cause failed deployments or infrastructure issues.

### 👥 Team Collaboration Scenario

In a collaborative environment with a diverse team:

1. **Setup Git Hooks**:
   ```bash
   # Add to your project setup script
   cp .git/hooks/pre-commit .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

2. **Configure VS Code Integration**:
   - Add `.vscode/settings.json` and `.vscode/tasks.json` to your repository
   - Team members can run Synx tasks directly from VS Code

3. **Add CI Pipeline**:
   - Include `.github/workflows/synx.yml` in your repository
   - Automatically validates PR changes

**Benefit**: Consistent code quality across the team with minimal setup, regardless of each developer's preferred tools.

### 📊 Comprehensive Project Validation

Before a major release or deployment:

```bash
# Validate all supported files in the project
find . -type f \( -name "*.js" -o -name "*.jsx" -o -name "*.py" -o -name "*.html" \
  -o -name "*.css" -o -name "*.json" -o -name "*.yaml" -o -name "*.yml" \
  -o -name "*.md" -o -name "*.toml" -o -name "*.sh" -o -name "Dockerfile" \) \
  -not -path "*/node_modules/*" -not -path "*/venv/*" -not -path "*/.git/*" | \
  xargs synx --strict
```

**Benefit**: Comprehensive validation gives confidence before deploying to production.

## 🗺️ Roadmap

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

## 🔄 Integration with CI/CD Systems

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

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

<div align="center">

### 🌟 Synx: One Validator to Rule Them All 🌟

<p align="center">
  <img src="https://img.shields.io/badge/Made%20with-Rust-orange?style=for-the-badge&logo=rust" alt="Made with Rust" />
</p>

**[Get Started](#quick-start)** • 
**[Documentation](#description)** • 
**[Examples](#example-files)** • 
**[Use Cases](#-real-world-scenarios)** • 
**[Contribute](#contributing)**

</div>

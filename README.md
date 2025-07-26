<div align="center">

<img src="https://raw.githubusercontent.com/A5873/synx/gh-pages/assets/images/synx-logo.svg" alt="Synx Logo" width="256" height="256">

# ✨ Synx - Universal Syntax Validator ✨

[![CI](https://img.shields.io/github/actions/workflow/status/A5873/synx/ci.yml?branch=main&style=for-the-badge&logo=github&label=CI)](https://github.com/A5873/synx/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/A5873/synx?style=for-the-badge&logo=github&label=Release)](https://github.com/A5873/synx/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Platform Support](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-brightgreen.svg?style=for-the-badge&logo=windowsterminal)](https://github.com/A5873/synx/releases)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org)

**🚀 A high-performance, enterprise-grade CLI tool for validating syntax across 15+ programming languages with advanced security features**

</div>

<p align="center">
  <img src="https://img.shields.io/badge/javascript-%23F7DF1E.svg?style=flat-square&logo=javascript&logoColor=black" alt="JavaScript" />
  <img src="https://img.shields.io/badge/python-%233776AB.svg?style=flat-square&logo=python&logoColor=white" alt="Python" />
  <img src="https://img.shields.io/badge/rust-%23000000.svg?style=flat-square&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/c++-%2300599C.svg?style=flat-square&logo=cplusplus&logoColor=white" alt="C++" />
  <img src="https://img.shields.io/badge/java-%23ED8B00.svg?style=flat-square&logo=java&logoColor=white" alt="Java" />
  <img src="https://img.shields.io/badge/go-%2300ADD8.svg?style=flat-square&logo=go&logoColor=white" alt="Go" />
  <img src="https://img.shields.io/badge/typescript-%233178C6.svg?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/html-%23E34F26.svg?style=flat-square&logo=html5&logoColor=white" alt="HTML" />
  <img src="https://img.shields.io/badge/css-%231572B6.svg?style=flat-square&logo=css3&logoColor=white" alt="CSS" />
  <img src="https://img.shields.io/badge/json-%23000000.svg?style=flat-square&logo=json&logoColor=white" alt="JSON" />
  <img src="https://img.shields.io/badge/yaml-%23CB171E.svg?style=flat-square&logo=yaml&logoColor=white" alt="YAML" />
  <img src="https://img.shields.io/badge/docker-%232496ED.svg?style=flat-square&logo=docker&logoColor=white" alt="Docker" />
</p>

---

## 🎯 What is Synx?

Synx is a **blazingly fast**, **security-focused** universal syntax validator built from the ground up in Rust. Designed for enterprise environments, it provides comprehensive code validation across multiple programming languages while maintaining strict security policies and audit trails.

### ⚡ Key Highlights

- 🔒 **Enterprise Security**: Sandboxed execution with audit logging
- 🚀 **High Performance**: Parallel processing with Rayon, handles 1000+ files/minute
- 🎛️ **Advanced CLI**: Rich terminal interface with progress bars and colored output
- 📊 **Multiple Formats**: Text, JSON, and detailed report generation
- 🔧 **Extensible**: Plugin architecture for custom validators
- 📈 **CI/CD Ready**: Perfect for automated pipelines and code quality gates

## 🚀 Quick Start

```bash
# Install Synx
cargo install synx

# Validate a single file
synx validate main.rs

# Scan entire directories with progress tracking
synx scan ./src ./tests --format json --report validation_report.json

# Watch files for changes and revalidate
synx validate *.py --watch --verbose

# Enterprise-grade scanning with exclusions
synx scan ./codebase --exclude "node_modules/*" "*.test.*" --parallel 8
```

## ✨ Core Features

### 🔍 **Universal Language Support**
- **15+ Programming Languages**: Rust, Python, JavaScript, TypeScript, Java, Go, C/C++, C#, HTML, CSS, JSON, YAML, Shell scripts, Dockerfiles
- **Smart Detection**: Automatic file type detection based on extensions and content analysis
- **Extensible Architecture**: Easy plugin system for adding new language validators

### 🛡️ **Enterprise Security**
- **Sandboxed Execution**: All validation runs in secure, isolated environments
- **Audit Logging**: Comprehensive audit trails for compliance and monitoring
- **Resource Limits**: CPU and memory usage controls to prevent resource exhaustion
- **Policy Enforcement**: Configurable security policies for different environments

### ⚡ **High Performance**
- **Parallel Processing**: Multi-threaded validation with configurable worker pools
- **Smart Caching**: File hash-based caching for faster repeated validations
- **Progress Tracking**: Real-time progress bars for large codebases
- **Memory Optimized**: Efficient memory usage even with large directory structures

### 🎨 **Rich User Experience**
- **Colorized Output**: Beautiful terminal interface with syntax highlighting
- **Multiple Formats**: Text, JSON, and structured report generation
- **Watch Mode**: Real-time validation on file changes
- **Verbose Logging**: Detailed error messages and debugging information

## 📋 Usage Examples

### Basic File Validation
```bash
# Validate a single Python file
synx validate script.py

# Validate multiple files with verbose output
synx validate *.js *.ts --verbose

# Strict mode validation (stricter rules)
synx validate main.go --strict
```

### Directory Scanning
```bash
# Basic directory scan
synx scan ./src

# Advanced scanning with exclusions and parallel processing
synx scan ./project --exclude "*.test.*" "node_modules/*" --parallel 12

# Generate JSON report for CI/CD integration
synx scan ./codebase --format json --report ci_validation.json
```

### Watch Mode for Development
```bash
# Watch files and revalidate on changes
synx validate *.py --watch --interval 1

# Watch entire directories
synx scan ./src --watch --verbose
```

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Validate Code Quality
  run: |
    synx scan ./src ./tests --format json --report validation.json
    synx scan ./src --strict --parallel 4
```

## Repository Structure

This repository is organized into two main directories:

### [`source/`](source/README.md)
Contains the core Synx implementation:
- Rust source code
- Documentation
- Examples
- Tests
- CI/CD configurations

### [`packaging/`](packaging/README.md)
Contains all package distribution related files:
- AUR (Arch User Repository) package
- Debian packaging
- RPM spec and builds
- Homebrew formula

## Quick Links

- [Installation Guide](source/README.md#installation)
- [Usage Documentation](source/README.md#usage)
- [Package Builds](packaging/README.md#building-packages)
- [Contributing Guide](source/CONTRIBUTING.md)

## Installation

### Automated Installation

Synx provides an automated installation script that handles both Synx itself and all necessary dependencies:

```bash
# Clone the repository
git clone https://github.com/A5873/synx.git
cd synx

# Run the installation script
./source/install.sh
```

The script automatically detects your Linux distribution (Debian/Ubuntu, Fedora/RHEL, or Arch Linux) and installs all required dependencies for the supported languages.

### Dependencies

Synx requires various tools to validate different languages. Here's a comprehensive list of dependencies:

#### Build Essentials
- `gcc`, `g++` - For C/C++ validation
- `pkg-config` - For system libraries integration
- Go compiler - For Go validation
- JDK (Java Development Kit) - For Java validation
- Node.js and npm - For JavaScript/TypeScript validation
- Python 3 and pip - For Python validation

#### Language-Specific Tools
| Language | Required Tools | Optional Tools (Strict Mode) |
|----------|---------------|------------------------------|
| Rust     | `rustc`       | `clippy`                     |
| C        | `gcc`         | `valgrind` for memory checks |
| C++      | `g++`         | -                            |
| C#       | .NET SDK or Mono | `dotnet format`           |
| Python   | `python3`     | `mypy`, `pylint`             |
| JavaScript | `node`      | `eslint`                     |
| TypeScript | `tsc`       | `@typescript-eslint/parser`, `@typescript-eslint/eslint-plugin` |
| Java     | `javac`       | `checkstyle`                 |
| Go       | `go`          | `gofmt`, `golangci-lint`     |
| HTML     | `tidy`        | -                            |
| CSS      | `csslint`     | -                            |
| JSON     | `jq`          | -                            |
| YAML     | `yamllint`    | -                            |
| Shell    | `shellcheck`  | -                            |
| Dockerfile | `hadolint`  | -                            |

### OS-Specific Installation

#### Debian/Ubuntu

```bash
# System packages
sudo apt-get update
sudo apt-get install -y build-essential pkg-config python3 python3-pip golang openjdk-21-jdk tidy shellcheck jq yamllint

# Node.js via nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
nvm install node
nvm use node

# Node.js tools
npm install -g typescript @typescript-eslint/parser @typescript-eslint/eslint-plugin eslint csslint

# Python tools
pip3 install mypy pylint

# Go tools
go install golang.org/x/lint/golint@latest
curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin v1.56.2
```

#### Fedora/RHEL

```bash
# System packages
sudo dnf install -y gcc gcc-c++ golang java-latest-openjdk-devel nodejs npm python3 python3-pip tidy ShellCheck jq yamllint

# Node.js tools
npm install -g typescript @typescript-eslint/parser @typescript-eslint/eslint-plugin eslint csslint

# Python tools
pip3 install mypy pylint

# Go tools
go install golang.org/x/lint/golint@latest
curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin v1.56.2
```

#### Arch Linux

```bash
# System packages
sudo pacman -Sy --needed base-devel go nodejs npm python python-pip jdk-openjdk tidy shellcheck jq yamllint

# Node.js tools
npm install -g typescript @typescript-eslint/parser @typescript-eslint/eslint-plugin eslint csslint

# Python tools
pip3 install mypy pylint

# Go tools
go install golang.org/x/lint/golint@latest
curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin v1.56.2
```

## Development Status

Synx is actively maintained and developed. Current status:

- ✅ Core Functionality: Complete and stable
- ✅ Multiple Package Formats Available
- 🔄 Ongoing Package Distribution Improvements
- 🔄 CI/CD Integration Enhancements

See the [source README](source/README.md#development-status) for detailed status information.

## License

This project is licensed under the MIT License - see the [LICENSE](source/LICENSE) file for details.

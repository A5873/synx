<div align="center">

<img src="https://raw.githubusercontent.com/A5873/synx/main/source/src/assets/synx-logo-v2.svg" alt="Synx Logo" width="256" height="256">

# ✨ Synx ✨

[![CI](https://img.shields.io/github/actions/workflow/status/A5873/synx/ci.yml?branch=main&style=for-the-badge&logo=github&label=CI)](https://github.com/A5873/synx/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/A5873/synx?style=for-the-badge&logo=github&label=Release)](https://github.com/A5873/synx/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Platform Support](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-brightgreen.svg?style=for-the-badge&logo=windowsterminal)](https://github.com/A5873/synx/releases)

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

---

## 🚀 Development Status

Synx is actively developed and maintained with a focus on enterprise-grade code validation. Here's the current development status:

### ✅ **Completed Features**
- **Core Validation Engine**: Rust-based syntax validation for 15+ languages
- **Interactive TUI**: Full-featured terminal UI for interactive issue review and fixing
- **Parallel Processing**: Multi-threaded file validation with Rayon
- **Smart Caching**: File hash-based validation caching for performance
- **Rich CLI Interface**: Colored output, progress bars, and multiple output formats
- **Configuration System**: Flexible TOML-based configuration with defaults
- **Enterprise Security**: Sandboxed execution with audit logging
- **Package Distribution**: Debian, RPM, AUR, and Homebrew packages

### 🔄 **In Progress**
- **Advanced Analytics**: File complexity analysis and code quality metrics
- **Plugin Architecture**: Custom validator plugin system
- **Web Dashboard**: Optional web interface for team collaboration
- **Integration APIs**: REST API for CI/CD pipeline integration

### 📋 **Planned Features**
- **Language Server Protocol**: LSP support for real-time validation in editors
- **Cloud Integration**: Support for cloud-based validation services
- **Advanced Reporting**: Detailed compliance and quality reports
- **Team Management**: Multi-user authentication and access control

### 🛠️ **Development Environment**

#### Requirements
- Rust 1.70+ with Cargo
- Language validators (see main README for complete list)
- Git for version control

#### Quick Development Setup
```bash
# Clone and enter the source directory
cd source

# Install dependencies and build
cargo build

# Run tests
cargo test

# Run in development mode
cargo run -- --help
```

#### Testing
```bash
# Run all tests
cargo test

# Run specific test module
cargo test config_test

# Run with verbose output
cargo test -- --nocapture
```

#### Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines and contribution process.

### 📊 **Current Metrics**
- **Lines of Code**: ~6,500+ (Rust)
- **Test Coverage**: 85%+
- **Supported Languages**: 15+
- **UI Modes**: 2 (CLI + Interactive TUI)
- **Package Formats**: 4 (Debian, RPM, AUR, Homebrew)
- **CI/CD Pipelines**: GitHub Actions, GitLab CI

### 🎯 **Performance Targets**
- **Validation Speed**: 1000+ files/minute
- **Memory Usage**: <100MB for typical projects
- **Startup Time**: <500ms
- **Cache Hit Rate**: 90%+ for repeated scans

---

## 📁 **Source Structure**

```
source/
├── src/
│   ├── analysis/         # Code analysis and metrics
│   ├── config/          # Configuration management
│   ├── tui/             # Interactive Terminal User Interface
│   │   ├── mod.rs       # Main TUI application and event loop
│   │   ├── syntax.rs    # Syntax highlighting and tree-sitter integration
│   │   ├── issue_state.rs # Issue state management and tracking
│   │   └── widgets.rs   # Custom TUI widgets
│   ├── tools/           # Utility tools and helpers
│   ├── validators/      # Language-specific validators
│   ├── lib.rs          # Library interface
│   └── main.rs         # CLI application entry
├── tests/              # Integration tests
├── examples/           # Usage examples
├── docs/               # Additional documentation
│   ├── TUI_GUIDE.md    # Interactive TUI user guide
│   └── TUI_ARCHITECTURE.md # TUI technical documentation
└── Cargo.toml         # Rust package configuration
```

## 🔗 **Additional Resources**

- [Main Project README](../README.md) - Overview and installation
- [Enterprise Features](../ENTERPRISE_PLAN.md) - Advanced features roadmap
- [Progress Updates](../PROGRESS_WEEK1.md) - Development progress
- [Package Directory](../packaging/README.md) - Distribution packages

---

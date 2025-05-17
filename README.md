<div align="center">

<img src="https://raw.githubusercontent.com/A5873/synx/gh-pages/assets/images/synx-logo.svg" alt="Synx Logo" width="256" height="256">

[![CI](https://img.shields.io/github/actions/workflow/status/A5873/synx/ci.yml?branch=main&style=for-the-badge&logo=github&label=CI)](https://github.com/A5873/synx/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/A5873/synx?style=for-the-badge&logo=github&label=Release)](https://github.com/A5873/synx/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

# Synx Project Repository

A CLI-first universal syntax validator and linter dispatcher built with Rust.
</div>

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

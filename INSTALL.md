# Installation Instructions

Synx can be installed through various package managers or built from source.

## Pre-built Packages

### Debian/Ubuntu
```bash
# Download the .deb package from the latest release
wget https://github.com/A5873/synx/releases/download/v0.2.0/synx_0.2.0_amd64.deb
sudo dpkg -i synx_0.2.0_amd64.deb
```

### Fedora/RHEL
```bash
# Download the .rpm package from the latest release
sudo dnf install https://github.com/A5873/synx/releases/download/v0.2.0/synx-0.2.0-1.fc38.x86_64.rpm
```

### Arch Linux
```bash
# Using the PKGBUILD
git clone https://github.com/A5873/synx.git
cd synx
makepkg -si
```

### macOS
```bash
# Using Homebrew
brew tap A5873/synx
brew install synx
```

## Building from Source

### Requirements
- Rust 1.75.0 or later
- Cargo
- gcc/clang
- make

### Build Steps
```bash
# Clone the repository
git clone https://github.com/A5873/synx.git
cd synx

# Build with Cargo
cargo build --release

# Install
sudo cp target/release/synx /usr/local/bin/
```

## Dependencies

Synx requires various tools for different file types:
- Python: python3
- JavaScript/TypeScript: node, tsc
- Rust: rustc
- C/C++: gcc/clang
- YAML: yamllint
- JSON: jq
- HTML: tidy
- Shell: shellcheck

Install these dependencies as needed for your use case.

## Post-Installation

After installation, verify the installation with:
```bash
synx --version
```

For full functionality, ensure all required dependencies for your target file types are installed.

# Installing Synx

Synx is available for various platforms. Below are installation instructions for different operating systems and distributions.

## Pre-built Packages

### Debian/Ubuntu

Download the `.deb` package from the releases page and install it using:

```bash
sudo dpkg -i synx_0.2.1_amd64.deb
# If there are dependency issues, run:
sudo apt-get install -f
```

### Generic Linux (tarball)

Download the tarball and extract it:

```bash
curl -L https://github.com/A5873/synx/releases/download/v0.2.1/synx-linux-amd64.tar.gz | tar xz
sudo mv synx /usr/local/bin/
```

### Arch Linux

Synx is available in the AUR. You can install it using an AUR helper like `yay`:

```bash
yay -S synx
```

Or manually:

```bash
git clone https://aur.archlinux.org/synx.git
cd synx
makepkg -si
```

### Fedora/RHEL

Download the `.rpm` package from the releases page and install it using:

```bash
sudo dnf install synx-0.2.1-1.x86_64.rpm
```

## Building from Source

### Prerequisites

- Rust toolchain (rustc, cargo)
- Build essentials (gcc, make, etc.)

### Linux

```bash
# Clone the repository
git clone https://github.com/A5873/synx.git
cd synx

# Build the project
cargo build --release

# Install the binary
sudo cp target/release/synx /usr/local/bin/
```

### macOS

```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone the repository
git clone https://github.com/A5873/synx.git
cd synx

# Build the project
cargo build --release

# Install the binary
sudo cp target/release/synx /usr/local/bin/
```

### Windows

1. Install Rust from https://www.rust-lang.org/tools/install
2. Install Visual Studio Build Tools with C++ support
3. Clone and build:

```powershell
# Clone the repository
git clone https://github.com/A5873/synx.git
cd synx

# Build the project
cargo build --release

# The binary will be at target\release\synx.exe
# You can add it to your PATH or copy it to a location in your PATH
```

## Platform-Specific Notes

### Linux

Synx requires the following runtime dependencies:
- libc6
- libgcc-s1

### Windows

Make sure to have Visual C++ Redistributable installed on your system.

### macOS

Synx requires macOS 10.12 (Sierra) or newer.

## Verifying Installation

After installation, verify that synx is working correctly:

```bash
synx --version
# Should output: synx 0.2.1
```

## Troubleshooting

If you encounter any issues, please check the project's GitHub repository for known issues or to report a new one: https://github.com/A5873/synx/issues

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

# Synx Distribution Packages

This repository contains package distribution files for **Synx**, a CLI-first universal syntax validator and linter dispatcher.

## About Synx

Synx is a command-line tool that inspects any file and attempts to validate its syntax or structure by automatically detecting the filetype and dispatching to the appropriate validator or linter. It includes comprehensive memory analysis capabilities and performance profiling features.

## Installation

Choose your distribution below for installation instructions:

- [Debian/Ubuntu](#debianubuntu)
- [Arch Linux](#arch-linux)
- [Fedora/RHEL](#fedorarhel)
- [Generic Linux](#generic-linux)

### Debian/Ubuntu

#### Option 1: Using APT Repository (Recommended)

1. Install required dependencies:
```bash
sudo apt-get update
sudo apt-get install -y ca-certificates curl gnupg
sudo mkdir -p /etc/apt/keyrings
```

2. Add the repository GPG key:
```bash
curl -fsSL https://raw.githubusercontent.com/A5873/synx/master/repo/synx-key.asc | sudo gpg --dearmor -o /etc/apt/keyrings/synx-archive-keyring.gpg
```

3. Add the repository:
```bash
echo "deb [signed-by=/etc/apt/keyrings/synx-archive-keyring.gpg] https://raw.githubusercontent.com/A5873/synx/master/repo noble main" | sudo tee /etc/apt/sources.list.d/synx.list
```

4. Update and install:
```bash
sudo apt-get update
sudo apt-get install synx
```

#### Option 2: Manual Package Installation

1. Download the .deb package from the [releases page](https://github.com/A5873/synx/releases/latest)
2. Install with:
```bash
sudo dpkg -i synx_0.2.1_amd64.deb
sudo apt-get install -f # To resolve any dependencies
```

### Arch Linux

#### Option 1: AUR Helper (Recommended)

If you use an AUR helper like `yay` or `paru`:

```bash
yay -S synx
# OR
paru -S synx
```

#### Option 2: Manual AUR Installation

```bash
git clone https://aur.archlinux.org/synx.git
cd synx
makepkg -si
```

### Fedora/RHEL

#### Option 1: DNF Repository (Recommended)

1. Add the repository:
```bash
sudo dnf config-manager --add-repo https://raw.githubusercontent.com/A5873/synx/master/repo/synx.repo
```

2. Install synx:
```bash
sudo dnf install synx
```

#### Option 2: Manual RPM Installation

1. Download the .rpm package from the [releases page](https://github.com/A5873/synx/releases/latest)
2. Install with:
```bash
sudo dnf install ./synx-0.2.1-1.fc38.x86_64.rpm
```

### Generic Linux

For distributions not listed above, you can use the portable tarball:

1. Download the latest tarball from the [releases page](https://github.com/A5873/synx/releases/latest)
2. Extract and install:
```bash
curl -L https://github.com/A5873/synx/releases/download/v0.2.1/synx-linux-amd64.tar.gz | tar xz
sudo mv synx /usr/local/bin/
sudo chmod +x /usr/local/bin/synx
```

## Verifying Package Signatures

All packages are signed with our GPG key. Verify package integrity with:

```bash
# Download the public key
curl -fsSL https://raw.githubusercontent.com/A5873/synx/master/repo/synx-key.asc | gpg --import

# Verify a package (example for .deb)
gpg --verify synx_0.2.1_amd64.deb.asc synx_0.2.1_amd64.deb
```

## Building from Source

If you prefer to build from source, please see our [INSTALL.md](https://github.com/A5873/synx/blob/master/INSTALL.md) guide.

## Troubleshooting

### Package Signature Verification Issues

If you encounter signature verification issues:
- Make sure you have imported the correct key
- Check that you're using the latest version of the key
- Try refreshing your GPG keyring: `gpg --refresh-keys`

### Missing Dependencies

- For Debian/Ubuntu: `sudo apt-get install -f`
- For Arch Linux: Check PKGBUILD dependencies
- For Fedora/RHEL: `sudo dnf install --allowerasing synx`

## Support & Contributing

For issues, feature requests, or to contribute, please visit:
- [GitHub Repository](https://github.com/A5873/synx)
- [Issue Tracker](https://github.com/A5873/synx/issues)

## License

Synx is distributed under the MIT License. See [LICENSE](https://github.com/A5873/synx/blob/master/LICENSE) for details.


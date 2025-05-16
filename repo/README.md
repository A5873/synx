# Synx Package Repository

[![Latest Release](https://img.shields.io/github/v/release/A5873/synx?style=for-the-badge&logo=github&label=Release)](https://github.com/A5873/synx/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

This is the official package repository for Synx, a CLI-first universal syntax validator and linter dispatcher. For the main project documentation, please visit the [main repository](https://github.com/A5873/synx).

## Quick Installation

### Debian/Ubuntu

```bash
# Add the repository
echo "deb [trusted=yes] https://a5873.github.io/synx/repo/deb/ noble main" | sudo tee /etc/apt/sources.list.d/synx.list

# Update and install
sudo apt update
sudo apt install synx
```

### Fedora/RHEL

```bash
# Add the repository
sudo dnf config-manager --add-repo https://a5873.github.io/synx/repo/rpm/synx.repo

# Install synx
sudo dnf install synx
```

### Arch Linux (AUR)

```bash
# Using yay
yay -S synx

# Or using paru
paru -S synx
```

## Available Packages

### Debian (.deb)
- synx_0.2.1_amd64.deb - Latest stable release
- [View all Debian packages](repo/deb/pool/main/s/synx/)

### RPM
- synx-0.2.1-1.fc38.x86_64.rpm - Latest stable release
- [View all RPM packages](repo/rpm/)

## Repository Structure

```
repo/
├── deb/                    # Debian repository
│   ├── pool/              # Package pool
│   └── dists/             # Distribution indices
├── rpm/                    # RPM repository
└── index.md               # Repository index
```

## Verifying Package Signatures

All packages are signed with our release key. To verify package signatures:

```bash
# Import our public key
curl -fsSL https://a5873.github.io/synx/repo/synx-key.asc | sudo gpg --dearmor -o /usr/share/keyrings/synx-archive-keyring.gpg

# Verify a package
gpg --verify synx_0.2.1_amd64.deb.asc synx_0.2.1_amd64.deb
```

## Support

For issues, feature requests, or general support:
- [GitHub Issues](https://github.com/A5873/synx/issues)
- [Documentation](https://github.com/A5873/synx/blob/main/README.md)

## License

This project is licensed under the MIT License. See the [LICENSE](https://github.com/A5873/synx/blob/main/LICENSE) file in the main repository for details.

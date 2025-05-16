<div align="center">

<img src="/synx/assets/images/synx-logo.svg" alt="Synx Logo" width="256" height="256">

[![Latest Release](https://img.shields.io/github/v/release/A5873/synx?style=for-the-badge&logo=github&label=Release)](https://github.com/A5873/synx/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

# Synx Package Repository

Welcome to the official Synx package repository. Choose your distribution below for installation instructions.
</div>

## Quick Installation

### Debian/Ubuntu

```bash
# Add the repository
echo "deb [trusted=yes] https://a5873.github.io/synx/repo/deb/dists/noble main" | sudo tee /etc/apt/sources.list.d/synx.list

# Update and install
sudo apt update
sudo apt install synx
```

### Fedora/RHEL

```bash
# Add the repository configuration
sudo tee /etc/yum.repos.d/synx.repo << REPO
[synx]
name=Synx Package Repository
baseurl=https://a5873.github.io/synx/repo/rpm
enabled=1
gpgcheck=0
REPO

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
- [Browse Debian packages](https://a5873.github.io/synx/repo/deb/pool/main/s/synx/)

### RPM
- synx-0.2.1-1.fc38.x86_64.rpm - Latest stable release
- [Browse RPM repository](https://a5873.github.io/synx/repo/rpm/)

## Direct Download Links

- [Download synx_0.2.1_amd64.deb](https://a5873.github.io/synx/repo/deb/pool/main/s/synx/synx_0.2.1_amd64.deb)
- [Download source package (synx_0.2.1.tar.xz)](https://a5873.github.io/synx/repo/deb/pool/main/s/synx/synx_0.2.1.tar.xz)

## Support

For issues, feature requests, or general support:
- [GitHub Issues](https://github.com/A5873/synx/issues)
- [Documentation](https://github.com/A5873/synx/blob/main/README.md)

## License

This project is licensed under the MIT License. See the [LICENSE](https://github.com/A5873/synx/blob/main/LICENSE) file in the main repository for details.

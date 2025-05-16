# Synx Package Distribution

This directory contains packaging configurations for distributing Synx across different platforms and package managers.

## Package Formats

### AUR (Arch User Repository)
- Location: `aur/`
- Package name: `synx`
- Status: ✅ Available in AUR
- Install: `yay -S synx` or `paru -S synx`

### Debian/Ubuntu
- Location: `deb/`
- Status: 🔄 In Progress
- Target: Ubuntu (Noble) and Debian (Bookworm)
- Build: See `deb/README.md` for build instructions

### RPM (Red Hat/Fedora)
- Location: `rpm/`
- Status: 🔄 Ready for testing
- Target: Fedora, RHEL, and compatible distributions
- Build: Use `rpmbuild -ba rpm/synx.spec`

### Homebrew
- Location: `brew/`
- Status: 🔄 Formula under review
- Install: Coming soon via `brew install synx`

## Building Packages

### Prerequisites
- Rust toolchain
- Cargo
- Platform-specific build tools:
  - Debian: `build-essential`, `debhelper`, `dh-make`
  - RPM: `rpm-build`, `rpmdevtools`
  - AUR: `base-devel`

### General Build Process
1. Build the Rust binary: `cargo build --release`
2. Package using the appropriate tool for your target format
3. Test the package in a clean environment
4. Submit for distribution

See individual package directories for specific build instructions.

## Package Maintenance

### Version Updates
When updating Synx version:
1. Update version in `source/Cargo.toml`
2. Update the following files:
   - `aur/PKGBUILD`
   - `rpm/synx.spec`
   - `deb/debian/changelog`
   - `brew/synx.rb`

### Distribution Status
- ✅ AUR: Active and maintained
- 🔄 Debian: In progress
- 🔄 RPM: Ready for testing
- 🔄 Homebrew: Under review

## Contributing
To add support for a new package format:
1. Create a new directory for the package format
2. Add necessary build configurations
3. Document build and installation process
4. Test thoroughly before submitting

For detailed contribution guidelines, see the main repository README.


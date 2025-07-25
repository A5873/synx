#!/bin/bash
# Synx 0.2.2 Installation Script for macOS
# This script installs synx binary and man page to system directories

set -e

echo "Installing Synx 0.2.2 for macOS..."

# Check if running as root or with sudo
if [[ $EUID -eq 0 ]]; then
    PREFIX="/usr/local"
else
    echo "This script requires sudo privileges to install to /usr/local"
    echo "Re-running with sudo..."
    exec sudo "$0" "$@"
fi

# Create directories if they don't exist
mkdir -p "${PREFIX}/bin"
mkdir -p "${PREFIX}/share/man/man1"

# Install binary
echo "Installing synx binary to ${PREFIX}/bin/"
cp usr/local/bin/synx "${PREFIX}/bin/synx"
chmod +x "${PREFIX}/bin/synx"

# Install man page
echo "Installing man page to ${PREFIX}/share/man/man1/"
cp usr/local/share/man/man1/synx.1.gz "${PREFIX}/share/man/man1/synx.1.gz"

# Verify installation
echo "Verifying installation..."
if command -v synx &> /dev/null; then
    echo "✅ Synx installed successfully!"
    echo "Version: $(synx --version | head -n 1)"
    echo ""
    echo "You can now use:"
    echo "  synx <files>        - Validate files"
    echo "  synx --help         - Show help"
    echo "  man synx            - View manual page"
    echo "  synx --init-config  - Generate configuration file"
else
    echo "❌ Installation verification failed. synx command not found in PATH."
    echo "You may need to restart your terminal or add ${PREFIX}/bin to your PATH."
    exit 1
fi

echo ""
echo "Installation complete! 🎉"


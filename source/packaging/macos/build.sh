#!/bin/bash
set -e

# Configuration
APP_NAME="synx"
VERSION="0.2.1"
MACOS_TARGET="x86_64-apple-darwin"

# Build with platform-specific features
cargo build --release --target $MACOS_TARGET --features macos-security

# Create app structure
PKG_ROOT="packaging/macos/root"
mkdir -p "$PKG_ROOT/usr/local/bin"
mkdir -p "$PKG_ROOT/usr/local/share/doc/$APP_NAME"

# Copy binary and documentation
cp "target/$MACOS_TARGET/release/$APP_NAME" "$PKG_ROOT/usr/local/bin/"
cp README.md LICENSE "$PKG_ROOT/usr/local/share/doc/$APP_NAME/"

# Create package
pkgbuild \
    --root "$PKG_ROOT" \
    --identifier "com.synx.pkg" \
    --version "$VERSION" \
    --install-location "/" \
    "packaging/macos/$APP_NAME-$VERSION.pkg"

echo "macOS package created at packaging/macos/$APP_NAME-$VERSION.pkg"

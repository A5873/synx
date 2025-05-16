#!/bin/bash

set -e

echo "Installing Synx and dependencies..."

# Detect OS and package manager
if [ -f /etc/debian_version ]; then
    # Debian/Ubuntu
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        pkg-config \
        python3 \
        python3-pip \
        golang \
        openjdk-21-jdk \
        tidy \
        shellcheck \
        jq \
        yamllint

    # Install Node.js and npm using nvm
    echo "Installing Node.js and npm using nvm..."
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
    export NVM_DIR="$HOME/.nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    nvm install node
    nvm use node

    # Install Node.js tools
    npm install -g \
        typescript \
        @typescript-eslint/parser \
        @typescript-eslint/eslint-plugin \
        eslint \
        csslint

    # Install Python tools
    sudo pip3 install \
        mypy \
        pylint

    # Install Go tools
    go install golang.org/x/lint/golint@latest
    curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin v1.56.2

elif [ -f /etc/fedora-release ]; then
    # Fedora/RHEL
    sudo dnf install -y \
        gcc \
        gcc-c++ \
        golang \
        java-latest-openjdk-devel \
        nodejs \
        npm \
        python3 \
        python3-pip \
        tidy \
        ShellCheck \
        jq \
        yamllint

    # Rest of the installations same as above
    npm install -g \
        typescript \
        @typescript-eslint/parser \
        @typescript-eslint/eslint-plugin \
        eslint \
        csslint

    sudo pip3 install \
        mypy \
        pylint

    go install golang.org/x/lint/golint@latest
    curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin v1.56.2

elif [ -f /etc/arch-release ]; then
    # Arch Linux
    sudo pacman -Sy --needed \
        base-devel \
        go \
        nodejs \
        npm \
        python \
        python-pip \
        jdk-openjdk \
        tidy \
        shellcheck \
        jq \
        yamllint

    # Rest of the installations same as above
    npm install -g \
        typescript \
        @typescript-eslint/parser \
        @typescript-eslint/eslint-plugin \
        eslint \
        csslint

    sudo pip3 install \
        mypy \
        pylint

    go install golang.org/x/lint/golint@latest
    curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin v1.56.2
else
    echo "Unsupported operating system"
    exit 1
fi

# Install synx
echo "Building and installing synx..."
cargo install --path .

# Install man pages
echo "Installing man pages..."
sudo mkdir -p /usr/local/share/man/man1/
sudo cp packaging/man/synx.1.gz /usr/local/share/man/man1/
sudo mandb

echo "Installation complete! Run 'synx --help' to get started."

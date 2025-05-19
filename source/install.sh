#!/bin/bash

set -e

# Display SYNX banner
echo -e "\033[1;36m"
cat << "EOF"

 ___ _   _ _ __ __  __
/ __| | | | '_ \\ \/ /
\__ \ |_| | | | |>  < 
|___/\__, |_| |_/_/\_\
     |___/  
                              
EOF
echo -e "\033[0m"

echo -e "\033[1;32m==================================================\033[0m"
echo -e "\033[1;32m            SYNX INSTALLATION SCRIPT             \033[0m"
echo -e "\033[1;32m==================================================\033[0m"
echo -e "Installing Synx and dependencies...\n"

# Detect OS and package manager
if [ -f /etc/debian_version ]; then
    # Debian/Ubuntu
    echo -e "\n\033[1;33m[Step 1/5]\033[0m Installing system dependencies..."
    echo -e "Detected: \033[1;36mDebian/Ubuntu\033[0m based system"
    
    echo "→ Updating package lists..."
    sudo apt-get update
    
    echo "→ Installing essential packages..."
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
    
    # Check if installation was successful
    if [ $? -ne 0 ]; then
        echo -e "\033[1;31mError:\033[0m Failed to install essential packages. Please check your internet connection and try again."
        exit 1
    fi

    echo -e "\n\033[1;33m[Step 2/5]\033[0m Setting up Node.js environment..."
    # Install Node.js and npm using nvm
    echo "→ Installing Node.js and npm using nvm..."
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
    export NVM_DIR="$HOME/.nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    nvm install node
    nvm use node
    
    if ! command -v node &> /dev/null; then
        echo -e "\033[1;31mWarning:\033[0m Node.js installation may not have been successful."
        echo "You might need to restart your terminal and run 'nvm use node' manually."
    else
        echo "✓ Node.js $(node -v) and npm $(npm -v) installed successfully."
    fi

    echo -e "\n\033[1;33m[Step 3/5]\033[0m Installing Node.js tools..."
    # Install Node.js tools
    npm install -g \
        typescript \
        @typescript-eslint/parser \
        @typescript-eslint/eslint-plugin \
        eslint \
        csslint
    
    if [ $? -eq 0 ]; then
        echo "✓ TypeScript and linting tools installed successfully."
    fi

    echo -e "\n\033[1;33m[Step 4/5]\033[0m Installing Python tools..."
    # Install Python tools
    sudo pip3 install \
        mypy \
        pylint
    
    if [ $? -eq 0 ]; then
        echo "✓ Python tools installed successfully."
    fi

    echo -e "\n\033[1;33m[Step 5/5]\033[0m Installing Go tools..."
    # Install Go tools
    echo "→ Installing golint..."
    go install golang.org/x/lint/golint@latest
    
    echo "→ Installing golangci-lint..."
    curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin v1.56.2
    
    if command -v golint &> /dev/null && command -v golangci-lint &> /dev/null; then
        echo "✓ Go tools installed successfully."
    else
        echo -e "\033[1;33mNote:\033[0m You may need to add Go bin directory to your PATH:"
        echo "export PATH=\$PATH:\$(go env GOPATH)/bin"
    fi

elif [ -f /etc/fedora-release ]; then
    # Fedora/RHEL
    echo -e "\n\033[1;33m[Step 1/5]\033[0m Installing system dependencies..."
    echo -e "Detected: \033[1;36mFedora/RHEL\033[0m based system"
    
    echo "→ Installing essential packages..."
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
    
    if [ $? -ne 0 ]; then
        echo -e "\033[1;31mError:\033[0m Failed to install essential packages. Please check your internet connection and try again."
        exit 1
    fi

    echo -e "\n\033[1;33m[Step 2/5]\033[0m Verifying Node.js environment..."
    # Check Node.js installation
    if command -v node &> /dev/null; then
        echo "✓ Node.js $(node -v) and npm $(npm -v) installed successfully."
    else
        echo -e "\033[1;31mError:\033[0m Node.js installation failed. Please install manually and retry."
        exit 1
    fi

    echo -e "\n\033[1;33m[Step 3/5]\033[0m Installing Node.js tools..."
    # Install Node.js tools
    npm install -g \
        typescript \
        @typescript-eslint/parser \
        @typescript-eslint/eslint-plugin \
        eslint \
        csslint
    
    if [ $? -eq 0 ]; then
        echo "✓ TypeScript and linting tools installed successfully."
    fi

    echo -e "\n\033[1;33m[Step 4/5]\033[0m Installing Python tools..."
    # Install Python tools
    sudo pip3 install \
        mypy \
        pylint
    
    if [ $? -eq 0 ]; then
        echo "✓ Python tools installed successfully."
    fi

    echo -e "\n\033[1;33m[Step 5/5]\033[0m Installing Go tools..."
    # Install Go tools
    echo "→ Installing golint..."
    go install golang.org/x/lint/golint@latest
    
    echo "→ Installing golangci-lint..."
    curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin v1.56.2
    
    if command -v golint &> /dev/null && command -v golangci-lint &> /dev/null; then
        echo "✓ Go tools installed successfully."
    else
        echo -e "\033[1;33mNote:\033[0m You may need to add Go bin directory to your PATH:"
        echo "export PATH=\$PATH:\$(go env GOPATH)/bin"
    fi

elif [ -f /etc/arch-release ]; then
    # Arch Linux
    echo -e "\n\033[1;33m[Step 1/5]\033[0m Installing system dependencies..."
    echo -e "Detected: \033[1;36mArch Linux\033[0m based system"
    
    echo "→ Installing essential packages..."
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
    
    if [ $? -ne 0 ]; then
        echo -e "\033[1;31mError:\033[0m Failed to install essential packages. Please check your internet connection and try again."
        exit 1
    fi

    echo -e "\n\033[1;33m[Step 2/5]\033[0m Verifying Node.js environment..."
    # Check Node.js installation
    if command -v node &> /dev/null; then
        echo "✓ Node.js $(node -v) and npm $(npm -v) installed successfully."
    else
        echo -e "\033[1;31mError:\033[0m Node.js installation failed. Please install manually and retry."
        exit 1
    fi

    echo -e "\n\033[1;33m[Step 3/5]\033[0m Installing Node.js tools..."
    # Install Node.js tools
    npm install -g \
        typescript \
        @typescript-eslint/parser \
        @typescript-eslint/eslint-plugin \
        eslint \
        csslint
    
    if [ $? -eq 0 ]; then
        echo "✓ TypeScript and linting tools installed successfully."
    fi

    echo -e "\n\033[1;33m[Step 4/5]\033[0m Installing Python tools..."
    # Install Python tools
    sudo pip3 install \
        mypy \
        pylint
    
    if [ $? -eq 0 ]; then
        echo "✓ Python tools installed successfully."
    fi

    echo -e "\n\033[1;33m[Step 5/5]\033[0m Installing Go tools..."
    # Install Go tools
    echo "→ Installing golint..."
    go install golang.org/x/lint/golint@latest
    
    echo "→ Installing golangci-lint..."
    curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin v1.56.2
    
    if command -v golint &> /dev/null && command -v golangci-lint &> /dev/null; then
        echo "✓ Go tools installed successfully."
    else
        echo -e "\033[1;33mNote:\033[0m You may need to add Go bin directory to your PATH:"
        echo "export PATH=\$PATH:\$(go env GOPATH)/bin"
    fi
else
    echo -e "\033[1;31mError:\033[0m Unsupported operating system."
    echo "This script currently supports Debian/Ubuntu, Fedora/RHEL, and Arch Linux."
    echo "For other distributions, please refer to the manual installation guide in the README."
    exit 1
fi

cargo install --path .

# Install man pages
echo "Installing man pages..."
sudo mkdir -p /usr/local/share/man/man1/
sudo cp packaging/man/synx.1.gz /usr/local/share/man/man1/
sudo mandb

echo "Installation complete! Run 'synx --help' to get started."

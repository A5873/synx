# Synx 0.2.2 - macOS x86_64 Package

Universal syntax validator and linter dispatcher for macOS.

## Installation

### Quick Install
```bash
./install.sh
```

### Manual Install
```bash
# Copy binary
sudo cp usr/local/bin/synx /usr/local/bin/
sudo chmod +x /usr/local/bin/synx

# Copy man page
sudo cp usr/local/share/man/man1/synx.1.gz /usr/local/share/man/man1/
```

## Verification

After installation, verify synx is working:

```bash
synx --version
man synx
```

## Supported Languages

- **Rust** (.rs) - Uses rustc
- **Python** (.py) - Uses python3 -m py_compile
- **JavaScript** (.js) - Uses node --check
- **TypeScript** (.ts, .tsx) - Uses tsc
- **JSON** (.json) - Uses jq
- **YAML** (.yaml, .yml) - Uses yamllint
- **C** (.c, .h) - Uses gcc
- **C++** (.cpp, .hpp) - Uses g++
- **Go** (.go) - Uses go vet
- **Java** (.java) - Uses javac
- **Shell** (.sh, .bash) - Uses shellcheck
- **HTML** (.html) - Uses tidy
- **CSS** (.css) - Uses csslint

## Quick Usage

```bash
# Validate single file
synx script.py

# Validate multiple files
synx main.rs config.json style.css

# Watch mode
synx --watch script.py

# Verbose output
synx --verbose --strict config.json

# Generate config
synx --init-config
```

## Dependencies

Install language-specific tools as needed:

```bash
# Essential tools
brew install jq node python3 shellcheck

# Language-specific (install as needed)
brew install rust go openjdk
npm install -g typescript eslint
pip3 install yamllint mypy pylint
```

## System Requirements

- macOS 10.12 or later
- x86_64 architecture
- 10 MB disk space

## Support

- **GitHub**: https://github.com/A5873/synx
- **Issues**: https://github.com/A5873/synx/issues
- **Documentation**: `man synx`

## License

MIT License - see the source repository for full details.


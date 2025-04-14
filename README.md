# Synx

A CLI-first universal syntax validator and linter dispatcher.

## Description

Synx is a command-line tool that inspects any file and attempts to validate its syntax or structure by automatically detecting the filetype and dispatching the appropriate validator or linter. Whether you're checking Python scripts, YAML configs, JSON data, Shell scripts, Dockerfiles, or HTML—Synx streamlines the process into a single command.

Think of it as a universal "linter dispatcher" for developers, sysadmins, and hackers.

### Why Synx?

- **No more guessing which linter to use**: Synx automatically detects file types and runs the appropriate validator
- **One command for everything**: Whether it's Python, JSON, YAML, or HTML, use the same command
- **Fast and efficient**: Built in Rust for performance
- **Configurable**: Easily customize validators and settings
- **Ideal for CI/CD pipelines**: Batch validate multiple files with a single command

## Installation

### From Source

Ensure you have Rust and Cargo installed. Then:

```bash
git clone https://github.com/your-username/synx.git
cd synx
cargo build --release
# Optional: copy to a directory in your PATH
cp target/release/synx ~/.local/bin/
```

### Dependencies

Synx relies on external validators being installed on your system. Based on the file types you plan to validate, you may need to install:

- **Python**: Python interpreter (for `.py` files)
- **JavaScript**: Node.js (for `.js` files)
- **JSON**: jq (for `.json` files)
- **YAML**: yamllint (for `.yaml`/`.yml` files)
- **HTML**: html-tidy (for `.html` files)
- **CSS**: csslint (for `.css` files)
- **Docker**: hadolint (for `Dockerfile` files)
- **Shell**: shellcheck (for `.sh` files)

## Usage

### Basic Usage

```bash
# Check a single file
synx script.py

# Check multiple files
synx config.yaml script.py Dockerfile

# Check all files of a specific type
synx *.json
```

### Advanced Options

```bash
# Show verbose output
synx --verbose script.py

# Watch for file changes and revalidate
synx --watch script.py

# Use a custom config file
synx --config ~/.config/synx/custom-config.toml script.py
```

## Supported File Types

Synx can validate the following file types:

- **Python** (`.py`)
- **JavaScript** (`.js`)
- **TypeScript** (`.ts`)
- **HTML** (`.html`, `.htm`)
- **CSS** (`.css`)
- **JSON** (`.json`)
- **YAML** (`.yaml`, `.yml`)
- **TOML** (`.toml`)
- **Dockerfile** (`Dockerfile`)
- **Shell** (`.sh`, `.bash`, `.zsh`)
- **Markdown** (`.md`, `.markdown`)
- **C** (`.c`)
- **C++** (`.cpp`, `.cc`, `.cxx`)
- **Rust** (`.rs`)

Support for additional file types can be added through configuration.

## Configuration

Synx uses a TOML configuration file located at `~/.config/synx/config.toml`. You can customize validators, add file type mappings, and configure default behavior.

### Default Configuration

```toml
[general]
verbose = false
strict = false
timeout = 30

[validators.python]
command = "python"
args = ["-m", "py_compile"]
strict = false
timeout = 10
enabled = true

[validators.json]
command = "jq"
args = ["."]
strict = true
timeout = 5
enabled = true

# File name to validator mappings
[file_mappings]
"Dockerfile" = "dockerfile"
"Jenkinsfile" = "groovy"
"Makefile" = "makefile"
```

### Custom Validators

You can add or modify validators in your config file:

```toml
[validators.custom_lang]
command = "custom-validator"
args = ["--option1", "--option2"]
strict = true
timeout = 15
enabled = true
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.


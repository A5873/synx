# Contributing to Synx

Thank you for considering contributing to Synx! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

By participating in this project, you are expected to uphold our Code of Conduct: be respectful, considerate, and collaborative.

## How Can I Contribute?

### Reporting Bugs

- Before submitting a bug report, please check the issue tracker to avoid duplicates
- Use a clear and descriptive title
- Describe the exact steps to reproduce the problem
- Provide specific examples (command line used, expected vs. actual output, etc.)
- Include details about your environment (OS, Rust version, etc.)

### Suggesting Enhancements

- Use a clear and descriptive title
- Provide a step-by-step description of the suggested enhancement
- Explain why this enhancement would be useful to most Synx users

### Adding Support for New File Types

Synx is designed to be extensible. To add support for a new file type:

1. Add the new file type to the `FileType` enum in `src/detectors/mod.rs`
2. Implement detection logic in the `detect_file_type` function
3. Create a validator implementation in `src/validators/mod.rs`
4. Update the `get_validator` function to return your new validator

### Pull Requests

1. Fork the repository
2. Create a new branch for your feature or bugfix
3. Make your changes
4. Run the tests to ensure everything works
5. Submit a pull request

## Development Workflow

### Setting Up Development Environment

```bash
# Clone your fork
git clone https://github.com/YOUR-USERNAME/synx.git
cd synx

# Install development dependencies
cargo build
```

### Style Guidelines

- Follow the Rust style guide
- Use `cargo fmt` to format your code
- Use `cargo clippy` to catch common mistakes

### Testing

- Write tests for new features or bug fixes
- Run tests with `cargo test`
- Add example files for new file type support in the `examples` directory

## Release Process

Synx follows semantic versioning (SemVer).

## Questions?

Feel free to reach out to the maintainer (Alex Ngugi) with any questions.

Thank you for contributing to Synx!


# Contributing to FerrisFetcher

Thank you for your interest in contributing to FerrisFetcher! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or higher
- Git
- Basic knowledge of web scraping and async programming

### Development Setup

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/your-username/FerrisFetcher.git
   cd FerrisFetcher
   ```
3. Install dependencies and run tests:
   ```bash
   cargo build
   cargo test
   ```

## 📋 Development Guidelines

### Code Style

- Follow Rust's standard formatting: `cargo fmt`
- Use clippy for linting: `cargo clippy -- -D warnings`
- Write clear, documented code with appropriate comments
- Use meaningful variable and function names

### Testing

- Write unit tests for new functionality
- Add integration tests for complex features
- Ensure all tests pass: `cargo test`
- Maintain test coverage

### Documentation

- Document public APIs with `///` doc comments
- Include examples in documentation
- Update README.md for user-facing changes
- Keep CHANGELOG.md updated

## 🏗️ Architecture

FerrisFetcher is organized into several modules:

- **client**: HTTP client with retry logic and rate limiting
- **config**: Configuration management and validation
- **error**: Comprehensive error handling
- **extractor**: Data extraction with configurable rules
- **html_parser**: HTML parsing and CSS selector support
- **scraper**: Main API and orchestration
- **types**: Core data structures and enums

### Adding New Features

1. Consider which module the feature belongs to
2. Add appropriate types to `types.rs` if needed
3. Implement the feature in the relevant module
4. Add comprehensive tests
5. Update documentation
6. Update examples if applicable

## 🐛 Bug Reports

When reporting bugs, please include:

- Rust version (`rustc --version`)
- Operating system
- Minimum reproducible example
- Expected vs actual behavior
- Any relevant logs or error messages

## ✨ Feature Requests

Feature requests are welcome! Please:

- Check if the feature already exists or is planned
- Provide a clear description of the use case
- Consider if it fits the project's goals
- Be open to discussion and refinement

## 🔄 Pull Request Process

1. Create a new branch for your feature: `git checkout -b feature-name`
2. Make your changes and commit them with clear messages
3. Ensure all tests pass and code is formatted
4. Push to your fork: `git push origin feature-name`
5. Open a pull request with a descriptive title and body

### Pull Request Guidelines

- Keep PRs focused and reasonably sized
- Link to relevant issues
- Include tests for new functionality
- Update documentation as needed
- Ensure CI passes

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run examples
cargo run --example example_name
```

### Test Categories

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test component interactions
- **Example tests**: Verify examples work correctly

## 📚 Documentation

### Building Documentation

```bash
# Generate documentation
cargo doc --open

# Check documentation coverage
cargo doc --document-private-items
```

### Documentation Standards

- All public items must have documentation
- Include examples in doc comments
- Use proper markdown formatting
- Cross-reference related items

## 🔧 Development Tools

### Useful Commands

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for security vulnerabilities
cargo audit

# Update dependencies
cargo update

# Benchmark performance
cargo bench
```

### Pre-commit Hooks

Consider using pre-commit hooks to ensure code quality:

```bash
# Install pre-commit
pip install pre-commit

# Setup hooks
pre-commit install
```

## 🤝 Code of Conduct

Please be respectful and professional in all interactions. We're here to build great software together.

## 📝 Release Process

Releases are managed through semantic versioning:

- **Major**: Breaking changes
- **Minor**: New features (backward compatible)
- **Patch**: Bug fixes

### Release Checklist

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Tag the release: `git tag v0.1.0`
4. Push tag: `git push origin v0.1.0`
5. Create GitHub release

## 🎯 Priority Areas

We're currently focusing on:

1. **Performance**: Optimizing for large-scale scraping
2. **Reliability**: Improving error handling and retry logic
3. **Usability**: Making the API more intuitive
4. **Documentation**: Comprehensive guides and examples
5. **Testing**: Increasing test coverage

## 📞 Getting Help

- **Issues**: For bugs and feature requests
- **Discussions**: For questions and ideas
- **Discord/Slack**: For real-time conversation (if available)

## 🙏 Recognition

Contributors are recognized in:

- README.md contributors section
- Release notes
- Git commit history
- GitHub contributors page

Thank you for contributing to FerrisFetcher! 🦀✨

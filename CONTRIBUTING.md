# Contributing to cltree

Thank you for your interest in contributing to cltree! This document provides guidelines and instructions for contributing.

## Language Policy

- **English is preferred** for all contributions including code, comments, documentation, issues, and pull requests.
- This ensures the project is accessible to the global community.

## Development Requirements

- **Rust**: 1.70 or later
- **Cargo**: Rust's package manager (included with Rust)
- **Git**: For version control

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/cltree.git
   cd cltree
   ```
3. Add the upstream repository as a remote:
   ```bash
   git remote add upstream https://github.com/jsleemaster/cltree.git
   ```

## Build Commands

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Apply code formatting
cargo fmt

# Run linter
cargo clippy

# Run linter with warnings as errors
cargo clippy -- -D warnings

# Run the application
cargo run
```

## Alternative Installation (for Rust developers)

If you have Rust installed, you can install cltree directly:

```bash
# From crates.io
cargo install cltree

# From source
git clone https://github.com/jsleemaster/cltree.git
cd cltree
cargo install --path .
```

## Code Conventions

- **Edition**: Rust 2021
- **Formatting**: Follow `rustfmt` defaults (run `cargo fmt` before committing)
- **Linting**: Code must pass `cargo clippy` without warnings
- **Naming**:
  - `snake_case` for functions, methods, variables, and modules
  - `PascalCase` for types, traits, and enums
  - `SCREAMING_SNAKE_CASE` for constants
- **Error Handling**: Use `anyhow::Result` for application code, `thiserror` for library boundaries
- **Comments**: Public APIs should have English doc comments

## Contribution Workflow

1. **Create a branch** for your work:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. **Make your changes** following the code conventions above.

3. **Test your changes**:
   ```bash
   cargo test
   cargo fmt --check
   cargo clippy -- -D warnings
   ```

4. **Commit your changes** with a clear message:
   ```bash
   git commit -m "feat: add amazing new feature"
   # or
   git commit -m "fix: resolve issue with file tree"
   ```

   Commit message prefixes:
   - `feat:` - New feature
   - `fix:` - Bug fix
   - `docs:` - Documentation changes
   - `refactor:` - Code refactoring
   - `test:` - Adding or updating tests
   - `chore:` - Maintenance tasks

5. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request** on GitHub against the `main` branch.

## Pull Request Guidelines

- Fill out the PR template completely
- Link any related issues
- Ensure all CI checks pass
- Keep PRs focused on a single change
- Update documentation if needed
- Add tests for new functionality

## Reporting Issues

- Check existing issues before creating a new one
- Use the issue templates provided
- Include reproduction steps for bugs
- Provide environment details (OS, Rust version, terminal)

## Code of Conduct

Be respectful and inclusive. We welcome contributors from all backgrounds and experience levels.

## Questions?

If you have questions, feel free to:
- Open a [Discussion](https://github.com/jsleemaster/cltree/discussions)
- Ask in your PR or issue

Thank you for contributing!

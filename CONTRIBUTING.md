# Contributing to Sena1996 AI Tool

Thank you for your interest in contributing to Sena1996 AI Tool! This document provides guidelines and information for contributors.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/Sena1996/Sena1996-AI/issues)
2. If not, create a new issue with:
   - Clear, descriptive title
   - Steps to reproduce
   - Expected vs actual behavior
   - System information (OS, Rust version)
   - Relevant logs or screenshots

### Suggesting Features

1. Check existing issues and discussions
2. Create a new issue with the `enhancement` label
3. Describe the feature and its use case
4. Explain why it would benefit users

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes following our coding standards
4. Write/update tests as needed
5. Run the test suite: `cargo test`
6. Run clippy: `cargo clippy -- -D warnings`
7. Format code: `cargo fmt`
8. Commit with clear messages
9. Push and create a Pull Request

## Development Setup

```bash
# Clone the repository
git clone https://github.com/Sena1996/Sena1996-AI.git
cd Sena1996-AI

# Build
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

## Coding Standards

We follow the **SENA1996-AI Elite Coding Standards** (see CLAUDE.md):

### Critical Rules
- Use `Result<T, SenaError>` for fallible operations
- Never use `unwrap()` in production code
- Never add comments - code must self-explain
- Keep functions under 20 lines
- Zero warnings: `cargo build --release`

### Style Guidelines
- Prefer `&str` over `String` in parameters
- Prefer iterators over `for` loops
- Use early return / guard clauses
- Follow Rust API Guidelines

## Commit Messages

Format: `<type>: <description>`

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance

Example: `feat: Add multi-AI provider routing`

## Testing

- Write unit tests for new functionality
- Ensure all tests pass before submitting PR
- Add integration tests for complex features
- Test edge cases

## Questions?

- Open a [Discussion](https://github.com/Sena1996/Sena1996-AI/discussions)
- Check existing documentation

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to Sena1996 AI Tool!**

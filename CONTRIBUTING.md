# Contributing to LC-3 Assembler

Thank you for your interest in contributing to the LC-3 Assembler! This document provides guidelines for contributing to the project.

## Code of Conduct

Be respectful and inclusive. We're all here to learn and build great tools together.

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues. When creating a bug report, include:

- **Clear title and description**
- **Steps to reproduce** the issue
- **Expected behavior** vs actual behavior
- **LC-3 assembly code** that triggers the bug
- **Environment details** (OS, Rust version, assembler version)

**Example:**
```markdown
## Bug: Parser fails on labels with underscores

### Steps to Reproduce
1. Create file with label `MY_LABEL`
2. Run `lc3-assembler test.asm`

### Expected
Assembly completes successfully

### Actual
Error: "Unexpected character '_'"

### Environment
- OS: Ubuntu 22.04
- Rust: 1.75.0
- Assembler: v1.0.0
```

### Suggesting Enhancements

Enhancement suggestions are welcome! Please:

1. **Check FUTURE_IMPROVEMENTS.md** to see if it's already planned
2. **Explain the use case** - why is this useful?
3. **Provide examples** of how it would work
4. **Consider implementation** complexity

### Pull Requests

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/my-feature`
3. **Make your changes** (see coding guidelines below)
4. **Add tests** for new functionality
5. **Run the test suite**: `cargo test`
6. **Run fmt and clippy**:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   ```
7. **Commit your changes**: See commit guidelines below
8. **Push to your fork**: `git push origin feature/my-feature`
9. **Open a Pull Request** against `main`

## Development Setup

### Prerequisites

- Rust 1.60 or later
- Cargo (comes with Rust)

### Initial Setup

```bash
# Clone your fork
git clone https://github.com/your-username/lc3-assembler
cd lc3-assembler

# Build
cargo build

# Run tests
cargo test

# Install development tools
rustup component add rustfmt clippy
```

### Project Structure

```
lc3-assembler/
├── src/
│   ├── lib.rs                # Public API
│   ├── main.rs               # CLI binary
│   ├── error.rs              # Error types
│   ├── lexer/                # Tokenization
│   ├── parser/               # AST generation
│   ├── first_pass/           # Symbol table building
│   └── encoder/              # Machine code generation
├── tests/
│   ├── integration_tests.rs  # End-to-end tests
│   └── test_programs/        # Example .asm files
└── syntax-highlighting/      # Editor support files
```

## Coding Guidelines

### Rust Style

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

- Use `cargo fmt` for formatting
- Fix all Clippy warnings
- Write idiomatic Rust (use iterators, ? operator, etc.)
- Prefer owned types over lifetimes when reasonable
- Use descriptive variable names

**Good:**
```rust
pub fn calculate_pc_offset(target_address: u16, current_address: u16) -> Result<i16, AsmError> {
    let offset = target_address.wrapping_sub(current_address).wrapping_sub(1);
    Ok(offset as i16)
}
```

**Bad:**
```rust
pub fn calc(t: u16, c: u16) -> i16 {
    (t - c - 1) as i16  // No error handling!
}
```

### Documentation

- **All public items** must have doc comments
- **Complex logic** should have inline comments
- **Examples** in doc comments are highly encouraged

```rust
/// Calculates the PC-relative offset from current to target address.
///
/// The LC-3 PC points to the next instruction during execution, so:
/// `offset = target_address - (current_address + 1)`
///
/// # Examples
///
/// ```
/// let offset = calculate_pc_offset(0x3010, 0x3005)?;
/// assert_eq!(offset, 10);
/// ```
///
/// # Errors
///
/// Returns `OffsetOutOfRange` if the offset exceeds 9-bit signed range.
pub fn calculate_pc_offset(target_address: u16, current_address: u16) -> Result<i16, AsmError> {
    // Implementation...
}
```

### Testing

- **Write tests** for all new functionality
- **Unit tests** for individual functions
- **Integration tests** for full pipeline
- **Test edge cases** (boundary values, error conditions)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pc_offset_calculation() {
        assert_eq!(calculate_pc_offset(0x3010, 0x3005).unwrap(), 10);
    }

    #[test]
    fn test_pc_offset_negative() {
        assert_eq!(calculate_pc_offset(0x3000, 0x3010).unwrap(), -17);
    }

    #[test]
    fn test_pc_offset_out_of_range() {
        let result = calculate_pc_offset(0x3000, 0x3200);
        assert!(result.is_err());
    }
}
```

### Error Handling

- Use `Result<T, AsmError>` for operations that can fail
- Provide **helpful error messages** with context
- Include **span information** for source location

```rust
// Good
return Err(AsmError {
    kind: ErrorKind::OffsetOutOfRange,
    message: format!(
        "PC offset {} to label '{}' exceeds 9-bit range [-256, 255]",
        offset, label
    ),
    span,
});

// Bad
return Err(AsmError {
    kind: ErrorKind::InvalidOffset,
    message: "Bad offset".to_string(),
    span,
});
```

## Commit Guidelines

Follow [Conventional Commits](https://www.conventionalcommits.org/):

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code formatting (no functional change)
- **refactor**: Code refactoring
- **test**: Adding or updating tests
- **chore**: Maintenance tasks

### Examples

```
feat(parser): add support for .MACRO directive

Implements macro expansion in the parser. Macros are expanded
during a pre-processing pass before the first pass.

Closes #42
```

```
fix(encoder): correct PC offset calculation for backward branches

The offset calculation was off by one for backward branches due to
not accounting for the PC increment. This fixes the issue by
subtracting an additional 1.

Fixes #58
```

```
docs(readme): add syntax highlighting installation guide

Added comprehensive guide for installing syntax highlighting in
VS Code, Vim, Sublime Text, and Emacs.
```

## Areas for Contribution

See [FUTURE_IMPROVEMENTS.md](FUTURE_IMPROVEMENTS.md) for ideas. High-impact areas:

### Easy (Good First Issues)

- Add more test cases
- Improve error messages
- Write documentation
- Create example programs

### Medium

- Better error messages with suggestions (see FUTURE_IMPROVEMENTS.md #4.3)
- Expression evaluation in operands (see FUTURE_IMPROVEMENTS.md #3.6)
- Listing file generation (see FUTURE_IMPROVEMENTS.md #3.4)

### Hard

- Language Server Protocol implementation
- Macro system
- Include file support
- Conditional assembly

## Review Process

1. **Automated checks** must pass (CI)
2. **Code review** by maintainer
3. **Tests** must be included for new features
4. **Documentation** must be updated
5. **Changelog** entry added (for features/fixes)

### Review Checklist

Reviewers will check:
- ✅ Tests pass
- ✅ Code follows style guidelines
- ✅ Documentation is clear
- ✅ No clippy warnings
- ✅ Commit messages follow conventions
- ✅ CHANGES.md updated (if applicable)

## Questions?

- Open an issue for questions
- Tag with `question` label
- We're happy to help!

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing! 🚀

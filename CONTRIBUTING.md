# Contributing to Zush Prompt

Thank you for your interest in contributing to Zush Prompt! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git
- A terminal emulator that supports 24-bit color

### Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/zush-prompt-rust.git
   cd zush-prompt-rust
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Install locally for testing:**
   ```bash
   cargo build --release
   cp target/release/zush-prompt ~/.local/bin/
   ```

## Commit Message Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/) for automated versioning and changelog generation.

### Format

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

### Types

| Type | Description | Version Bump |
|------|-------------|--------------|
| `feat` | New feature | Minor (0.1.0 → 0.2.0) |
| `fix` | Bug fix | Patch (0.1.0 → 0.1.1) |
| `docs` | Documentation only | None |
| `style` | Code style/formatting | None |
| `refactor` | Code refactoring | Patch |
| `perf` | Performance improvement | Patch |
| `test` | Adding tests | None |
| `build` | Build system changes | None |
| `ci` | CI configuration | None |
| `chore` | Maintenance tasks | None |

### Examples

**New Feature:**
```bash
git commit -m "feat(themes): add ocean theme with wave icons"
```

**Bug Fix:**
```bash
git commit -m "fix(template): resolve color tag parsing error

The template preprocessor was not properly closing nested color tags.
This fixes the issue by tracking tag depth."
```

**Breaking Change:**
```bash
git commit -m "feat(config)!: change theme configuration format

BREAKING CHANGE: Theme files now use TOML format instead of JSON.
See migration guide in docs/migration.md"
```

**Documentation:**
```bash
git commit -m "docs: add installation guide for Homebrew"
```

**Multiple Changes:**
```bash
git commit -m "feat(themes): add candy and sakura themes

- Add candy theme with pastel colors
- Add sakura theme with cherry blossom aesthetics
- Update theme documentation"
```

## Pull Request Process

1. **Fork the repository** and create a new branch from `main`:
   ```bash
   git checkout -b feat/my-new-feature
   ```

2. **Make your changes** following the coding standards

3. **Write tests** for new features or bug fixes

4. **Run the test suite:**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

5. **Commit your changes** using conventional commits

6. **Push to your fork:**
   ```bash
   git push origin feat/my-new-feature
   ```

7. **Open a Pull Request** with:
   - Clear title following conventional commit format
   - Description of changes
   - Screenshots for UI changes
   - Link to related issues

### PR Title Format

Use the same format as commit messages:
```
feat(themes): add galaxy theme with cosmic colors
fix(git): resolve branch detection in detached HEAD state
docs: improve theme creation guide
```

## Coding Standards

### Rust Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Add documentation comments for public APIs
- Keep functions focused and small

### Theme Files

- Use TOML format
- Include metadata (name, description, author, version)
- Provide examples in `examples/themes/`
- Document custom colors and symbols

### Documentation

- Update README.md for user-facing changes
- Update THEMES_GUIDE.md for theme-related changes
- Add inline comments for complex logic
- Include examples in documentation

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Adding Tests

Add tests for:
- New features
- Bug fixes
- Template parsing
- Color calculations
- Git integration

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        let color = Color::from_hex("#ff0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }
}
```

## Creating Themes

See [THEMES_GUIDE.md](examples/THEMES_GUIDE.md) for detailed theme creation instructions.

### Quick Start

1. **Create a new TOML file** in `examples/themes/`:
   ```toml
   name = "my-theme"
   description = "A beautiful theme"
   author = "Your Name"
   version = "1.0.0"

   [colors]
   primary = "#ff6b9d"
   secondary = "#c5f1ff"

   [symbols]
   arrow = "❯"

   [templates]
   main = "(fg primary){{user}}(/fg) (fg secondary){{pwd_short}}(/fg)\n(fg primary)@arrow(/fg) "
   ```

2. **Test your theme:**
   ```bash
   cp examples/themes/my-theme.toml ~/.config/zush/themes/
   zush-theme my-theme
   ```

3. **Submit a PR** with your theme in `examples/themes/`

## Release Process

Releases are automated via GitHub Actions:

1. **Commit changes** using conventional commits
2. **Push to main** branch
3. **Automatic versioning** creates a new tag
4. **Automatic release** builds binaries for all platforms

See [.github/WORKFLOWS.md](.github/WORKFLOWS.md) for details.

## Getting Help

- **Issues:** [GitHub Issues](https://github.com/YOUR_USERNAME/zush-prompt-rust/issues)
- **Discussions:** [GitHub Discussions](https://github.com/YOUR_USERNAME/zush-prompt-rust/discussions)
- **Documentation:** See README.md and docs/

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Help others learn and grow

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

## Recognition

Contributors will be recognized in:
- CHANGELOG.md for their commits
- GitHub releases for their contributions
- README.md for significant contributions

Thank you for contributing to Zush Prompt! 🎨✨

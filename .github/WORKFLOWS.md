# GitHub Actions Workflows

This document explains the automated workflows for zush-prompt.

## Workflows

### 1. CI (`ci.yml`)

**Trigger:** Push to main/master/develop, Pull Requests

**Purpose:** Continuous Integration - ensures code quality on every push and PR

**Jobs:**
- **Test:** Runs on Ubuntu and macOS
  - Check code formatting with `cargo fmt`
  - Run linter with `cargo clippy`
  - Build the project
  - Run test suite

- **Build:** Builds release binaries on Ubuntu, macOS, and Windows
  - Uploads artifacts for verification

### 2. Version and Tag (`version.yml`)

**Trigger:** Push to main/master branch

**Purpose:** Automatically determines version bump based on conventional commits and creates tags

**How it works:**
1. Analyzes commit messages since last release
2. Determines version bump type:
   - `feat:` → Minor version bump (0.1.0 → 0.2.0)
   - `fix:` → Patch version bump (0.1.0 → 0.1.1)
   - `BREAKING CHANGE:` → Major version bump (0.1.0 → 1.0.0)
3. Updates `Cargo.toml` version
4. Updates `CHANGELOG.md`
5. Creates Git tag (e.g., `v0.2.0`)
6. Pushes tag to trigger release workflow

### 3. Release (`release.yml`)

**Trigger:** Push of version tags (v*.*.*)

**Purpose:** Builds and publishes release artifacts

**Jobs:**
1. **Create Release:**
   - Generates changelog from commits
   - Creates GitHub release with notes

2. **Build and Upload:**
   - Builds for multiple platforms:
     - macOS ARM64 (Apple Silicon)
     - macOS x86_64 (Intel)
     - Linux x86_64
   - Creates tarballs with checksums
   - Uploads to GitHub release

3. **Publish Homebrew:**
   - Provides instructions for Homebrew formula update

## Conventional Commits

This project uses [Conventional Commits](https://www.conventionalcommits.org/) for automated versioning.

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat:** A new feature (triggers minor version bump)
- **fix:** A bug fix (triggers patch version bump)
- **docs:** Documentation changes (no version bump)
- **style:** Code style changes (no version bump)
- **refactor:** Code refactoring (triggers patch version bump)
- **perf:** Performance improvements (triggers patch version bump)
- **test:** Adding tests (no version bump)
- **build:** Build system changes (no version bump)
- **ci:** CI configuration changes (no version bump)
- **chore:** Other changes (no version bump)

### Breaking Changes

To trigger a major version bump, include `BREAKING CHANGE:` in the footer:

```
feat: redesign theme API

BREAKING CHANGE: The theme configuration format has changed. Users must update their theme files.
```

Or use the `!` suffix:

```
feat!: redesign theme API
```

### Examples

**Feature (minor bump):**
```
feat(themes): add unicorn theme with rainbow gradients

Added a new pastel theme with rainbow colors and sparkle effects.
```

**Bug Fix (patch bump):**
```
fix(template): resolve unclosed tag error in sunset theme

Fixed template parsing error where color tags were not properly closed.
```

**Breaking Change (major bump):**
```
feat(config)!: change default theme to minimal

BREAKING CHANGE: The default fallback theme is now 'minimal' instead of 'split'.
Users who relied on 'split' being the default must explicitly set it in their config.
```

**Documentation (no bump):**
```
docs: update installation instructions for Homebrew
```

**Chore (no bump):**
```
chore: update dependencies
```

## Release Process

### Automatic (Recommended)

1. Make changes and commit with conventional commit messages
2. Push to main branch
3. Version workflow automatically:
   - Determines new version
   - Updates `Cargo.toml` and `CHANGELOG.md`
   - Creates and pushes version tag
4. Release workflow automatically:
   - Builds binaries for all platforms
   - Creates GitHub release with artifacts

### Manual

If you need to create a release manually:

1. Update version in `Cargo.toml`:
   ```toml
   [package]
   version = "0.2.0"
   ```

2. Create and push a tag:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

3. Release workflow will automatically build and publish

## Skipping CI

To skip CI on a commit (e.g., for documentation changes):

```
docs: update README [skip ci]
```

## Viewing Workflow Status

- Go to the **Actions** tab in GitHub
- Click on a workflow to see run history
- Click on a specific run to see logs

## Secrets Required

The workflows use these GitHub secrets (automatically available):
- `GITHUB_TOKEN` - For creating releases and pushing tags

No additional configuration needed!

## Release Assets

Each release includes:
- `zush-prompt-macos-arm64.tar.gz` - macOS Apple Silicon binary
- `zush-prompt-macos-x86_64.tar.gz` - macOS Intel binary
- `zush-prompt-linux-x86_64.tar.gz` - Linux x86_64 binary
- `*.sha256` - Checksums for each binary

## Troubleshooting

### Version workflow not creating tags

Check that:
- Commits follow conventional commit format
- Commits contain changes that warrant a version bump (feat/fix/etc.)
- You're pushing to main/master branch

### Release workflow failing

Check that:
- The tag follows the format `v*.*.*` (e.g., `v0.2.0`)
- Cargo.toml is valid
- All dependencies can be resolved

### Build failures

Check:
- Rust code compiles locally with `cargo build --release`
- All tests pass with `cargo test`
- Code is formatted with `cargo fmt`
- No clippy warnings with `cargo clippy`

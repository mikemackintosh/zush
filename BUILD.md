# Building Zush for Multiple Platforms

This guide explains how to build Zush for different operating systems and architectures.

## Quick Cross-Compilation

### Using cargo with target triples

```bash
# Install cross-compilation tool
cargo install cross

# Or use cargo directly with rustup targets
```

## Supported Platforms

Zush should work on any platform that supports:
- Rust toolchain
- Zsh shell
- ANSI terminal with 24-bit color support

### Tested Platforms

- ✅ macOS (arm64/Apple Silicon)
- ✅ macOS (x86_64/Intel)
- ✅ Linux (x86_64)
- ✅ Linux (aarch64/ARM64)
- ⚠️  Windows (WSL only - requires Zsh)

## Building for Specific Targets

### macOS (Apple Silicon - M1/M2/M3)

```bash
# Current architecture (if on Apple Silicon Mac)
cargo build --release

# Or explicitly
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Binary location
ls -lh target/aarch64-apple-darwin/release/zush-prompt
```

### macOS (Intel x86_64)

```bash
# On Intel Mac
cargo build --release

# Cross-compile from Apple Silicon
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Binary location
ls -lh target/x86_64-apple-darwin/release/zush-prompt
```

### Linux (x86_64)

```bash
# On Linux x86_64
cargo build --release

# Cross-compile from macOS (requires additional setup)
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# Note: Cross-compiling to Linux from macOS may require a cross-compiler
# Use Docker or cross tool for easier cross-compilation
```

### Linux (ARM64/aarch64)

```bash
# On ARM64 Linux
cargo build --release

# Cross-compile
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

### Using Docker for Linux builds (from macOS)

```bash
# Build for Linux x86_64 using Docker
docker run --rm -v "$PWD":/app -w /app rust:latest \
  cargo build --release --target x86_64-unknown-linux-gnu

# Build for Linux ARM64
docker run --rm -v "$PWD":/app -w /app rust:latest \
  cargo build --release --target aarch64-unknown-linux-gnu
```

## Using the `cross` Tool (Recommended for Cross-Platform Builds)

The `cross` tool provides Docker-based cross-compilation for Rust:

```bash
# Install cross
cargo install cross

# Build for different platforms
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-apple-darwin
cross build --release --target aarch64-apple-darwin

# Binaries are in target/<triple>/release/zush-prompt
```

## Building Universal macOS Binary

To create a universal binary that works on both Intel and Apple Silicon Macs:

```bash
# Build for both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create universal binary using lipo
lipo -create \
  target/x86_64-apple-darwin/release/zush-prompt \
  target/aarch64-apple-darwin/release/zush-prompt \
  -output zush-prompt-universal

# Verify
lipo -info zush-prompt-universal
# Output: Architectures in the fat file: zush-prompt-universal are: x86_64 arm64
```

## Automated Build Script

Create a `build-all.sh` script:

```bash
#!/bin/bash
set -e

echo "Building Zush for all platforms..."

# macOS targets
echo "Building for macOS (Apple Silicon)..."
cargo build --release --target aarch64-apple-darwin

echo "Building for macOS (Intel)..."
cargo build --release --target x86_64-apple-darwin

# Create universal macOS binary
echo "Creating universal macOS binary..."
mkdir -p dist
lipo -create \
  target/x86_64-apple-darwin/release/zush-prompt \
  target/aarch64-apple-darwin/release/zush-prompt \
  -output dist/zush-prompt-macos-universal

# Linux targets (requires cross tool)
if command -v cross &> /dev/null; then
    echo "Building for Linux x86_64..."
    cross build --release --target x86_64-unknown-linux-gnu
    cp target/x86_64-unknown-linux-gnu/release/zush-prompt dist/zush-prompt-linux-x86_64

    echo "Building for Linux ARM64..."
    cross build --release --target aarch64-unknown-linux-gnu
    cp target/aarch64-unknown-linux-gnu/release/zush-prompt dist/zush-prompt-linux-aarch64
else
    echo "cross tool not found, skipping Linux builds"
    echo "Install with: cargo install cross"
fi

echo "Build complete! Binaries in ./dist/"
ls -lh dist/
```

Make it executable and run:

```bash
chmod +x build-all.sh
./build-all.sh
```

## GitHub Actions CI/CD

Create `.github/workflows/build.yml` for automated builds:

```yaml
name: Build

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  release:
    types: [ created ]

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
            name: macos-intel
          - os: macos-latest
            target: aarch64-apple-darwin
            name: macos-arm64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: linux-x86_64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            name: linux-aarch64

    runs-on: ${{ matrix.os }}

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
        override: true

    - name: Build
      run: cargo build --release --target ${{ matrix.target }}

    - name: Rename binary
      run: |
        mkdir -p dist
        cp target/${{ matrix.target }}/release/zush-prompt \
           dist/zush-prompt-${{ matrix.name }}

    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: zush-prompt-${{ matrix.name }}
        path: dist/zush-prompt-${{ matrix.name }}
```

## Distribution

After building, you can distribute binaries:

```bash
# Create release archives
tar -czf zush-prompt-macos-universal.tar.gz -C dist zush-prompt-macos-universal
tar -czf zush-prompt-linux-x86_64.tar.gz -C dist zush-prompt-linux-x86_64
tar -czf zush-prompt-linux-aarch64.tar.gz -C dist zush-prompt-linux-aarch64

# Or create a single archive with all binaries
tar -czf zush-prompt-all-platforms.tar.gz dist/
```

## Installation From Pre-built Binaries

Users can install pre-built binaries:

```bash
# Download the appropriate binary for your platform
curl -L https://github.com/yourusername/zush-prompt/releases/latest/download/zush-prompt-macos-universal -o zush-prompt

# Make executable
chmod +x zush-prompt

# Move to PATH
mv zush-prompt ~/.local/bin/
```

## Target Triple Reference

Common target triples:

| Platform | Architecture | Target Triple |
|----------|-------------|---------------|
| macOS | Intel (x86_64) | `x86_64-apple-darwin` |
| macOS | Apple Silicon (ARM64) | `aarch64-apple-darwin` |
| Linux | x86_64 | `x86_64-unknown-linux-gnu` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` |
| Linux | ARMv7 | `armv7-unknown-linux-gnueabihf` |
| FreeBSD | x86_64 | `x86_64-unknown-freebsd` |

List all available targets:
```bash
rustc --print target-list | grep -E "(darwin|linux|freebsd)"
```

## Troubleshooting

### Cross-compilation errors

If you get linker errors when cross-compiling:

```bash
# Install cross tool instead
cargo install cross
cross build --release --target <target-triple>
```

### Missing dependencies

On Linux, you may need development packages:

```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config

# Fedora/RHEL
sudo dnf install gcc pkg-config
```

### musl target (static Linux binaries)

For fully static Linux binaries:

```bash
rustup target add x86_64-unknown-linux-musl
cross build --release --target x86_64-unknown-linux-musl
```

This creates binaries with no runtime dependencies.

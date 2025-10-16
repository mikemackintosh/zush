# Supported Platforms

Zush Prompt provides pre-built binaries for the following platforms:

## macOS

| Architecture | Platform | Binary Format | Target Triple |
|--------------|----------|---------------|---------------|
| Apple Silicon (M1/M2/M3) | macOS 11+ | `.tar.gz` | `aarch64-apple-darwin` |
| Intel x86_64 | macOS 10.13+ | `.tar.gz` | `x86_64-apple-darwin` |

**Installation:**
```bash
# Download and extract
tar -xzf zush-prompt-macos-arm64.tar.gz  # or macos-x86_64
mv zush-prompt ~/.local/bin/
chmod +x ~/.local/bin/zush-prompt
```

## Linux

| Architecture | Platform | Binary Format | Target Triple |
|--------------|----------|---------------|---------------|
| x86_64 (AMD64) | Most Linux distributions | `.tar.gz` | `x86_64-unknown-linux-gnu` |
| ARM64 (aarch64) | ARM servers, Raspberry Pi 4+ | `.tar.gz` | `aarch64-unknown-linux-gnu` |

**Requirements:**
- GNU libc (glibc) 2.17 or later
- Most modern Linux distributions (Ubuntu 16.04+, Debian 9+, RHEL 7+, etc.)

**Installation:**
```bash
# Download and extract
tar -xzf zush-prompt-linux-x86_64.tar.gz  # or linux-arm64
sudo mv zush-prompt /usr/local/bin/
chmod +x /usr/local/bin/zush-prompt
```

## Windows

| Architecture | Platform | Binary Format | Target Triple |
|--------------|----------|---------------|---------------|
| x86_64 (AMD64) | Windows 10+ | `.zip` | `x86_64-pc-windows-msvc` |

**Requirements:**
- Windows 10 or later
- Visual C++ Redistributable (usually pre-installed)

**Installation:**
```powershell
# Extract the zip file
Expand-Archive zush-prompt-windows-x86_64.zip

# Move to a directory in your PATH
Move-Item zush-prompt.exe $env:USERPROFILE\.local\bin\

# Or add to PATH in PowerShell profile
```

**Note:** Windows support is experimental. Zush Prompt works best in:
- Windows Terminal
- PowerShell 7+
- Git Bash
- WSL (use Linux binaries instead)

## Verification

All binaries include SHA256 checksums for verification:

### Unix/Linux/macOS:
```bash
shasum -a 256 -c zush-prompt-*.sha256
```

### Windows (PowerShell):
```powershell
$hash = (Get-FileHash zush-prompt-windows-x86_64.zip).Hash
$expected = Get-Content zush-prompt-windows-x86_64.zip.sha256
if ($hash -eq $expected.Split()[0]) { "✓ Checksum verified" }
```

## Platform-Specific Notes

### macOS
- **ARM64 binary** works on Apple Silicon Macs (M1/M2/M3)
- **x86_64 binary** works on Intel Macs and Apple Silicon via Rosetta 2
- Use ARM64 binary on Apple Silicon for best performance

### Linux ARM64
- Tested on:
  - Raspberry Pi 4 (64-bit OS)
  - AWS Graviton instances
  - Oracle ARM servers
  - Other ARM64 Linux systems
- Requires 64-bit ARM Linux (not 32-bit)

### Windows
- **Limited shell support**: Works best in modern terminals
- **Git Bash users**: Can use Windows binary or WSL
- **WSL users**: Use the Linux binary for better compatibility
- **Command Prompt**: Basic support only
- **PowerShell**: Recommended for Windows users

## Unsupported Platforms

The following platforms are **not** currently supported:

- **32-bit systems** (x86, ARM32)
- **BSD variants** (FreeBSD, OpenBSD, NetBSD) - may work from source
- **Windows ARM64** - not yet built
- **musl libc Linux** - use source build or Docker
- **Android/Termux** - may work with Linux ARM64 binary

## Building from Source

If your platform is not supported, you can build from source:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/YOUR_USERNAME/zush-prompt-rust.git
cd zush-prompt-rust
cargo build --release

# Binary will be at target/release/zush-prompt
```

## Cross-Platform Compatibility

### Shell Support

| Shell | macOS | Linux | Windows | Notes |
|-------|-------|-------|---------|-------|
| Zsh | ✅ | ✅ | ⚠️ | Primary target |
| Bash | 🔶 | 🔶 | 🔶 | Limited support |
| Fish | ❌ | ❌ | ❌ | Not supported |
| PowerShell | ❌ | ❌ | 🔶 | Windows only, limited |

**Legend:**
- ✅ Fully supported
- 🔶 Partial support
- ⚠️ Works but not recommended
- ❌ Not supported

### Terminal Emulator Support

| Terminal | Support | Notes |
|----------|---------|-------|
| iTerm2 | ✅ | Recommended for macOS |
| Terminal.app | ✅ | Built-in macOS terminal |
| Alacritty | ✅ | Cross-platform |
| Kitty | ✅ | Modern features |
| Windows Terminal | ✅ | Recommended for Windows |
| GNOME Terminal | ✅ | Linux default |
| Konsole | ✅ | KDE default |
| Git Bash | 🔶 | Limited colors |
| Command Prompt | ⚠️ | Basic support only |

## Release Assets

Each release includes:

1. **Binary archives**: `.tar.gz` (Unix) or `.zip` (Windows)
2. **Checksums**: `.sha256` files for verification
3. **Source code**: Automatic GitHub archives

## Getting Help

- **Platform issues**: [GitHub Issues](https://github.com/YOUR_USERNAME/zush-prompt-rust/issues)
- **Build problems**: See [CONTRIBUTING.md](../CONTRIBUTING.md)
- **Installation**: See [README.md](../README.md)

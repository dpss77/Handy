# Build Instructions

This guide covers how to set up the development environment and build Handy from source across different platforms.

## Prerequisites

### All Platforms
- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) package manager
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/)

### Platform-Specific Requirements

#### macOS
- Xcode Command Line Tools
- Install with: `xcode-select --install`

#### Windows  
- Microsoft C++ Build Tools
- Visual Studio 2019/2022 with C++ development tools
- Or Visual Studio Build Tools 2019/2022

#### Linux
- Build essentials
- ALSA development libraries
- Install with:
  ```bash
  # Ubuntu/Debian
  sudo apt update
  sudo apt install build-essential libasound2-dev pkg-config libssl-dev libvulkan-dev vulkan-tools glslc libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf

  # Fedora/RHEL
  sudo dnf groupinstall "Development Tools"
  sudo dnf install alsa-lib-devel pkgconf openssl-devel vulkan-devel \
    gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel

  # Arch Linux
  sudo pacman -S base-devel alsa-lib pkgconf openssl vulkan-devel \
    gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg
  ```

## Setup Instructions

### 1. Clone the Repository
```bash
git clone git@github.com:cjpais/Handy.git
cd Handy
```

### 2. Install Dependencies
```bash
bun install
```

### 3. Download Required Models
Handy requires a VAD (Voice Activity Detection) model to function:

```bash
# Create models directory
mkdir -p src-tauri/resources/models

# Download Silero VAD model
curl -o src-tauri/resources/models/silero_vad_v4.onnx \
  https://blob.handy.computer/silero_vad_v4.onnx
```

**Verify download**:
```bash
ls -lh src-tauri/resources/models/
# Should show: silero_vad_v4.onnx (approximately 2-3MB)
```

### 4. Build the Application

#### Development Build (with hot-reload)

```bash
# macOS (if you encounter cmake errors)
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev

# All platforms
bun run tauri dev
```

This starts:
- Frontend dev server (Vite) on port 1420
- Tauri app with hot-reload enabled
- Console output for debugging

#### Production Build

```bash
# macOS (if you encounter cmake errors)
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri build

# All platforms
bun run tauri build
```

**Build times**:
- First build: 10-20 minutes (compiles all Rust dependencies)
- Subsequent builds: 2-5 minutes

**Build artifacts location**:

**macOS**:
```
src-tauri/target/release/bundle/macos/Handy.app
```

**Linux**:
```
src-tauri/target/release/bundle/appimage/handy_*.AppImage
src-tauri/target/release/bundle/deb/handy_*.deb
```

**Windows**:
```
src-tauri\target\release\bundle\msi\Handy_*_x64_en-US.msi
```

## Platform-Specific Notes

### macOS

**Metal GPU Acceleration**:
- Automatically enabled for Apple Silicon (M1/M2/M3) and Intel Macs with Metal support
- Provides 2-5x faster Whisper inference

**Code Signing**:
```bash
# For development, you may need to allow the app to run
xattr -cr /Applications/Handy.app
```

**Accessibility Permissions**:
Required for pasting transcriptions. Grant in:
System Settings → Privacy & Security → Accessibility

### Linux

**Vulkan Acceleration**:
- Requires Vulkan drivers and libraries
- Verify with: `vulkaninfo` or `vkcube`

**Audio Issues**:
If you encounter audio device errors:
```bash
# Check PulseAudio
pactl list sources short

# Check ALSA
arecord -l
```

**AppImage Permissions**:
```bash
chmod +x handy_*.AppImage
```

### Windows

**GPU Acceleration**:
- Vulkan acceleration for NVIDIA/AMD GPUs
- Ensure latest graphics drivers installed

**Visual Studio Build Tools**:
Must include:
- C++ Build Tools
- Windows 10/11 SDK

**Firewall**:
- May need to allow Handy through Windows Firewall
- Required for model downloads

## Testing

### Run Rust Tests

```bash
cd src-tauri

# Run all tests
cargo test

# Run specific test module
cargo test --lib audio
cargo test --lib transcription

# Run with output
cargo test -- --nocapture
```

### Run Benchmarks

```bash
cd src-tauri
cargo bench
```

### Frontend Tests

```bash
# Type checking
bun run type-check

# Linting
bun run lint
```

## Troubleshooting

### Common Build Errors

**"cmake not found"** (macOS):
```bash
brew install cmake
# or use the CMAKE_POLICY flag
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri build
```

**"could not find native TLS library"** (Linux):
```bash
sudo apt install libssl-dev pkg-config
```

**"failed to run custom build command for vad-rs"**:
```bash
# Make sure VAD model exists
ls src-tauri/resources/models/silero_vad_v4.onnx

# If missing, download it
curl -o src-tauri/resources/models/silero_vad_v4.onnx \
  https://blob.handy.computer/silero_vad_v4.onnx
```

**"Bun command not found"**:
```bash
# Restart terminal after installing Bun
source ~/.bashrc  # or ~/.zshrc on macOS
```

**Tauri build fails with "WebView2 not found"** (Windows):
- Download and install WebView2 Runtime from Microsoft
- Should be automatic on Windows 11

### Performance Issues

**Slow compilation**:
```bash
# Enable parallel compilation
export CARGO_BUILD_JOBS=8  # Adjust based on CPU cores

# Use faster linker (Linux/macOS)
cargo install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

**Large binary size**:
The release build is optimized with LTO and stripping (see `Cargo.toml`):
```toml
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization
strip = true            # Remove debug symbols
panic = "abort"         # Smaller panic handler
```

## Development Workflow

### Recommended Setup

1. **Use development mode** for rapid iteration:
   ```bash
   bun run tauri dev
   ```

2. **Frontend changes**: Hot-reload automatically

3. **Backend changes**: Requires rebuild (automatic in dev mode)

4. **Test before committing**:
   ```bash
   cd src-tauri
   cargo test
   cargo clippy
   cargo fmt
   ```

### Code Style

**Rust**:
```bash
# Format code
cargo fmt

# Run linter
cargo clippy
```

**TypeScript**:
```bash
# Format and lint
bun run lint
```

## Advanced Build Options

### Cross-Platform Builds

**macOS Universal Binary** (Intel + Apple Silicon):
```bash
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
bun run tauri build -- --target universal-apple-darwin
```

**Windows ARM**:
```bash
rustup target add aarch64-pc-windows-msvc
bun run tauri build -- --target aarch64-pc-windows-msvc
```

### Debug Builds

```bash
# Build without optimizations (faster compilation, larger binary)
cargo build
```

### Release with Debug Symbols

```bash
# Add to Cargo.toml temporarily:
[profile.release]
debug = true

cargo build --release
```

## Development Branch

For the **latest features** (Phases 1-3 implementation), use the development branch:

```bash
git checkout claude/codebase-review-011CUpFFLwx7sb5q5LfW8MfZ
```

See `DEVELOPMENT_INSTALL.md` for complete setup and testing instructions.

**New features in development branch**:
- Real-time streaming transcription UI
- Ollama LLM integration
- Hot-plug device detection
- Webhook system
- 5 export formats (txt, md, json, srt, vtt)
- 320+ tests

## Additional Resources

- **Development Install Guide**: `DEVELOPMENT_INSTALL.md`
- **Architecture Overview**: `CLAUDE.md`
- **Contributing Guide**: `CONTRIBUTING.md`
- **Implementation Details**: `COMPLETE_IMPLEMENTATION_SUMMARY.md`
- **Tauri Documentation**: [tauri.app](https://tauri.app)
- **Rust Documentation**: [rust-lang.org](https://www.rust-lang.org/learn)

## Getting Help

- **GitHub Issues**: [github.com/cjpais/Handy/issues](https://github.com/cjpais/Handy/issues)
- **Discord Community**: [discord.com/invite/WVBeWsNXK4](https://discord.com/invite/WVBeWsNXK4)
- **Email**: contact@handy.computer

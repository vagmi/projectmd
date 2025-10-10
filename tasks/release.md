---
issue_id: 1
type: task
tags:
- release
- distribution
- infra
created_at: 2025-10-10T16:50:40.103749380+00:00
updated_at: 2025-10-10T16:50:40.103749380+00:00
---


# Setup cargo-dist for automated releases

Setup cargo-dist to automate the build and release process for projectmd binaries across multiple platforms.

## What is cargo-dist?

[cargo-dist](https://github.com/axodotdev/cargo-dist) is a tool that automates building and distributing Rust binaries. It generates GitHub Actions workflows that:
- Build binaries for multiple platforms (Linux, macOS, Windows)
- Create GitHub releases with pre-built binaries
- Generate installers (shell scripts, PowerShell, etc.)
- Publish to package managers (Homebrew, npm, etc.)

## Why this is important

Currently, users must build projectmd from source using `cargo build --release`. This is a barrier to adoption because:
1. Not all users have Rust toolchain installed
2. Compilation takes time and resources
3. Cross-platform builds are difficult

With cargo-dist, users can:
- Download pre-built binaries for their platform
- Use one-line install scripts
- Get automatic updates through package managers

## Implementation steps

### 1. Install cargo-dist

```bash
cargo install cargo-dist
```

### 2. Initialize cargo-dist

```bash
cargo dist init
```

This will:
- Add `[workspace.metadata.dist]` section to Cargo.toml
- Generate `.github/workflows/release.yml`
- Configure supported platforms and installers

### 3. Configure distribution settings

Update Cargo.toml with cargo-dist metadata:
- Choose target platforms (x86_64-linux, aarch64-macos, etc.)
- Select installer types (shell, powershell, homebrew, etc.)
- Configure CI/CD settings

### 4. Test the release workflow locally

```bash
# Build for all platforms
cargo dist build

# Generate a full release
cargo dist plan
```

### 5. Create a test release

- Tag a version: `git tag v0.1.0`
- Push the tag: `git push origin v0.1.0`
- Verify GitHub Actions builds and publishes artifacts

### 6. Update README with installation instructions

Add installation methods to README:
- Shell installer for Linux/macOS
- PowerShell installer for Windows
- Direct binary downloads
- Cargo install (existing method)

## Expected outcomes

After setup:
- GitHub Actions workflow automatically triggers on version tags
- Binaries built for: Linux (x86_64), macOS (x86_64, ARM64), Windows (x86_64)
- GitHub release created with downloadable artifacts
- Installation scripts generated and tested
- README updated with new installation options

## Technical considerations

### Platform support
- Start with tier 1 platforms: x86_64-unknown-linux-gnu, x86_64-apple-darwin, x86_64-pc-windows-msvc
- Consider adding: aarch64-apple-darwin (M1/M2 Macs), aarch64-unknown-linux-gnu (ARM Linux)

### Dependencies
- Ensure all dependencies have compatible licenses
- Check for platform-specific dependencies or issues
- Verify builds work in clean CI environment

### Versioning
- Establish versioning strategy (semantic versioning)
- Decide on release cadence
- Document release process in CONTRIBUTING.md

### Security
- Sign binaries (code signing for macOS/Windows)
- Generate and publish checksums
- Consider reproducible builds

## Acceptance criteria

- [ ] cargo-dist installed and initialized
- [ ] Cargo.toml contains dist configuration
- [ ] GitHub Actions workflow file created (`.github/workflows/release.yml`)
- [ ] Test release successfully creates binaries for all target platforms
- [ ] Binaries are executable and functional on each platform
- [ ] Installation scripts work correctly
- [ ] README updated with installation instructions
- [ ] At least one successful release published to GitHub with artifacts
- [ ] Documentation added about the release process

## Resources

- [cargo-dist documentation](https://opensource.axo.dev/cargo-dist/)
- [cargo-dist GitHub repository](https://github.com/axodotdev/cargo-dist)
- [Example projects using cargo-dist](https://github.com/axodotdev/cargo-dist#who-is-using-cargo-dist)
- [GitHub Actions: Creating releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)

## Notes

- Start with basic setup, can expand installer options later
- Test thoroughly in CI before announcing pre-built binaries
- Consider adding auto-update functionality in future iterations
- May want to set up Homebrew tap for easier macOS installation

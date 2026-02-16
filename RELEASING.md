# Release Process

This document describes the release process for the LC-3 Assembler.

## Prerequisites

Before creating a release, ensure:

1. ✅ All tests pass (`cargo test`)
2. ✅ Code is formatted (`cargo fmt --check`)
3. ✅ No Clippy warnings (`cargo clippy -- -D warnings`)
4. ✅ Documentation builds (`cargo doc --no-deps`)
5. ✅ CHANGES.md is updated
6. ✅ Version bumped in Cargo.toml
7. ✅ README.md reflects new features

## Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version: Incompatible API changes
- **MINOR** version: New functionality (backward-compatible)
- **PATCH** version: Bug fixes (backward-compatible)

### Version Examples

- `1.0.0` - Initial stable release
- `1.1.0` - Added macro system feature
- `1.1.1` - Fixed parser bug
- `2.0.0` - Changed CLI interface (breaking change)

## Release Checklist

### 1. Prepare the Release

```bash
# Ensure clean working directory
git status

# Update version in Cargo.toml
# Example: version = "1.1.0"

# Update CHANGES.md with release notes
# Add release date and version

# Run full test suite
cargo test --all-features

# Build in release mode
cargo build --release

# Test the binary
./target/release/lc3-assembler --version
./target/release/lc3-assembler tests/test_programs/subroutine.asm

# Update documentation
cargo doc --no-deps --all-features
```

### 2. Commit Changes

```bash
# Commit version bump and changelog
git add Cargo.toml Cargo.lock CHANGES.md
git commit -m "chore: bump version to 1.1.0"

# Push to main
git push origin main
```

### 3. Create Git Tag

```bash
# Create annotated tag
git tag -a v1.1.0 -m "Release version 1.1.0

## New Features
- Added macro system (.MACRO/.ENDM)
- Improved error messages with suggestions

## Bug Fixes
- Fixed PC offset calculation for edge cases

## Documentation
- Updated README with macro examples
"

# Push tag to trigger release workflow
git push origin v1.1.0
```

### 4. Monitor Release Build

1. Go to GitHub Actions: `https://github.com/your-repo/lc3-assembler/actions`
2. Watch the "Release" workflow
3. Verify all platform builds succeed:
   - ✅ Linux x86_64
   - ✅ Linux x86_64 (musl)
   - ✅ macOS x64 (Intel)
   - ✅ macOS ARM64 (Apple Silicon)
   - ✅ Windows x64

### 5. Verify Release Assets

Check that the following files are attached to the release:

- `lc3-assembler-linux-x64`
- `lc3-assembler-linux-x64.sha256`
- `lc3-assembler-linux-x64-musl`
- `lc3-assembler-linux-x64-musl.sha256`
- `lc3-assembler-macos-x64`
- `lc3-assembler-macos-x64.sha256`
- `lc3-assembler-macos-arm64`
- `lc3-assembler-macos-arm64.sha256`
- `lc3-assembler-windows-x64.exe`
- `lc3-assembler-windows-x64.exe.sha256`

### 6. Test Release Binaries

Download and test each platform binary:

```bash
# Linux / macOS
curl -LO https://github.com/your-repo/lc3-assembler/releases/download/v1.1.0/lc3-assembler-linux-x64
chmod +x lc3-assembler-linux-x64
./lc3-assembler-linux-x64 --version
./lc3-assembler-linux-x64 tests/test_programs/subroutine.asm

# Verify checksum
curl -LO https://github.com/your-repo/lc3-assembler/releases/download/v1.1.0/lc3-assembler-linux-x64.sha256
shasum -c lc3-assembler-linux-x64.sha256
```

### 7. Publish to crates.io (Optional)

The GitHub Actions workflow attempts to publish automatically, but you can do it manually:

```bash
# Login to crates.io
cargo login <your-token>

# Dry run
cargo publish --dry-run

# Publish
cargo publish
```

### 8. Announce the Release

1. **GitHub Release Notes**: Edit the auto-generated release notes
2. **Reddit**: Post to r/rust, r/ComputerScience
3. **Twitter/X**: Announce with #rustlang #LC3
4. **Discord**: Post in Rust community servers

## Hotfix Process

For urgent bug fixes:

```bash
# Create hotfix branch from tag
git checkout -b hotfix/1.1.1 v1.1.0

# Make fix
vim src/encoder/mod.rs

# Test
cargo test

# Commit
git commit -am "fix: critical PC offset bug"

# Bump version
# Edit Cargo.toml: version = "1.1.1"

# Update CHANGES.md

# Commit version bump
git commit -am "chore: bump version to 1.1.1"

# Merge to main
git checkout main
git merge hotfix/1.1.1

# Tag and push
git tag -a v1.1.1 -m "Hotfix release 1.1.1"
git push origin main v1.1.1

# Delete hotfix branch
git branch -d hotfix/1.1.1
```

## CI/CD Configuration

### GitHub Secrets Required

For full automation, configure these secrets in GitHub repository settings:

1. **CARGO_TOKEN**: Token from crates.io for publishing
   - Get from: https://crates.io/settings/tokens
   - Scopes: `publish-new`, `publish-update`

2. **DOCKER_USERNAME**: Docker Hub username (optional)
3. **DOCKER_PASSWORD**: Docker Hub token (optional)
   - Get from: https://hub.docker.com/settings/security

### Setting Secrets

```bash
# Via GitHub UI
Settings → Secrets and variables → Actions → New repository secret

# Or via GitHub CLI
gh secret set CARGO_TOKEN < token.txt
gh secret set DOCKER_USERNAME -b "your-username"
gh secret set DOCKER_PASSWORD -b "your-token"
```

## Rollback Process

If a release has critical issues:

### 1. Delete the Release

```bash
# Delete tag locally
git tag -d v1.1.0

# Delete tag remotely
git push origin :refs/tags/v1.1.0

# Delete GitHub release (via UI or CLI)
gh release delete v1.1.0 --yes
```

### 2. Yank from crates.io

```bash
# Yank the version (keeps it in registry but marks as broken)
cargo yank --vers 1.1.0
```

### 3. Revert Changes

```bash
# Revert the version bump commit
git revert <commit-sha>

# Push
git push origin main
```

### 4. Create Fixed Release

Follow the normal release process with a patch version (e.g., 1.1.1).

## Release Frequency

- **Patch releases**: As needed for bug fixes (weekly if necessary)
- **Minor releases**: Monthly for new features
- **Major releases**: Yearly or when breaking changes are necessary

## Pre-Release Testing

For major releases, create a release candidate:

```bash
# Tag as release candidate
git tag -a v2.0.0-rc.1 -m "Release candidate 1 for version 2.0.0"
git push origin v2.0.0-rc.1

# Test for 1-2 weeks
# Gather feedback
# Fix issues

# Create final release
git tag -a v2.0.0 -m "Release version 2.0.0"
git push origin v2.0.0
```

## Automation Details

### CI Workflow (.github/workflows/ci.yml)

Runs on every push and PR:
- Code formatting check
- Clippy lints
- Tests (Linux, macOS, Windows)
- Code coverage
- Security audit
- Documentation build
- MSRV (Minimum Supported Rust Version) check

### Release Workflow (.github/workflows/release.yml)

Triggered by version tags:
1. Creates GitHub release
2. Builds binaries for all platforms
3. Generates checksums
4. Uploads assets to release
5. Publishes to crates.io (optional)
6. Builds and pushes Docker image (optional)

## Troubleshooting

### Build Fails on Specific Platform

1. Check GitHub Actions logs
2. Reproduce locally with cross-compilation:
   ```bash
   # Install target
   rustup target add x86_64-pc-windows-msvc

   # Build
   cargo build --release --target x86_64-pc-windows-msvc
   ```

### crates.io Publish Fails

- Ensure version not already published
- Check Cargo.toml metadata is complete
- Verify CARGO_TOKEN is valid
- Check dependency versions are on crates.io

### Docker Build Fails

- Test locally first: `docker build -t lc3-assembler .`
- Check Dockerfile syntax
- Verify .dockerignore excludes unnecessary files

## Contact

For release questions:
- Open an issue: https://github.com/your-repo/lc3-assembler/issues
- Email: maintainer@example.com

---

**Last Updated:** February 15, 2026
**Document Version:** 1.0

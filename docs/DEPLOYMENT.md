# Deployment Guide

This document provides comprehensive deployment instructions for the rust-template project.

## Overview

The project includes automated deployment workflows for:

- **GitHub Releases** - Multi-platform binaries
- **Docker** - Container images on GitHub Container Registry
- **crates.io** - Rust package registry

## Prerequisites

### Required Secrets and Setup

1. **crates.io Trusted Publishing** - publishing uses OIDC, not a token, so no `CARGO_REGISTRY_TOKEN` secret exists
   - One-time setup on crates.io: crate Settings → Trusted Publishing → add this GitHub repo with workflow `publish.yml` and environment `copilot`

2. **HOMEBREW_TAP_TOKEN** (optional secret) - For Homebrew formula updates (`package-homebrew.yml`)
   - Fine-grained PAT with write access to your `homebrew-tap` repository
   - Override the tap repo name with the `HOMEBREW_TAP_REPO` repository variable (default: `homebrew-tap`)

3. **GITHUB_TOKEN** - Automatically provided by GitHub Actions (no setup needed)

### GitHub Packages

Enable GitHub Packages for Docker image publishing:
- Settings → Actions → General → Workflow permissions → "Read and write permissions"

## Creating a Release

### 1. Prepare Release

Update version in `Cargo.toml`:

```toml
[package]
version = "0.2.0"  # Update this
```

Run checks locally:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo deny check
```

### 2. Create and Push Tag

```bash
# Commit version bump
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
git push

# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

### 3. Automated Workflows

Pushing the tag automatically triggers:

1. **Release Workflow** (`release.yml`)
   - Resolves the binary name and version from `cargo metadata`
   - Builds binaries for all 5 platforms, named `rust_template-<version>-<platform>`
   - Attaches SLSA build provenance and a CycloneDX SBOM attestation to every binary
   - Verifies every attestation fail-closed, then creates the GitHub release with auto-generated notes and a checksums file

2. **Publish Workflow** (`publish.yml`)
   - Runs all pre-publish checks
   - Publishes to crates.io via Trusted Publishing (OIDC)
   - Downloads the registry-served `.crate`, byte-compares it to the local package, and attests it

3. **Pipeline Workflow** (`pipeline.yml`, container chain)
   - Builds multi-platform images and pushes to ghcr.io with version tag and 'latest'
   - Images are signed and attested by the centralized signer workflow, then verified fail-closed

4. **Homebrew Workflow** (`package-homebrew.yml`)
   - Runs after the Release workflow completes
   - Regenerates the source formula in `{owner}/homebrew-tap`

## Deployment Targets

### GitHub Releases

**Access:** https://github.com/zircote/rust-template/releases

**Artifacts** (version embedded in every name):
- `rust_template-<version>-linux-amd64` - Linux x86_64
- `rust_template-<version>-linux-arm64` - Linux ARM64
- `rust_template-<version>-macos-amd64` - macOS x86_64
- `rust_template-<version>-macos-arm64` - macOS ARM64 (Apple Silicon)
- `rust_template-<version>-windows-amd64.exe` - Windows x86_64
- `rust_template-<version>-sbom.cdx.json` - CycloneDX SBOM
- `rust_template-<version>-checksums.txt` - SHA-256 checksums

**Download and Verify Example:**

```bash
# Linux
wget https://github.com/zircote/rust-template/releases/download/v0.1.0/rust_template-0.1.0-linux-amd64
gh attestation verify rust_template-0.1.0-linux-amd64 --repo zircote/rust-template
chmod +x rust_template-0.1.0-linux-amd64
./rust_template-0.1.0-linux-amd64 --version
```

Full verification commands (provenance, SBOM, checksums, container images, crate) are in [SECURITY.md](../SECURITY.md#verifying-release-artifacts).

### Docker (GitHub Container Registry)

**Registry:** ghcr.io/zircote/rust-template

**Supported Platforms:**
- linux/amd64
- linux/arm64

**Pull and Run:**

```bash
# Latest version
docker pull ghcr.io/zircote/rust-template:latest
docker run --rm ghcr.io/zircote/rust-template:latest --version

# Specific version
docker pull ghcr.io/zircote/rust-template:v0.1.0
docker run --rm ghcr.io/zircote/rust-template:v0.1.0 --version

# With volumes
docker run --rm -v $(pwd):/data ghcr.io/zircote/rust-template:latest
```

**Image Details:**
- Base: distroless/cc-debian12 (minimal attack surface)
- User: nonroot:nonroot (unprivileged)
- Healthcheck: Built-in with `--version` command
- Size: ~10-15 MB (optimized multi-stage build)

### crates.io

**Package:** https://crates.io/crates/rust_template

**Install:**

```bash
# Latest version
cargo install rust_template

# Specific version
cargo install rust_template@0.1.0

# From source
cargo install --git https://github.com/zircote/rust-template
```

**Use in Project:**

```toml
[dependencies]
rust_template = "0.1"
```

## Versioning

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0) - Incompatible API changes
- **MINOR** (0.1.0) - Backwards-compatible functionality
- **PATCH** (0.0.1) - Backwards-compatible bug fixes

## Changelog

Changelogs are automatically generated from conventional commits:

- `feat:` → Added section
- `fix:` → Fixed section
- `docs:` → Documentation section
- `perf:` → Performance section
- `refactor:` → Refactored section
- `test:` → Testing section
- `chore:` → Miscellaneous section

**Example Commit:**

```bash
git commit -m "feat(auth): add JWT token validation"
```

## Rollback

### GitHub Release

Delete the release and tag:

```bash
# Delete remote tag
git push --delete origin v0.2.0

# Delete local tag
git tag -d v0.2.0

# Delete release via GitHub UI or gh CLI
gh release delete v0.2.0
```

### Docker

Images are immutable; use previous version tags:

```bash
docker pull ghcr.io/zircote/rust-template:v0.1.0
```

### crates.io

**Cannot unpublish** - crates.io doesn't allow unpublishing. Options:

1. Yank the version (prevents new projects from using it):
   ```bash
   cargo yank --vers 0.2.0
   ```

2. Publish a patch version with fixes:
   ```bash
   # Update to 0.2.1
   git tag -a v0.2.1 -m "Release v0.2.1 (fixes v0.2.0)"
   git push origin v0.2.1
   ```

## Monitoring

### GitHub Actions

Monitor workflow runs:
- Actions tab: https://github.com/zircote/rust-template/actions

### Security Audits

Daily automated security scans run at 00:00 UTC:
- Workflow: `.github/workflows/security-audit.yml`
- Uses: cargo-audit
- Notifications: GitHub Actions UI

### Dependencies

Dependabot automatically opens PRs for:
- Cargo dependencies
- GitHub Actions versions

## Troubleshooting

### Release Workflow Fails

**Build Error:**
- Check Cargo.toml version matches tag
- Verify MSRV compatibility (1.92+)
- Test locally: `cargo build --release`

**Cross-compilation Error:**
- Linux ARM64 requires `gcc-aarch64-linux-gnu`
- macOS ARM64 requires macOS 11+ runner

### Docker Build Fails

**Context Issue:**
- Verify .dockerignore excludes target/
- Check Dockerfile paths match `crates/` structure

**Push Permission:**
- Verify GitHub Actions workflow permissions
- Check ghcr.io login succeeds

### Publish to crates.io Fails

**Trusted Publishing Issue:**
- "No Trusted Publishing config found": complete the one-time setup on crates.io (crate Settings → Trusted Publishing → workflow `publish.yml`, environment `copilot`)
- No registry token is used; do not set `CARGO_REGISTRY_TOKEN`

**Pre-publish Checks:**
- All tests must pass
- No clippy warnings
- cargo-deny checks must pass

## Best Practices

1. **Test Before Tagging**
   ```bash
   cargo build --release
   cargo test --all-features
   cargo clippy --all-targets --all-features -- -D warnings
   ```

2. **Use Conventional Commits**
   - Enables automatic changelog generation
   - Clearly communicates changes

3. **Version Bump in Separate Commit**
   ```bash
   git commit -m "chore: bump version to 0.2.0"
   git tag -a v0.2.0 -m "Release v0.2.0"
   ```

4. **Monitor Release Progress**
   - Watch GitHub Actions for workflow completion
   - Verify artifacts are uploaded
   - Test Docker image immediately after push

5. **Document Breaking Changes**
   - Use `BREAKING CHANGE:` in commit body
   - Update migration guide in CHANGELOG

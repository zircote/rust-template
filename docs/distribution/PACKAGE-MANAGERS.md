---
diataxis_type: how-to
---
# Package Manager Distribution

## Overview

Automated distribution for the channels this template ships with:

**Workflows:**
- `.github/workflows/package-homebrew.yml` - Homebrew tap formula (macOS/Linux)
- `.github/workflows/publish.yml` - crates.io via Trusted Publishing
- `.github/workflows/release.yml` - attested multi-platform binaries on GitHub Releases

Both workflows resolve project specificity (crate name, binary name, description, license) from `cargo metadata` at runtime — instantiating the template requires no edits to the workflow files.

> **Removed from the template:** Debian (`.deb`), RPM, Snap, and Windows MSI packaging workflows are no longer included. See [Re-adding Removed Channels](#re-adding-removed-channels) if your project needs them.

## Installation Methods

### Homebrew (macOS/Linux)

```bash
# Add tap
brew tap zircote/tap

# Install
brew install rust-template

# Update
brew upgrade rust-template
```

**Setup Requirements:**
1. Create a `homebrew-tap` repository: `https://github.com/USER/homebrew-tap` (override the repo name with the `HOMEBREW_TAP_REPO` repository variable)
2. Add secret `HOMEBREW_TAP_TOKEN` with write access to the tap repo
3. Formula auto-updates after each release

**How the formula is generated:**

`package-homebrew.yml` triggers via `workflow_run` when the Release workflow completes (bot-authored release events do not trigger workflows, so `workflow_run` is the reliable path). It checks out the project at the released tag, resolves the binary name, description, and license from `cargo metadata`, and writes a **source formula** (builds with `cargo install` from the tag tarball) to `Formula/<name>.rb` in the tap.

### crates.io (Trusted Publishing)

```bash
# Install the binary
cargo install rust_template

# Or use as a dependency
cargo add rust_template
```

Publishing runs in `publish.yml` on every `v*.*.*` tag using crates.io **Trusted Publishing** (OIDC). There is no `CARGO_REGISTRY_TOKEN` secret.

**One-time setup:**
1. On crates.io, open the crate's **Settings > Trusted Publishing**
2. Add this GitHub repository with workflow `publish.yml` and environment `copilot`

After publishing, the workflow downloads the `.crate` the registry serves, byte-compares it to the local package, and attests it:

```bash
curl -fsSL -A 'release-check' \
  -O https://static.crates.io/crates/rust_template/rust_template-0.1.0.crate
gh attestation verify rust_template-0.1.0.crate --repo USER/REPO
```

### GitHub Releases (prebuilt binaries)

Every release attaches attested binaries named `{bin}-{version}-{platform}`:

- `rust_template-0.1.0-linux-amd64`
- `rust_template-0.1.0-linux-arm64`
- `rust_template-0.1.0-macos-amd64`
- `rust_template-0.1.0-macos-arm64`
- `rust_template-0.1.0-windows-amd64.exe`

Plus a CycloneDX SBOM and a `{bin}-{version}-checksums.txt` file. Verify before use:

```bash
gh release download v0.1.0 --repo USER/REPO
gh attestation verify rust_template-0.1.0-linux-amd64 --repo USER/REPO
shasum -a 256 -c rust_template-0.1.0-checksums.txt
```

See [SECURITY.md](../../SECURITY.md#verifying-release-artifacts) for the full verification reference.

## CI/CD Integration

### On Release

1. Tag release: `git tag v0.1.0 && git push origin v0.1.0`
2. `release.yml` builds, attests, verifies, and publishes the GitHub Release
3. `publish.yml` publishes to crates.io and attests the served `.crate`
4. `package-homebrew.yml` fires on Release completion and updates the tap formula

### Manual Trigger

```bash
# Preview the Homebrew formula without pushing
gh workflow run package-homebrew.yml -f version=0.1.0 -f dry_run=true

# Regenerate and push the formula for an existing release
gh workflow run package-homebrew.yml -f version=0.1.0 -f dry_run=false

# Dry-run the publish chain from a branch (tag-gated steps are skipped)
gh workflow run publish.yml
```

## Troubleshooting

### Homebrew Formula Push Fails

- `HOMEBREW_TAP_TOKEN` missing, expired, or lacking write access to the tap repo
- Tap repository does not exist (create `USER/homebrew-tap` or set `HOMEBREW_TAP_REPO`)
- In dry-run mode the formula is printed to the job log and nothing is pushed

### Homebrew Workflow Did Not Run

The `workflow_run` trigger only proceeds for **successful, tag-triggered** Release runs. Check the Release workflow conclusion, then fall back to manual dispatch:

```bash
gh workflow run package-homebrew.yml -f version=X.Y.Z -f dry_run=false
```

### crates.io Publish Fails

- `No Trusted Publishing config found`: complete the one-time setup (workflow `publish.yml`, environment `copilot`)
- `crate ... already exists`: versions are immutable; a duplicate attempt after a successful publish is benign
- Crate download/byte-compare step fails after retries: CDN propagation delay — re-run the failed job; the publish itself succeeded

## Re-adding Removed Channels

If your project needs Debian, RPM, Snap, or MSI packages, these tools generate them from a Rust project:

- [cargo-deb](https://github.com/kornelski/cargo-deb) - Debian packages
- [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm) - RPM packages
- [cargo-wix](https://github.com/volks73/cargo-wix) - Windows MSI installers
- [Snapcraft](https://snapcraft.io/docs) - Snap packages

They were removed from the template in favor of attested GitHub Release binaries, Homebrew, and crates.io. If you add one back, route it through the attestation flow (attest the package, verify fail-closed) to keep the "nothing publishes unattested" guarantee.

## Publishing to Stores

### Homebrew Core (Official)

For official Homebrew inclusion:

1. Formula must be popular and stable
2. Create PR to [homebrew-core](https://github.com/Homebrew/homebrew-core)
3. Follow [Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)

### Windows Package Manager (winget)

winget can install the release binary directly as a portable package. Create a manifest in [winget-pkgs](https://github.com/microsoft/winget-pkgs):

```yaml
# manifests/r/rust-template/rust-template/0.1.0/rust-template.rust-template.yaml
PackageIdentifier: rust-template.rust-template
PackageVersion: 0.1.0
PackageLocale: en-US
Publisher: Your Name
PackageName: rust-template
License: MIT
ShortDescription: Modern Rust template
Installers:
  - Architecture: x64
    InstallerType: portable
    InstallerUrl: https://github.com/USER/REPO/releases/download/v0.1.0/rust_template-0.1.0-windows-amd64.exe
    InstallerSha256: HASH  # from rust_template-0.1.0-checksums.txt
ManifestType: singleton
ManifestVersion: 1.0.0
```

## Verification

### Test Installations

```bash
# Homebrew
brew install USER/tap/rust-template && rust-template --version

# crates.io
cargo install rust_template && rust_template --version

# GitHub Release binary (Linux)
gh attestation verify rust_template-0.1.0-linux-amd64 --repo USER/REPO && \
  chmod +x rust_template-0.1.0-linux-amd64 && \
  ./rust_template-0.1.0-linux-amd64 --version
```

## Links

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing)
- [GitHub Artifact Attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations)

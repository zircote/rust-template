# Releasing

End-to-end runbook for creating, monitoring, and rolling back releases of rust-template.

> **Prefer the `/release` skill.** Releases are orchestrated end-to-end by the `/release` skill (`.claude/skills/release/SKILL.md`): release-prep PR, tag, monitoring of every workflow chain, and independent workstation verification. The manual procedure below is the same process the skill drives.

## Version Numbering (SemVer)

This project follows [Semantic Versioning 2.0.0](https://semver.org/):

| Change type | Version bump | Example | When to use |
|---|---|---|---|
| Breaking API change | **MAJOR** | `1.0.0` -> `2.0.0` | Removed public types, changed function signatures |
| New feature (backward-compatible) | **MINOR** | `0.1.0` -> `0.2.0` | New public functions, new optional fields |
| Bug fix (backward-compatible) | **PATCH** | `0.1.0` -> `0.1.1` | Fix incorrect behavior, performance improvement |

**Pre-1.0 policy:** While on `0.x.y`, MINOR bumps may include breaking changes. Document these clearly in commit messages with `BREAKING CHANGE:` in the body.

---

## Prerequisites

### Required Secrets and Setup

Configure in GitHub repository settings (**Settings > Secrets and variables > Actions**) and on crates.io:

| Item | Purpose | How to set up |
|---|---|---|
| crates.io Trusted Publishing | Publish to crates.io via OIDC — no token secret exists | One-time, on crates.io: crate **Settings > Trusted Publishing** > add this repo, workflow `publish.yml`, environment `copilot` |
| `HOMEBREW_TAP_TOKEN` (secret, optional) | Push formula updates to your Homebrew tap | Fine-grained PAT with write access to `{owner}/homebrew-tap` |
| `HOMEBREW_TAP_REPO` (variable, optional) | Override the tap repo name (default `homebrew-tap`) | **Settings > Secrets and variables > Actions > Variables** |
| `GITHUB_TOKEN` | Provided automatically | No setup needed |

### Permissions

- **GitHub Packages (Docker):** Settings > Actions > General > Workflow permissions > "Read and write permissions"

---

## Pre-Release Checklist

Run through this checklist before every release.

- [ ] All CI checks pass on `main` (check [Actions](https://github.com/zircote/rust-template/actions/workflows/ci.yml))
- [ ] Update version in `Cargo.toml`:
  ```toml
  [package]
  version = "X.Y.Z"  # New version
  ```
- [ ] Run the full local check suite:
  ```bash
  cargo fmt -- --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-features
  cargo deny check
  cargo doc --no-deps --all-features
  ```
- [ ] Build a release binary locally to verify:
  ```bash
  cargo build --release
  ```
- [ ] Review `CHANGELOG.md` and recent commits since last tag:
  ```bash
  git log $(git describe --tags --abbrev=0)..HEAD --oneline
  ```
- [ ] Verify conventional commit messages are correct (they drive changelog generation)
- [ ] If breaking changes exist, confirm MAJOR version bump and `BREAKING CHANGE:` in commit bodies
- [ ] Commit the version bump separately:
  ```bash
  git add Cargo.toml Cargo.lock
  git commit -m "chore: bump version to X.Y.Z"
  git push
  ```

---

## Step-by-Step: Create and Push a Release Tag

### 1. Create an Annotated Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

### 2. Push the Tag

```bash
git push origin vX.Y.Z
```

This single push triggers all release automation.

### 3. Triggered Workflows

Pushing a `v*.*.*` tag triggers these workflows in parallel:

| Workflow | File | What it does |
|---|---|---|
| **Release** | `release.yml` | Resolves bin/version from `cargo metadata`, builds 5 platform binaries (`{bin}-{version}-{platform}`) with SLSA build provenance, generates + attests a CycloneDX SBOM, verifies every attestation **fail-closed**, then creates the GitHub Release with auto-generated notes and a `{bin}-{version}-checksums.txt` file |
| **Publish** | `publish.yml` | Runs pre-publish checks, publishes to crates.io via Trusted Publishing (OIDC), then downloads the registry-served `.crate`, byte-compares it, and attests it |
| **Pipeline (container)** | `pipeline.yml` | Builds multi-platform images (linux/amd64, linux/arm64), pushes to `ghcr.io/zircote/rust-template` with version + `latest` tags; images are signed/attested by the central signer workflow and verified fail-closed |

After the Release workflow completes, `package-homebrew.yml` fires via `workflow_run` and regenerates the source formula in `{owner}/homebrew-tap`.

**Never re-run `release.yml` against an existing tag.** Builds are not reproducible; a re-run would overwrite published assets with different bytes, violating the immutability the attestations protect.

---

## Monitoring Workflow Progress

### GitHub Actions Dashboard

- **All workflows:** https://github.com/zircote/rust-template/actions
- **Filter by tag:** Click the specific workflow run triggered by the tag push

### CLI Monitoring

```bash
# List recent workflow runs
gh run list --limit 10

# Watch a specific run
gh run watch <run-id>

# View logs for a failed run
gh run view <run-id> --log-failed
```

### What to Watch For

| Stage | Expected duration | Common failure point |
|---|---|---|
| Build Binaries (5 legs) | ~5-10 min | The macos-amd64 leg (cross-target on `macos-latest`) |
| Test + Cargo Audit gates | ~3 min | New advisory in `Cargo.lock` (audit scans the raw lockfile; deny may not have flagged it) |
| SBOM (generate + attest) | ~1 min | Attestation permissions (`id-token`, `attestations`) |
| Verify Attestations | <1 min | Fail-closed gate: any missing/unverifiable attestation blocks the release |
| Create Release | ~1 min | Only runs on tags, after verify passes |
| Publish (crates.io) | ~3 min | Trusted Publishing config missing, pre-publish checks |
| Docker chain (pipeline) | ~5-10 min | Buildx multi-platform, central signer/verify |
| Homebrew (after Release) | ~2 min | `workflow_run` trigger, tap token |

---

## Post-Release Verification

Run through this after all workflows complete.

- [ ] **GitHub Release** exists with correct version:
  ```bash
  gh release view vX.Y.Z
  ```
- [ ] **All 7 assets** are attached (version embedded in every name):
  - `rust_template-X.Y.Z-linux-amd64`
  - `rust_template-X.Y.Z-linux-arm64`
  - `rust_template-X.Y.Z-macos-amd64`
  - `rust_template-X.Y.Z-macos-arm64`
  - `rust_template-X.Y.Z-windows-amd64.exe`
  - `rust_template-X.Y.Z-sbom.cdx.json`
  - `rust_template-X.Y.Z-checksums.txt`
- [ ] **Attestations verify** from an independent machine (full reference: [SECURITY.md](../../SECURITY.md#verifying-release-artifacts)):
  ```bash
  gh release download vX.Y.Z --repo zircote/rust-template
  gh attestation verify rust_template-X.Y.Z-linux-amd64 --repo zircote/rust-template
  gh attestation verify rust_template-X.Y.Z-linux-amd64 --repo zircote/rust-template \
    --predicate-type https://cyclonedx.org/bom
  shasum -a 256 -c rust_template-X.Y.Z-checksums.txt
  ```
- [ ] **Release notes** are generated correctly
- [ ] **Docker image** is available:
  ```bash
  docker pull ghcr.io/zircote/rust-template:vX.Y.Z
  docker run --rm ghcr.io/zircote/rust-template:vX.Y.Z --version
  ```
- [ ] **Docker `latest` tag** points to the new release:
  ```bash
  docker pull ghcr.io/zircote/rust-template:latest
  docker run --rm ghcr.io/zircote/rust-template:latest --version
  ```
- [ ] **crates.io** package updated, and the served `.crate` attestation verifies:
  ```bash
  curl -fsSL -A 'release-check' \
    -O https://static.crates.io/crates/rust_template/rust_template-X.Y.Z.crate
  gh attestation verify rust_template-X.Y.Z.crate --repo zircote/rust-template
  # Or check: https://crates.io/crates/rust_template
  ```
- [ ] **Homebrew formula** updated in the tap (a `package-homebrew.yml` run appeared after Release completed)
- [ ] Download and test a binary on at least one platform:
  ```bash
  wget https://github.com/zircote/rust-template/releases/download/vX.Y.Z/rust_template-X.Y.Z-linux-amd64
  chmod +x rust_template-X.Y.Z-linux-amd64
  ./rust_template-X.Y.Z-linux-amd64 --version
  ```

---

## Rollback Procedures

### Roll Back a GitHub Release

```bash
# Delete the release
gh release delete vX.Y.Z --yes

# Delete the remote tag
git push --delete origin vX.Y.Z

# Delete the local tag
git tag -d vX.Y.Z
```

### Roll Back a crates.io Publish

**You cannot unpublish from crates.io.** Your options:

1. **Yank the version** (prevents new projects from depending on it):
   ```bash
   cargo yank --version X.Y.Z
   ```
2. **Publish a fix** as a patch release:
   ```bash
   # Fix the issue, bump to X.Y.Z+1
   git tag -a vX.Y.(Z+1) -m "Release vX.Y.(Z+1) (fixes vX.Y.Z)"
   git push origin vX.Y.(Z+1)
   ```

### Roll Back Docker Images

Docker images on GHCR are immutable by tag. To mitigate:

1. **Point users to a previous version:**
   ```bash
   docker pull ghcr.io/zircote/rust-template:vPREVIOUS
   ```
2. **Delete the package version** via GitHub UI: Packages > rust-template > Package versions > Delete
3. **Re-tag `latest`** to the previous good version by re-pushing a known-good tag

---

## Hotfix Release Process

When a critical bug or security issue is found in the latest release:

### 1. Create a Hotfix Branch

```bash
# Branch from the release tag
git checkout -b hotfix/vX.Y.(Z+1) vX.Y.Z
```

### 2. Apply the Fix

```bash
# Make the fix, then:
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

### 3. Bump Version and Tag

```bash
# Update Cargo.toml to X.Y.(Z+1)
git add -A
git commit -m "fix: <description of the critical fix>"
git commit --allow-empty -m "chore: bump version to X.Y.(Z+1)"
```

### 4. Merge and Release

```bash
# Merge hotfix into main
git checkout main
git merge hotfix/vX.Y.(Z+1)
git push origin main

# Tag and push
git tag -a vX.Y.(Z+1) -m "Release vX.Y.(Z+1)"
git push origin vX.Y.(Z+1)
```

### 5. If the Bad Version Was on crates.io

```bash
# Yank the bad version
cargo yank --version X.Y.Z

# The hotfix tag push triggers automatic publish of X.Y.(Z+1)
```

---

## Changelog and Release Notes

`CHANGELOG.md` is maintained by hand (Keep a Changelog format) and updated **before** tagging: the release-prep step moves the `## [Unreleased]` entries under a new `## [X.Y.Z] - <date>` heading and updates the compare links (the `/release` skill does this in the prep PR). GitHub Release notes are auto-generated by the Release workflow (`generate_release_notes: true`).

Conventional commit prefixes still map cleanly onto changelog sections:

| Commit prefix | Changelog section |
|---|---|
| `feat:` | Added |
| `fix:` | Fixed |
| `docs:` | Documentation |
| `perf:` | Performance |
| `refactor:` | Refactored |
| `test:` | Testing |
| `chore:` | Miscellaneous |

**Best practices:**
- Use scoped prefixes for clarity: `feat(auth): add JWT validation`
- Include `BREAKING CHANGE:` in the commit body for breaking changes
- A release with an empty `[Unreleased]` section is a red flag — stop and confirm what the release contains

---

## Deployment Targets Quick Reference

### GitHub Releases

- **URL:** https://github.com/zircote/rust-template/releases
- **Platforms:** Linux (amd64, arm64), macOS (amd64, arm64), Windows (amd64)
- **Attestations:** SLSA build provenance + CycloneDX SBOM attestation per binary, single `{bin}-{version}-checksums.txt` file; verify per [SECURITY.md](../../SECURITY.md#verifying-release-artifacts)

### Docker (GHCR)

- **Registry:** `ghcr.io/zircote/rust-template`
- **Platforms:** linux/amd64, linux/arm64
- **Base image:** distroless/cc-debian12 (minimal attack surface)
- **User:** nonroot:nonroot (unprivileged)
- **Tags:** `vX.Y.Z`, `X.Y`, `X`, `latest`, `sha-<commit>`

### crates.io

- **Package:** https://crates.io/crates/rust_template
- **Note:** Publishing runs on every `v*.*.*` tag via crates.io Trusted Publishing (OIDC, no token secret). It requires the one-time Trusted Publishing setup described under Prerequisites; without it, the publish job fails and the other release channels are unaffected.

### Homebrew Tap

- **Tap:** `{owner}/homebrew-tap` (override with the `HOMEBREW_TAP_REPO` variable)
- **Formula:** source formula generated from Cargo.toml metadata after each release

### Install Methods

```bash
# From GitHub release (Linux)
wget https://github.com/zircote/rust-template/releases/download/vX.Y.Z/rust_template-X.Y.Z-linux-amd64
chmod +x rust_template-X.Y.Z-linux-amd64

# From Docker
docker pull ghcr.io/zircote/rust-template:vX.Y.Z

# From crates.io
cargo install rust_template

# From source
cargo install --git https://github.com/zircote/rust-template
```

---

## Troubleshooting

| Problem | Cause | Fix |
|---|---|---|
| Release workflow fails at build | Cargo.toml version doesn't match tag | Ensure `version = "X.Y.Z"` matches tag `vX.Y.Z` |
| macos-amd64 build fails | Cross-target leg (`x86_64-apple-darwin` on `macos-latest`) | Check the `targets:` input on the toolchain step; the other 4 legs build natively |
| Cargo Audit gate fails | Real advisory in `Cargo.lock` | Fix the dependency (usually `cargo update <crate>`) via a normal PR, then restart the release |
| Verify Attestations job fails | Missing/unverifiable attestation | The fail-closed gate worked: the release was never created. Fix the cause and release with a **new** tag — never re-run against an existing one |
| Docker push fails | Insufficient permissions | Verify workflow permissions include `packages: write` |
| crates.io publish fails ("No Trusted Publishing config found") | Trusted Publishing not configured | One-time setup on crates.io: workflow `publish.yml`, environment `copilot`. Then `gh workflow run publish.yml --ref vX.Y.Z` |
| Homebrew formula not updated | `workflow_run` missed or tap token missing | `gh workflow run package-homebrew.yml -f version=X.Y.Z -f dry_run=false`; check `HOMEBREW_TAP_TOKEN` |
| Attestation step fails | Missing permissions | Check `id-token: write` and `attestations: write` on the job in `release.yml` / `publish.yml` |
| Tag push doesn't trigger workflows | Tag format wrong | Must match `v*.*.*` pattern exactly (e.g., `v1.0.0`, not `1.0.0`) |

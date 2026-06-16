# `rust_template`

<!-- Badges -->
[![GitHub Template](https://img.shields.io/badge/template-zircote%2Frust--template-blue?logo=github)](https://github.com/zircote/rust-template)
[![CI](https://github.com/zircote/rust-template/actions/workflows/pipeline.yml/badge.svg)](https://github.com/zircote/rust-template/actions/workflows/pipeline.yml)
[![Crates.io](https://img.shields.io/crates/v/rust_template.svg?logo=rust&logoColor=white)](https://crates.io/crates/rust_template)
[![Documentation](https://docs.rs/rust_template/badge.svg)](https://docs.rs/rust_template)
[![Rust Version](https://img.shields.io/badge/rust-1.92%2B-dea584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green)](https://github.com/zircote/rust-template/blob/main/LICENSE)
[![Clippy](https://img.shields.io/badge/linting-clippy-orange?logo=rust&logoColor=white)](https://github.com/rust-lang/rust-clippy)
[![cargo-deny](https://img.shields.io/badge/security-cargo--deny-blue?logo=rust&logoColor=white)](https://github.com/EmbarkStudios/cargo-deny)
[![Security: gitleaks](https://img.shields.io/badge/security-gitleaks-blue?logo=git&logoColor=white)](https://github.com/gitleaks/gitleaks)
[![Dependabot](https://img.shields.io/badge/dependabot-enabled-025e8c?logo=dependabot)](https://docs.github.com/en/code-security/dependabot)

A Rust template crate with modern tooling and best practices.

## Features

- **Type-safe error handling** with `thiserror` for clear error types
- **Builder pattern** for configuration with compile-time const functions
- **Comprehensive testing** including unit, integration, and property-based tests
- **Modern tooling** with clippy pedantic lints and cargo-deny supply chain security
- **Full documentation** with examples in all public APIs

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust_template = "0.1"
```

Or use cargo add:

```bash
cargo add rust_template
```

## Quick Start

```rust
use rust_template::{add, divide, Config};

fn main() -> Result<(), rust_template::Error> {
    // Basic arithmetic
    let sum = add(2, 3);
    println!("2 + 3 = {sum}");

    // Safe division with error handling
    let quotient = divide(10, 2)?;
    println!("10 / 2 = {quotient}");

    // Using configuration builder
    let config = Config::new()
        .with_verbose(true)
        .with_max_retries(5)
        .with_timeout(60);

    println!("Config: verbose={}, retries={}, timeout={}s",
        config.verbose(), config.max_retries(), config.timeout_secs());

    Ok(())
}
```

## API Overview

### Functions

| Function | Description |
|----------|-------------|
| `add(a, b)` | Adds two numbers |
| `divide(a, b)` | Divides with error handling |

### Types

| Type | Description |
|------|-------------|
| `Config` | Configuration with builder pattern |
| `Error` | Error type for operations |
| `Result<T>` | Type alias for `Result<T, Error>` |

## Getting Started

**New to this template?** See the [Getting Started Guide](docs/template/GETTING-STARTED.md) for a step-by-step walkthrough from "Use this template" to your first CI pass.

## Development

### Prerequisites

- Rust 1.92+ (2024 edition) — install via [rustup](https://rustup.rs/), **not** Homebrew
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) for supply chain security

### Setup

```bash
# Install Rust via rustup (not Homebrew)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Clone the repository
git clone https://github.com/zircote/rust-template.git
cd rust-template

# Build
cargo build

# Run tests
cargo test

# Run linting
cargo clippy --all-targets --all-features

# Format code
cargo fmt

# Check supply chain security
cargo deny check

# Generate documentation
cargo doc --open
```

### Project Structure

```text
crates/
├── lib.rs           # Library entry point
├── main.rs          # Binary entry point
└── ...              # Additional modules

tests/
└── integration_test.rs

Cargo.toml           # Project manifest
clippy.toml          # Clippy configuration
rustfmt.toml         # Formatter configuration
deny.toml            # cargo-deny configuration
CLAUDE.md            # AI assistant instructions
AGENTS.md            # AI coding agent instructions
.editorconfig        # Cross-editor defaults
.devcontainer/       # Codespaces / dev container config
.vscode/             # VS Code settings and extensions
```

### Code Quality

This project maintains high code quality standards:

- **Linting**: clippy with pedantic and nursery lints
- **Formatting**: rustfmt with custom configuration
- **Testing**: Unit tests, integration tests, and property-based tests
- **Documentation**: All public APIs documented with examples
- **Supply Chain**: cargo-deny for dependency auditing
- **CI/CD**: GitHub Actions for automated testing

### Running Checks

```bash
# Run all checks
cargo fmt -- --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test && \
cargo doc --no-deps && \
cargo deny check

# Run with MIRI for undefined behavior detection
cargo +nightly miri test
```

## CI/CD and Deployment

This template includes production-ready workflows:

### Continuous Integration

- **CI** (`.github/workflows/pipeline.yml`) - Format, lint, test, docs, supply chain security, MSRV check, coverage
- **Security Audit** (`.github/workflows/security-audit.yml`) - Daily cargo-audit scans
- **Quality Gates** (`.github/workflows/quality-gates.yml`) - CodeQL SAST, OSV/dependency-review SCA, OpenSSF Scorecard, and Trivy IaC/license on push/PR and weekly schedule
- **Benchmark** (`.github/workflows/benchmark.yml`) - Performance tracking with criterion
- **ADR Validation** (`.github/workflows/adr-validation.yml`) - Architectural decision records validation

### Release and Deployment

> **Template state: publication disabled.** `publish = false` in Cargo.toml gates GitHub Release creation, crates.io publishing, and Homebrew updates (workflows read it via `cargo metadata`); the build → attest → verify chain still runs as CI validation. Delete that line in your project to arm all three channels.

- **Release** (`.github/workflows/release.yml`) - Attested GitHub releases with multi-platform binaries
  - Builds for: Linux (`x86_64`, ARM64), macOS (`x86_64`, ARM64), Windows (`x86_64`)
  - Artifacts named `{bin}-{version}-{platform}` (e.g. `rust_template-0.2.0-linux-amd64`)
  - SLSA build provenance and `CycloneDX` SBOM attestations on every binary
  - Fail-closed `gh attestation verify` gate runs before the release is published
  - Single `{bin}-{version}-checksums.txt` checksums file

- **Docker** (`.github/workflows/release-docker.yml` via `pipeline.yml`) - Multi-platform container builds
  - Platforms: linux/amd64, linux/arm64
  - Distroless base image for security
  - Published to GitHub Container Registry (ghcr.io)
  - Tagged with version and 'latest'
  - Signed and attested by a centralized signer workflow, then verified fail-closed

- **Publish** (`.github/workflows/publish.yml`) - Automated crates.io publishing
  - Full pre-publish validation
  - Triggered on version tags
  - crates.io Trusted Publishing (OIDC) - no registry token secret
  - The registry-served `.crate` is downloaded back, byte-compared, and attested

- **Homebrew** (`.github/workflows/package-homebrew.yml`) - Tap formula updates
  - Runs after each Release completes
  - Generates a source formula from Cargo.toml metadata into `{owner}/homebrew-tap`

### Creating a Release

Releases are orchestrated end-to-end by the `/release` skill (`.claude/skills/release/SKILL.md`). The manual equivalent:

1. Update version in `Cargo.toml`
2. Create and push a version tag:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```
3. Workflows automatically:
   - Build binaries for all platforms with SLSA build provenance
   - Generate and attest a `CycloneDX` SBOM
   - Verify every attestation (fail-closed) before publishing anything
   - Create GitHub release with artifacts and checksums
   - Build, sign, and push Docker images
   - Publish to crates.io via Trusted Publishing
   - Update the Homebrew tap formula

Verification commands for every artifact type live in [SECURITY.md](SECURITY.md#verifying-release-artifacts).

### AI Coding Agent

- **Copilot Setup** (`.github/workflows/copilot-setup-steps.yml`) - Environment for GitHub Copilot coding agent
- **Agent Instructions**: `AGENTS.md`, `.github/copilot-instructions.md`, `CLAUDE.md`
- **Path-Specific Instructions**: `.github/instructions/` for Rust code and test patterns
- **Reusable Prompts**: `.github/prompts/` for common development tasks

### Docker Usage

Pull and run the container:

```bash
# Pull latest
docker pull ghcr.io/zircote/rust-template:latest

# Run specific version
docker pull ghcr.io/zircote/rust-template:v0.1.0
docker run --rm ghcr.io/zircote/rust-template:v0.1.0 --version
```

## MSRV Policy

The Minimum Supported Rust Version (MSRV) is **1.92**. Increasing the MSRV is considered a minor breaking change.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, PR checklist, and coding standards.

Please also review:
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) - Community guidelines
- [SECURITY.md](SECURITY.md) - Vulnerability reporting
- [GOVERNANCE.md](GOVERNANCE.md) - Decision-making process

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/zircote/rust-template/blob/main/LICENSE) file for details.

## Acknowledgments

- [The Rust Programming Language](https://www.rust-lang.org/)
- [Cargo](https://doc.rust-lang.org/cargo/)
- [clippy](https://github.com/rust-lang/rust-clippy)

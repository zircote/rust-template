# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-06-17

### Security

- **deps**: Force `esbuild` >= 0.28.1 in the docs site (GHSA-gv7w-rqvm-qjhr RCE via `NPM_CONFIG_REGISTRY`, GHSA-g7r4-m6w7-qqqr Windows dev-server path traversal)
- **docker**: Pin Dockerfile base images by digest (OpenSSF Scorecard Pinned-Dependencies, Trivy DS-0001) and add a `docker` Dependabot ecosystem to keep them fresh
- **ci**: Harden GitHub Actions token permissions to least privilege across all workflows (OpenSSF Scorecard Token-Permissions)
- **ci**: Scope the Trivy supply-chain scan away from the dev-only docs site, clearing license-classification noise from the code-scanning hub

### Added

- **fuzz**: Add a minimal cargo-fuzz harness targeting the public `process()` parser
- **docs**: Add an end-to-end "Attested Delivery" guide — which `zircote/.github` reusables run, the `publish` gate, the build → sign → verify chain, and a downstream adoption runbook
- **docs-site**: Add Astro Starlight documentation site at `site/`
  - 73 browsable, searchable pages deployed to GitHub Pages
  - Auto-generated content from `docs/` markdown, `.github/workflows/*.yml`, and `CLAUDE.md` reference sections
  - Embedded rustdoc API reference at `/api/`
  - Pagefind full-text search, Mermaid diagram support, OG/Twitter social meta
  - Content generation scripts with freshness checking (`npm run check:freshness`)
  - Splash landing page with feature cards, 11-section sidebar navigation
- **workflows**: Add `docs-freshness.md` gh-aw workflow for weekly staleness detection
- **ci**: Add template-init workflow for automatic repo renaming
- Add community and governance files
- Add editor, devcontainer, and VS Code configuration
- Add GitHub config, Copilot setup, and CodeQL workflow
- Add documentation structure and ADR-0002
- Add justfile for local CI parity
- **commands**: Add `/spec-orchestrator` slash command for parallel agent team orchestration
  - Phase-based workflow: bootstrap, discovery, synthesis, execution, verification, cleanup
  - `jq`-based inventory processing to conserve agent context windows
  - Just-in-time teammate spawning with staleness prevention and heartbeat monitoring
  - Anti-takeover rules preventing the orchestrator from writing code itself
  - Mnemonic blackboard storage for persistent, project-isolated work directory
- **commands**: Add `/init-project` toolchain verification (Phase 1.5) requiring rustup over Homebrew
- Add `template-sync` recipe to justfile for syncing shared tooling from upstream

### Changed

- **ci**: Decouple the GitHub Release from the external-publish gate — any pushed tag now produces an attested GitHub Release (binaries + SBOM + source snapshot); `publish = false` gates only crates.io, the container image, and Homebrew
- **release**: Align the `/release` skill with the current attested workflows (source-snapshot and gate-attestation jobs; 8 release assets)
- **workflows**: Replace rustdoc+mdBook docs-deploy workflow with Astro Starlight site deployment
  - Builds Node.js site alongside rustdoc, embeds API docs at `/api/`
  - Triggers on `docs/**`, `site/**`, `CLAUDE.md`, and `Cargo.toml` changes

### Build

- Bump thiserror 2.0.18 and proptest 1.10.0
- Bump taiki-e/install-action to v2.67.25

### CI/CD

- Use GitHub API for signed changelog commits
- Consolidate CI/release into unified pipeline

### Documentation

- Rewrite Copilot Jumpstart prompts for 500-char limit
- Update project docs, rustfmt config, and tests
- Add commit signing guidance for contributors
- Add rustup toolchain setup guidance to GETTING-STARTED.md, README.md, and CONTRIBUTING.md (not Homebrew)
- Add 90% code coverage requirement across all metrics to CLAUDE.md

### Fixed

- **docs**: Fix the GitHub Pages base path so the published site renders styled (assets were 404ing) and base-prefix internal navigation links
- **docs**: Align workflow-reference coverage with the actual workflows (remove 6 stale reference pages) and regenerate all pages so committed content matches the generators
- Rename copilot-setup-steps job ID
- Add cargo deny check and rustls constraints to jumpstart prompts

## [0.2.0] - 2026-02-07

### Added

- Add ADR validation and viewer workflows
- Add production-ready CI/CD and deployment workflows
- **phase1**: Add security & quality workflows with comprehensive docs
- **phase2**: Add comprehensive testing enhancements
- **phase3**: Add packaging & distribution for all major platforms
- **phase4**: Add UX enhancements and automation workflows
- **phase5**: Add advanced security and observability features

### CI/CD

- Disable Docker Hub and crates.io publish triggers

### Documentation

- Update documentation to reflect current codebase
- Add comprehensive deployment guide
- Add Copilot Jumpstart prompts for template users

### Fixed

- **workflows**: Correct SHAs, disable heavy triggers, fix SLSA structure
- **docs**: Add backticks to x86_64 in README for clippy doc_markdown lint
- **docker**: Keep Cargo.lock in Docker context and fix FROM casing
- **ci**: Correct git-cliff-action SHA in release and changelog workflows
- **ci**: Fix release asset upload and ARM64 strip
- **ci**: Rename binaries to unique asset names before upload
- **ci**: Add shell: bash to release upload step for Windows compat

### Refactored

- Rename src directory to crates

## [0.1.0] - 2026-02-07

### Added

- Update rust-template
- Add Claude Code agents for development workflow

### CI/CD

- Add dependabot auto-merge workflow
- Update MSRV check to Rust 1.92

### Documentation

- Add MIT LICENSE file
- Fix LICENSE links in README for rustdoc
- Update copilot-instructions.md

### Fixed

- Update deny.toml to cargo-deny v2 format
- Update dtolnay/rust-toolchain action to v1
- Restore commit SHA pinning for rust-toolchain action

### Miscellaneous

- Update GitHub Actions and dependencies to December 2025 latest
- Update Rust dependencies to December 2025 latest versions
- **deps**: Bump the github-actions group with 2 updates
- **deps**: Bump taiki-e/install-action in the github-actions group
- **deps**: Bump taiki-e/install-action in the github-actions group
- **deps**: Bump actions/cache from 4.2.3 to 5.0.2
- **deps**: Bump taiki-e/install-action in the github-actions group
- **deps**: Bump actions/checkout from 4.2.2 to 6.0.2 (#15)
- **deps**: Bump taiki-e/install-action in the github-actions group (#14)
- **deps**: Bump the github-actions group with 2 updates (#16)

### Refactored

- Simplify code and update to Rust 1.92 best practices

<!-- generated by git-cliff -->

# CI/CD Workflows Reference

Comprehensive guide to every GitHub Actions workflow included in the
`zircote/rust-template` repository. Workflows are organized by purpose and
annotated with trigger conditions, required secrets, and activation status.

---

## Architecture

All CI and release work is orchestrated through a single `pipeline.yml`
that calls reusable workflows via `workflow_call`. This ensures CI passes
before any release work begins and eliminates duplicate checks.

```text
                       push/PR/tag
                            |
                  +---------+---------+
                  |         |         |
                [ci]   [coverage] [test-matrix]*
                  |                   (* PR only)
        +---------+---------+
        |                   |
     [docker]          [release]**
   (PR=build-only)     (** tags only)
                            |
       +--------+--------+--+--------+--------+
       |        |        |  |        |        |
    [sign] [publish] [pkgs] [sbom] [slsa-build]
                                        |
                                  [slsa-provenance]
```

---

## Overview Table

### Pipeline Workflows (Orchestrator + Reusable)

| Workflow | File | Called By | Purpose |
|---|---|---|---|
| Pipeline | `pipeline.yml` | push, PR, tag, manual | Orchestrator — routes triggers, enforces dependencies |
| CI Checks | `ci-checks.yml` | `pipeline.yml` | fmt, clippy, test (3-OS), doc, deny, msrv, gate |
| Code Coverage | `ci-coverage.yml` | `pipeline.yml` | LCOV/HTML/JSON coverage, Codecov, PR comment |
| Test Matrix | `ci-test-matrix.yml` | `pipeline.yml` (PR only) | 12-combo matrix, integration tests, Miri |
| Create Release | `release-create.yml` | `pipeline.yml` (tags) | GH release, git-cliff body, 5 binaries, CHANGELOG.md |
| Sign Release | `release-sign.yml` | `pipeline.yml` (tags) | Cosign signing, SHA256/SHA512 checksums |
| Publish | `release-publish.yml` | `pipeline.yml` (tags) | cargo package + crates.io publish |
| Docker | `release-docker.yml` | `pipeline.yml` | Multi-platform Docker build/push to GHCR |
| Packages | `release-packages.yml` | `pipeline.yml` (tags) | Homebrew, Snap, MSI, deb, rpm |
| SBOM | `release-sbom.yml` | `pipeline.yml` (tags) | SPDX SBOM generation, attach to release |

### Standalone Workflows

| Workflow | File | Trigger | Required Secrets | Status |
|---|---|---|---|---|
| Security Audit | `security-audit.yml` | schedule (daily), push, manual | -- | Active |
| CodeQL Analysis | `codeql-analysis.yml` | push, PR, schedule (weekly), manual | -- | Active |
| Secrets Scan | `secrets-scan.yml` | push, PR, manual | `GITLEAKS_LICENSE` | Active |
| Benchmark | `benchmark.yml` | push, PR, manual | -- | Active |
| Benchmark Regression | `benchmark-regression.yml` | PR, manual | -- | Active |
| Mutation Testing | `mutation-testing.yml` | PR (src/tests paths), manual | -- | Active |
| Fuzz Testing | `fuzz-testing.yml` | manual | -- | Opt-in |
| Code Quality Metrics | `code-quality.yml` | PR, manual | -- | Active |
| Spell Check | `spell-check.yml` | push, PR, manual | -- | Active |
| Dependabot Auto-Merge | `dependabot-automerge.yml` | PR (dependabot actor) | -- | Active |
| Stale Issue Management | `stale.yml` | manual | -- | Opt-in |
| Contributor Recognition | `contributors.yml` | manual | -- | Opt-in |
| Template Init | `template-init.yml` | push to main, manual | -- | Active |
| Nightly Builds | `nightly.yml` | manual | -- | Opt-in |
| Deploy Documentation | `docs-deploy.yml` | push to main (docs/site/CLAUDE.md/Cargo.toml paths), manual | -- | Active |
| ADR Validation | `adr-validation.yml` | push, PR (docs/adr paths), manual | -- | Active |
| ADR Viewer | `adr-viewer.yml` | push (docs/adr paths), manual | -- | Active |
| Docker Hub Multi-Registry | `docker-hub.yml` | manual | `DOCKERHUB_USERNAME`, `DOCKERHUB_TOKEN` | Opt-in |
| Copilot Setup Steps | `copilot-setup-steps.yml` | manual | -- | Active |

> **"Active"** means the workflow has at least one automatic trigger (push, PR,
> schedule, release, or tag). **"Opt-in"** means only `workflow_dispatch`
> (manual) is enabled; automatic triggers are commented out and must be
> uncommented to activate.

---

## Pipeline Orchestrator

### pipeline.yml

**What it does:** Single entry point for all CI and release work. Routes
triggers to reusable workflows with explicit `needs:` dependencies ensuring
CI passes before any release work begins.

**Trigger:** Push to `main`/`master`, pull request to `main`/`master`, push
tag `v*.*.*`, manual (with stage selector).

**Concurrency:** Cancels in-progress runs for branches/PRs; never cancels
tag runs.

**Manual dispatch stages:** `all`, `ci`, `release`, `sign`, `publish`,
`docker`, `packages`, `sbom`, `slsa`.

**Job dependency chain:**

- `ci`, `coverage`, `test-matrix` — run in parallel (test-matrix is PR only)
- `docker` — needs `ci` (PR = build-only via `push: false`)
- `release` — needs `ci` (tags only)
- `sign`, `publish`, `packages`, `sbom`, `slsa-build` — need `release`
- `slsa-provenance` — needs `slsa-build`

---

## CI Reusable Workflows

### ci-checks.yml

**What it does:** The primary quality gate. Runs formatting, linting, tests
on three operating systems, documentation build, dependency license/advisory
checks (cargo-deny), MSRV verification. A final `all-checks-pass` job gates
merge readiness.

**Inputs:** `rust-version` (default: stable), `msrv` (default: 1.92).

**Secrets:** `CODECOV_TOKEN` (optional).

### ci-coverage.yml

**What it does:** Generates detailed code coverage reports with
`cargo-llvm-cov` in LCOV, HTML, and JSON formats. Uploads to Codecov, posts a
summary as a PR comment, and checks against an 80% coverage threshold.

**Secrets:** `CODECOV_TOKEN` (optional).

### ci-test-matrix.yml

**What it does:** Runs the full test suite across a matrix of operating systems
(Ubuntu, macOS, Windows) and Rust toolchains (stable, beta, nightly, MSRV).
Includes integration tests, Miri undefined-behavior detection, and a summary
report.

**Inputs:** `msrv` (default: 1.92).

---

## Release Reusable Workflows

### release-create.yml

**What it does:** Creates a GitHub Release with an auto-generated changelog
(via git-cliff), builds release binaries for five targets (Linux x86_64,
Linux aarch64, macOS x86_64, macOS aarch64, Windows x86_64), and commits the
full CHANGELOG.md to the `main` branch via GitHub API. Eliminates the separate
changelog workflow.

**Inputs:** `tag` (required).

### release-sign.yml

**What it does:** Downloads all assets from the GitHub release, signs each
with Sigstore Cosign (keyless OIDC), generates SHA-256 and SHA-512 checksum
files, signs the checksums, and uploads everything back. Appends verification
instructions to the release notes.

**Inputs:** `tag` (required).

### release-publish.yml

**What it does:** Publishes the crate to crates.io. Runs `cargo package`
validation and a dry-run before the actual publish. CI is guaranteed by the
`needs: [ci] -> needs: [release]` chain — no duplicate checks.

**Inputs:** `tag` (required).

**Secrets:** `CARGO_REGISTRY_TOKEN` (required).

### release-docker.yml

**What it does:** Builds a multi-platform Docker image (linux/amd64,
linux/arm64) and optionally pushes to GHCR. Uses GitHub Actions cache for
layer caching. Tags follow semver and include `latest` for the default branch.

**Inputs:** `push` (boolean, default: false).

### release-packages.yml

**What it does:** Builds distribution packages in five parallel jobs: Homebrew
formula update, Snap package, Windows MSI installer, Debian package (.deb),
and RPM package (.rpm). All packages are attached to the GitHub release.

**Inputs:** `tag` (required).

**Secrets:** `HOMEBREW_TAP_TOKEN` (optional), `SNAPCRAFT_TOKEN` (optional).

### release-sbom.yml

**What it does:** Generates a Software Bill of Materials in SPDX 2.3 JSON
format using `cargo-sbom` and attaches it to the GitHub release.

**Inputs:** `tag` (required).

---

## Security & Compliance

### security-audit.yml

**What it does:** Runs `cargo audit` against the RustSec advisory database to
detect known vulnerabilities in dependencies.

**Trigger:** Daily schedule at 00:00 UTC, push when `Cargo.toml` or
`Cargo.lock` change, manual.

### codeql-analysis.yml

**What it does:** Performs GitHub CodeQL static analysis on Rust code. Results
surface in the repository Security tab.

**Trigger:** Push to `main`, pull request to `main`, weekly schedule, manual.

### secrets-scan.yml

**What it does:** Scans the repository history for accidentally committed
secrets using Gitleaks.

**Trigger:** Every push, every pull request, manual.

**Secrets:** `GITLEAKS_LICENSE` (optional).

---

## Testing & Quality

### benchmark.yml

**What it does:** Runs `cargo bench --workspace` and uploads Criterion results
as artifacts.

**Trigger:** Push to `main`/`master`, pull request, manual.

### benchmark-regression.yml

**What it does:** Compares benchmark results against a cached baseline from
the main branch. Posts a performance report as a PR comment.

**Trigger:** Pull request, manual.

### mutation-testing.yml

**What it does:** Runs `cargo-mutants` to evaluate test suite effectiveness.
Posts results as a PR comment.

**Trigger:** Pull request when source or test files change, manual.

### fuzz-testing.yml

**What it does:** Runs `cargo-fuzz` against all fuzz targets. Opens GitHub
issues for crashes.

**Trigger:** Manual only (daily schedule commented out).

### code-quality.yml

**What it does:** Collects code quality metrics including unsafe code analysis,
binary size breakdown, and documentation coverage.

**Trigger:** Pull request, manual.

### spell-check.yml

**What it does:** Checks spelling across all project files using
`crate-ci/typos`.

**Trigger:** Push, pull request, manual.

---

## Maintenance & Automation

### dependabot-automerge.yml

**What it does:** Automatically enables auto-merge for Dependabot patch and
minor version PRs.

**Trigger:** Pull request (dependabot actor only).

### stale.yml

**What it does:** Marks issues and PRs as stale after inactivity.

**Trigger:** Manual only (daily schedule commented out).

### contributors.yml

**What it does:** Generates `CONTRIBUTORS.md` from git history.

**Trigger:** Manual only (monthly schedule commented out).

### template-init.yml

**What it does:** Automatically renames the project when a new repository is
created from this template.

**Trigger:** Push to `main`, manual.

### nightly.yml

**What it does:** Builds with Rust nightly, creates rolling pre-release.

**Trigger:** Manual only (daily schedule commented out).

---

## Documentation & ADRs

### docs-deploy.yml

**What it does:** Builds API docs and optionally an mdBook guide. Deploys to
GitHub Pages.

**Trigger:** Manual only.

### adr-validation.yml

**What it does:** Validates Architecture Decision Records using adrscope.

**Trigger:** Push/PR when ADR files change, manual.

### adr-viewer.yml

**What it does:** Generates an HTML viewer for all ADRs.

**Trigger:** Push when ADR files change, manual.

---

## AI Coding Agent

### copilot-setup-steps.yml

**What it does:** Prepares the CI environment for GitHub Copilot coding agent
sessions.

**Trigger:** Manual only.

---

## Enabling/Disabling Workflows

### Running a specific pipeline stage manually

Navigate to **Actions > Pipeline > Run workflow** and select a stage from the
dropdown: `all`, `ci`, `release`, `sign`, `publish`, `docker`, `packages`,
`sbom`, or `slsa`.

### Running a reusable workflow standalone

All reusable workflows also support `workflow_dispatch`. Navigate to
**Actions > (workflow name) > Run workflow** to trigger any reusable workflow
independently.

### Activating an opt-in standalone workflow

1. Open the workflow file in `.github/workflows/`.
2. Uncomment the desired triggers under the `on:` key.
3. Configure required secrets in **Settings > Secrets and variables > Actions**.
4. Commit and push.

### Disabling a standalone workflow

**Option A:** Replace automatic triggers with only `workflow_dispatch:`.

**Option B:** Delete the workflow YAML file.

**Option C:** **Actions > (workflow) > ... > Disable workflow** in GitHub UI.

---

## Required Secrets Summary

| Secret | Used By | Purpose | Required |
|---|---|---|---|
| `GITHUB_TOKEN` | multiple | GitHub API access | Built-in |
| `CODECOV_TOKEN` | `ci-coverage.yml`, `ci-checks.yml` | Coverage upload | Optional |
| `CARGO_REGISTRY_TOKEN` | `release-publish.yml` | crates.io publish | If publishing |
| `GITLEAKS_LICENSE` | `secrets-scan.yml` | Gitleaks license | Optional |
| `DOCKERHUB_USERNAME` | `docker-hub.yml` | Docker Hub username | If using Docker Hub |
| `DOCKERHUB_TOKEN` | `docker-hub.yml` | Docker Hub token | If using Docker Hub |
| `HOMEBREW_TAP_TOKEN` | `release-packages.yml` | Homebrew tap write access | If using Homebrew |
| `SNAPCRAFT_TOKEN` | `release-packages.yml` | Snap Store credentials | Optional |

Configure secrets at **Settings > Secrets and variables > Actions > New
repository secret**.

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
pipeline.yml (push/PR/tag/manual)
                       push/PR/tag
                            |
            +---------+-----+-----+-------------+
            |         |           |             |
          [ci]   [coverage] [test-matrix]* [pin-check]
            |                 (* PR only)
         [docker]
       (PR=build-only)
            |
      [docker-sign]       (push/tags only — central signer)
            |
     [docker-verify]      (fail-closed gate)

release.yml (tag push; dispatch = dry-run)
  [meta] -> [build x5 + attest] -> [sbom + attest] -> [verify]** -> [release]*
            [test] [audit] ----------------------------^
  (* tags only; ** fail-closed BEFORE the release exists)

publish.yml (tag push; dispatch = dry-run)
  checks -> Trusted Publishing (OIDC) -> registry .crate byte-compare -> attest

package-homebrew.yml
  release.yml completion --workflow_run--> source formula -> <owner>/homebrew-tap
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
| Docker | `release-docker.yml` | `pipeline.yml` | Multi-platform Docker build/push to GHCR; outputs the manifest digest |
| Release | `release.yml` | tags, manual (dry-run) | 5 attested binaries + attested SBOM + fail-closed verify → GH Release |
| Publish | `publish.yml` | tags, manual (dry-run) | crates.io Trusted Publishing (OIDC) + registry `.crate` attestation |
| Homebrew | `package-homebrew.yml` | `workflow_run` (Release), release, manual | Source formula in `<owner>/homebrew-tap`, metadata from Cargo.toml |
| Pin Check | `zircote/.github` » `pin-check.yml` | `pipeline.yml` | Asserts every `uses:` is pinned to a 40-char SHA |
| Sign and Attest Image | `zircote/.github` » `sign-and-attest.yml` | `pipeline.yml` (push/tags) | Cosign signature, SLSA provenance, SBOM + vuln report as OCI referrers |
| Verify Image Attestations | `zircote/.github` » `verify-attestation.yml` | `pipeline.yml` (push/tags) | Fail-closed verification gate before release |

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

**Manual dispatch stages:** `all`, `ci`, `docker`. Stage `docker` also
runs `ci` (a dependency), making the build → push → sign → verify chain
exercisable without cutting a tag. Releases are NOT orchestrated by the
pipeline — they run as independent tag-triggered workflows
(`release.yml`, `publish.yml`, `package-homebrew.yml`).

**Job dependency chain:**

- `ci`, `coverage`, `test-matrix`, `pin-check` — run in parallel
  (test-matrix is PR only)
- `docker` — needs `ci` (PR = build-only via `push: false`)
- `docker-sign` — needs `docker` (push/tags only); calls the centralized
  `zircote/.github` signer workflow, pinned by full commit SHA (SLSA
  Build L3 isolation — the signing identity is the central workflow,
  not this repository)
- `docker-verify` — needs `docker-sign`; fail-closed verification of
  signature, provenance, and SBOM attestations

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

## Release Workflows

**Publication is disabled in the template**: `publish = false` in
Cargo.toml gates the Create Release job, crates.io publishing, and
Homebrew tap updates (each workflow reads it via `cargo metadata` at
runtime). The build → attest → SBOM → fail-closed-verify chain still
runs as CI validation. Downstream projects delete that single line to
arm all three channels.

All release workflows are **var-driven**: crate name, binary name,
version, description, and license are resolved at runtime from
`cargo metadata`; owner/repo come from the GitHub context. Instantiating
the template requires editing only `Cargo.toml` — nothing in these files
is renamed. They mirror the verified architecture of `zircote/rlm-rs`.

### release.yml

**What it does:** On a `v*.*.*` tag: resolves project metadata, builds
five platform binaries (`{bin}-{version}-{platform}` naming;
linux-amd64, linux-arm64 on native arm runners, macos-arm64, macos-amd64
via cross-target, windows-amd64) with SLSA build provenance attested at
build time, runs test and cargo-audit gates (tags are untrusted input),
generates and attests a CycloneDX SBOM bound to every binary, then
**fail-closed verifies every attestation before the GitHub Release is
created**. The tag-gated release job attaches binaries, the SBOM, and a
checksums file, with auto-generated release notes. A tag publishes
nothing unattested.

**Triggers:** tag push; `workflow_dispatch` from any branch is a dry-run
(version suffixed `-dev`, publish job skipped).

**Secrets:** `HOMEBREW_TAP_TOKEN` (optional — PAT lets the release event
propagate to downstream workflows; `workflow_run` is the fallback).

### publish.yml

**What it does:** Pre-publish gauntlet (fmt, clippy, test, doc, deny,
package, dry-run publish), then crates.io **Trusted Publishing** via
OIDC — no long-lived registry token. After publish it downloads the
`.crate` the registry actually serves, byte-compares it against the
local package, and attests the registry bytes.

**One-time setup per crate:** crates.io → crate Settings → Trusted
Publishing → this repo, workflow `publish.yml`, environment `copilot`.

**Triggers:** tag push; `workflow_dispatch` is a dry-run (publish and
attest steps are tag-gated).

### package-homebrew.yml

**What it does:** Generates a source formula (name, description, and
license read from Cargo.toml at the released tag) and pushes it to
`{owner}/homebrew-tap` (override with repo variable
`HOMEBREW_TAP_REPO`). Triggered by `workflow_run` on Release completion
because bot-authored release events do not trigger workflows; idempotent
on repeated firings.

**Triggers:** `workflow_run` (Release), release published, manual
dispatch with `version` and `dry_run` inputs.

**Secrets:** `HOMEBREW_TAP_TOKEN` (required for tap pushes).

### release-docker.yml

**What it does:** Builds a multi-platform Docker image (linux/amd64,
linux/arm64) and optionally pushes to GHCR. Uses GitHub Actions cache for
layer caching. Tags follow semver and include `latest` for the default branch.

**Inputs:** `push` (boolean, default: false).

**Outputs:** `image-digest` — the pushed manifest digest (`sha256:...`),
consumed by the `docker-sign`/`docker-verify` attestation chain.

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

Navigate to **Actions > Pipeline > Run workflow** and select a stage from
the dropdown: `all`, `ci`, or `docker`. To dry-run the release chain
without a tag, dispatch **Release** or **Publish to crates.io** instead —
both tag-gate their publish-side steps.

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
| `GITLEAKS_LICENSE` | `secrets-scan.yml` | Gitleaks license | Optional |
| `DOCKERHUB_USERNAME` | `docker-hub.yml` | Docker Hub username | If using Docker Hub |
| `DOCKERHUB_TOKEN` | `docker-hub.yml` | Docker Hub token | If using Docker Hub |
| `HOMEBREW_TAP_TOKEN` | `package-homebrew.yml`, `release.yml` | Homebrew tap write access; release-event propagation | If using Homebrew |

crates.io publishing uses **Trusted Publishing (OIDC)** — there is no
registry token secret. One-time setup on crates.io: crate Settings →
Trusted Publishing → this repo, workflow `publish.yml`, environment
`copilot`.

### Repository Variables

| Variable | Used By | Purpose | Default |
|---|---|---|---|
| `HOMEBREW_TAP_REPO` | `package-homebrew.yml` | Tap repository name under the owner | `homebrew-tap` |

All other project specificity (crate name, binary name, version,
description, license) is resolved from `cargo metadata` at runtime.

Configure secrets at **Settings > Secrets and variables > Actions > New
repository secret**.

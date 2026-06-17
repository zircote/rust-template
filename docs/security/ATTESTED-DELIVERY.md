---
diataxis_type: explanation
---

# Attested Delivery, End to End

How a change in this repository travels from a pull request to a signed,
independently verifiable release — and exactly which gate signs what.

> **Diátaxis mode: Explanation**, with one embedded **How-to** section
> ("How to Adopt This in Your Own Project", numbered steps + a verification
> command). Reference lookups (per-workflow triggers, verify commands) are
> *not* duplicated here — they live in
> [`CI-WORKFLOWS.md`](../template/CI-WORKFLOWS.md) and
> [`SIGNED-RELEASES.md`](./SIGNED-RELEASES.md). This document explains how the
> pieces compose.

This repository is a **Mode-A consumer** of the central reusable-workflow
repository `zircote/.github`: it does not re-implement signing, scanning, or
verification — it *calls* central reusables pinned to a full commit SHA, and
each gate's verdict normalizes on SARIF.

---

## The Two Seams

Attested delivery has two distinct points where evidence is produced and bound
to a subject. Keeping them separate is the key to understanding the whole
system.

| Seam | When | Subject | Evidence form | Where it lands |
|---|---|---|---|---|
| **Merge-time gates** | every push / PR to `main` | the source tree / commit | SARIF | the repo's **code-scanning hub** (Security tab); the *Code scanning results* required check is the merge gate |
| **Deploy-time attestation** | tag push (release) / main push (container) | a *published* artifact bound by digest | signed in-toto attestation (Sigstore keyless) | GitHub artifact attestations, verifiable with `gh attestation verify` |

The same four central reusables power the merge-time seam. A **subset** of them
re-runs at deploy time, where the SARIF they emit is no longer just uploaded —
it becomes a **signed predicate** bound to a release subject.

---

## Stage 1 — Merge-Time Quality Gates (`quality-gates.yml`)

`quality-gates.yml` is a thin caller of four `zircote/.github` central
reusables. It runs on push and PR to `main`, on a weekly Monday 06:00 UTC
schedule, and on manual dispatch. Top-level permissions are `contents: read`;
each job widens scope only as its reusable requires.

| Job | Reusable (pinned SHA) | Emits | SARIF → code-scanning category |
|---|---|---|---|
| `sast` | `reusable-sast-codeql.yml@740cb8efb57af0187f88e9b4f939355b871a5895` | CodeQL (Rust, `build-mode: none`) | `/language:rust` |
| `sca` | `reusable-sca-osv.yml@77a87549a65c6c978a0e87efe0168ed3517f7ca4` | OSV-Scanner (`--lockfile=Cargo.lock`, `fail-on-severity: high`) + dependency-review (PR gate) | OSV SARIF; artifact `OSV Scanner SARIF file` |
| `posture` | `reusable-scorecard.yml@77a87549a65c6c978a0e87efe0168ed3517f7ca4` | OpenSSF Scorecard (push/schedule only — needs the default branch) | `scorecard` |
| `trivy` | `reusable-trivy.yml@77a87549a65c6c978a0e87efe0168ed3517f7ca4` | Trivy filesystem scan (`scan-iac: true`: Dockerfile, manifests, licenses) | `trivy-iac-license` |

**Why a caller must over-grant permissions.** A reusable workflow's job
permissions are a *floor*: a caller must grant **≥** every permission any of
the reusable's jobs declares, or GitHub fails the call at startup — even for
jobs that are conditionally skipped. That is why `sast` and `trivy` grant
`packages: read` (the CodeQL analyze job and the Trivy *image* job declare it)
even though no package is read in template state, and why `sca` grants
`pull-requests: write` (its dependency-review job declares it).

**What each reusable outputs (the attestation seam).** Every gate reusable
publishes its SARIF as a named artifact *and* exposes `sarif-artifact` /
`sarif-filename` outputs. At merge time those outputs are unused (the SARIF is
uploaded to code scanning via `github/codeql-action/upload-sarif`). At release
time they become the input to the signing seam — see Stage 3.

---

## Stage 2 — The `publish = false` Template Gate

This template ships **publishing-disabled**. `Cargo.toml` carries:

```toml
publish = false
```

Every release-side workflow resolves this at runtime via
`cargo metadata --no-deps --locked --format-version 1`, mapping
`.packages[0].publish == []` to `publishable = "false"`. That single boolean
is the master switch. Deleting the `publish = false` line arms all four
outward-facing channels at once.

**Exactly what `publish = false` disables:**

| Channel | Mechanism in template state |
|---|---|
| **Container build → sign → verify** | `pipeline.yml`'s `gate` job resolves `publishable=false`; the `docker` job's `if:` requires `publishable == 'true'`, so `docker`, `docker-sign`, `docker-verify`, `gate-image`, and `attest-container-scan` are all **skipped** — the template builds no image. |
| **crates.io publish** | `publish.yml`'s `guard` job sets `publishable=false`; the `publish` job's `if:` gates the whole job off. (cargo itself also refuses `cargo publish` while `publish = false`.) |
| **GitHub Release creation** | `release.yml`'s `release` job requires `startsWith(github.ref, 'refs/tags/') && needs.meta.outputs.publishable == 'true'`. The build → attest → SBOM → **fail-closed verify** chain still runs as CI validation; only the final *publish of the release* is gated off. |
| **Homebrew tap update** | `package-homebrew.yml` reads `publishable` from `Cargo.toml` *at the released tag*; every formula-push step is gated on `publishable == 'true'`. |

The deploy-time *evidence chain still executes* in template state — binaries
build, provenance and SBOM are attested, verification runs fail-closed. What
the gate withholds is the *outward publication* of those artifacts. This lets
the template prove the whole pipeline works without shipping anything.

---

## Stage 3 — Build → Sign → Verify

There are two independent deploy-time chains: **release binaries** (`release.yml`)
and the **container image** (`pipeline.yml`). Both are fail-closed.

### 3a. Release binaries, SBOM, and the gate-verdict seam (`release.yml`)

Triggered by a `v*.*.*` tag (a `workflow_dispatch` from any branch is a
dry-run: version suffixed `-dev`, release job skipped). Flow:

1. **`meta`** — resolve `bin`, `version`, `publishable` from `cargo metadata`.
2. **`build`** (5-platform matrix: `linux-amd64`, `linux-arm64`, `macos-arm64`,
   `macos-amd64` via cross-target, `windows-amd64`) — each binary is staged as
   `{bin}-{version}-{platform}` and gets **SLSA build provenance** attached at
   build time via `actions/attest-build-provenance`
   (`@a2bbfa25375fe432b6a289bc6b6cd05ecd0c4c32`, v4.1.0).
3. **`source`** — `git archive` produces a *published* source snapshot
   `{bin}-{version}-source.tar.gz` and attests its provenance. Because the exact
   bytes ship as a release asset, the gate attestations below can be verified
   from any workstation against a real, downloadable subject.
4. **`sbom`** — a CycloneDX SBOM (`anchore/sbom-action`) is generated over the
   binaries and bound to **every** binary via `actions/attest-sbom`
   (`@c604332985a26aa8cf1bdc465b92731239ec6b9e`, v4.1.0).
5. **The gate-verdict seam** — this is the bridge from Stage 1. Two of the four
   merge-time reusables **re-run** here over the shipped source:
   - `gate-sca` → `reusable-sca-osv.yml@77a87549a65c6c978a0e87efe0168ed3517f7ca4`
   - `gate-trivy` → `reusable-trivy.yml@77a87549a65c6c978a0e87efe0168ed3517f7ca4`

   Their SARIF outputs are then fed to the central signing reusable
   `reusable-attest-scan.yml@740cb8efb57af0187f88e9b4f939355b871a5895`, which
   signs each verdict as an in-toto attestation **bound to the source snapshot
   digest**:

   | Attest job | predicate-type | predicate from |
   |---|---|---|
   | `attest-sca` | `https://zircote.github.io/attestations/sca/v1` | `gate-sca.outputs.sarif-{artifact,filename}` |
   | `attest-iac-license` | `https://zircote.github.io/attestations/iac-license/v1` | `gate-trivy.outputs.sarif-{artifact,filename}` |

   SAST (CodeQL) and posture (Scorecard) are **not** re-run here — they are
   repo/source-level and already enforced at merge in `quality-gates.yml`.

6. **`verify`** (fail-closed, **before** the release exists) — downloads the
   `{bin}-{version}-*` artifacts, asserts **exactly 7** are present (5 binaries
   + SBOM document + source tarball; a partial set must never ship), then for
   each:
   - platform binary → verify provenance + SBOM (`--predicate-type https://cyclonedx.org/bom`);
   - source tarball → verify provenance + both gate verdicts, asserting the
     signer with `--signer-workflow zircote/.github/.github/workflows/reusable-attest-scan.yml`
     and the predicate-type URIs above.
7. **`release`** (tag-gated + `publishable == 'true'`) — attaches binaries, the
   SBOM, and `{bin}-{version}-checksums.txt` with auto-generated notes.
   `test` and `audit` (cargo-audit) are `needs` of this job because *a tag is
   untrusted input* — it is not guaranteed to point at a CI-green commit.

A tag therefore publishes **nothing unattested and nothing unverified**.

### 3b. Container image (`pipeline.yml`, dormant in template state)

Gated on `publishable == 'true'` (skipped while `publish = false`). On a
non-PR push/tag:

1. **`docker`** → `release-docker.yml` builds `linux/amd64,linux/arm64` and
   pushes to `ghcr.io/{owner}/{repo}`, outputting the manifest `image-digest`.
2. **`docker-sign`** → central `sign-and-attest.yml@740cb8efb57af0187f88e9b4f939355b871a5895`.
   Under **SLSA Build L3** the signing identity is the *central* workflow, not
   this repo — the isolation boundary is what makes the provenance
   non-falsifiable. It attaches a Cosign signature, SLSA provenance, an SBOM,
   and a Grype vuln report as OCI referrers.
3. **`docker-verify`** → central `verify-attestation.yml@e8f0dbde068cc0701e443e7b8d57ae9917de7da3`,
   a **fail-closed** gate. It verifies against `attestation-repo` (this repo,
   where the build ran) while the Fulcio certificate identity defaults to the
   central signer — so verification asserts *both* where the build ran and who
   signed.
4. **`gate-image` → `attest-container-scan`** — a parallel container-vuln seam:
   `reusable-trivy.yml` scans the image by digest (artifact
   `container-scan-sarif`) and `reusable-attest-scan.yml` signs the verdict
   under predicate-type `https://zircote.github.io/attestations/container-scan/v1`,
   bound to the image digest.

> Container *verification* commands (including the mandatory `--signer-workflow`
> for centrally-signed images) live in
> [`SIGNED-RELEASES.md` § Container Image Attestations](./SIGNED-RELEASES.md#container-image-attestations).
> They are not repeated here.

---

## Stage 4 — crates.io Trusted Publishing (`publish.yml`)

Triggered by a `v*.*.*` tag (dispatch = dry-run). Gated on `publishable`.

1. A **pre-publish gauntlet** runs `cargo fmt --check`, `clippy -D warnings`,
   `test`, `doc`, `cargo deny check`, `cargo package`, and `cargo publish
   --dry-run`. It also asserts the **tag version matches `Cargo.toml`** — a
   mismatch fails *before* the irreversible publish.
2. **Trusted Publishing (OIDC)** — `rust-lang/crates-io-auth-action`
   (`@c6f97d42243bad5fab37ca0427f495c86d5b1a18`, v1.0.4) mints a short-lived
   token from the workflow's OIDC identity. **There is no long-lived registry
   token secret.** This requires the `publish` job to run in the `copilot`
   environment with `id-token: write`.
3. **Registry `.crate` attestation** — after publish, the workflow downloads
   the `.crate` *the registry actually serves* from `static.crates.io`,
   byte-compares its SHA-256 against the locally packaged archive (cargo
   packaging is deterministic for a commit; a mismatch fails loudly), then
   attaches SLSA provenance to the **registry bytes** via
   `actions/attest-build-provenance`. The attestation covers what users
   download, not a local rebuild.

---

## Stage 5 — Homebrew Propagation (`package-homebrew.yml`)

The release in Stage 3 is authored by `github-actions[bot]`, and **bot-authored
release events do not trigger other workflows**. So Homebrew is driven by a
`workflow_run` trigger on **Release** completion (with a `release: published`
trigger and manual dispatch as alternates).

The job only proceeds for a *successful tag run*
(`workflow_run.conclusion == 'success'` and `head_branch` starts with `v`),
resolves `bin`/`description`/`license`/`publishable` from `Cargo.toml`
**at the released tag**, computes the source tarball SHA-256 (failing loudly on
any download error), and generates a source formula
(`class {CamelCase} < Formula`, `cargo install`-based, with a `--version`
smoke test). It pushes to `{owner}/{HOMEBREW_TAP_REPO|homebrew-tap}` using
`HOMEBREW_TAP_TOKEN` (a PAT scoped to the tap repo — this workflow only ever
*reads* the source repo). The push is idempotent: a re-fire for the same
version that produces no diff is a no-op.

---

## How to Adopt This in Your Own Project

> **Diátaxis mode: How-to.** Numbered, task-oriented, with a verification
> command at the end.

1. **Arm the channels.** Delete this one line from `Cargo.toml`:

   ```toml
   publish = false
   ```

   This single deletion arms the container chain, crates.io publish, GitHub
   Release creation, and Homebrew — all four resolve it from `cargo metadata`.

2. **Set crate identity.** Edit `Cargo.toml` `name`, `version`, `description`,
   `license`, `repository`, and the `[[bin]]` name. Every release workflow is
   var-driven from this metadata plus the GitHub context — **no workflow file
   is renamed.**

3. **Configure crates.io Trusted Publishing (one-time, per crate).** On
   crates.io → your crate → **Settings → Trusted Publishing → Add**:
   - Repository: `{owner}/{repo}`
   - Workflow filename: `publish.yml`
   - Environment: `copilot`

   No registry token is stored anywhere; the OIDC exchange replaces it.

4. **Set Homebrew secrets/variables (only if shipping a tap).**
   - Secret `HOMEBREW_TAP_TOKEN` — a PAT with write access to your tap repo
     (also used by `release.yml` so the release event can propagate;
     `workflow_run` is the fallback either way).
   - Variable `HOMEBREW_TAP_REPO` — *optional*; defaults to `homebrew-tap`
     under your owner.

   Set both at **Settings → Secrets and variables → Actions**.

5. **(Optional) Other secrets.** `CODECOV_TOKEN` (coverage upload),
   `GITLEAKS_LICENSE` (secrets scan). Neither is required for attested
   delivery. See the
   [Required Secrets Summary](../template/CI-WORKFLOWS.md#required-secrets-summary).

   > **Note:** no `DEPLOY_DOCS` variable exists in any attested-delivery
   > workflow in this repository; documentation deployment is configured
   > separately in `docs-deploy.yml` and is out of scope for delivery.

6. **Keep the SHA-pinning convention.** Every `uses:` in this repository —
   third-party actions *and* the `zircote/.github` central reusables — is
   pinned to a **full 40-character commit SHA**, never a tag or branch. The
   `pin-check` required check (central
   `pin-check.yml@740cb8efb57af0187f88e9b4f939355b871a5895`) fails the build
   if any reference is not SHA-pinned. Let Dependabot (github-actions
   ecosystem) bump these pins; do not hand-edit them to a moving ref.

7. **Verify.** Cut a dry-run before a real tag — dispatch **Release** and
   **Publish to crates.io** from any branch; both exercise the full
   build → attest → verify chain and tag-gate only the publish step:

   ```bash
   gh workflow run release.yml
   gh workflow run publish.yml
   gh run watch "$(gh run list --workflow=release.yml --limit=1 --json databaseId -q '.[0].databaseId')"
   ```

   A green dry-run with the `Verify Attestations` job passing confirms the
   pipeline is wired correctly before you publish anything irreversible.

---

## See Also

- [`SIGNED-RELEASES.md`](./SIGNED-RELEASES.md) — verification commands, SLSA
  levels, keyless Sigstore, "who signs what", troubleshooting.
- [`CI-WORKFLOWS.md`](../template/CI-WORKFLOWS.md) — per-workflow reference:
  triggers, inputs, secrets, the full pipeline dependency chain.
- `SECURITY.md` § Verifying Release Artifacts — canonical copy-paste verify
  commands.

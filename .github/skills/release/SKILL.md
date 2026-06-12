---
name: release
argument-hint: v<X.Y.Z> | patch | minor | major
description: >-
  Orchestrate and monitor a full attested release of this project
  end-to-end: release-prep PR, tag, attested binaries + SBOM, crates.io
  Trusted Publishing with .crate attestation, Homebrew propagation, and
  independent workstation verification. Use this skill whenever the user
  invokes /release v<n.n.n> or /release patch|minor|major, or says "cut a
  release", "ship a release", "release version X", "bump and release",
  "do a patch/minor/major release", "tag a new version", or anything else
  that means publishing a new version of this project. Do not improvise
  the release process from memory — this skill encodes hard-won fixes for
  failure modes that are invisible until a release breaks.
---

# Release Orchestration

Run a complete attested release for this repository. The argument is
either an explicit version (`/release v1.4.0`) or a bump type
(`/release patch|minor|major`), which computes the target from the
current `Cargo.toml` version. Every phase below ends with a verification;
do not proceed past a failure — fix it or stop and report.

Project names are never hardcoded: resolve the crate/binary name from
`cargo metadata --no-deps` and the `owner/repo` from
`gh repo view --json nameWithOwner` at the start, and use those values
everywhere `<bin>` and `<owner>/<repo>` appear below.

The pipeline this skill drives (all already wired in `.github/workflows/`):

```
prep PR ──merge──> tag push ──┬─> release.yml  (test + audit gates → 5 platform
                              │    binaries → provenance + SBOM attestations →
                              │    fail-closed verify → GitHub Release)
                              ├─> publish.yml  (pre-publish checks → crates.io
                              │    Trusted Publishing → download registry .crate
                              │    → sha256 match → attest)
                              └─> pipeline.yml (container: build → central
                                   sign-and-attest → fail-closed verify)
release.yml completion ─workflow_run─> package-homebrew.yml (formula update)
```

## Help / no argument

If invoked with no argument, `--help`, or `help`, print this and stop —
do not start a release:

```
/release — attested release orchestration

USAGE
    /release v<X.Y.Z>     release an explicit version (e.g. /release v1.4.0)
    /release patch        bump X.Y.Z -> X.Y.(Z+1) and release
    /release minor        bump X.Y.Z -> X.(Y+1).0 and release
    /release major        bump X.Y.Z -> (X+1).0.0 and release

WHAT IT DOES
    prep PR (version locations) -> required-checks green -> squash merge
    -> annotated tag -> monitors: attested binaries + SBOM + fail-closed
    verify -> GitHub Release; crates.io Trusted Publishing + .crate
    attestation; attested container image; Homebrew auto-update ->
    independent workstation verification of every artifact.

NOTES
    - Publishing to crates.io is irreversible; versions are immutable.
    - CHANGELOG [Unreleased] must have content; empty means stop and ask.
    - Never re-runs release.yml against an existing tag (asset immutability).
```

## Phase 0 — Preflight

0. **Publication gate** — this project ships from rust-template, where
   all publication channels are disabled by `publish = false` in
   Cargo.toml (the workflows read it via `cargo metadata`; release
   creation, crates.io publishing, and Homebrew updates all skip while
   it is set). Check it first:
   ```bash
   cargo metadata --no-deps --format-version 1 \
     | jq -r 'if .packages[0].publish == [] then "DISABLED" else "ENABLED" end'
   ```
   If DISABLED, stop and tell the user: releasing requires deleting the
   `publish = false` line (and its comment block) from Cargo.toml. For
   the template repository itself this is by design — do not release.
1. `git checkout main && git pull`; working tree must be clean of tracked
   changes (untracked noise is fine).
2. Resolve the target version from the argument:
   - `v<major>.<minor>.<patch>` → use as given.
   - `patch` | `minor` | `major` → read `version` from `Cargo.toml` on
     freshly-pulled main and bump that component, zeroing the lower ones
     (`1.3.1` + `minor` → `1.4.0`; `1.3.1` + `major` → `2.0.0`).
   - Anything else → stop and ask.
   Strip the `v` for file edits; keep it for the tag. State the resolved
   version in the first progress message so a wrong bump is caught early.
3. Sanity checks, all hard stops:
   - New version is greater than `version` in `Cargo.toml`.
   - Tag does not already exist (`git tag -l v<X.Y.Z>`, and check remote).
   - `CHANGELOG.md` has content under `## [Unreleased]` — a release with
     an empty changelog means something is off; ask the user what this
     release contains.
   - Latest pipeline run on main is green
     (`gh run list --workflow=pipeline.yml --branch main --limit 1`).
4. Semver gut-check: if Unreleased contains breaking changes or an MSRV
   bump and the user asked for a patch, raise it before proceeding.

## Phase 1 — Release prep

Branch `release/v<X.Y.Z>` off main, then update **all** version
locations (missing one ships inconsistent metadata):

| File | What to change |
| --- | --- |
| `Cargo.toml` | `version = "<X.Y.Z>"` |
| `Cargo.lock` | run `cargo check` after the Toml edit — never hand-edit |
| `CHANGELOG.md` | insert `## [<X.Y.Z>] - <today>` under `## [Unreleased]`; update the `[Unreleased]:` compare link and add the new version's compare link at the bottom |
| `SECURITY.md` | any `<bin>-<version>-<platform>` example versions |
| `CITATION.cff` | if present: `version:` and `date-released:` — every occurrence |

Validate locally before the PR: `cargo fmt -- --check` and `cargo check`
minimum. The PR's CI and the release workflow's own gates run the full
suite, so don't duplicate the entire gauntlet here — but a broken lockfile
or fmt failure should never reach the PR.

Commit as `chore(release): v<X.Y.Z>`, push, open the PR. The body should
list what the release contains and note anything operator-relevant.

## Phase 2 — PR through merge

Required status checks on main are `CI Checks / All Checks Pass` and
`pin-check / pin-check`. Monitor the PR with the Monitor tool — and use
the aggregate-gate guard, not a naive all-non-pending check:

```bash
# Terminal only when the aggregate gate check itself has reported and
# nothing is pending. Right after a push there is a window where only 1-2
# checks are registered; "zero pending" alone declares victory in that
# window (this produced a false ALL GREEN once).
gate=$(jq -r '[.[] | select(.name=="CI Checks / All Checks Pass")][0].bucket // "absent"' <<<"$checks")
```

When green, merge with `gh pr merge --squash --delete-branch`, then
`git checkout main && git pull` and confirm HEAD is the release commit.

## Phase 3 — Tag

Annotated tag on the merge commit, then push:

```bash
git tag -a v<X.Y.Z> -m "Release v<X.Y.Z>

<one-paragraph summary from the changelog>" <merge-sha>
git push origin v<X.Y.Z>
```

The tag push is the release trigger. Facts that matter here:

- Tag pushes bypass branch protection — release.yml carries its own test
  and audit gates precisely because the tag is untrusted input.
- **Never re-dispatch release.yml against an existing tag.** Builds are
  not reproducible; it would overwrite published release assets with
  different bytes, violating the immutability the attestations exist to
  protect.

## Phase 4 — Monitor the chains

Four things run; watch all of them with the Monitor tool (one monitor,
multiple conditions — report each as it lands):

1. **Release run** (`release.yml`). Expect: Resolve Project Metadata,
   Test, Cargo Audit, 5 × Build, SBOM (generate + attest), Verify
   Attestations, Create Release — all success.
2. **Publish run** (`publish.yml`). Expect: pre-publish checks, Trusted
   Publishing auth, `cargo publish`, then the crate-attestation steps
   ("Download published crate from registry" and "Attest crate
   provenance"). Report these step conclusions explicitly.
3. **Pipeline run** (`pipeline.yml`, container chain on the tag).
   Expect: Docker build/push, Sign and Attest Image (central signer),
   Verify Image Attestations — all success.
4. **Homebrew run** (`package-homebrew.yml`) must appear **on its own**
   after the Release run completes, via `workflow_run`. If no run appears
   within a few minutes of Release success, the trigger regressed — fall
   back to manual dispatch
   (`gh workflow run package-homebrew.yml -f version=<X.Y.Z> -f dry_run=false`)
   and investigate.

### Failure playbook

| Symptom | Cause | Action |
| --- | --- | --- |
| Publish auth fails: "No Trusted Publishing config found" | crates.io TP not configured | One-time setup on crates.io: crate → Settings → Trusted Publishing → repo `<owner>/<repo>`, workflow filename `publish.yml`, environment `copilot`. Then `gh workflow run publish.yml --ref v<X.Y.Z>` (dispatch-on-tag makes `github.ref` the tag, so the tag-gated steps run). |
| Publish fails: "crate <name>@X.Y.Z already exists" | Duplicate publish attempt raced a successful one | Benign. Verify the version is live (`cargo search <name>`), report, move on. crates.io versions are immutable. |
| Crate download step exhausts retries | static.crates.io CDN propagation | Re-run the failed job; the publish itself succeeded. |
| Crate sha256 mismatch (registry vs local package) | Should never happen — cargo packaging is deterministic per commit | Hard stop. Do not attest. Investigate before anything else. |
| Cargo Audit job fails | Real advisory in `Cargo.lock` | Fix the dependency (usually `cargo update <crate>`) via a normal PR, then start the release over at Phase 0. Note: cargo-deny may NOT have flagged it — deny analyzes the feature/target graph, audit scans the raw lockfile; an unreachable phantom lock entry trips audit only. Both gates are intentional; keep both. |
| A build leg fails | Platform/toolchain issue | The five legs are linux-amd64, linux-arm64 (`ubuntu-24.04-arm`), macos-arm64, macos-amd64 (cross-target on macos-latest), windows-amd64. Binaries build with **default features** (matches `cargo install`). |
| Release event didn't trigger Homebrew | Releases are authored by `github-actions[bot]`; bot events don't trigger workflows | The `workflow_run` trigger handles this; `head_branch` in the workflow_run payload IS the tag name for tag-triggered runs (verified empirically — and the payload has no `ref` field, whatever a reviewer may claim). |
| Image verify fails on the tag run | Central signer/verify regression | Check the central repo pin in pipeline.yml and `references/platform-constraints.md` of the attested-delivery skill before anything else. |

## Phase 5 — Independent workstation verification

In-pipeline success is necessary; this is the acceptance test. Run from
the local machine, in a scratch dir:

```bash
gh release download v<X.Y.Z> --repo <owner>/<repo>
# Expect 7 assets: 5 binaries, <bin>-<X.Y.Z>-sbom.cdx.json, checksums.txt

for f in <bin>-<X.Y.Z>-{linux-amd64,linux-arm64,macos-arm64,macos-amd64,windows-amd64.exe}; do
  gh attestation verify "$f" --repo <owner>/<repo>                  # provenance
  gh attestation verify "$f" --repo <owner>/<repo> \
    --predicate-type https://cyclonedx.org/bom                      # SBOM binding
done
shasum -a 256 -c <bin>-<X.Y.Z>-checksums.txt

# crates.io: needs a User-Agent or the API/CDN rejects silently
curl -fsSL -A 'release-check' \
  -O https://static.crates.io/crates/<name>/<name>-<X.Y.Z>.crate
gh attestation verify <name>-<X.Y.Z>.crate --repo <owner>/<repo>

# Container image (digest from the pipeline run's docker job output):
gh attestation verify "oci://ghcr.io/<owner>/<repo>@<digest>" \
  --repo <owner>/<repo> \
  --signer-workflow <owner>/.github/.github/workflows/sign-and-attest.yml \
  --predicate-type https://slsa.dev/provenance/v1
```

Check **exit codes**, not grepped output — a filtered pipe that matches
nothing looks identical to success (this mistake was made once; silence
is not success). No `--signer-workflow` flag for binaries and crates:
those are attested by this repo's own workflows. The container image DOES
need `--signer-workflow`: it is signed by the central workflow.

Confirm crates.io shows the version:
`curl -s -A 'release-check' https://crates.io/api/v1/crates/<name> | jq .crate.max_version`

## Final report

Summarize for the user: version, merge commit, tag; per-channel status
(GitHub Release / crates.io / container image / Homebrew); workstation
verification results; and anything from the failure playbook that fired.
If any channel is incomplete, say exactly what is pending and what
unblocks it.

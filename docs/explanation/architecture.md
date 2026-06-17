---
diataxis_type: explanation
---
# Template Architecture & Design Rationale

This document explains *why* `rust_template` is built the way it is. It is the
single place to understand the design trade-offs behind the source layout, the
CI orchestration model, the supply-chain story, and the library conventions. It
deliberately contains no step-by-step instructions — for those, follow the
how-to guides linked throughout.

For the formal, dated decisions behind two of these choices, see the
[Architectural Decision Records](../adr/README.md): in particular
[ADR-0001 (use ADRs)](../adr/0001-use-architectural-decision-records.md) and
[ADR-0002 (documentation directory structure)](../adr/0002-documentation-directory-structure.md).

## Why `crates/` instead of `src/`

The library root is `crates/lib.rs` and the binary entry point is
`crates/main.rs`, wired up explicitly through the `[lib]` and `[[bin]]` `path`
keys in `Cargo.toml`. This is a deliberate departure from the conventional
`src/` layout.

The reason is that this is a *template*, not an ordinary crate. Using `crates/`
signals at a glance that the directory holds example scaffolding meant to be
restructured, not production code to be preserved verbatim. A developer who
clicks "Use this template" sees an unfamiliar path and is nudged to make a
conscious decision about structure, rather than inheriting a layout by inertia.

The trade-off is real but small: some tooling and muscle memory assume `src/`,
so the `path` keys must be set explicitly and downstream projects that prefer
the convention have one rename to perform. We accept that friction because the
"this is scaffolding" signal is worth more in a template than convention
adherence is.

## The CI orchestration model

CI is a single orchestrator, `pipeline.yml`, that calls focused reusable
workflows rather than one monolithic file. The trade-off is intentional: a thin
orchestrator with named child workflows is easier to reason about, reuse, and
pin than a single file with dozens of inlined jobs.

`pipeline.yml` runs on every push, pull request, and tag, and coordinates these
jobs:

- `gate` — resolves a single boolean, `publishable`, from `cargo metadata`. This
  is the publish gate described below, and every publication-bearing job depends
  on it.
- `ci` — calls `ci-checks.yml` (fmt, clippy, multi-OS test, doc build,
  cargo-deny, MSRV check, and an `all-checks-pass` aggregator).
- `coverage` — calls `ci-coverage.yml` (LCOV to Codecov), in parallel with `ci`.
- `test-matrix` — calls `ci-test-matrix.yml` (the broad feature/version matrix),
  on pull requests.
- `pin-check` — calls the central `zircote/.github` workflow that asserts every
  `uses:` reference is pinned to a full 40-character commit SHA.
- `docker` → `docker-sign` → `docker-verify`, plus `gate-image` and
  `attest-container-scan` — the container build, sign, and fail-closed verify
  chain.

Releases are deliberately **not** part of this orchestrator. Tag-triggered
publication runs through flat, independent workflows — `release.yml`,
`publish.yml`, and `package-homebrew.yml` — each triggered by a tag (or a
dispatch dry-run) and owning one channel. The rationale is blast-radius
isolation: a failure in the Homebrew step must not be entangled with the
crates.io publish or the GitHub Release. Flat, single-purpose release workflows
fail independently and are re-runnable in isolation. This mirrors the verified
reference architecture in `zircote/rlm-rs`.

A second design rule keeps the template instantiable with almost no edits:
**project specificity is var-driven.** The release workflows resolve the crate
name, binary name, version, description, and license from `cargo metadata` at
runtime, and the owner/repo from the GitHub context. Nothing in the workflow
files is renamed when you adopt the template — you edit `Cargo.toml` and,
optionally, one repository variable. Hard-coding those values into the workflows
would have been simpler to write but would have turned every template adoption
into a find-and-replace chore and a source of drift.

## The `publish = false` gate pattern

A template should ship *no* published artifacts — no container on a registry, no
crate on crates.io, no GitHub Release, no Homebrew formula. But the same
workflows that must stay dormant in template state must arm themselves the
moment a real project adopts them. The mechanism that achieves both from a
single switch is the `publish = false` line in `Cargo.toml`.

`cargo` itself refuses `cargo publish` while that line is present. The workflows
read the same fact independently: the `gate` job runs `cargo metadata` and
resolves `publishable=false` when `.publish == []`. While `publishable` is
false, the Docker build → sign → verify chain is **skipped** (not built and
discarded — a template ships no container), and the crates.io, GitHub Release,
and Homebrew channels are all gated off.

Deleting that one line in a downstream project flips `publishable` to true and
arms all four channels at once. The trade-off is that the gate is a single point
of control — a downstream project that wants, say, container images but not
crates.io must add finer-grained conditions. We accept that, because the
overwhelmingly common need is "off in the template, all-on in real projects,"
and one line serving that case beats four independent toggles a new adopter
would have to discover.

## Attested delivery, conceptually

The guiding principle of the release machinery is **nothing publishes
unverified**. Every release artifact carries cryptographic attestations, and a
fail-closed verification job runs *before* the artifact becomes visible — the
GitHub Release does not exist until verification passes.

Three kinds of evidence are attached:

- **SLSA provenance** — binds each artifact to the exact commit, workflow, and
  run that produced it, signed keyless through Sigstore (no private keys to
  manage or rotate).
- **SBOM** — a CycloneDX Software Bill of Materials, generated and attested over
  every binary, so consumers can audit the dependency set behind a release.
- **Gate attestations** — for container images, the central
  `zircote/.github` signer workflow attests under SLSA Build L3, which means the
  signing identity is the *central* workflow, not this repository. Verification
  therefore asserts both where the build ran and who signed.

The non-negotiable property is **fail-closed verification**: a tag that produces
an artifact failing verification publishes nothing. In-pipeline success is
necessary but not sufficient; consumers are expected to verify on their side
too. This is a conceptual overview — for the full attestation chain, who signs
what, and the keyless-signing rationale, see
[Signed Releases & SLSA Provenance](../security/SIGNED-RELEASES.md). The
copy-paste verification commands live in
[SECURITY.md](../../SECURITY.md#verifying-release-artifacts).

## The lint philosophy: pedantic by default

Clippy runs with the `pedantic`, `nursery`, and `cargo` lint groups enabled, and
a strict deny list turns several patterns into hard errors: `unwrap_used`,
`expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro`, and the print
macros. `unsafe` code is forbidden outright.

The rationale is that a template sets the ceiling for the projects built from it.
Strictness is far cheaper to relax than to retrofit: a downstream project that
finds pedantic lints noisy can downgrade specific lints in minutes, whereas a
project that started permissive and later wants rigor faces a large remediation.
Denying `unwrap_used` and `panic` in particular forces library code to handle
errors explicitly and push failures to the API boundary, where callers — not the
library — decide what to do. Test code is exempt from these denials (via
`clippy.toml`), because the cost/benefit inverts in tests, where `unwrap` is
legible and the blast radius is contained.

## Why `thiserror` and consuming-self builders

Two library conventions in `crates/lib.rs` are worth their own rationale.

**`thiserror` for the error type.** The crate's `Error` enum derives
`thiserror::Error`, and a crate-level alias `Result<T>` reduces boilerplate
across the API. `thiserror` generates the `Display` and `From` implementations
from attributes at zero runtime cost, keeping error definitions concise and
consistent. The alternative — hand-writing `std::error::Error` impls, or
reaching for a dynamic-error crate like `anyhow` — is either more boilerplate or
less precise. `anyhow` suits applications that only need to report errors;
`thiserror` suits a *library* that wants callers to be able to match on
structured variants. This template is library-first, so `thiserror` is the
right default.

**Consuming-self builders.** `Config` uses builder methods shaped
`fn with_field(mut self, value: T) -> Self` rather than `&mut self`. Three
benefits follow. First, `const` evaluation: a `const fn` is compatible with owned
`self` but not with `&mut self`, so the consuming form lets the entire builder be
`const`. Second, chaining reads naturally —
`Config::new().with_a(1).with_b(2)`. Third, move semantics avoid hidden shared
state, since the builder is consumed at each step. The cost is that the builder
cannot be reused after a method call without cloning, but configuration builders
are almost always used once, so that cost rarely bites.

## Related reading

- [ADR-0001 — Use Architectural Decision Records](../adr/0001-use-architectural-decision-records.md)
- [ADR-0002 — Documentation Directory Structure](../adr/0002-documentation-directory-structure.md)
- [Signed Releases & SLSA Provenance](../security/SIGNED-RELEASES.md) — the attestation chain in depth
- [Deployment Guide](../DEPLOYMENT.md) — how to actually cut a release
- The project `CLAUDE.md` Reference and Explanation sections — the authoritative source for lint tables, cargo profiles, and conventions

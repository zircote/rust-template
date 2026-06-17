# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **GitHub template repository** for Rust crates. The crate name is `rust_template` (Rust edition 2024, MSRV 1.92). It ships both a library (`crates/lib.rs`) and a binary (`crates/main.rs`). Source lives in `crates/`, not the standard `src/` directory.

---

## Documentation Standard: Diátaxis

All documentation in this project follows the [Diátaxis framework](https://diataxis.fr/). When adding or updating documentation — including this file, `docs/`, doc comments, and README files — classify content into one of four modes:

| Mode | Purpose | Prompt | Example |
|---|---|---|---|
| **How-to** | Task-oriented steps | "How do I…?" | Adding a new error variant, running tests |
| **Reference** | Precise, factual lookup | "What is…?" | Lint tables, cargo profiles, API signatures |
| **Explanation** | Design rationale | "Why does…?" | Why `thiserror`, why `panic = "abort"` |
| **Tutorial** | Learning-oriented walkthrough | "Teach me…" | (Not used in CLAUDE.md; use `docs/` for tutorials) |

**Rules for contributors (human and AI):**

- Before writing documentation, decide which Diátaxis mode it belongs to. Do not mix modes in a single section.
- **How-to** sections use numbered steps and end with a verification command.
- **Reference** sections use tables or structured lists. No rationale — just facts.
- **Explanation** sections use "Why X" headings and focus on trade-offs and decisions.
- New `docs/` files must declare their Diátaxis mode in a frontmatter comment or heading.
- When extending this CLAUDE.md, place new content under the correct Diátaxis heading below.

---

<!-- Diátaxis: How-to Guides — task-oriented, practical steps -->

## How-to Guides

### Build and Run

[`just`](https://github.com/casey/just) is the local task runner. Run `just` to list all recipes.

```bash
just                  # List all recipes
just check            # Full CI check (fmt + clippy + test + doc + deny)
just build            # Debug build
just build-release    # Release build
just run              # Run the binary
just template-sync    # Sync shared tooling from rust-template upstream
```

<details>
<summary>Raw cargo equivalents</summary>

```bash
cargo build                                              # Build
cargo test --all-features                                # Run all tests
cargo test test_name                                     # Run specific test
cargo test -- --nocapture                                # Run tests with stdout
cargo clippy --all-targets --all-features -- -D warnings # Lint (CI uses -D warnings)
cargo fmt                                                # Format
cargo fmt -- --check                                     # Check formatting
cargo deny check                                         # Supply chain audit
cargo doc --no-deps --all-features                       # Build docs
cargo +nightly miri test                                 # UB detection

# Full CI check (run before pushing)
cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test && cargo doc --no-deps && cargo deny check
```

</details>

### Run Tests

```bash
just test             # All tests (unit + integration + doc)
just test-verbose     # Tests with stdout visible
just test-single NAME # Single test by name
just coverage         # LCOV coverage report
just coverage-html    # HTML coverage report
just msrv             # Check against MSRV 1.92
just miri             # Miri undefined behavior detection
just mutants          # Mutation testing
```

### Lint and Format

```bash
just fmt              # Format code
just fmt-check        # Check formatting (no modify)
just lint             # Clippy with CI-equivalent flags
just lint-fix         # Clippy auto-fix
just deny             # Supply chain audit
just audit            # Advisory database check
```

### Add a New Public Function

1. Add the function in `crates/lib.rs` (or a module under `crates/`).
2. Annotate with `#[must_use]` if it returns a value without side effects.
3. Use `const fn` if the body permits.
4. Write a doc comment with `# Arguments`, `# Returns`, `# Errors` (if fallible), and `# Examples`.
5. Add a unit test in the `#[cfg(test)] mod tests` block within the same file.
6. Add an integration test in `tests/integration_test.rs`.
7. Run `just check` before committing.

### Add a New Error Variant

1. Add the variant to the `Error` enum in `crates/lib.rs`.
2. Include a `#[error("...")]` format string with meaningful context.
3. Prefer structured variants (named fields) over tuple variants when there are multiple pieces of context.
4. Add a display test in the `test_error_display` test.

### Add a Builder Field to Config

1. Add the field to the `Config` struct with a doc comment.
2. Set a sensible default in `Config::new()`.
3. Add a `with_<field>(mut self, value: T) -> Self` method marked `#[must_use]` and `const fn`.
4. Add a test case in `test_config_builder` and `test_config_default`.

---

<!-- Diátaxis: Reference — precise, factual, information-oriented -->

## Reference

### Source Layout

| Path | Purpose |
|---|---|
| `crates/lib.rs` | Library root: `Error` (thiserror), `Result<T>`, `Config` (builder), `add()`, `divide()` |
| `crates/main.rs` | Binary entry point: `main() -> ExitCode`, delegates to `run() -> Result` |
| `tests/integration_test.rs` | Integration tests including property-based tests (proptest) |
| `clippy.toml` | Clippy thresholds and test-mode exemptions |
| `rustfmt.toml` | Formatter settings (stable options active, nightly options commented) |
| `deny.toml` | Supply chain policy: licenses, bans, source restrictions |
| `justfile` | Local task runner recipes (CI parity) |

### Error Handling

- **Crate error type**: `Error` enum derived with `thiserror::Error`.
- **Result alias**: `pub type Result<T> = std::result::Result<T, Error>`.
- **Propagation**: use `?` operator. Never `unwrap()`, `expect()`, or `panic!()` in library code.
- **Binary**: `main()` returns `ExitCode`; delegates to `run() -> Result`. On `Err`, the binary renders the error to stderr in the format selected by `--format` / TTY (below).

#### Dual-Consumer Error Output (RFC 9457)

The crate emits errors for two consumers from one `Error` value: the human (the `thiserror` `Display`, unchanged) and the LLM agent (a serializable `application/problem+json` envelope). The envelope type is `ProblemDetails` in `crates/problem.rs`, re-exported from the crate root alongside `Applicability`, `CodeAction`, `SuggestedFix`, and `OutputFormat`. Map any error with `Error::to_problem()`; render for a format with `Error::render(OutputFormat)`.

**Envelope members** (`ProblemDetails`):

| Field | RFC 9457 role | Notes |
|---|---|---|
| `type` | standard | Stable, version-embedded URI (`.../v1`). |
| `title` | standard | Short summary, stable per `type`. |
| `status` | standard | Numeric status class. |
| `detail` | standard | This-occurrence text; equals the `Display` string. |
| `instance` | standard | `urn:` occurrence reference. |
| `retry_after` | agent extension | Delta-seconds, or `null` (serialized) on non-transient errors. |
| `suggested_fix` | agent extension | `{ description, applicability }`, or `null`. |
| `code_actions` | agent extension | Array of LSP-`CodeAction`-shaped `{ title, kind, applicability }`. |
| `exit_code` | optional extension | Process exit code; omitted from JSON when absent. |

**Applicability markers** (on every `suggested_fix` and `code_action`): `machine_applicable` (auto-apply), `maybe_incorrect` (escalate to human), `has_placeholders` (fill slots first), `unspecified` (default; treat as `maybe_incorrect`).

**Type URIs** (one per variant, distinct, versioned): `InvalidInput` → `https://zircote.com/errors/invalid-input/v1`; `OperationFailed` → `https://zircote.com/errors/operation-failed/v1`. A breaking change ships a new `/v2` URI rather than redefining `/v1`.

**Format selection** (`OutputFormat::select(explicit, is_terminal)`): JSON when `--format=json` or (no flag and stderr is not a TTY); pretty otherwise. Pretty output is byte-identical to the historical `Error: {e}` line.

For the rationale, see the **Dual-Consumer Error Output** explanation doc (`docs/explanation/error-architecture.md`).

### Ownership and Borrowing

- Prefer `&str` over `String` in function parameters.
- Prefer `&[T]` over `Vec<T>` in function parameters.
- Use `Cow<'_, str>` when a function may or may not allocate.
- Pass large structs by reference; pass `Copy` types by value.
- Avoid unnecessary `.clone()` — if you need ownership, take owned types in the signature.

### Type Design

- Use newtypes to enforce domain invariants (e.g., `struct Port(u16)` over bare `u16`).
- Derive `Debug` on all types. Derive `Clone`, `PartialEq`, `Eq`, `Hash` when semantically correct.
- Use `#[non_exhaustive]` on public enums and structs that may grow.
- Prefer `enum` for closed sets, `trait` for open extension.

### Builder Pattern

This project uses consuming-self builders with `const fn`:

```rust
#[must_use]
pub const fn with_field(mut self, value: T) -> Self {
    self.field = value;
    self
}
```

- `Config::new()` is `const fn` and `#[must_use]`.
- `Default` impl delegates to `new()`.
- Every builder method is `const fn` and `#[must_use]`.

### Const and Must-Use Annotations

- `#[must_use]` on all pure functions that return a value.
- `const fn` wherever the compiler allows it.
- Both annotations on builder methods.

### Lint Configuration

Clippy runs with **pedantic + nursery + cargo** lint groups. All are set to `warn` with priority -1.

**Denied lints** (hard errors):

| Lint | Reason |
|---|---|
| `unwrap_used` | Use `?` or explicit match |
| `expect_used` | Use `?` or explicit match |
| `panic` | Return errors instead |
| `todo` | No placeholder code |
| `unimplemented` | No placeholder code |
| `dbg_macro` | No debug prints in production |
| `print_stdout` | Use logging; binary exempts itself with `#[allow]` |
| `print_stderr` | Use logging; binary exempts itself with `#[allow]` |

**Allowed lints**:

| Lint | Reason |
|---|---|
| `missing_errors_doc` | Opt-in documentation |
| `missing_panics_doc` | Opt-in documentation |
| `module_name_repetitions` | Common in Rust API design |
| `must_use_candidate` | Applied manually where meaningful |
| `redundant_pub_crate` | Allow `pub(crate)` for clarity |

**Clippy thresholds** (from `clippy.toml`):

| Threshold | Value |
|---|---|
| `too-many-lines-threshold` | 100 |
| `too-many-arguments-threshold` | 7 |
| `cognitive-complexity-threshold` | 25 |
| `excessive-nesting-threshold` | 4 |
| `max-struct-bools` | 3 |
| `max-fn-params-bools` | 3 |
| `pass-by-value-size-limit` | 256 bytes |
| `type-complexity-threshold` | 250 |

**Test exemptions**: `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-dbg-in-tests`, `allow-print-in-tests` are all `true`.

### Formatting

Configured in `rustfmt.toml` (stable options active):

| Setting | Value |
|---|---|
| `max_width` | 100 |
| `edition` | 2024 |
| `tab_spaces` | 4 |
| `hard_tabs` | false |
| `use_field_init_shorthand` | true |
| `reorder_imports` | true |
| `reorder_modules` | true |
| `newline_style` | Unix |
| `match_block_trailing_comma` | true |

Nightly-only options (`imports_granularity`, `group_imports`, `trailing_comma`, `brace_style`, etc.) are commented out but documented for when nightly is used.

### Import Ordering

Group imports in this order, separated by blank lines:

1. `std` / `core` / `alloc`
2. External crates
3. `crate` / `super` / `self`

Within each group, alphabetical order (enforced by `reorder_imports = true`).

### Doc Comments

All public items require doc comments. Structure:

```rust
/// Brief one-line summary.
///
/// Extended description (optional, for complex items).
///
/// # Arguments
///
/// * `param` - Description.
///
/// # Returns
///
/// What this function returns.
///
/// # Errors
///
/// When and why this function returns an error (required for fallible functions).
///
/// # Examples
///
/// ```rust
/// use rust_template::my_function;
///
/// let result = my_function(42);
/// assert_eq!(result, 42);
/// ```
```

- Doc examples must compile (`cargo test` runs them as doctests).
- Use `#![doc = include_str!("../README.md")]` at the crate root to pull in README as crate docs.

### Unsafe Code

`unsafe` code is **forbidden** (`unsafe_code = "forbid"` in `[lints.rust]`). No exceptions.

### Supply Chain Security

`deny.toml` enforces:

- **Licenses**: only permissive (MIT, Apache-2.0, BSD-2/3, ISC, Zlib, MPL-2.0, Unicode, CC0, BSL-1.0, 0BSD).
- **Sources**: crates.io only; unknown registries and git sources denied.
- **Bans**: `openssl` (use `rustls`), `atty` (use `std::io::IsTerminal`).
- **Advisories**: all advisory types (vulnerability, unmaintained, unsound, notice, yanked) denied.
- **Wildcards**: wildcard version requirements denied.

### Testing

| Test type | Location | Crate |
|---|---|---|
| Unit tests | `#[cfg(test)] mod tests` inside source files | — |
| Integration tests | `tests/integration_test.rs` | — |
| Property tests | `tests/integration_test.rs::property_tests` | `proptest` |
| Parameterized tests | anywhere, via `#[test_case]` | `test-case` |
| Doc tests | `///` examples on public items | — |

**Code coverage requirement**: 90% minimum. Run `just coverage` to generate an LCOV report and verify. CI enforces this threshold via Codecov.

**Property test pattern** (proptest):

```rust
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn my_property(input in any::<i32>()) {
            prop_assert!(some_invariant(input));
        }
    }
}
```

### CI/CD

CI and the container chain run through `pipeline.yml`; releases run through flat, independent tag-triggered workflows (`release.yml`, `publish.yml`, `package-homebrew.yml`) — the same architecture as `zircote/rlm-rs`, the verified reference.

**Project specificity is var-driven**: the release workflows resolve the crate name, binary name, version, description, and license from `cargo metadata` at runtime, and owner/repo from the GitHub context. Instantiating the template requires editing only `Cargo.toml` (plus optional repo variable `HOMEBREW_TAP_REPO`, default `homebrew-tap`, and optional secret `HOMEBREW_TAP_TOKEN`). Nothing in the workflow files is renamed.

**Publication is disabled in the template**: `publish = false` in Cargo.toml gates the container build, crates.io publishing, GitHub Release creation, and Homebrew tap updates (the workflows read it via `cargo metadata`; cargo itself also refuses `cargo publish`). In template state the `pipeline.yml` `gate` job resolves `publishable=false`, so the Docker build → sign → verify chain is **skipped** rather than built — a template ships no container. Deleting that one line in a downstream project arms all four channels.

**CI stage** (`ci-checks.yml`): fmt, clippy, test (Linux/macOS/Windows), doc build, cargo-deny, MSRV check (1.92), `all-checks-pass` gate. Runs in parallel with `ci-coverage.yml` (LCOV/Codecov), `ci-test-matrix.yml` (12-combo matrix, PR only), and `pin-check` (central `zircote/.github` workflow asserting every `uses:` is pinned to a full commit SHA).

**Docker** (`release-docker.yml`): multi-platform build after CI passes, **gated on `publish` (skipped in template state)**. PR = build-only; push on main/tags. Pushed images flow through `docker-sign` (centralized `zircote/.github` `sign-and-attest.yml`, pinned by full SHA — under SLSA Build L3 the signing identity is the central workflow, not this repo) and `docker-verify` (fail-closed attestation verification).

**Release** (`release.yml`, tags + dispatch dry-run): resolve metadata → 5-platform build matrix with per-binary SLSA provenance attested at build time (`{bin}-{version}-{platform}` naming) → test + cargo-audit gates (tags are untrusted input) → CycloneDX SBOM generated and attested over every binary → **fail-closed `gh attestation verify` before the release exists** → tag-gated GitHub Release with checksums. A tag publishes nothing unattested.

**Publish** (`publish.yml`, tags + dispatch dry-run): pre-publish gauntlet → crates.io **Trusted Publishing** (OIDC, no long-lived token; one-time crates.io setup: workflow `publish.yml`, environment `copilot`) → download the registry-served `.crate`, byte-compare against the local package, attest the registry bytes.

**Homebrew** (`package-homebrew.yml`): `workflow_run` on Release completion (bot-authored release events don't trigger workflows) → source formula generated from Cargo.toml metadata into `{owner}/homebrew-tap`.

Releases are orchestrated by the `/release` skill (`.claude/skills/release/`). Artifact verification commands live in `SECURITY.md` § Verifying Release Artifacts.

See `docs/template/CI-WORKFLOWS.md` for the full reference.

### Cargo Profiles

| Profile | Optimization | LTO | Codegen Units | Panic | Strip | Debug |
|---|---|---|---|---|---|---|
| `dev` | 0 | off | default | unwind | no | 1 (line tables) |
| `release` | 3 | thin | 1 | abort | yes | no |
| `release-debug` | 3 | thin | 1 | abort | no | full |

---

<!-- Diátaxis: Explanation — understanding-oriented, design rationale -->

## Explanation

### Why `crates/` Instead of `src/`

This template uses `crates/` as the source directory to distinguish it from the common `src/` layout. This is a template convention — downstream projects may restructure. The `[lib]` and `[[bin]]` paths in `Cargo.toml` point to `crates/lib.rs` and `crates/main.rs`.

### Why `thiserror` for Errors

`thiserror` provides derive macros for `std::error::Error` with zero runtime overhead. It generates `Display` and `From` implementations from attributes, keeping error definitions concise and consistent. The crate-level `Result<T>` alias reduces boilerplate across the API.

### Why Consuming-Self Builders

The builder pattern uses `fn with_field(mut self, ...) -> Self` instead of `&mut self`. This enables:

- **Const evaluation**: `const fn` is compatible with owned self, not `&mut self`.
- **Chaining**: `Config::new().with_a(1).with_b(2)` reads naturally.
- **Move semantics**: no hidden shared state; the builder is consumed on each call.

### Why Pedantic Clippy

Enabling `pedantic`, `nursery`, and `cargo` lint groups catches subtle issues early: missing docs, inefficient patterns, cargo metadata problems. The strict deny list (`unwrap_used`, `panic`, etc.) enforces that library code handles all errors explicitly, pushing failures to the API boundary where callers can make decisions.

### Why `panic = "abort"` in Release

Release builds use `panic = "abort"` to eliminate unwinding tables, reducing binary size. Combined with `strip = true` and `lto = "thin"`, this produces small, fast binaries. The `release-debug` profile inherits these optimizations but preserves debug symbols for profiling.

### Why Ban `openssl` and `atty`

- **`openssl`**: links to a system C library with complex build requirements and CVE history. `rustls` is a pure-Rust TLS implementation with smaller attack surface.
- **`atty`**: unmaintained and unnecessary since Rust 1.70 added `std::io::IsTerminal` to the standard library.

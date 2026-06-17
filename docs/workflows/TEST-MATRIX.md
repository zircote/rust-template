---
diataxis_type: how-to
---
# Test Matrix - Multi-Platform Testing

A comprehensive test matrix validating code across multiple platforms, Rust versions, and feature combinations.

## Reference

| Field | Value |
|---|---|
| Workflow | `.github/workflows/ci-test-matrix.yml` |
| Platforms | Linux, macOS, Windows |
| Rust versions | Stable, Beta, Nightly, MSRV (1.92) |
| Triggers | Pull requests (via pipeline.yml) |

### Operating systems

- **ubuntu-latest** — Linux (primary platform).
- **macos-latest** — macOS (Apple Silicon + Intel).
- **windows-latest** — Windows (x64).

### Rust toolchains

- **stable** — latest stable Rust.
- **beta** — beta channel (upcoming stable).
- **nightly** — nightly builds (experimental features).
- **1.92** — MSRV (Minimum Supported Rust Version).

### Feature combinations

- `--all-features` — all features enabled.
- `--no-default-features` — minimal build.
- Default — standard feature set.

**Total jobs:** ~12-15 (optimized to skip redundant combinations).

### What gets tested

| Scope | Checks |
|---|---|
| All combinations | `cargo build`, `cargo test`, `cargo test --doc` |
| Stable + Ubuntu only | `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo doc --no-deps` |
| Separate jobs | Integration tests (`cargo test --test '*'`), Miri (nightly) |

### Result interpretation

| Result | Meaning | Action |
|---|---|---|
| Success ✅ | All platforms and versions pass | Code is portable and compatible |
| Partial failure ⚠️ | One OS/version fails | Fix the platform-specific issue (path, filesystem, or API difference) |
| MSRV failure ❌ | MSRV job fails, stable passes | Code uses features newer than MSRV — bump MSRV, remove the feature, or gate it |

Example partial failure:

```text
ubuntu-latest / stable: ✅
macos-latest / stable: ❌
windows-latest / stable: ✅
```

Example MSRV failure:

```text
ubuntu-latest / 1.92: ❌
ubuntu-latest / stable: ✅
```

### Miri coverage

Miri (nightly only, on a separate job) detects use-after-free, out-of-bounds memory access, data races in unsafe code, invalid pointer arithmetic, and uninitialized memory reads. It is 100-1000x slower than normal tests and does not support file/network I/O.

## How-to

### Check MSRV locally

Current MSRV: **1.92**.

```bash
# Install the MSRV toolchain
rustup install 1.92

# Test with MSRV
cargo +1.92 check
cargo +1.92 test
```

Verify: both commands succeed under the pinned toolchain.

### Test feature combinations locally

```bash
# Individual features
cargo test --no-default-features --features feature1
cargo test --no-default-features --features feature2
cargo test --features feature1,feature2

# Maximum and minimal builds
cargo test --all-features
cargo test --no-default-features
```

`--all-features` ensures no feature conflicts; `--no-default-features` ensures optional features don't leak into the default set. Verify: every combination compiles and passes.

### Run integration tests

```bash
cargo test --test '*' --verbose
```

Integration tests live in `tests/*.rs` and exercise the public API, not internal units. Verify: all integration tests pass.

### Run Miri locally

```bash
# Install the miri component
rustup +nightly component add miri

# Run miri tests
cargo +nightly miri test

# Run a specific test
cargo +nightly miri test test_name
```

Skip a test that does I/O (unsupported under Miri):

```rust
#[test]
#[cfg_attr(miri, ignore)]  // Skip in miri
fn test_file_io() {
    // File I/O test
}
```

Verify: `cargo +nightly miri test` completes with no UB reports.

### Test multiple platforms locally

```bash
# Cross-compilation with cross
cargo install cross
cross test --target x86_64-pc-windows-gnu
cross test --target x86_64-apple-darwin

# Linux container
docker run --rm -v $(pwd):/app -w /app rust:latest cargo test
```

Verify: tests pass for each target you exercise.

### Write portable code

Avoid the common cross-platform pitfalls:

```rust
// Path separators — use PathBuf, not string formatting
use std::path::PathBuf;
let path = PathBuf::from(dir).join("file.txt");

// Line endings — compare structurally, not byte-for-byte
assert_eq!(content.lines().count(), 2);

// Filesystem case sensitivity — match exact case
let file = File::open("config.toml")?;

// Process signals — use a cross-platform crate instead of `kill`
use subprocess::Popen;
```

Verify: the test matrix passes on all three operating systems.

### Resolve MSRV failures

Common causes and fixes:

```toml
# Edition vs MSRV conflict
[package]
edition = "2024"      # Requires a recent toolchain
rust-version = "1.92" # Keep these consistent
```

```rust
// Newer std APIs may exceed MSRV — use an older API or bump MSRV
let mut buf = Vec::new();
reader.read_buf(&mut buf)?;
```

```toml
# A dependency may require a newer Rust — pin a compatible version or bump MSRV
[dependencies]
serde = "1.0"
```

Diagnostic steps:

1. Check dependencies: `cargo tree --edges no-dev` for version conflicts.
2. Check APIs: search for recently stabilized features.
3. Update or lower MSRV based on the requirement.

Verify: `cargo +1.92 check` succeeds.

### Optimize CI time

Skip redundant combinations:

```yaml
matrix:
  exclude:
    - os: macos-latest
      rust: beta
    - os: windows-latest
      rust: beta
```

Cache dependencies:

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-${{ matrix.rust }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

See all failures rather than stopping at the first:

```yaml
strategy:
  fail-fast: false
```

Verify: total matrix runtime drops while still covering each platform.

### Troubleshooting

**Tests fail only on Windows** — usually path separators (`/` vs `\`), case-insensitive filesystem, a different temp directory, or CRLF vs LF line endings:

```rust
#[cfg(windows)]
#[test]
fn test_windows_specific() {
    println!("Temp dir: {:?}", std::env::temp_dir());
}
```

**Tests fail only on macOS** — usually the case-insensitive but case-preserving filesystem, different system APIs, or Apple-specific security restrictions.

**MSRV test fails** — check dependency versions, check for recently stabilized APIs, then update or lower MSRV.

### Best practices

1. **Test locally first** — use `cross` or Docker before pushing.
2. **Fix MSRV issues early** — don't let them accumulate.
3. **Use `PathBuf`** — always for cross-platform file paths.
4. **Gate platform-specific tests** — `#[cfg(target_os = "linux")]`.
5. **Monitor CI time** — optimize the matrix if jobs run long.

## Why this matters

"It works on my machine" is the failure mode this matrix exists to prevent. Rust compiles per target, and behavior diverges on path separators, filesystem case sensitivity, line endings, and platform APIs — bugs that only surface on an OS the author doesn't run. Pinning the MSRV job alongside stable catches the silent dependency on a too-new language feature before downstream users on the supported floor hit it, and the beta/nightly lanes give early warning of upcoming compiler changes while Miri probes for undefined behavior that ordinary tests pass right over.

## Links

- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [Cross-Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Miri Documentation](https://github.com/rust-lang/miri)
- [GitHub Actions Matrix](https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs)
- [CI Workflows reference](../template/CI-WORKFLOWS.md)

---
diataxis_type: how-to
---
# Code Quality Metrics

Automated collection of code quality metrics — unsafe code detection, binary size analysis, and documentation coverage — emitted as a single report artifact.

## Reference

| Field | Value |
|---|---|
| Workflow | `.github/workflows/code-quality.yml` |
| Tools | `cargo-geiger`, `cargo-bloat`, `rustdoc` |
| Output | Markdown report artifact |

### Metrics collected

| Metric | Tool | What it detects |
|---|---|---|
| Unsafe code analysis | `cargo-geiger` | Unsafe function calls, blocks, trait impls, and unsafe in dependencies |
| Binary size analysis | `cargo-bloat` | Size by crate and by function; bloat sources |
| Documentation coverage | `rustdoc` | Missing doc comments, broken doc links, doc test failures |

### Report access

The workflow generates a combined report. Access it via **Actions → Code Quality Metrics → Artifacts → code-quality-metrics**.

Example report:

```markdown
## Unsafe Code Analysis
Functions  Expressions  Impls  Traits  Methods  Dependency
0/10       0/100        0/5    0/2     0/20     rust_template

## Binary Size Analysis
File   .text   Size    Crate
 71.0%  59.0%   1.2MiB  std
  8.5%   7.1%   147KiB  rust_template

## Documentation Coverage
Documenting rust_template v0.1.0
warning: missing documentation for public function
```

### Interpreting the unsafe code report

```text
Functions  Expressions  Impls  Traits  Methods  Dependency
2/10       5/100        0/5    0/2     0/20     ✓ rust_template
```

- **Functions**: 2 functions contain unsafe code.
- **Expressions**: 5 unsafe expressions total.
- **✓** = no unsafe in this crate's API.

### Interpreting the binary size report

```text
File   .text   Size    Crate
71.0%  59.0%   1.2MiB  std        ← Standard library
 8.5%   7.1%   147KiB  rust_template
 5.2%   4.3%   89KiB   serde
```

A large dependency contribution (e.g. `serde`) is a candidate for feature-flag trimming.

### Interpreting documentation coverage

```text
warning: missing documentation for public function `add`
  --> crates/lib.rs:10
```

Each warning names the public item that needs a doc comment.

## How-to

### Run the analysis locally

```bash
# Install tools
cargo install cargo-geiger cargo-bloat

# Run unsafe code analysis
cargo geiger --all-features

# Analyze binary size
cargo build --release
cargo bloat --release --crates

# Check documentation
cargo doc --no-deps --all-features
```

Verify: each command prints a report section matching the formats above.

### Configure unsafe code policy

Set the unsafe policy in `Cargo.toml`:

```toml
[lints.rust]
unsafe_code = "forbid"  # No unsafe allowed
# or
unsafe_code = "warn"    # Warn but allow
```

Verify: `cargo geiger` reports `0/N` for a `forbid` crate.

### Configure binary size optimization

```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true            # Link-time optimization
codegen-units = 1     # Better optimization
strip = true          # Remove symbols
panic = "abort"       # Smaller panic handler
```

Verify: `cargo build --release && cargo bloat --release --crates` and compare the total size.

### Configure documentation requirements

```toml
[lints.rust]
missing_docs = "warn"                     # Warn on missing docs
rustdoc::broken_intra_doc_links = "deny"  # Fail on broken links
```

Verify: `cargo doc --no-deps` surfaces missing-doc warnings.

### Improve the metrics

**Reduce unsafe code** — replace raw pointer writes with safe abstractions:

```rust
// Before
unsafe {
    *ptr = value;
}

// After - use safe abstraction
vec[index] = value;
```

**Reduce binary size** — find the largest contributors, then enable size optimizations:

```bash
cargo bloat --release -n 20
```

**Improve documentation** — add the missing doc comment the report named:

```rust
/// Adds two numbers together.
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}
```

```bash
cargo doc --no-deps                          # Check coverage
cargo test --doc                             # Run doc tests
cargo doc --no-deps --document-private-items # Generate private docs
```

Verify: re-run the relevant tool and confirm the count dropped.

### Troubleshooting

**cargo-geiger errors**:

```bash
cargo install cargo-geiger --force
cargo clean
cargo geiger
```

**Binary size analysis fails** — ensure a release build exists first:

```bash
cargo build --release
cargo bloat --release
```

**Documentation warnings** — surface them all as errors:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

## Why this matters

These three metrics each guard a property the compiler alone won't enforce. `unsafe_code = "forbid"` is the template default, so `cargo-geiger` exists to confirm that guarantee holds transitively — unsafe code bypasses Rust's safety model, and a dependency can reintroduce it silently. Binary size matters for download time and memory footprint, and `cargo-bloat` makes the cost of each dependency visible so feature trimming is an informed decision. Documentation coverage is a usability property: a well-documented public API is the difference between a crate people can adopt and one they have to reverse-engineer.

## Links

- [cargo-geiger](https://github.com/rust-secure-code/cargo-geiger)
- [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat)
- [rustdoc Documentation](https://doc.rust-lang.org/rustdoc/)
- [Unsafe Code Guidelines](https://rust-lang.github.io/unsafe-code-guidelines/)
- [CI Workflows reference](../template/CI-WORKFLOWS.md)

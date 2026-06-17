---
diataxis_type: how-to
---
# Code Coverage Tracking

Automated code coverage measurement and tracking using [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), with optional Codecov reporting.

## Reference

| Field | Value |
|---|---|
| Workflow | `.github/workflows/ci-coverage.yml` |
| Tool | cargo-llvm-cov |
| Integration | Codecov (optional) |
| Triggers | Via `pipeline.yml` on push/PR/tag, plus manual (`workflow_dispatch`) |
| Target | ≥90% coverage |

### CI pipeline stages

The workflow automatically:

1. **Instrument** — compile with coverage instrumentation.
2. **Execute** — run all tests (unit, integration, doc).
3. **Collect** — gather coverage data.
4. **Report** — generate HTML, LCOV, and JSON reports.
5. **Upload** — send to Codecov (if a token is configured) and upload the `coverage-report` artifact.
6. **Enforce** — fail CI if total coverage is below the 90% threshold.

Reports are available via **Actions → Workflow Run → Artifacts → coverage-report** (30-day retention).

### Summary output

```text
Filename              Regions  Missed Regions  Coverage
---------------------------------------------------------
crates/lib.rs              45              3     93.33%
crates/parser.rs           78             12     84.62%
crates/utils.rs            23              0    100.00%
---------------------------------------------------------
TOTAL                     146             15     89.73%
```

- **Regions**: code regions (branches, statements).
- **Missed Regions**: not executed during tests.
- **Coverage**: percent of regions executed.

### HTML report

Interactive report at `target/llvm-cov/html/index.html`:

- **Green**: covered lines.
- **Red**: uncovered lines.
- **Yellow**: partially covered branches.

### Coverage types

1. **Line coverage**: percent of lines executed.
2. **Branch coverage**: percent of conditional branches taken.
3. **Function coverage**: percent of functions called.

### Coverage goals

| Coverage | Quality | Action |
|----------|---------|--------|
| `< 50%` | Poor ❌ | Critical gaps |
| `50-70%` | Fair ⚠️ | Needs improvement |
| `70-85%` | Good ✅ | Acceptable |
| `> 85%` | Excellent 🌟 | High quality |

**Project target: ≥90%**

### What coverage does not measure

Coverage shows **execution**, not:

- **Correctness**: executed code may still be wrong.
- **Edge cases**: may miss unusual inputs.
- **Logic errors**: all branches covered ≠ all cases tested.
- **Race conditions**: concurrency issues are invisible.

Use coverage alongside mutation testing, property-based testing, and manual review.

## How-to

### Install and generate coverage locally

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Install the llvm-tools component
rustup component add llvm-tools-preview

# Generate coverage for all tests
cargo llvm-cov

# Generate HTML report
cargo llvm-cov --html --open

# Generate LCOV format (for Codecov)
cargo llvm-cov --lcov --output-path lcov.info

# Generate JSON report
cargo llvm-cov --json --output-path coverage.json
```

Verify: `cargo llvm-cov` prints a `TOTAL` coverage line.

### Find and close coverage gaps

1. Show uncovered lines:

   ```bash
   cargo llvm-cov --show-missing-lines
   cargo llvm-cov --ignore-filename-regex tests/
   ```

2. Add tests for the uncovered branch. For example, this error path is not covered:

   ```rust
   pub fn divide(a: i32, b: i32) -> Result<i32, Error> {
       if b == 0 {
           return Err(Error::DivideByZero);  // ❌ Not covered
       }
       Ok(a / b)  // ✅ Covered
   }
   ```

   Add the missing error-case test:

   ```rust
   #[test]
   fn test_divide_by_zero() {
       assert!(divide(10, 0).is_err());
   }
   ```

3. Cover the common gap categories:

   ```rust
   // Error paths
   #[test]
   fn test_errors() {
       assert!(parse("").is_err());
       assert!(parse("invalid").is_err());
       assert!(parse("too_long_".repeat(1000).as_str()).is_err());
   }

   // Edge cases / boundaries
   #[test]
   fn test_boundaries() {
       assert_eq!(clamp(0, 0, 10), 0);     // min
       assert_eq!(clamp(10, 0, 10), 10);   // max
       assert_eq!(clamp(-1, 0, 10), 0);    // below min
       assert_eq!(clamp(11, 0, 10), 10);   // above max
   }

   // Conditional branches — exercise both sides
   #[test]
   fn test_both_branches() {
       assert!(process(b"valid", true).is_ok());
       assert!(process(b"data", false).is_ok());
   }
   ```

   For `match` expressions, ensure every arm is exercised by a test.

Verify: re-run `cargo llvm-cov` and confirm the coverage percentage increased.

### Configure coverage

```bash
# Exclude test files
cargo llvm-cov --ignore-filename-regex tests/

# Exclude generated code
cargo llvm-cov --ignore-filename-regex generated/

# Coverage with all features
cargo llvm-cov --all-features

# Coverage with specific features
cargo llvm-cov --features feature1,feature2
```

Enforce a local threshold:

```bash
coverage=$(cargo llvm-cov --summary-only | grep -oP 'TOTAL.*\K\d+\.\d+')
if (( $(echo "$coverage < 90" | bc -l) )); then
    echo "Coverage ${coverage}% below threshold 90%"
    exit 1
fi
```

Verify: the script exits non-zero when coverage drops below the threshold.

### Set up Codecov

1. Sign up at https://codecov.io/.
2. Add the repository (GitHub integration).
3. Get the token: **Settings → Repository Upload Token**.
4. Add the secret: **GitHub repo → Settings → Secrets → `CODECOV_TOKEN`**.
5. Upload (CI does this automatically; manual command below):

   ```bash
   cargo llvm-cov --lcov --output-path lcov.info
   bash <(curl -s https://codecov.io/bash) -f lcov.info
   ```

Codecov then provides PR coverage diffs, trend tracking, a README badge, and a sunburst map. Add the badge to `README.md`:

```markdown
[![codecov](https://codecov.io/gh/USER/REPO/branch/main/graph/badge.svg)](https://codecov.io/gh/USER/REPO)
```

Verify: the Codecov dashboard shows the uploaded report.

### Advanced usage

```bash
# Coverage for changed files only (in a PR)
git diff --name-only main | grep '\.rs$' | xargs cargo llvm-cov --include-ffi

# Generate profdata for analysis
cargo llvm-cov --no-report --profdata-output rust_template.profdata
llvm-profdata show rust_template.profdata

# Include documentation tests
cargo llvm-cov --doc

# Coverage across a workspace
cargo llvm-cov --workspace
cargo llvm-cov --workspace --exclude member1
```

### Exclude code from coverage

```rust
// Ignore unreachable safety invariants
#[cfg(not(tarpaulin_include))]
fn internal_safety_check() {
    unreachable!("Safety invariant violated");
}

// Ignore debug-only code
#[cfg(debug_assertions)]
fn debug_only_function() {
    // Not covered in release builds
}
```

### Troubleshooting

**Zero coverage**:

```bash
rustup component add llvm-tools-preview
cargo clean
cargo llvm-cov
```

**Incomplete coverage**:

```bash
cargo llvm-cov --all-targets
cargo llvm-cov --doc
```

**Slow coverage**:

```bash
cargo llvm-cov -- --test-threads=4
cargo llvm-cov -- --skip slow_test
```

**Codecov upload fails**:

```bash
echo $CODECOV_TOKEN
bash <(curl -s https://codecov.io/bash) -f lcov.info -v
```

### Best practices

1. **Aim for ≥90%** — a good balance of quality and effort.
2. **Test error paths** — don't just test the happy path.
3. **Exclude test code** — focus on production code.
4. **Use integration tests** — cover real usage patterns.
5. **Track trends** — coverage should improve over time.
6. **Don't game metrics** — meaningful tests beat a coverage number.

## Why this matters

Coverage answers exactly one question: which lines ran during the tests. That is necessary but not sufficient — code that never executes is certainly untested, but executed code with a weak assertion is also effectively untested. That is why the target sits at ≥90% rather than 100%: the last few percent are usually unreachable error branches or defensive code where the cost of contrived tests outweighs the value, and chasing the number invites assertion-free tests that inflate coverage without catching bugs. The honest use of coverage is as a gap-finder feeding real tests, paired with mutation and property testing to check whether those tests actually assert anything.

## Links

- [cargo-llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Codecov Documentation](https://docs.codecov.com/)
- [LLVM Coverage Mapping](https://llvm.org/docs/CoverageMappingFormat.html)
- [Coverage Best Practices](https://testing.googleblog.com/2020/08/code-coverage-best-practices.html)
- [Mutation Testing](MUTATION-TESTING.md)
- [CI Workflows reference](../template/CI-WORKFLOWS.md)

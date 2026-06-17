---
diataxis_type: how-to
---
# Fuzz Testing with cargo-fuzz

Automated fuzz testing to discover crashes, panics, and edge cases using [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).

## Reference

| Field | Value |
|---|---|
| Workflow | `.github/workflows/fuzz-testing.yml` |
| Tool | `cargo-fuzz` (libFuzzer) |
| Trigger | Manual (`workflow_dispatch`); a daily cron is present but commented out |
| Goal | Find unexpected inputs that cause crashes |

### How fuzzing works

The fuzzer generates random or mutated inputs and feeds them to a target function:

1. **Generate inputs** — create random or mutated test inputs.
2. **Execute** — run the target function with each input.
3. **Monitor** — detect crashes, panics, timeouts, and memory errors.
4. **Minimize** — reduce a crashing input to a minimal reproducible case.
5. **Report** — save crash artifacts for investigation.

### CI behavior

The workflow runs:

- **On demand** via `workflow_dispatch` (the only active trigger).
- **Duration:** 5 minutes per target (configurable).

A daily `schedule:` cron (`0 2 * * *`) is present in the workflow but commented out. Uncomment the `schedule:` block in `.github/workflows/fuzz-testing.yml` to run fuzzing daily.

On a crash it creates a GitHub issue and uploads crash artifacts (90-day retention).

### Successful run output (no crashes)

```text
#0  READ units: 1234
#1  pulse  cov: 234 ft: 456 corp: 10/1234b
...
Done 10000 runs in 300 seconds
```

- **units**: inputs tested.
- **cov**: code coverage.
- **ft**: features covered.
- **corp**: corpus size.

### Crash output

```text
==1234==ERROR: AddressSanitizer: heap-buffer-overflow
READ of size 1 at 0x...
```

The crashing input is saved to `fuzz/artifacts/<target>/crash-<hash>`.

### Corpus layout

```text
fuzz/corpus/parse_input/
├── 0a1b2c3d4e5f...  # Auto-generated interesting cases
├── 1b2c3d4e5f6a...
└── seed_inputs/     # Your seed corpus
```

The fuzzer automatically saves interesting inputs that reach new coverage.

### Security benefits

Fuzz testing finds buffer overflows, integer overflows, assertion failures, panics and unwraps, memory leaks, and logic errors triggered by edge-case inputs.

## How-to

### Initialize fuzzing

```bash
# Install cargo-fuzz (requires nightly Rust)
cargo install cargo-fuzz

# Initialize fuzz targets
cargo fuzz init
```

This creates:

```text
fuzz/
├── Cargo.toml
└── fuzz_targets/
    └── fuzz_target_1.rs
```

Verify: `cargo fuzz list` prints the generated target.

### Create a fuzz target

Write `fuzz/fuzz_targets/parse_input.rs`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_template::parse;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string
    if let Ok(s) = std::str::from_utf8(data) {
        // Fuzz the parse function
        let _ = parse(s);
    }
});
```

For structured input, derive `Arbitrary`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    value: i32,
    flag: bool,
    data: Vec<u8>,
}

fuzz_target!(|input: FuzzInput| {
    // Fuzz with structured input
    process(input.value, input.flag, &input.data);
});
```

Verify: `cargo fuzz list` shows the new target.

### Run fuzzing locally

```bash
# List fuzz targets
cargo fuzz list

# Run a target for 60 seconds
cargo fuzz run parse_input -- -max_total_time=60

# Run with more parallel jobs
cargo fuzz run parse_input -- -jobs=4

# Run against a saved corpus
cargo fuzz run parse_input fuzz/corpus/parse_input
```

Verify: the run prints `cov:`/`corp:` lines and ends with `Done N runs`.

### Investigate a crash

1. Reproduce with the saved artifact:

   ```bash
   cargo fuzz run parse_input fuzz/artifacts/parse_input/crash-*
   ```

2. Minimize the crashing input:

   ```bash
   cargo fuzz tmin parse_input crash_artifact
   ```

3. Add debug output to the target if needed:

   ```rust
   fuzz_target!(|data: &[u8]| {
       eprintln!("Input length: {}", data.len());
       if let Ok(s) = std::str::from_utf8(data) {
           eprintln!("Input: {:?}", s);
           let _ = parse(s);
       }
   });
   ```

Verify: re-running with the minimized artifact still reproduces the crash, then fix the bug and confirm it no longer does.

### Manage the corpus

Seed initial inputs in `fuzz/corpus/<target>/`:

```bash
mkdir -p fuzz/corpus/parse_input
echo "valid input" > fuzz/corpus/parse_input/valid1
echo "" > fuzz/corpus/parse_input/empty
echo "🦀" > fuzz/corpus/parse_input/unicode
```

Verify: `cargo fuzz run parse_input fuzz/corpus/parse_input` loads the seeds.

### Configure fuzzing

```yaml
# In the workflow — adjust per-target duration
duration: '600'  # 10 minutes
```

```bash
# Limit memory usage
cargo fuzz run target -- -rss_limit_mb=2048
```

Add a dictionary of domain keywords at `fuzz/dict/target.dict`:

```text
"keyword1"
"keyword2"
"special_token"
```

```bash
cargo fuzz run target -- -dict=fuzz/dict/target.dict
```

Verify: the run reports the dictionary loaded.

### Common fuzz target patterns

```rust
// Parsers
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parser::parse(s);
    }
});

// Deserialization
fuzz_target!(|data: &[u8]| {
    let _: Result<MyStruct, _> = serde_json::from_slice(data);
});

// Binary protocols
fuzz_target!(|data: &[u8]| {
    let _ = decode_packet(data);
});
```

State machines via a sequence of arbitrary actions:

```rust
#[derive(Arbitrary, Debug)]
enum Action {
    Start,
    Process(u8),
    Stop,
}

fuzz_target!(|actions: Vec<Action>| {
    let mut state = State::new();
    for action in actions {
        state.handle(action);
    }
});
```

A complete structured target with input constraints:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct Config {
    timeout: u32,
    retries: u8,
    url: String,
}

fuzz_target!(|config: Config| {
    // Validate constraints
    if config.timeout > 0 && config.timeout < 10000 {
        if config.retries <= 10 {
            if config.url.len() < 256 {
                // Fuzz the actual function
                let _ = process_request(&config);
            }
        }
    }
});

fn process_request(config: &Config) -> Result<(), Error> {
    // Implementation
    Ok(())
}
```

### Troubleshooting

**Slow fuzzing**:

```bash
cargo fuzz run target -- -jobs=8
cargo fuzz run target -- -max_len=1024
```

**Out of memory**:

```bash
cargo fuzz run target -- -rss_limit_mb=2048
rm -rf fuzz/corpus/target/*
```

**No new coverage** — the fuzzer may be stuck. Add a better seed corpus, a dictionary, or switch to structured fuzzing with `arbitrary`.

### Best practices

1. **Start simple** — fuzz one function at a time.
2. **Use a seed corpus** — guide the fuzzer with valid examples.
3. **Run long sessions** — hours or days, not minutes.
4. **Minimize crashes** — use `cargo fuzz tmin` for debugging.
5. **Fuzz continuously** — run in CI regularly.
6. **Fuzz multiple targets** — cover different entry points.

## Why this matters

Hand-written tests check the inputs a developer thought of; fuzzing checks the inputs nobody thought of. By mutating inputs toward new code coverage, a fuzzer drives execution into the malformed, adversarial, and boundary cases where parsers, decoders, and deserializers actually break. Because it runs unattended and saves any crash as a minimized, replayable artifact, fuzzing turns "we hope this handles bad input" into a reproducible bug report — and enabling the (commented-out) daily schedule keeps probing as the code evolves, catching regressions long-running campaigns would otherwise surface only by luck.

## Links

- [cargo-fuzz Book](https://rust-fuzz.github.io/book/)
- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [Arbitrary Crate](https://docs.rs/arbitrary/)
- [Fuzzing Rust Code](https://rust-fuzz.github.io/book/introduction.html)
- [CI Workflows reference](../template/CI-WORKFLOWS.md)

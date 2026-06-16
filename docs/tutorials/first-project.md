---
diataxis_type: tutorial
---

# Your First Project with rust-template

Welcome! By the end of this tutorial you will have created your own repository
from **zircote/rust-template**, built it, run its test suite, added your very
first function, and watched GitHub's CI turn green. No prior experience with
this template is required — just follow each step in order and you will end up
with a working, fully-tested Rust crate.

This is a **learning walkthrough**: every step is designed to succeed, and we
explain the *why* as we go. When you finish you will understand how the template
is laid out, why its source lives in `crates/` instead of `src/`, and how its
quality gates protect your code. For task-focused recipes and lookup tables,
each section links you to the matching how-to and reference docs at the end.

> **Time:** about 20 minutes. **You will need:** a GitHub account and a
> terminal. We install the Rust toolchain together in Step 2.

---

## What we will build

You will turn the template's example crate into *your* crate and teach it one
new trick: a `multiply` function that sits right next to the template's existing
`add` function. Small on purpose — the point is to learn the full loop (edit →
test → commit → CI) on a change that is guaranteed to work.

Here is the whole journey:

```text
1. Use this template  ──>  your new GitHub repo
2. Install Rust       ──>  rustc 1.92+
3. Clone & build      ──>  first green test run locally
4. Tour the layout    ──>  understand crates/, the example API
5. Add multiply()     ──>  your first change, tested
6. Commit & push      ──>  green CI on GitHub
7. (Optional) /init   ──>  make it truly yours
8. (Optional) release ──>  ship v0.1.0
```

Let's go.

---

## Step 1 — Create your repository from the template

1. Open [zircote/rust-template](https://github.com/zircote/rust-template).
2. Click the green **"Use this template"** button, then **"Create a new
   repository"**.
3. Choose an **owner** (your username or an organization).
4. Give it a name. For this tutorial we will use `hello-rust` — pick anything
   you like and substitute your name wherever you see `hello-rust` below.
5. Pick **Public** or **Private**. Either works.
6. Click **"Create repository"**.

GitHub copies every file from the template into a brand-new repository that
belongs to you. (It does *not* copy stars, issues, or commit history — a
template gives you a clean starting point. The full breakdown of what copies
lives in the [GitHub Template Features](../template/GITHUB-TEMPLATE-FEATURES.md)
reference.)

### What happens automatically

The moment your repository exists, a workflow named **Template Init**
(`template-init.yml`) runs once and rewrites the template's placeholder names to
match your project:

| Placeholder | Becomes |
|---|---|
| `zircote/rust-template` | `your-owner/hello-rust` |
| `rust-template` | `hello-rust` (your repo name) |
| `rust_template` | `hello_rust` (your crate name, underscored) |

It takes about a minute and lands a commit titled
`chore: initialize from rust-template ...`. After it runs, `Cargo.toml`,
`README.md`, and the docs all point at your project.

> **Good to know:** this automatic rename only *renames*. It keeps the
> template's example code (the `add`, `divide`, and `Config` items you will meet
> in Step 4) so you have something real to build on. Later, in Step 7, the
> optional `/init-project` command strips that example code out when you are
> ready to write your own.

Give the workflow a minute to finish (you can watch it under the **Actions**
tab), then continue.

---

## Step 2 — Install the Rust toolchain

The template targets **Rust 1.92 or newer** (edition 2024). Install Rust with
the official `rustup` installer, which manages your compiler and lets you switch
versions later:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts, then reload your shell and verify:

```bash
source "$HOME/.cargo/env"
rustup default stable
rustc --version
```

You should see something like:

```text
rustc 1.92.0 (or newer)
```

> **Why rustup, not Homebrew?** `brew install rust` installs an unmanaged
> toolchain that cannot switch versions, add cross-compilation targets, or run
> the `rustup` commands this project uses. If you already installed Rust through
> Homebrew, remove it first with
> `brew uninstall rust rust-analyzer 2>/dev/null` and use rustup instead.

That is everything you need to build and test. (A couple of optional convenience
tools — `just` and `cargo-deny` — show up in Step 6; we will install them when
we get there.)

---

## Step 3 — Clone and run your first build

Clone the repository you created in Step 1 (replace the owner and name with
yours):

```bash
git clone https://github.com/your-owner/hello-rust.git
cd hello-rust
```

Now build and run the test suite — the most important habit in Rust:

```bash
cargo build
cargo test
```

The first `cargo build` downloads dependencies and compiles the crate, so it
takes a little longer than later runs. When `cargo test` finishes you will see
output that looks like this (exact counts will vary):

```text
   Compiling hello_rust v0.1.0 (/path/to/hello-rust)
    Finished test [unoptimized + debuginfo] target(s)
     Running unittests crates/lib.rs

running 9 tests
test tests::test_add ... ok
test tests::test_divide_success ... ok
test tests::test_config_builder ... ok
...
test result: ok. 9 passed; 0 failed; 0 ignored

   Doc-tests hello_rust
test result: ok. 3 passed; 0 failed
```

🎉 **That is your first green run.** Everything compiled and every test passed.

Notice the two kinds of tests that ran: the `running ... tests` block is the
unit tests, and the `Doc-tests` block at the bottom ran the code examples
embedded in the documentation comments. In this template, **your examples are
tests** — if a `# Examples` block stops compiling, `cargo test` fails. That is a
feature, and you will use it in Step 5.

You can also run the example binary:

```bash
cargo run
```

```text
2 + 3 = 5
10 / 2 = 5
```

---

## Step 4 — Tour the layout

Before changing anything, let's understand what you are looking at. This is the
knowledge that makes every later step obvious.

```text
hello-rust/
├── crates/
│   ├── lib.rs              # Library root — the public API lives here
│   └── main.rs             # Binary entry point (uses the library)
├── tests/
│   └── integration_test.rs # Tests that use the crate as an outside user would
├── Cargo.toml              # Package manifest (name, deps, lints, profiles)
├── clippy.toml             # Linter thresholds
├── rustfmt.toml            # Formatter rules
├── deny.toml               # Supply-chain policy (licenses, banned crates)
└── justfile                # Shortcuts for the CI commands
```

### Why `crates/`, not `src/`?

Most Rust projects keep code in `src/`. This template deliberately uses
`crates/` to signal that it is a template you are meant to reshape — and to make
room for growing into a multi-crate workspace later. The paths are wired up in
`Cargo.toml` under `[lib]` and `[[bin]]`, so the compiler always knows where to
look. Everything else (`cargo build`, `cargo test`, `cargo run`) works exactly
as normal.

### Meet the example API

Open `crates/lib.rs`. It is short and worth reading top to bottom. It
demonstrates the patterns this template wants you to follow:

```rust
/// Error type for `hello_rust` operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("operation '{operation}' failed: {cause}")]
    OperationFailed { operation: String, cause: String },
}

/// Result type alias for `hello_rust` operations.
pub type Result<T> = std::result::Result<T, Error>;
```

A few concepts to take away:

- **`Error` is one enum derived with `thiserror`.** The `#[error("...")]`
  attribute generates the human-readable message for each variant, so you never
  hand-write a `Display` implementation. `#[non_exhaustive]` lets you add new
  variants later without breaking callers.
- **`Result<T>` is a crate-wide alias** for `std::result::Result<T, Error>`. It
  means functions can return `Result<i64>` instead of repeating the full type
  everywhere — and the `?` operator just works.

Below that are the example functions:

| Item | What it shows |
|---|---|
| `add(a, b)` | A pure `const fn`, marked `#[must_use]`, with a doctest |
| `divide(dividend, divisor)` | A fallible function returning `Result<i64>` |
| `process(input)` | Parsing input and returning typed errors |
| `Config` | A consuming-self builder (`Config::new().with_verbose(true)`) |

These exist purely to teach the patterns. In Step 5 you will add a function in
the same style as `add` — copying a known-good shape is the safest way to make a
change that passes every gate on the first try.

> Want the full design rationale (why `thiserror`, why consuming builders, why
> the strict lints)? It lives in the project's `CLAUDE.md` under
> **Explanation**. For now, you know enough to make your first change.

---

## Step 5 — Make your first change: add `multiply`

You will add a `multiply` function right next to `add`. We model it on `add`
because `add` already passes every check — so by mirroring its shape (a pure
`const fn`, `#[must_use]`, a full doc comment with a runnable example), your new
function is correct by construction.

### 5a. Add the function to `crates/lib.rs`

Open `crates/lib.rs` and add this function just below the existing `add`
function (before `divide` is a fine spot):

```rust
/// Multiplies two numbers together.
///
/// # Arguments
///
/// * `a` - The first factor.
/// * `b` - The second factor.
///
/// # Returns
///
/// The product of `a` and `b`.
///
/// # Examples
///
/// ```rust
/// use hello_rust::multiply;
///
/// assert_eq!(multiply(2, 3), 6);
/// assert_eq!(multiply(-4, 5), -20);
/// ```
#[must_use]
pub const fn multiply(a: i64, b: i64) -> i64 {
    a * b
}
```

Use your own crate name in the `use` line — if you named your repo `hello-rust`,
the crate is `hello_rust` (hyphens become underscores).

Two things this small function teaches:

- **`#[must_use]`** tells the compiler to warn if a caller ignores the return
  value. The template's pedantic lints expect it on pure functions like this.
- **The `# Examples` block is a real test.** `cargo test` compiles and runs it.
  If you typo the assertion, the build fails — your documentation can never drift
  out of sync with your code.

### 5b. Add a unit test

Scroll to the `#[cfg(test)] mod tests` block near the bottom of the same file
and add a test alongside `test_add`:

```rust
    #[test]
    fn test_multiply() {
        assert_eq!(multiply(2, 3), 6);
        assert_eq!(multiply(-4, 5), -20);
        assert_eq!(multiply(0, 100), 0);
    }
```

### 5c. Add an integration test

Unit tests live *inside* the library. Integration tests live in `tests/` and
exercise your crate the way an outside user would — through its public API only.
Open `tests/integration_test.rs` and add:

```rust
#[test]
fn multiply_works_from_outside() {
    assert_eq!(hello_rust::multiply(6, 7), 42);
}
```

Again, swap `hello_rust` for your crate name.

### 5d. Run the tests

```bash
cargo test
```

You should now see your new tests among the output, all passing (cargo prints a
separate `test result:` line for each group — unit tests, integration tests, and
doctests):

```text
test tests::test_multiply ... ok
...
test multiply_works_from_outside ... ok
...
test result: ok. 0 failed
```

If something failed, read the message — Rust's compiler errors are famously
helpful. The usual culprits are a wrong crate name in the `use` line or a
mismatched bracket. Fix it and re-run; this loop (`edit → cargo test`) is the
heartbeat of Rust development.

You just added a function, documented it, and proved it works three different
ways. Nice.

---

## Step 6 — Run the full checks, then commit

`cargo test` is the fast inner loop. Before pushing, run the same gates CI will
run so there are no surprises. The template bundles them behind a single command
using [`just`](https://github.com/casey/just), a small task runner. Install
`just` and `cargo-deny` (used by one of the checks):

```bash
cargo install just cargo-deny
```

> **Shortcut:** if you opened this repo in a GitHub Codespace or the provided
> dev container, `just` and `cargo-deny` are already installed — skip the
> install command.

Now run the full check suite:

```bash
just check
```

This runs formatting, the Clippy linter, the tests, a documentation build, and
the supply-chain audit — exactly what GitHub runs on every push. If you prefer
the raw commands (no `just` needed), they are:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
cargo deny check
```

When everything is green, commit and push:

```bash
git add -A
git commit -m "feat: add multiply function"
git push
```

### Watch CI go green

Open your repository on GitHub and click the **Actions** tab. Your push kicked
off the pipeline. After a few minutes you will see a green check mark next to
your commit — the same checks you just ran locally, now confirmed on Linux,
macOS, and Windows.

✅ **You shipped a tested change and CI is green.** That is the complete
professional loop, start to finish.

If a check ever fails on GitHub but passed locally (or vice versa), the
[CI Troubleshooting](../runbooks/CI-TROUBLESHOOTING.md) runbook maps each
failure to its fix.

---

## Step 7 — (Optional) Make it truly yours with `/init-project`

So far you have been building on the template's example code. When you are ready
to start your *real* project, you have two ways to clear the examples out:

- **By hand:** the [Customization](../template/CUSTOMIZATION.md) guide walks
  through removing the example functions and replacing the README.
- **Automated:** if you use Claude Code, run the **`/init-project`** command. It
  interviews you (crate name, description, author, library-only vs.
  library+binary, distribution channels), then renames everything, strips the
  example `add`/`divide`/`Config` code down to a clean stub, configures your
  distribution channels, and verifies the result builds clean.

> **Heads up — this is destructive.** `/init-project` *removes* the example API
> you used in Steps 4 and 5. That is the point: it hands you an empty, correctly
> wired crate to fill with your own code. Run it only once you no longer need the
> examples. After it finishes, your "first change" becomes adding a brand-new
> function to the stub (the same pattern you practiced with `multiply`).

This step is genuinely optional. Many people keep building on the example code
for a while first. Do it whenever it feels right.

---

## Step 8 — (Optional) Cut your first release

When you have something worth sharing, the template can build signed,
attested release binaries for five platforms, generate a software bill of
materials, and (in a real project) publish to crates.io and Homebrew.

In the **template** itself, publication is intentionally switched off
(`publish = false` in `Cargo.toml`), so a release runs the full build-and-verify
chain as a dry run without pushing anything outward. Deleting that one line — or
letting `/init-project` remove it for you — arms the real publishing channels.

The short version of cutting a release:

```bash
# Bump the version in Cargo.toml, commit it, then:
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

Pushing a `v*.*.*` tag triggers the release automation. The full, careful
procedure — pre-release checklist, monitoring each workflow, and rollback — is
in the [Releasing](../runbooks/RELEASING.md) runbook. Read it before your first
real release.

---

## What you learned

In one sitting you:

- **Created a repository from a template** and saw the automatic rename wire up
  your project name everywhere.
- **Installed a rustup-managed toolchain** the way the project expects.
- **Built and tested** a Rust crate, and learned that *doctests are real tests*.
- **Understood the layout** — why source lives in `crates/`, and how `Error`,
  the `Result<T>` alias, the example functions, and the `Config` builder
  demonstrate the template's patterns.
- **Added a function the safe way** — mirroring a known-good shape — with a doc
  example, a unit test, and an integration test.
- **Ran the full quality gate** locally and **watched the same checks pass in
  CI** across three operating systems.
- **Learned the two optional next moves:** stripping the examples with
  `/init-project`, and cutting an attested release.

You now have everything you need to build something real.

## Where to go next

| If you want to… | Read |
|---|---|
| Follow a task-focused setup checklist (secrets, signing, options) | [Getting Started](../template/GETTING-STARTED.md) (how-to) |
| Configure `Cargo.toml`, features, profiles, and lints | [Configuration](../template/CONFIGURATION.md) (how-to) |
| Add modules, go library-only, adjust lints, customize Docker | [Customization](../template/CUSTOMIZATION.md) (how-to) |
| Have AI scaffold a whole project for you | [Copilot Jumpstart](../template/COPILOT-JUMPSTART.md) (how-to) |
| Look up exactly what every CI workflow does | [CI Workflows](../template/CI-WORKFLOWS.md) (reference) |
| See what copies when you "Use this template" | [GitHub Template Features](../template/GITHUB-TEMPLATE-FEATURES.md) (reference) |
| Fix a failing CI check | [CI Troubleshooting](../runbooks/CI-TROUBLESHOOTING.md) (how-to) |
| Cut and verify a release | [Releasing](../runbooks/RELEASING.md) (how-to) |
| Understand the design rationale (why `thiserror`, `crates/`, strict lints) | `CLAUDE.md` → Explanation |

Happy building. 🦀

---
name: init-project
description: Initialize a new Rust project from this template. Runs an interactive interview to gather project details, then renames, strips example code, configures distribution channels, and verifies the result.
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
  - AskUserQuestion
argument-hint: "[optional: crate_name]"
---

# Template Project Initializer

You are initializing a new Rust project from the `zircote/rust-template` GitHub template. This is a standalone replacement for the `template-init.yml` GitHub Actions workflow -- you will handle everything locally and interactively.

**IMPORTANT**: This command is destructive and irreversible. It transforms the template into a real project.

**Before making ANY changes**:
1. Confirm the user wants to proceed
2. Create a safety branch: `git checkout -b pre-init-backup && git checkout -` so the user can recover if needed

---

## Phase 1: Interview

Gather project details from the user using AskUserQuestion. If the user passed a crate name as an argument (`$ARGUMENTS`), use it and skip that question.

### Round 1: Identity

Ask these questions (use AskUserQuestion with up to 4 questions):

1. **Crate name** (skip if provided as `$ARGUMENTS`):
   - Must be a valid Rust identifier: lowercase, underscores, no hyphens, no leading digits
   - Example: `my_awesome_crate`
   - This becomes the `name` in Cargo.toml, lib name, and binary name

2. **Project description**:
   - One-line description for Cargo.toml and README
   - Example: "A high-performance HTTP proxy with TLS termination"

3. **Author**:
   - Format: "Name \<email\>" for Cargo.toml authors field
   - Example: "Jane Doe \<jane@example.com\>"

4. **GitHub owner/org**:
   - The GitHub username or organization that owns this repo
   - Example: "acme-corp"

### Round 2: Architecture

Ask these questions:

1. **Project type**: Library-only or Library + Binary?
   - Library-only: removes `[[bin]]` from Cargo.toml, removes `crates/main.rs`, removes Docker/Snap configs
   - Library + Binary: keeps both, binary name matches crate name

2. **Distribution channels** (multi-select, only if Library + Binary):
   - Docker (Dockerfile, `.dockerignore`, release-docker workflows)
   - Snap (`snap/snapcraft.yaml`, release-packages Snap job)
   - Homebrew (release-packages Homebrew job)
   - None of the above (remove all distribution packaging)

### Round 3: Optional Metadata

Ask if the user wants to customize these or accept defaults:

1. **Keywords** (default: empty array, max 5 for crates.io)
2. **Categories** (default: empty array, from crates.io category list)
3. **License** (default: "MIT")
4. **MSRV** (default: "1.92")

---

## Phase 1.5: Verify Rust Toolchain

Before making any changes, verify the user has a rustup-managed toolchain (not Homebrew):

```bash
# Check if rustup exists
if ! command -v rustup &>/dev/null; then
  echo "ERROR: rustup not found."
  echo ""
  echo "Install Rust via the official installer — do NOT use Homebrew:"
  echo ""
  echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo "  source \"\$HOME/.cargo/env\""
  echo ""
  echo "If you installed Rust via Homebrew, remove it first:"
  echo "  brew uninstall rust rust-analyzer 2>/dev/null"
  exit 1
fi
```

If `rustup` is missing, show the installation instructions above via `AskUserQuestion` and **stop**. Do not proceed with initialization until the user has a working `rustup` installation.

If `rustup` is present, verify the toolchain meets MSRV:

```bash
rustup default stable
rustup update
RUSTC_VERSION=$(rustc --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
echo "Rust toolchain: ${RUSTC_VERSION} (managed by rustup)"
```

Report the detected version to the user and continue.

---

## Phase 2: Derive Names

From the interview answers, compute:

- `CRATE_NAME`: the crate name (snake_case), e.g. `my_awesome_crate`
- `REPO_NAME`: crate name with underscores replaced by hyphens, e.g. `my-awesome-crate`
- `OWNER`: GitHub owner/org, e.g. `acme-corp`
- `DESCRIPTION`: one-line description
- `AUTHOR`: author string
- `LICENSE`: license identifier
- `MSRV`: minimum supported Rust version
- `KEYWORDS`: array of keywords
- `CATEGORIES`: array of categories
- `IS_BINARY`: whether to include binary target
- `DIST_DOCKER`: whether to keep Docker
- `DIST_SNAP`: whether to keep Snap
- `DIST_HOMEBREW`: whether to keep Homebrew

---

## Phase 3: Global Rename

Perform systematic find-and-replace across the entire project. **Order matters** -- replace longer/more specific patterns first to avoid partial matches.

### Replacement Order

Apply these replacements to ALL text files in the project (skip `.git/`, binary files like `*.png`, `*.jpg`, `*.ico`):

1. `zircote/rust-template` -> `{OWNER}/{REPO_NAME}`
2. `zircote/rust_template` -> `{OWNER}/{CRATE_NAME}`
3. `zircote` -> `{OWNER}`
4. `rust-template` -> `{REPO_NAME}`
5. `rust_template` -> `{CRATE_NAME}`

### Files to Process

Use Bash with `find` and `sed` (this is the one case where Bash is appropriate for bulk operations):

```bash
find . -type f \
  ! -path './.git/*' \
  ! -path './.claude/commands/*' \
  ! -name '*.png' ! -name '*.jpg' ! -name '*.ico' ! -name '*.gif' \
  ! -name '*.woff' ! -name '*.woff2' ! -name '*.ttf' ! -name '*.eot' \
  ! -name 'Cargo.lock' \
  -exec sed -i '' \
    -e "s|zircote/rust-template|${OWNER}/${REPO_NAME}|g" \
    -e "s|zircote/rust_template|${OWNER}/${CRATE_NAME}|g" \
    -e "s|zircote|${OWNER}|g" \
    -e "s|rust-template|${REPO_NAME}|g" \
    -e "s|rust_template|${CRATE_NAME}|g" \
    {} +
```

**Note**: On macOS, `sed -i ''` requires the empty string argument. This is the correct form.

### Workflow Files

Unlike `template-init.yml` which skips `.github/workflows/`, you MUST also process workflow files. They contain references to binary names, Docker image names, and Homebrew formulas that need updating.

---

## Phase 4: Update Cargo.toml

Use the Edit tool to update Cargo.toml with the interview answers:

1. `description` -> user's description
2. `authors` -> user's author string (as array)
3. `keywords` -> user's keywords (or empty array)
4. `categories` -> user's categories (or empty array)
5. `license` -> user's license choice
6. `rust-version` -> user's MSRV choice
7. `version` -> reset to `"0.1.0"`

If **library-only** (no binary):
- Remove the entire `[[bin]]` section
- Remove `crates/main.rs` (use Bash `rm`)

---

## Phase 5: Strip Example Code

### crates/lib.rs

Replace the entire file contents with a clean stub that keeps the project's patterns but removes example functions:

```rust
#![doc = include_str!("../README.md")]

use thiserror::Error;

/// Error type for `{CRATE_NAME}` operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid input was provided.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// An operation failed.
    #[error("operation '{operation}' failed: {cause}")]
    OperationFailed {
        /// The operation that failed.
        operation: String,
        /// The underlying cause.
        cause: String,
    },
}

/// Result type alias for `{CRATE_NAME}` operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::InvalidInput("test error".to_string());
        assert_eq!(err.to_string(), "invalid input: test error");

        let err = Error::OperationFailed {
            operation: "test".to_string(),
            cause: "failed".to_string(),
        };
        assert_eq!(err.to_string(), "operation 'test' failed: failed");
    }
}
```

### crates/main.rs (only if binary target retained)

Replace with a clean stub:

```rust
//! Binary entry point for `{CRATE_NAME}`.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::process::ExitCode;

/// Runs the application logic.
fn run() -> Result<(), {CRATE_NAME}::Error> {
    // TODO: implement application logic
    Ok(())
}

/// Main entry point.
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        },
    }
}
```

### tests/integration_test.rs

Replace with a clean stub:

```rust
//! Integration tests for `{CRATE_NAME}`.

use {CRATE_NAME}::Error;

#[test]
fn test_error_types() {
    let err = Error::InvalidInput("test message".to_string());
    let display = format!("{err}");
    assert!(display.contains("invalid input"));
    assert!(display.contains("test message"));

    let err = Error::OperationFailed {
        operation: "read".to_string(),
        cause: "file not found".to_string(),
    };
    let display = format!("{err}");
    assert!(display.contains("read"));
    assert!(display.contains("file not found"));
}
```

---

## Phase 6: Update README.md

Replace the entire README with a clean version for the new project:

```markdown
# `{CRATE_NAME}`

<!-- Badges -->
[![CI](https://github.com/{OWNER}/{REPO_NAME}/actions/workflows/ci.yml/badge.svg)](https://github.com/{OWNER}/{REPO_NAME}/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/{CRATE_NAME}.svg?logo=rust&logoColor=white)](https://crates.io/crates/{CRATE_NAME})
[![Documentation](https://docs.rs/{CRATE_NAME}/badge.svg)](https://docs.rs/{CRATE_NAME})
[![Rust Version](https://img.shields.io/badge/rust-{MSRV}%2B-dea584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-{LICENSE}-green)](https://github.com/{OWNER}/{REPO_NAME}/blob/main/LICENSE)

{DESCRIPTION}

## Installation

Add this to your `Cargo.toml`:

\`\`\`toml
[dependencies]
{CRATE_NAME} = "0.1"
\`\`\`

Or use cargo add:

\`\`\`bash
cargo add {CRATE_NAME}
\`\`\`

## Quick Start

\`\`\`rust
use {CRATE_NAME}::Result;

fn main() -> Result<()> {
    // TODO: add usage example
    Ok(())
}
\`\`\`

## Development

### Prerequisites

- Rust {MSRV}+ (2024 edition) — install via [rustup](https://rustup.rs/), **not** Homebrew
- [just](https://github.com/casey/just) task runner
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) for supply chain security

### Setup

\`\`\`bash
# Install Rust via rustup (not Homebrew)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

git clone https://github.com/{OWNER}/{REPO_NAME}.git
cd {REPO_NAME}

just check    # Full CI check (fmt + clippy + test + doc + deny)
just build    # Debug build
just test     # Run all tests
\`\`\`

### Project Structure

\`\`\`text
crates/
{PROJECT_STRUCTURE_LIB_OR_BOTH}

tests/
└── integration_test.rs

Cargo.toml           # Project manifest
clippy.toml          # Clippy configuration
rustfmt.toml         # Formatter configuration
deny.toml            # cargo-deny configuration
justfile             # Task runner recipes
\`\`\`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and coding standards.

## License

This project is licensed under the {LICENSE} License - see the [LICENSE](LICENSE) file for details.
```

**Note**: `{PROJECT_STRUCTURE_LIB_OR_BOTH}` is NOT a placeholder from Phase 2. You must compute it inline when writing the README:
- If library-only: use `├── lib.rs           # Library root`
- If library + binary: use `├── lib.rs           # Library root` followed by `└── main.rs          # Binary entry point` on the next line

---

## Phase 7: Update CLAUDE.md

Use the Edit tool to update the Project Overview section in CLAUDE.md:

1. Replace the first paragraph of "## Project Overview" with:
   ```
   This is **{DESCRIPTION}**. The crate name is `{CRATE_NAME}` (Rust edition 2024, MSRV {MSRV}). It ships {LIB_OR_BOTH}. Source lives in `crates/`, not the standard `src/` directory.
   ```
   Where `{LIB_OR_BOTH}` is either "a library (`crates/lib.rs`)" or "both a library (`crates/lib.rs`) and a binary (`crates/main.rs`)".

2. In the How-to section "Add a New Public Function", the examples reference `add()` and `divide()` -- these are now just patterns to follow. Leave them as generic guidance.

3. Update any remaining `rust_template` references that the global rename might have missed (check with Grep).

---

## Phase 8: Remove Template-Specific Content

### Remove docs/template/ directory
```bash
rm -rf docs/template/
```

### Remove template-init.yml workflow
```bash
rm .github/workflows/template-init.yml
```

### Remove distribution configs (based on interview)

If **Docker not selected**:
```bash
rm Dockerfile .dockerignore
```
Also remove/comment Docker-related workflow references if they exist.

If **Snap not selected**:
```bash
rm -rf snap/
```

If **Homebrew not selected**:
The Homebrew formula is generated in the release-packages workflow. Note this in the summary but don't remove workflow jobs (they're conditional).

If **library-only** (no binary):
- Remove Dockerfile, .dockerignore, snap/ (all distribution is binary-specific)
- Remove `crates/main.rs`

### Update CODEOWNERS (if it exists)

If `.github/CODEOWNERS` exists, edit it to replace `@zircote` with `@{OWNER}` and fix paths:
- Change `/src/` to `/crates/` (template has a known mismatch here)

If it does not exist, skip this step.

### Reset CHANGELOG.md

Replace contents with:
```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]
```

### Update LICENSE copyright

Replace the copyright line with:
```
Copyright (c) {CURRENT_YEAR} {CRATE_NAME} contributors
```
Use the current year from `date +%Y`.

---

## Phase 9: Regenerate and Verify

### Regenerate Cargo.lock
```bash
cargo generate-lockfile
```

### Run verification checks
```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
cargo deny check
```

If any check fails, fix the issue and re-run.

### Final grep for leftover template references
```bash
grep -r "rust_template\|rust-template\|zircote" --include='*.rs' --include='*.toml' --include='*.md' --include='*.yml' --include='*.yaml' --include='*.json' . | grep -v '.git/' | grep -v 'target/' | grep -v '.claude/commands/'
```

Fix any remaining references found.

---

## Phase 10: Summary

Print a summary of everything that was done:

```
## Project Initialized

**{CRATE_NAME}** ({OWNER}/{REPO_NAME})

### Configuration
- Description: {DESCRIPTION}
- Author: {AUTHOR}
- License: {LICENSE}
- MSRV: {MSRV}
- Type: {Library / Library + Binary}
- Distribution: {Docker, Snap, Homebrew / None}

### Changes Made
- Renamed all template references ({N} files processed)
- Updated Cargo.toml with project metadata
- Stripped example code from lib.rs, main.rs, integration tests
- Updated README.md with project-specific content
- Updated CLAUDE.md project overview
- Removed docs/template/ directory
- Removed template-init.yml workflow
- {Removed Dockerfile / Kept Dockerfile}
- {Removed snap/ / Kept snap/}
- Reset CHANGELOG.md
- Updated LICENSE copyright
- Updated CODEOWNERS (if present)
- Regenerated Cargo.lock
- All checks passing (fmt, clippy, test, doc, deny)

### Next Steps
1. Review the changes: `git diff`
2. Commit: `git add -A && git commit -m "chore: initialize project from rust-template"`
3. Start building your project in `crates/lib.rs`
4. Add your first feature or implement from a spec
```

---

## Error Handling

- If `cargo fmt` or `cargo clippy` fails after transformation, fix the issue before reporting success
- If the user cancels during the interview, stop immediately and report no changes were made
- If a file referenced in transformations doesn't exist (e.g., Dockerfile already removed), skip it silently
- Always verify the crate name is a valid Rust identifier before proceeding

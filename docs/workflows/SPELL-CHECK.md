---
diataxis_type: how-to
---
# Spell Checking with typos

Automated spell checking for documentation, code comments, and string literals using [typos](https://github.com/crate-ci/typos). It warns on typos but does not fail CI.

## Reference

| Field | Value |
|---|---|
| Workflow | `.github/workflows/spell-check.yml` |
| Configuration | `.typos.toml` |
| Behavior | Warns on typos, does not fail CI |

### Triggers

The workflow runs on every push to main/master, all pull requests, and manual workflow dispatch. It scans all files (except excluded paths) for common typos and suggests corrections.

### Warning output

```text
warning: `recieve` should be `receive`
  --> crates/lib.rs:10
```

Results are visible in the Actions tab.

## How-to

### Check locally

```bash
# Install typos
cargo install typos-cli

# Check for typos
typos

# Auto-fix typos
typos --write-changes
```

Verify: `typos` exits without warnings on clean text.

### Configure behavior

Edit `.typos.toml`:

```toml
[default]
# Add regex patterns to ignore
extend-ignore-re = [
    "[0-9a-f]{40}",  # Git SHAs
]

[files]
# Exclude directories/files
extend-exclude = [
    "target/",
    "*.lock",
]

[default.extend-words]
# Project-specific dictionary
# "typo" = "correct"
```

Verify: `typos` no longer flags the configured patterns.

### Handle a false positive

Accept a word as correct:

```toml
[default.extend-words]
myword = "myword"  # Accept as correct
```

Verify: re-run `typos` and confirm the word is no longer flagged.

### Exclude specific files

```toml
[files]
extend-exclude = [
    "docs/legacy/",
]
```

Verify: `typos` skips the excluded paths.

### Maintain a custom dictionary

```toml
[default.extend-identifiers]
# Code identifiers
myvar = "myvar"

[default.extend-words]
# Documentation words
specialterm = "specialterm"
```

Verify: `typos` treats the listed identifiers and words as correct.

## Why this matters

Typos in public-facing documentation, API names, and error messages erode trust and make a project look unmaintained, but a hard CI failure on every misspelling would block merges for trivial reasons and tempt contributors to disable the check entirely. Running typos as a non-blocking warning keeps spelling visible on every change without gatekeeping, and the configurable dictionary means domain terms and identifiers are accepted once rather than fought repeatedly.

## Links

- [typos Documentation](https://github.com/crate-ci/typos)
- [Configuration Reference](https://github.com/crate-ci/typos/blob/master/docs/reference.md)
- [CI Workflows reference](../template/CI-WORKFLOWS.md)

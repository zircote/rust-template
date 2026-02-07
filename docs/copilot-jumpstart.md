# Jumpstart Your Project with Copilot

When you create a new repository from this template, GitHub offers an optional **"Jumpstart your project with Copilot"** prompt. Paste one of the prompts below into that field, and Copilot will open a pull request that scaffolds your project — renaming the template, wiring up dependencies, and writing starter code.

> **How it works:** After repository creation, Copilot reads your prompt, generates the files, and opens a PR for your review. All 31 CI/CD workflows, security tooling, Docker setup, and linting configuration are preserved automatically.

---

## Default Prompt

Use this generic prompt for any Rust project. Copilot will infer your crate name and purpose from the repository name and description you set during creation.

```text
Customize this Rust template for my project. Determine crate name, GitHub
owner/repo, and purpose from the repo name and description. (1) Replace every
"rust_template", "rust-template", "zircote/rust-template" with correct names
across Cargo.toml, Cargo.lock, README.md, CLAUDE.md, crates/lib.rs,
crates/main.rs, .github/copilot-instructions.md. (2) Update Cargo.toml: name,
authors, repository, homepage, documentation, keywords, categories, description.
(3) Replace crates/lib.rs with idiomatic types, thiserror error variants, and
public functions with doc comments and examples. (4) Replace crates/main.rs with
a binary that uses the library. (5) Update README.md with new name, description,
features, and usage. Do NOT modify .github/workflows/, deny.toml, clippy.toml,
rustfmt.toml, or Dockerfile. Verify: cargo fmt -- --check && cargo clippy
--all-targets --all-features -- -D warnings && cargo test
```

---

## Example Use-Case Prompts

### 1. CLI Tool — Fast File Search Utility

```text
Build a CLI file search tool called "fzgrep". Rename all "rust_template"/
"rust-template"/"zircote/rust-template" to "fzgrep" and my GitHub owner/repo.
Add to Cargo.toml: clap (derive), regex, ignore, colored. Create under crates/:
search.rs (search engine), pattern.rs (regex compilation), output.rs (colored
formatting), walker.rs (directory traversal). In lib.rs define SearchResult,
SearchConfig (builder pattern), and error variants. In main.rs build a clap CLI
with args: pattern, path, flags for case-insensitive, hidden files, type filters.
Write unit tests for pattern compilation, matching, and formatting. Add proptest
for pattern edge cases. Update README.md with name, features, install, and usage.
Do NOT modify .github/workflows/, deny.toml, clippy.toml, rustfmt.toml, or
Dockerfile. Verify: cargo fmt -- --check && cargo clippy --all-targets
--all-features -- -D warnings && cargo test
```

### 2. REST API Server — Task Manager

```text
Build a REST API task manager called "taskd" with axum, tokio, serde. Rename all
"rust_template"/"rust-template"/"zircote/rust-template" to "taskd" and my GitHub
owner/repo. Uncomment tokio (full) and serde (derive) in Cargo.toml, add: axum,
serde_json, uuid (v4+serde), tower-http (cors+trace). Create under crates/:
routes.rs (CRUD handlers), models.rs (Task with id/title/status/timestamps),
state.rs (Arc<RwLock> store), error.rs (NotFound, Conflict variants). In lib.rs
expose build_router() -> axum::Router. In main.rs start tokio and bind
0.0.0.0:3000. Write tests with axum::test for POST/GET/PUT/DELETE /tasks
endpoints. Update README.md with API docs, curl examples, Docker usage. Do NOT
modify .github/workflows/, deny.toml, clippy.toml, or rustfmt.toml. Verify:
cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings
&& cargo test
```

### 3. Library Crate — Data Validation

```text
Build a validation library called "validox" for structured data with composable
rules. Rename all "rust_template"/"rust-template"/"zircote/rust-template" to
"validox" and my GitHub owner/repo. Add serde (derive) and serde_json. Remove
[[bin]] from Cargo.toml and delete crates/main.rs (library-only). Create under
crates/: rule.rs (Validate trait, rules: Required, MinLength, MaxLength, Range,
Pattern, Email), schema.rs (Schema builder composing rules per field), result.rs
(ValidationResult with field errors), types.rs (Value enum). In lib.rs provide
fluent API: Schema::new().field("email", Rule::required().email()).validate(&data).
Write unit tests for every rule. Add proptest for string/numeric fuzzing. Include
doc tests on all public types. Update README.md with features and API examples.
Do NOT modify .github/workflows/ or config files. Verify: cargo fmt -- --check
&& cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

### 4. System Utility — Log Aggregator

```text
Build a log aggregator called "logpulse" that tails multiple files with unified
output. Rename all "rust_template"/"rust-template"/"zircote/rust-template" to
"logpulse" and my GitHub owner/repo. Uncomment tokio (full) and tracing in
Cargo.toml. Add: clap (derive), notify, chrono, serde (derive), serde_json,
colored. Create under crates/: watcher.rs (async file watcher), parser.rs
(syslog/JSON/plain text parsing), filter.rs (level/source/regex filtering),
output.rs (colored formatting). In lib.rs define LogEntry, LogSource, FilterConfig
with builder pattern. In main.rs build clap CLI with file paths, filter flags,
output format (text/json). Write tests for each parser and filter combo. Add
proptest for malformed input resilience. Update README.md with features, usage,
supported formats. Do NOT modify .github/workflows/ or config files. Verify:
cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings
&& cargo test
```

### 5. WebAssembly Module — Text Processing

```text
Build a WASM text processing library called "textkit-wasm" for browser and
Node.js. Rename all "rust_template"/"rust-template"/"zircote/rust-template" to
"textkit_wasm"/"textkit-wasm" and my GitHub owner/repo. Add: wasm-bindgen,
js-sys, unicode-segmentation, serde (derive), serde-wasm-bindgen. Set
crate-type=["cdylib","rlib"] in [lib]. Remove [[bin]] and delete crates/main.rs.
Create under crates/: transform.rs (wasm_bindgen exports: slug, title_case,
camel_case, truncate, word_wrap), analyze.rs (word_count, char_count,
reading_time, readability_score), sanitize.rs (strip_html, escape_html,
normalize_whitespace). In lib.rs re-export all wasm_bindgen functions. Write
unit tests for every function. Add proptest for Unicode edge cases (CJK, emoji,
RTL). Update README.md with wasm-pack build, npm usage, JS examples. Do NOT
modify .github/workflows/ or config files. Verify: cargo fmt -- --check &&
cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

### 6. MCP Server — AI Tool Provider

```text
Build an MCP (Model Context Protocol) server called "mcptools" exposing dev tools
to AI assistants. Rename all "rust_template"/"rust-template"/
"zircote/rust-template" to "mcptools" and my GitHub owner/repo. Uncomment tokio
(full) and serde (derive) in Cargo.toml, add: serde_json, rmcp (features:
transport-io, server). Create under crates/: server.rs (MCP server with stdio
transport), tools.rs (#[tool] functions: file_stats counting lines/words/chars,
json_format for pretty-printing, hash_digest for SHA-256), types.rs (tool
input/output structs). In lib.rs expose build_server() returning the MCP
ServerHandler. In main.rs init tokio and start stdio transport. Write unit tests
for each tool. Add proptest for edge cases. Update README.md with MCP overview,
tool docs, and Claude Desktop config example. Do NOT modify .github/workflows/
or config files. Verify: cargo fmt -- --check && cargo clippy --all-targets
--all-features -- -D warnings && cargo test
```

---

## Tips for Writing Your Own Prompt

- **Be specific about your crate name** — Copilot will search-and-replace across all files
- **List the dependencies you need** — mention which commented-out deps in Cargo.toml to uncomment
- **Describe your module structure** — files go under `crates/`, not `src/`
- **Mention if it's library-only** — tell Copilot to remove the `[[bin]]` section and `crates/main.rs`
- **Always include the verification line** — `cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`
- **Protect the CI/CD** — always say "Do NOT modify .github/workflows/ or config files"

---
diataxis_type: reference
---
# Error Type Registry

The canonical registry of problem types this crate emits as RFC 9457
`application/problem+json` envelopes. Each `type` URI below is dereferenceable —
it resolves to its own reference page, which is the source of truth for that
type's status, recovery action, and stability.

| Problem type | Status | Exit code | Retryable |
|---|---|---|---|
| [Invalid input](https://zircote.com/rust-template/errors/invalid-input/v1) | 400 | 2 | No |
| [Operation failed](https://zircote.com/rust-template/errors/operation-failed/v1) | 500 | 1 | No |

For the envelope structure, applicability markers, and renderer selection, see
the **Dual-Consumer Error Output** explanation.

## Customizing the base URI

This is a template, so the URI host is a single configurable knob, not a
hardcoded string. The constant `ERROR_TYPE_BASE_URI` in `crates/problem.rs`
holds the base (`https://zircote.com/rust-template/errors` by default); every
type URI is derived as `{base}/{slug}/{version}`. An adopter sets that one
constant to their own documentation host and all type URIs follow. The
occurrence `instance` URN namespace is derived from the crate name
(`CARGO_PKG_NAME`), so renaming the crate in `Cargo.toml` updates it
automatically — no edit here.

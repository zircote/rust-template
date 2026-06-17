---
diataxis_type: reference
---
# Invalid input

`InvalidInput` — a value supplied to an operation did not satisfy a documented
constraint, so the operation never ran.

## Identity

| Field      | Value                                                          |
|------------|----------------------------------------------------------------|
| Type URI   | `https://zircote.com/rust-template/errors/invalid-input/v1`    |
| Title      | `Invalid input`                                                |
| Status     | `400`                                                          |
| Exit code  | `2`                                                            |
| Retryable  | No — `retry_after` is `null` (non-transient).                  |

## When it occurs

Returned when a value fails validation *before* an operation runs — for example
`divide(_, 0)` (the divisor cannot be zero). The `detail` member carries the
occurrence-specific `Display` string, such as
`invalid input: divisor cannot be zero`.

## Recovery

| Field             | Value                                                                   | Applicability       |
|-------------------|-------------------------------------------------------------------------|---------------------|
| `suggested_fix`   | Correct the input so it satisfies the documented constraints, then retry. | `maybe_incorrect`   |
| `code_actions[0]` | Replace the offending input with a valid value (`quickfix`).            | `maybe_incorrect`   |

The applicability is `maybe_incorrect`, not `machine_applicable`: the correct
value is domain-specific, so a consumer must escalate to a human rather than
substitute a value automatically.

## Example envelope

```json
{
  "type": "https://zircote.com/rust-template/errors/invalid-input/v1",
  "title": "Invalid input",
  "status": 400,
  "detail": "invalid input: divisor cannot be zero",
  "instance": "urn:rust_template:invalid-input",
  "retry_after": null,
  "suggested_fix": {
    "description": "Correct the input so it satisfies the documented constraints, then retry.",
    "applicability": "maybe_incorrect"
  },
  "code_actions": [
    {
      "title": "Replace the offending input with a valid value",
      "kind": "quickfix",
      "applicability": "maybe_incorrect"
    }
  ],
  "exit_code": 2
}
```

## Stability

This is the `v1` definition. Its meaning is stable across releases; a breaking
change to the type's semantics ships a new version (`/v2`) rather than redefining
`v1`.

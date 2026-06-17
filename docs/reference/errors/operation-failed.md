---
diataxis_type: reference
---
# Operation failed

`OperationFailed` — a named operation could not complete. The inputs may have
been individually valid, but the operation itself failed.

## Identity

| Field      | Value                                                             |
|------------|-------------------------------------------------------------------|
| Type URI   | `https://zircote.com/rust-template/errors/operation-failed/v1`    |
| Title      | `Operation failed`                                                |
| Status     | `500`                                                             |
| Exit code  | `1`                                                               |
| Retryable  | No — `retry_after` is `null` (non-transient).                     |

## When it occurs

Returned when an operation runs but cannot produce a result — for example
`process("-7")` (a parsed value that is out of the operation's accepted range).
The structured `OperationFailed` variant names the failing operation and its
cause; the `detail` member carries the `Display` string, such as
`operation 'process' failed: value -7 is negative`.

## Recovery

| Field             | Value                                                              | Applicability   |
|-------------------|--------------------------------------------------------------------|-----------------|
| `suggested_fix`   | Inspect the operation's operands; the operation could not complete. | `unspecified`   |
| `code_actions[0]` | Adjust the operands so the operation can complete (`quickfix`).    | `unspecified`   |

The applicability is `unspecified` — the safe floor. A consumer treats it as
`maybe_incorrect` and escalates to a human; the cause is too general to prescribe
a machine-applicable edit.

## Example envelope

```json
{
  "type": "https://zircote.com/rust-template/errors/operation-failed/v1",
  "title": "Operation failed",
  "status": 500,
  "detail": "operation 'process' failed: value -7 is negative",
  "instance": "urn:rust_template:operation-failed",
  "retry_after": null,
  "suggested_fix": {
    "description": "Inspect the operation's operands; the operation could not complete.",
    "applicability": "unspecified"
  },
  "code_actions": [
    {
      "title": "Adjust the operands so the operation can complete",
      "kind": "quickfix",
      "applicability": "unspecified"
    }
  ],
  "exit_code": 1
}
```

## Stability

This is the `v1` definition. Its meaning is stable across releases; a breaking
change to the type's semantics ships a new version (`/v2`) rather than redefining
`v1`.

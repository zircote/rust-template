//! Integration tests for `rust_template`.

use rust_template::{
    Applicability, Config, Error, OutputFormat, ProblemDetails, Result, add, divide, process,
};

#[test]
fn test_add_integration() {
    // Test basic addition
    assert_eq!(add(1, 2), 3);
    assert_eq!(add(-5, 5), 0);

    // Test boundary conditions
    assert_eq!(add(i64::MAX, 0), i64::MAX);
    assert_eq!(add(i64::MIN, 0), i64::MIN);
}

#[test]
fn test_divide_integration() {
    // Test successful division
    assert_eq!(divide(100, 10).unwrap(), 10);
    assert_eq!(divide(-100, 10).unwrap(), -10);
    assert_eq!(divide(100, -10).unwrap(), -10);
    assert_eq!(divide(-100, -10).unwrap(), 10);

    // Test integer division truncation
    assert_eq!(divide(7, 3).unwrap(), 2);
    assert_eq!(divide(-7, 3).unwrap(), -2);
}

#[test]
fn test_divide_by_zero() {
    let result = divide(42, 0);
    assert!(
        matches!(result, Err(Error::InvalidInput(ref msg)) if msg.contains("zero")),
        "Expected InvalidInput error with message containing 'zero'"
    );
}

#[test]
fn test_config_builder_pattern() {
    let config = Config::new()
        .with_verbose(true)
        .with_max_retries(10)
        .with_timeout(120);

    assert!(config.verbose());
    assert_eq!(config.max_retries(), 10);
    assert_eq!(config.timeout_secs(), 120);
}

#[test]
fn test_config_clone() {
    let config1 = Config::new().with_verbose(true);
    let config2 = config1.clone();

    assert_eq!(config1.verbose(), config2.verbose());
    assert_eq!(config1.max_retries(), config2.max_retries());
    assert_eq!(config1.timeout_secs(), config2.timeout_secs());
}

#[test]
fn test_error_types() {
    // Test InvalidInput error
    let err = Error::InvalidInput("test message".to_string());
    let display = format!("{err}");
    assert!(display.contains("invalid input"));
    assert!(display.contains("test message"));

    // Test OperationFailed error
    let err = Error::OperationFailed {
        operation: "read".to_string(),
        cause: "file not found".to_string(),
    };
    let display = format!("{err}");
    assert!(display.contains("read"));
    assert!(display.contains("file not found"));
}

/// Adds `a` and `b`, then divides the sum by 2.
///
/// Demonstrates composing fallible operations — the `Result` from `divide`
/// propagates directly to the caller without explicit `match`.
fn process_numbers(a: i64, b: i64) -> Result<i64> {
    let sum = add(a, b);
    divide(sum, 2)
}

#[test]
fn test_result_chaining() {
    let result = process_numbers(10, 6);
    assert_eq!(result.unwrap(), 8);
}

/// Tests for the RFC 9457 Problem Details error-output architecture, exercised
/// across the crate boundary as a downstream consumer would.
mod problem_envelope_tests {
    use super::*;

    /// One forced instance of every `Error` variant the public API can produce.
    /// Built directly so this helper stays outside `#[test]` context without
    /// tripping the `unwrap_used` lint; `divide`/`process` produce these exact
    /// shapes (verified in `super::test_divide_by_zero` and the unit tests).
    fn each_variant() -> Vec<Error> {
        vec![
            Error::InvalidInput("divisor cannot be zero".to_string()),
            Error::OperationFailed {
                operation: "process".to_string(),
                cause: "value -7 is negative".to_string(),
            },
        ]
    }

    #[test]
    fn envelope_carries_five_standard_members_and_three_extensions() {
        for err in each_variant() {
            let json = err.to_problem().to_json();
            let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

            for member in ["type", "title", "status", "detail", "instance"] {
                assert!(
                    value.get(member).is_some(),
                    "missing standard member {member}"
                );
            }
            // retry_after is present even on non-transient errors (null), so the
            // agent never has to guess whether the class is retryable.
            assert!(value.get("retry_after").is_some());
            assert!(value["retry_after"].is_null());
            assert!(value.get("suggested_fix").is_some());
            assert!(value.get("code_actions").is_some());
        }
    }

    #[test]
    fn every_suggested_fix_and_code_action_has_an_applicability_marker() {
        for err in each_variant() {
            let problem = err.to_problem();
            assert!(problem.suggested_fix.is_some(), "missing suggested_fix");
            assert!(!problem.code_actions.is_empty(), "missing code_actions");

            let json = problem.to_json();
            let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
            assert!(value["suggested_fix"]["applicability"].is_string());
            for action in value["code_actions"].as_array().expect("array") {
                assert!(action["applicability"].is_string());
            }
        }
    }

    #[test]
    fn every_variant_has_a_distinct_versioned_type_uri() {
        let uris: Vec<String> = each_variant().iter().map(Error::type_uri).collect();
        for uri in &uris {
            assert!(uri.contains("/v1"), "type URI {uri} must embed a version");
        }
        // Distinctness: no two variants share a type URI.
        for (i, a) in uris.iter().enumerate() {
            for b in &uris[i + 1..] {
                assert_ne!(a, b, "type URIs must be distinct across variants");
            }
        }
    }

    #[test]
    fn dual_format_selection_picks_json_or_pretty() {
        let err = divide(10, 0).unwrap_err();

        // Pretty is byte-identical to the historical Display line.
        let pretty = err.render(OutputFormat::select(Some("pretty"), false));
        assert_eq!(pretty, format!("Error: {err}"));

        // JSON is the problem+json envelope.
        let json = err.render(OutputFormat::select(Some("json"), true));
        assert_eq!(json, err.to_problem().to_json());

        // Without a flag, TTY selects pretty and a pipe selects JSON.
        assert_eq!(err.render(OutputFormat::select(None, true)), pretty);
        assert_eq!(err.render(OutputFormat::select(None, false)), json);
    }

    #[test]
    fn reusable_envelope_builds_a_transient_error() {
        // A downstream adopter constructs its own envelope with retry_after.
        let problem = ProblemDetails::new(
            "https://example.com/errors/rate-limit/v1",
            "Rate limit exceeded",
            429,
            "Exceeded rate limit for this endpoint.",
            "urn:request:abc123",
        )
        .with_retry_after(180)
        .with_exit_code(2);

        let value: serde_json::Value =
            serde_json::from_str(&problem.to_json()).expect("valid JSON");
        assert_eq!(value["retry_after"], 180);
        assert_eq!(value["status"], 429);
    }

    #[test]
    fn applicability_default_is_unspecified() {
        assert_eq!(Applicability::default(), Applicability::Unspecified);
    }

    /// Pins the hand-built `each_variant()` shapes against the real error output
    /// of `divide`/`process`, so a change to either variant's `Display` message
    /// fails here rather than silently diverging from the literals the rest of
    /// this suite asserts against.
    #[test]
    fn each_variant_matches_real_error_output() {
        let variants = each_variant();
        assert_eq!(
            variants[0].to_string(),
            divide(10, 0).unwrap_err().to_string()
        );
        assert_eq!(
            variants[1].to_string(),
            process("-7").unwrap_err().to_string()
        );
    }

    /// Proof harness: prints the pretty and JSON renderings for every variant.
    /// Run `cargo test --all-features print_dual_renderings -- --nocapture` to
    /// see the evidence in the transcript.
    #[test]
    fn print_dual_renderings() {
        for err in each_variant() {
            println!("\n=== variant: {} ===", err.type_uri());
            println!("[pretty]\n{}", err.render(OutputFormat::Pretty));
            println!("[json]\n{}", err.to_problem().to_json_pretty());
        }
    }
}

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        // i32 inputs are widened to i64 to prevent overflow false positives:
        // two arbitrary i64 values can overflow on addition, but widened i32s
        // fit within i64 range, keeping the test valid for all sampled inputs.
        fn add_is_commutative(a in any::<i32>(), b in any::<i32>()) {
            let a = i64::from(a);
            let b = i64::from(b);
            prop_assert_eq!(add(a, b), add(b, a));
        }

        #[test]
        // Same i32→i64 widening strategy: ensures (a+b)+c and a+(b+c) never
        // overflow for any sampled triple, making the invariant always checkable.
        fn add_is_associative(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {
            let a = i64::from(a);
            let b = i64::from(b);
            let c = i64::from(c);
            prop_assert_eq!(add(add(a, b), c), add(a, add(b, c)));
        }

        #[test]
        fn add_zero_is_identity(n in any::<i64>()) {
            prop_assert_eq!(add(n, 0), n);
            prop_assert_eq!(add(0, n), n);
        }

        #[test]
        fn divide_by_one_is_identity(n in any::<i64>()) {
            prop_assert_eq!(divide(n, 1).unwrap(), n);
        }

        #[test]
        fn divide_by_nonzero_succeeds(
            (dividend, divisor) in
                (any::<i64>(), any::<i64>()).prop_filter(
                    "non-zero divisor and non-overflowing pair",
                    |(d, v)| *v != 0 && !(*d == i64::MIN && *v == -1),
                ),
        ) {
            prop_assert!(divide(dividend, divisor).is_ok());
        }
    }
}

/// Parameterized tests using the `test-case` crate.
mod parameterized_tests {
    use rust_template::{add, divide};
    use test_case::test_case;

    #[test_case(2, 3, 5 ; "positive numbers")]
    #[test_case(-1, 1, 0 ; "negative plus positive")]
    #[test_case(0, 0, 0 ; "both zero")]
    #[test_case(i64::MAX, 0, i64::MAX ; "max plus zero")]
    #[test_case(i64::MIN, 0, i64::MIN ; "min plus zero")]
    fn test_add_cases(a: i64, b: i64, expected: i64) {
        assert_eq!(add(a, b), expected);
    }

    #[test_case(10, 2, 5 ; "basic division")]
    #[test_case(-10, 2, -5 ; "negative dividend")]
    #[test_case(10, -2, -5 ; "negative divisor")]
    #[test_case(-10, -2, 5 ; "both negative")]
    #[test_case(7, 3, 2 ; "truncating toward zero positive")]
    #[test_case(-7, 3, -2 ; "truncating toward zero negative")]
    fn test_divide_cases(dividend: i64, divisor: i64, expected: i64) {
        assert_eq!(divide(dividend, divisor).ok(), Some(expected));
    }
}

/// Tests for derived trait implementations on public types.
mod trait_tests {
    use super::*;

    #[test]
    fn test_config_debug_format() {
        let config = Config::new();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("verbose"));
        assert!(debug_str.contains("max_retries"));
        assert!(debug_str.contains("timeout_secs"));
    }

    #[test]
    fn test_error_invalid_input_debug() {
        let err = Error::InvalidInput("msg".to_string());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("InvalidInput"));
        assert!(debug_str.contains("msg"));
    }

    #[test]
    fn test_error_operation_failed_debug() {
        let err = Error::OperationFailed {
            operation: "write".to_string(),
            cause: "disk full".to_string(),
        };
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("OperationFailed"));
        assert!(debug_str.contains("write"));
        assert!(debug_str.contains("disk full"));
    }

    #[test]
    fn test_config_clone_independence() {
        let original = Config::new().with_verbose(true).with_max_retries(9);
        let mut cloned = original.clone();
        // Modifying cloned via builder creates a new value; verify original is unchanged
        cloned = cloned.with_verbose(false);
        assert!(original.verbose(), "original should retain verbose=true");
        assert!(
            !cloned.verbose(),
            "cloned should have verbose=false after rebuild"
        );
        assert_eq!(original.max_retries(), cloned.max_retries());
    }
}

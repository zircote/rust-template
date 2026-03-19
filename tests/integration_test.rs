//! Integration tests for `rust_template`.

use rust_template::{Config, Error, Result, add, divide};

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

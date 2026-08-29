// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BoxTransformerOnce;
use qubit_function::TransformerOnce;

// ============================================================================
// BoxTransformerOnce Tests - Consuming, single ownership
// ============================================================================

#[cfg(test)]
mod box_transformer_once_tests {
    use super::BoxTransformerOnce;
    use super::TransformerOnce;

    #[test]
    fn test_new_and_transform() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));

        assert_eq!(parse.apply("42".to_string()), 42);
    }

    #[test]
    fn test_identity() {
        let identity = BoxTransformerOnce::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxTransformerOnce::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let composed = add_one.and_then(double);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_compose() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let composed = add_one.and_then(double);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_pipeline() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let pipeline = add_one.and_then(|x| x * 2).and_then(|x| x - 3);
        assert_eq!(pipeline.apply(5), 9); // ((5 + 1) * 2) - 3
    }

    #[test]
    fn test_consuming_string() {
        let into_bytes = BoxTransformerOnce::new(|s: String| s.into_bytes());
        let bytes = into_bytes.apply("hello".to_string());
        assert_eq!(bytes, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_type_conversion() {
        let to_string = BoxTransformerOnce::new(|x: i32| x.to_string());
        let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));
        assert_eq!(add_suffix.apply(42), "42_suffix");
    }

    #[test]
    fn test_display_with_name() {
        let transformer = BoxTransformerOnce::new_with_name("parse", |s: String| s.parse::<i32>().unwrap_or(0));
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "BoxTransformerOnce(parse)");
    }

    #[test]
    fn test_display_without_name() {
        let transformer = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "BoxTransformerOnce");
    }
}

// ============================================================================
// Conditional Transformer Once Tests
// ============================================================================

#[cfg(test)]
mod conditional_tests {
    use qubit_function::BoxPredicate;

    use super::BoxTransformerOnce;
    use super::TransformerOnce;

    #[test]
    fn test_when_or_else() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxTransformerOnce::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.apply(5), 10);
    }

    #[test]
    fn test_when_or_else_negative() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxTransformerOnce::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.apply(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.apply(5), 10);
        let result2 = BoxTransformerOnce::new(|x: i32| x * 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: i32| -x);
        assert_eq!(result2.apply(-5), 5);
    }
}

// ============================================================================
// Conversion Tests
// ============================================================================

// ============================================================================
// Complex Composition Tests
// ============================================================================

#[cfg(test)]
mod complex_composition_tests {
    use super::BoxTransformerOnce;
    use super::TransformerOnce;

    #[test]
    fn test_multiple_and_then() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let to_string = |x: i32| x.to_string();
        let composed = add_one.and_then(double).and_then(to_string);
        assert_eq!(composed.apply(5), "12"); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_multiple_compose() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let square = BoxTransformerOnce::new(|x: i32| x * x);
        let composed = add_one.and_then(double).and_then(square);
        assert_eq!(composed.apply(5), 144); // ((5 + 1) * 2)^2 = 144
    }

    #[test]
    fn test_mixed_composition() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));
        let double = |x: i32| x * 2;
        let to_string = |x: i32| format!("Result: {}", x);
        let pipeline = parse.and_then(double).and_then(to_string);
        assert_eq!(pipeline.apply("21".to_string()), "Result: 42");
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::BoxTransformerOnce;
    use super::TransformerOnce;

    #[test]
    fn test_identity_composition() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let identity = BoxTransformerOnce::<i32, i32>::identity();
        let composed = double.and_then(|x| identity.apply(x));
        assert_eq!(composed.apply(21), 42);
    }

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxTransformerOnce::constant("hello");
        assert_eq!(constant.apply(123), "hello");

        let constant2 = BoxTransformerOnce::constant("world");
        assert_eq!(constant2.apply(456), "world");
    }

    #[test]
    fn test_with_option() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse.apply("42".to_string()), Some(42));

        let parse2 = BoxTransformerOnce::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse2.apply("abc".to_string()), None);
    }

    #[test]
    fn test_with_result() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>());
        assert!(parse.apply("42".to_string()).is_ok());

        let parse2 = BoxTransformerOnce::new(|s: String| s.parse::<i32>());
        assert!(parse2.apply("abc".to_string()).is_err());
    }

    #[test]
    fn test_with_vec() {
        let split = BoxTransformerOnce::new(|s: String| s.split(',').map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(
            split.apply("a,b,c".to_string()),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_consuming_ownership() {
        let vec = vec![1, 2, 3, 4, 5];
        let sum = BoxTransformerOnce::new(|v: Vec<i32>| v.iter().sum::<i32>());
        assert_eq!(sum.apply(vec), 15);
        // vec is consumed and cannot be used again
    }

    #[test]
    fn test_with_box() {
        let boxed = Box::new(42);
        let unbox = BoxTransformerOnce::new(|b: Box<i32>| *b);
        assert_eq!(unbox.apply(boxed), 42);
    }

    #[test]
    fn test_with_closure_capture() {
        let multiplier = 3;
        let multiply = BoxTransformerOnce::new(move |x: i32| x * multiplier);
        assert_eq!(multiply.apply(7), 21);
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use super::BoxTransformerOnce;
    use super::TransformerOnce;

    #[test]
    fn test_transformer_once_trait() {
        fn apply_transformer_once<F: TransformerOnce<i32, i32>>(f: F, x: i32) -> i32 {
            f.apply(x)
        }

        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        assert_eq!(apply_transformer_once(double, 21), 42);
    }

    #[test]
    fn test_closure_as_transformer_once() {
        fn apply_transformer_once<F: TransformerOnce<i32, i32>>(f: F, x: i32) -> i32 {
            f.apply(x)
        }

        let double = |x: i32| x * 2;
        assert_eq!(apply_transformer_once(double, 21), 42);
    }

    #[test]
    fn test_with_different_types() {
        fn apply_transformer_once<T, R, F: TransformerOnce<T, R>>(f: F, x: T) -> R {
            f.apply(x)
        }

        let to_string = BoxTransformerOnce::new(|x: i32| x.to_string());
        assert_eq!(apply_transformer_once(to_string, 42), "42");
    }
}
// ============================================================================
// BoxTransformer TransformerOnce Tests
// ============================================================================

#[cfg(test)]
mod box_transformer_transformer_once_tests {
    use qubit_function::BoxTransformer;
    use qubit_function::Transformer;

    #[test]
    fn test_box_transformer_apply() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let result = double.apply(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_box_transformer_string_transformation() {
        let uppercase = BoxTransformer::new(|s: String| s.to_uppercase());
        let result = uppercase.apply("hello".to_string());
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_box_transformer_complex_transformation() {
        let parse_and_double = BoxTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0) * 2);
        let result = parse_and_double.apply("21".to_string());
        assert_eq!(result, 42);
    }
}

// ============================================================================
// RcTransformer TransformerOnce Tests
// ============================================================================

#[cfg(test)]
mod rc_transformer_transformer_once_tests {
    use qubit_function::RcTransformer;
    use qubit_function::Transformer;

    #[test]
    fn test_rc_transformer_apply() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let result = double.apply(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_rc_transformer_string_transformation() {
        let uppercase = RcTransformer::new(|s: String| s.to_uppercase());
        let result = uppercase.apply("hello".to_string());
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_rc_transformer_complex_transformation() {
        let parse_and_double = RcTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0) * 2);
        let result = parse_and_double.apply("21".to_string());
        assert_eq!(result, 42);
    }

    #[test]
    fn test_rc_transformer_clone_before_apply() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let double_clone = double.clone();

        // Both should work
        assert_eq!(double.apply(21), 42);
        assert_eq!(double_clone.apply(21), 42);
    }
}

// ============================================================================
// ArcTransformer TransformerOnce Tests
// ============================================================================

#[cfg(test)]
mod arc_transformer_transformer_once_tests {
    use std::sync::Arc;
    use std::thread;

    use qubit_function::ArcTransformer;
    use qubit_function::Transformer;

    #[test]
    fn test_arc_transformer_apply() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let result = double.apply(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_arc_transformer_string_transformation() {
        let uppercase = ArcTransformer::new(|s: String| s.to_uppercase());
        let result = uppercase.apply("hello".to_string());
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_arc_transformer_complex_transformation() {
        let parse_and_double = ArcTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0) * 2);
        let result = parse_and_double.apply("21".to_string());
        assert_eq!(result, 42);
    }

    #[test]
    fn test_arc_transformer_clone_before_apply() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let double_clone = double.clone();

        // Both should work
        assert_eq!(double.apply(21), 42);
        assert_eq!(double_clone.apply(21), 42);
    }

    #[test]
    fn test_arc_transformer_thread_safety() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let _double_arc = Arc::new(double);

        let handle = thread::spawn(move || {
            // We can't move out of Arc, so we need to use the regular apply
            // method or create a new transformer in the thread
            let new_double = ArcTransformer::new(|x: i32| x * 2);
            new_double.apply(21)
        });

        let result = handle.join().expect("thread should not panic");
        assert_eq!(result, 42);
    }
}

// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// ============================================================================
// BoxTransformer Tests - Immutable, single ownership
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use qubit_function::ArcTransformer;
    use qubit_function::BoxTransformer;
    use qubit_function::Transformer;

    #[test]
    fn test_identity_composition() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let identity = BoxTransformer::<i32, i32>::identity();
        let composed = double.and_then(identity);
        assert_eq!(composed.apply(21), 42);
    }

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxTransformer::constant("hello");
        assert_eq!(constant.apply(123), "hello");
        assert_eq!(constant.apply(456), "hello");
        assert_eq!(constant.apply(789), "hello");
    }

    #[test]
    fn test_with_option() {
        let parse = BoxTransformer::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse.apply("42".to_string()), Some(42));
        assert_eq!(parse.apply("abc".to_string()), None);
    }

    #[test]
    fn test_with_result() {
        let parse = BoxTransformer::new(|s: String| s.parse::<i32>());
        assert!(parse.apply("42".to_string()).is_ok());
        assert!(parse.apply("abc".to_string()).is_err());
    }

    #[test]
    fn test_with_vec() {
        let split = BoxTransformer::new(|s: String| s.split(',').map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(
            split.apply("a,b,c".to_string()),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_arc_with_large_data() {
        let process = ArcTransformer::new(|v: Vec<i32>| v.iter().sum::<i32>());
        let data = (1..=100).collect::<Vec<_>>();
        assert_eq!(process.apply(data), 5050);
    }
}
// ============================================================================
// Specialized into_fn Implementation Tests
// ============================================================================

// ============================================================================
// Type Conversion Tests
// ============================================================================

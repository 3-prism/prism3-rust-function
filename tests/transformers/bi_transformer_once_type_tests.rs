// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for BiTransformerOnce trait and implementations

use qubit_function::BiTransformerOnce;
use qubit_function::BoxBiTransformerOnce;

// ============================================================================
// Tests for BiTransformerOnce trait
// ============================================================================

#[cfg(test)]
mod type_tests {
    use super::BiTransformerOnce;
    use super::BoxBiTransformerOnce;

    #[test]
    fn test_with_integers() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(10, 20), 30);
    }

    #[test]
    fn test_with_floats() {
        let multiply = BoxBiTransformerOnce::new(|x: f64, y: f64| x * y);
        assert!((multiply.apply(3.5, 2.0) - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_with_strings() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}{}", x, y));
        assert_eq!(concat.apply("hello".to_string(), "world".to_string()), "helloworld");
    }

    #[test]
    fn test_with_mixed_types() {
        let format_pair = BoxBiTransformerOnce::new(|x: i32, y: String| format!("number: {}, text: {}", x, y));
        assert_eq!(format_pair.apply(42, "hello".to_string()), "number: 42, text: hello");
    }

    #[test]
    fn test_with_vectors() {
        let merge = BoxBiTransformerOnce::new(|mut x: Vec<i32>, y: Vec<i32>| {
            x.extend(y);
            x
        });
        assert_eq!(merge.apply(vec![1, 2], vec![3, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_with_options() {
        let combine = BoxBiTransformerOnce::new(|x: Option<i32>, y: Option<i32>| match (x, y) {
            (Some(a), Some(b)) => Some(a + b),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        });
        assert_eq!(combine.apply(Some(5), Some(3)), Some(8));
    }

    #[test]
    fn test_with_tuples() {
        let swap = BoxBiTransformerOnce::new(|x: (i32, String), y: (String, i32)| ((y.1, x.1), (x.0, y.0)));
        let result = swap.apply((42, "hello".to_string()), ("world".to_string(), 99));
        assert_eq!(result, ((99, "hello".to_string()), (42, "world".to_string())));
    }
}

// ============================================================================
// Tests for edge cases
// ============================================================================

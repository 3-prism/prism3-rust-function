// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::thread;

use qubit_function::ArcBiTransformer;
use qubit_function::BiTransformer;
use qubit_function::BoxBiTransformer;
use qubit_function::RcBiTransformer;

// ============================================================================
// BoxBiTransformer Tests - Immutable, single ownership
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::ArcBiTransformer;
    use super::BiTransformer;
    use super::BoxBiTransformer;

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxBiTransformer::constant("hello");
        assert_eq!(constant.apply(123, 456), "hello");
        assert_eq!(constant.apply(789, 101), "hello");
    }

    #[test]
    fn test_with_option() {
        let safe_divide = BoxBiTransformer::new(|x: i32, y: i32| if y == 0 { None } else { Some(x / y) });
        assert_eq!(safe_divide.apply(42, 2), Some(21));
        assert_eq!(safe_divide.apply(42, 0), None);
    }

    #[test]
    fn test_with_result() {
        let safe_divide = BoxBiTransformer::new(|x: i32, y: i32| -> Result<i32, String> {
            if y == 0 {
                Err("Division by zero".to_string())
            } else {
                Ok(x / y)
            }
        });
        assert_eq!(safe_divide.apply(42, 2), Ok(21));
        assert!(safe_divide.apply(42, 0).is_err());
    }

    #[test]
    fn test_with_vec() {
        let combine = BoxBiTransformer::new(|v1: Vec<i32>, v2: Vec<i32>| {
            let mut result = v1;
            result.extend(v2);
            result
        });
        assert_eq!(combine.apply(vec![1, 2, 3], vec![4, 5, 6]), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_arc_with_large_data() {
        let sum_vecs =
            ArcBiTransformer::new(|v1: Vec<i32>, v2: Vec<i32>| v1.iter().sum::<i32>() + v2.iter().sum::<i32>());
        let data1 = (1..=50).collect::<Vec<_>>();
        let data2 = (51..=100).collect::<Vec<_>>();
        assert_eq!(sum_vecs.apply(data1, data2), 5050);
    }

    #[test]
    fn test_with_tuples() {
        let swap = BoxBiTransformer::new(|x: i32, y: i32| (y, x));
        assert_eq!(swap.apply(1, 2), (2, 1));
    }

    #[test]
    fn test_string_operations() {
        let join = BoxBiTransformer::new(|s1: String, s2: String| format!("{} {}", s1, s2));
        assert_eq!(join.apply("Hello".to_string(), "World".to_string()), "Hello World");
    }
}

// ============================================================================
// Type Conversion Tests - Testing into_box, into_rc, into_arc methods
// ============================================================================

// ============================================================================
// Closure BiTransformer Tests - Testing blanket implementation for closures
// ============================================================================

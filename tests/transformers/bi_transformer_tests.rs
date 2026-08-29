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
mod box_bi_transformer_tests {
    use super::BiTransformer;
    use super::BoxBiTransformer;

    #[test]
    fn test_new_and_transform() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_multiple_calls() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(20, 22), 42);
        assert_eq!(add.apply(10, 10), 20);
        assert_eq!(add.apply(5, 3), 8);
    }

    #[test]
    fn test_multiply() {
        let multiply = BoxBiTransformer::new(|x: i32, y: i32| x * y);
        assert_eq!(multiply.apply(6, 7), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxBiTransformer::constant("hello");
        assert_eq!(constant.apply(123, 456), "hello");
        assert_eq!(constant.apply(789, 101), "hello");
    }

    #[test]
    fn test_with_string() {
        let concat = BoxBiTransformer::new(|s1: String, s2: String| format!("{}{}", s1, s2));
        assert_eq!(concat.apply("hello".to_string(), "world".to_string()), "helloworld");
    }

    #[test]
    fn test_captured_variable() {
        let multiplier = 3;
        let weighted_sum = BoxBiTransformer::new(move |x: i32, y: i32| x * multiplier + y * multiplier);
        assert_eq!(weighted_sum.apply(2, 3), 15); // (2 * 3) + (3 * 3) = 15
    }

    #[test]
    fn test_different_types() {
        let format = BoxBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(format.apply("Alice".to_string(), 30), "Alice is 30");
    }

    #[test]
    fn test_with_option() {
        let safe_divide = BoxBiTransformer::new(|x: i32, y: i32| if y == 0 { None } else { Some(x / y) });
        assert_eq!(safe_divide.apply(42, 2), Some(21));
        assert_eq!(safe_divide.apply(42, 0), None);
    }

    #[test]
    fn test_display_with_name() {
        let transformer = BoxBiTransformer::new_with_name("add", |x: i32, y: i32| x + y);
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "BoxBiTransformer(add)");
    }

    #[test]
    fn test_display_without_name() {
        let transformer = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "BoxBiTransformer");
    }
}

// ============================================================================
// ArcBiTransformer Tests - Immutable, thread-safe
// ============================================================================

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
mod box_bi_transformer_once_tests {
    use super::BiTransformerOnce;
    use super::BoxBiTransformerOnce;

    #[test]
    fn test_new() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_new_with_string() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{} {}", x, y));
        assert_eq!(concat.apply("hello".to_string(), "world".to_string()), "hello world");
    }

    #[test]
    fn test_constant() {
        let constant = BoxBiTransformerOnce::constant("hello");
        assert_eq!(constant.apply(123, 456), "hello");
    }

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxBiTransformerOnce::constant(42);
        assert_eq!(constant.apply("foo", "bar"), 42);
    }

    #[test]
    fn test_transform_consumes_inputs() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", x, y));
        let s1 = String::from("hello");
        let s2 = String::from("world");
        let result = concat.apply(s1, s2);
        assert_eq!(result, "hello-world");
        // s1 and s2 are moved and cannot be used here
    }

    #[test]
    fn test_and_then_with_closure() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let composed = add.and_then(double);
        assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2
    }

    #[test]
    fn test_and_then_with_to_string() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let to_string = |x: i32| x.to_string();
        let composed = add.and_then(to_string);
        assert_eq!(composed.apply(20, 22), "42");
    }

    #[test]
    fn test_and_then_chain_multiple() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let to_string = |x: i32| format!("Result: {}", x);
        let composed = add.and_then(double).and_then(to_string);
        assert_eq!(composed.apply(3, 5), "Result: 16");
    }

    #[test]
    fn test_and_then_with_string_transformation() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{} {}", x, y));
        let uppercase = |s: String| s.to_uppercase();
        let composed = concat.and_then(uppercase);
        assert_eq!(composed.apply("hello".to_string(), "world".to_string()), "HELLO WORLD");
    }

    #[test]
    fn test_and_then_type_conversion() {
        let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
        let to_float = |x: i32| x as f64 / 2.0;
        let composed = multiply.and_then(to_float);
        assert!((composed.apply(6, 7) - 21.0).abs() < 1e-10);
    }

    #[test]
    fn test_display_with_name() {
        let transformer = BoxBiTransformerOnce::new_with_name("multiply", |x: i32, y: i32| x * y);
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "BoxBiTransformerOnce(multiply)");
    }

    #[test]
    fn test_display_without_name() {
        let transformer = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "BoxBiTransformerOnce");
    }
}

// ============================================================================
// Tests for BoxBiTransformerOnce::when and conditional logic
// ============================================================================

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
mod rc_bi_transformer_tests {
    use super::BiTransformer;
    use super::RcBiTransformer;

    #[test]
    fn test_new_and_transform() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_clone() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let cloned = add.clone();

        assert_eq!(add.apply(20, 22), 42);
        assert_eq!(cloned.apply(20, 22), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcBiTransformer::constant("hello");
        assert_eq!(constant.apply(123, 456), "hello");
    }

    #[test]
    fn test_shared_usage() {
        let concat = RcBiTransformer::new(|s1: String, s2: String| format!("{}{}", s1, s2));

        let func1 = concat.clone();
        let func2 = concat.clone();

        assert_eq!(concat.apply("hello".to_string(), "world".to_string()), "helloworld");
        assert_eq!(func1.apply("foo".to_string(), "bar".to_string()), "foobar");
        assert_eq!(func2.apply("rust".to_string(), "lang".to_string()), "rustlang");
    }

    #[test]
    fn test_with_different_types() {
        let format = RcBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(format.apply("Alice".to_string(), 30), "Alice is 30");
    }

    #[test]
    fn test_display_with_name() {
        let transformer = RcBiTransformer::new_with_name("concat", |s1: String, s2: String| format!("{}{}", s1, s2));
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "RcBiTransformer(concat)");
    }

    #[test]
    fn test_display_without_name() {
        let transformer = RcBiTransformer::new(|s1: String, s2: String| format!("{}{}", s1, s2));
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "RcBiTransformer");
    }
}

// ============================================================================
// Conditional BiTransformer Tests
// ============================================================================

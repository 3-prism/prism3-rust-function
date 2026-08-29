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
mod trait_usage_tests {
    use super::BiTransformer;
    use super::BoxBiTransformer;

    #[test]
    fn test_bi_transformer_trait() {
        fn apply_bi_transformer<F: BiTransformer<i32, i32, i32>>(f: &F, x: i32, y: i32) -> i32 {
            f.apply(x, y)
        }

        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(apply_bi_transformer(&add, 20, 22), 42);
    }

    #[test]
    fn test_closure_as_bi_transformer() {
        fn apply_bi_transformer<F: BiTransformer<i32, i32, i32>>(f: &F, x: i32, y: i32) -> i32 {
            f.apply(x, y)
        }

        let add = |x: i32, y: i32| x + y;
        assert_eq!(apply_bi_transformer(&add, 20, 22), 42);
    }

    #[test]
    fn test_with_different_types() {
        fn apply_bi_transformer<T, U, R, F: BiTransformer<T, U, R>>(f: &F, x: T, y: U) -> R {
            f.apply(x, y)
        }

        let format = BoxBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(apply_bi_transformer(&format, "Alice".to_string(), 30), "Alice is 30");
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

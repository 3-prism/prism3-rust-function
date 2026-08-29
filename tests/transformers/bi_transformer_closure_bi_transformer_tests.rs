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
mod closure_bi_transformer_tests {
    use super::BiTransformer;

    #[test]
    fn test_closure_transform() {
        let add = |x: i32, y: i32| x + y;
        assert_eq!(add.apply(10, 20), 30);
    }

    #[test]
    fn test_closure_transform_with_string() {
        let concat = |s1: String, s2: String| format!("{}{}", s1, s2);
        assert_eq!(concat.apply("Hello".to_string(), "World".to_string()), "HelloWorld");
    }

    #[test]
    fn test_function_pointer_transform() {
        fn multiply(x: i32, y: i32) -> i32 {
            x * y
        }
        assert_eq!(multiply.apply(6, 7), 42);
    }

    #[test]
    fn test_closure_with_captured_variable() {
        let multiplier = 3;
        let multiply_by = move |x: i32, y: i32| (x + y) * multiplier;
        assert_eq!(multiply_by.apply(5, 5), 30);
    }
}

// ============================================================================
// Custom BiTransformer Tests - Testing default into_xxx() implementations
// ============================================================================

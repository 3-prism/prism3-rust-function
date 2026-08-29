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
mod box_bi_transformer_once_tests {
    use super::BiTransformer;
    use super::BoxBiTransformer;

    #[test]
    fn test_apply() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_multiply_once() {
        let multiply = BoxBiTransformer::new(|x: i32, y: i32| x * y);
        assert_eq!(multiply.apply(6, 7), 42);
    }

    #[test]
    fn test_string_concatenation_once() {
        let concat = BoxBiTransformer::new(|x: String, y: String| format!("{} {}", x, y));
        let result = concat.apply("Hello".to_string(), "World".to_string());
        assert_eq!(result, "Hello World");
    }
}

// ============================================================================
// RcBiTransformer BiTransformerOnce Tests
// ============================================================================

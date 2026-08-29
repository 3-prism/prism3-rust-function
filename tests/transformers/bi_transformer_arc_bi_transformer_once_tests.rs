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
mod arc_bi_transformer_once_tests {
    use std::thread;

    use super::ArcBiTransformer;
    use super::BiTransformer;

    #[test]
    fn test_apply() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_multiply_once() {
        let multiply = ArcBiTransformer::new(|x: i32, y: i32| x * y);
        assert_eq!(multiply.apply(6, 7), 42);
    }

    #[test]
    fn test_string_concatenation_once() {
        let concat = ArcBiTransformer::new(|x: String, y: String| format!("{} {}", x, y));
        let result = concat.apply("Hello".to_string(), "World".to_string());
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_thread_safety_apply() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let handle = thread::spawn(move || add.apply(10, 20));
        assert_eq!(handle.join().expect("thread should not panic"), 30);
    }
}

// ============================================================================
// Conditional Transformer Display/Debug Tests
// ============================================================================

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
mod arc_bi_transformer_tests {
    use super::ArcBiTransformer;
    use super::BiTransformer;
    use super::thread;

    #[test]
    fn test_new_and_transform() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_clone() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let cloned = add.clone();

        assert_eq!(add.apply(20, 22), 42);
        assert_eq!(cloned.apply(20, 22), 42);
    }

    #[test]
    fn test_thread_safe() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let cloned = add.clone();

        let handle = thread::spawn(move || cloned.apply(20, 22));

        assert_eq!(handle.join().expect("thread should not panic"), 42);
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_constant() {
        let constant = ArcBiTransformer::constant("hello");
        assert_eq!(constant.apply(123, 456), "hello");
    }

    #[test]
    fn test_multiple_threads() {
        let multiply = ArcBiTransformer::new(|x: i32, y: i32| x * y);

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let m = multiply.clone();
                thread::spawn(move || m.apply(i, i + 1))
            })
            .collect();

        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("thread should not panic"))
            .collect();

        assert_eq!(results, vec![0, 2, 6, 12]); // 0*1, 1*2, 2*3, 3*4
    }

    #[test]
    fn test_with_different_types() {
        let format = ArcBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(format.apply("Alice".to_string(), 30), "Alice is 30");
    }

    #[test]
    fn test_display_with_name() {
        let transformer = ArcBiTransformer::new_with_name("multiply", |x: i32, y: i32| x * y);
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "ArcBiTransformer(multiply)");
    }

    #[test]
    fn test_display_without_name() {
        let transformer = ArcBiTransformer::new(|x: i32, y: i32| x * y);
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "ArcBiTransformer");
    }
}

// ============================================================================
// RcBiTransformer Tests - Immutable, single-threaded
// ============================================================================

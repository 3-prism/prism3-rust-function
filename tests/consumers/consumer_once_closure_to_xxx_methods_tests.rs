// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::sync::Arc;
use std::sync::Mutex;

use qubit_function::BoxConsumerOnce;
///
/// # ConsumerOnce Tests
///
/// Unit tests for the ConsumerOnce trait and its implementations.
///
/// # ConsumerOnce Tests
///
/// Unit tests for the ConsumerOnce trait and its implementations.
use qubit_function::ConsumerOnce;

// ============================================================================
// BoxConsumerOnce Tests
// ============================================================================

#[cfg(test)]
mod closure_to_xxx_methods_tests {
    use super::Arc;
    use super::BoxConsumerOnce;
    use super::ConsumerOnce;
    use super::Mutex;

    /// Test a boxed closure with and_then().
    #[test]
    fn test_closure_fnonce_ops_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let chained = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x + 50);
        });

        chained.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 55]);
    }

    /// Test a boxed closure with multiple and_then() chains.
    #[test]
    fn test_closure_fnonce_ops_multiple_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();

        let chained = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x + 10);
        })
        .and_then(move |x: &i32| {
            l3.lock().expect("mutex should not be poisoned").push(*x * 3);
        });

        chained.accept(&5);
        // First: 5 * 2 = 10
        // Second: 5 + 10 = 15 (operates on original value, not on result of
        // first) Third: 5 * 3 = 15 (operates on original value, not on
        // result of second)
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15, 15]);
    }

    /// Test chain of closures with and_then() followed by conditional
    #[test]
    fn test_closure_chain_then_conditional() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let chained = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x + 10);
        });

        let boxed = chained;
        let conditional = boxed.when(|x: &i32| *x < 15);
        conditional.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15]); // Both execute because condition is true (5 < 15)
    }
}

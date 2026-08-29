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
mod closure_tests {
    use super::Arc;
    use super::BoxConsumerOnce;
    use super::ConsumerOnce;
    use super::Mutex;

    #[test]
    fn test_closure_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x * 2);
        };
        closure.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10]);
    }

    #[test]
    fn test_closure_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let chained = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x + 10);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15]);
    }

    #[test]
    fn test_closure_multi_step_chaining() {
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
            l3.lock().expect("mutex should not be poisoned").push(*x / 2);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15, 2]);
    }
}

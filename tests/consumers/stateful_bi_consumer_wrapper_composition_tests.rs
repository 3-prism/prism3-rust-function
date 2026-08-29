// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Comprehensive tests for StatefulBiConsumer types
//!
//! This module provides exhaustive test coverage for all StatefulBiConsumer
//! implementations including BoxStatefulBiConsumer, ArcStatefulBiConsumer,
//! RcStatefulBiConsumer, and their conditional variants.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use qubit_function::ArcStatefulBiConsumer;
use qubit_function::BiConsumerOnce;
use qubit_function::BoxStatefulBiConsumer;
use qubit_function::RcStatefulBiConsumer;
use qubit_function::StatefulBiConsumer;

// ============================================================================
// BoxStatefulBiConsumer Tests
// ============================================================================

/// Custom struct for testing StatefulBiConsumer trait default implementations
#[derive(Clone)]
struct CustomStatefulBiConsumer {
    multiplier: i32,
    log: Arc<Mutex<Vec<i32>>>,
}

impl StatefulBiConsumer<i32, i32> for CustomStatefulBiConsumer {
    fn accept(&mut self, first: &i32, second: &i32) {
        self.multiplier += 1;
        let result = (*first + *second) * self.multiplier;
        self.log.lock().expect("mutex should not be poisoned").push(result);
    }
}

#[cfg(test)]
mod wrapper_composition_tests {
    use super::Arc;
    use super::ArcStatefulBiConsumer;
    use super::BoxStatefulBiConsumer;
    use super::Mutex;
    use super::Rc;
    use super::RcStatefulBiConsumer;
    use super::RefCell;
    use super::StatefulBiConsumer;

    // Test and_then() on closure
    #[test]
    fn test_closure_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut chained = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, 15]);
    }

    // Test and_then() with multiple closures
    #[test]
    fn test_closure_and_then_chain() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let mut chained = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l3.lock().expect("mutex should not be poisoned").push(*x - *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, 15, 2]);
    }

    // Test and_then() with BoxStatefulBiConsumer
    #[test]
    fn test_closure_and_then_with_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let box_consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * *y);
        });
        let mut chained = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        })
        .and_then(box_consumer);

        chained.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, 15]);
    }

    // Test and_then() with ArcStatefulBiConsumer
    #[test]
    fn test_closure_and_then_with_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let arc_consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * *y);
        });
        let mut chained = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        })
        .and_then(arc_consumer);

        chained.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, 15]);
    }

    // Test and_then() with RcStatefulBiConsumer
    #[test]
    fn test_closure_and_then_with_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let rc_consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l2.borrow_mut().push(*x * *y);
        });
        let mut chained = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.borrow_mut().push(*x + *y);
        })
        .and_then(rc_consumer);

        chained.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8, 15]);
    }

    // Test and_then() returns BoxStatefulBiConsumer
}

// ============================================================================
// Closure Implementation Tests (StatefulBiConsumer trait for closures)
// ============================================================================

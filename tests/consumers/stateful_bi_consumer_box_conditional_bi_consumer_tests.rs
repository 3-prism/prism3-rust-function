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
mod box_conditional_bi_consumer_tests {
    use super::Arc;
    use super::BoxStatefulBiConsumer;
    use super::Mutex;
    use super::StatefulBiConsumer;

    // Test accept() with true condition
    #[test]
    fn test_accept_when_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);
    }

    // Test accept() with false condition
    #[test]
    fn test_accept_when_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![]);
    }

    // Test and_then() method
    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut chained = conditional.and_then(move |x: &i32, y: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * *y);
        });
        chained.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, 15]);
        chained.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, 15, -15]);
    }

    // Test or_else() method
    #[test]
    fn test_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let mut conditional = consumer
            .when(|x: &i32, _y: &i32| *x > 0)
            .or_else(move |x: &i32, y: &i32| {
                l2.lock().expect("mutex should not be poisoned").push(*x * *y);
            });

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);

        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, -15]);
    }

    // Test with always true predicate
    #[test]
    fn test_with_always_true_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let mut conditional = consumer.when(|_: &i32, _: &i32| true);
        conditional.accept(&5, &3);
        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, -2]);
    }

    // Test with always false predicate
    #[test]
    fn test_with_always_false_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let mut conditional = consumer.when(|_: &i32, _: &i32| false);
        conditional.accept(&5, &3);
        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), Vec::<i32>::new());
    }

    // Test complex predicate
    #[test]
    fn test_with_complex_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0 && *x + *y < 10);
        conditional.accept(&2, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
        conditional.accept(&5, &10);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    // Test Debug trait implementation
    #[test]
    fn test_debug() {
        let consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let debug_str = format!("{:?}", conditional);
        assert!(debug_str.contains("BoxConditionalStatefulBiConsumer"));
        assert!(debug_str.contains("consumer"));
        assert!(debug_str.contains("predicate"));
    }

    // Test Display trait implementation
    #[test]
    fn test_display() {
        let consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert!(display_str.contains("BoxConditionalStatefulBiConsumer"));
    }
}

// ============================================================================
// ArcStatefulBiConsumer Tests
// ============================================================================

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
mod arc_conditional_bi_consumer_tests {
    use super::Arc;
    use super::ArcStatefulBiConsumer;
    use super::Mutex;
    use super::StatefulBiConsumer;

    // Test accept() with true condition
    #[test]
    fn test_accept_when_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![]);
    }

    // Test clone() method
    #[test]
    fn test_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);
        clone2.accept(&10, &2);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, 12]);
    }

    // Test or_else() method
    #[test]
    fn test_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut with_else = conditional.or_else(move |x: &i32, y: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * *y);
        });
        with_else.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);
        with_else.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, -15]);
    }

    // Test Debug trait implementation
    #[test]
    fn test_debug() {
        let consumer = ArcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let debug_str = format!("{:?}", conditional);
        assert!(debug_str.contains("ArcConditionalStatefulBiConsumer"));
        assert!(debug_str.contains("consumer"));
        assert!(debug_str.contains("predicate"));
    }

    // Test Display trait implementation
    #[test]
    fn test_display() {
        let consumer = ArcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert!(display_str.contains("ArcConditionalStatefulBiConsumer"));
    }
}

// ============================================================================
// RcStatefulBiConsumer Tests
// ============================================================================

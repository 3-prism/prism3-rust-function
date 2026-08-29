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
mod rc_conditional_bi_consumer_tests {
    use super::Rc;
    use super::RcStatefulBiConsumer;
    use super::RefCell;
    use super::StatefulBiConsumer;

    // Test accept() with true condition
    #[test]
    fn test_accept_when_true() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test accept() with false condition
    #[test]
    fn test_accept_when_false() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![]);
    }

    // Test clone() method
    #[test]
    fn test_clone() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        clone2.accept(&10, &2);
        assert_eq!(*log.borrow(), vec![8, 12]);
    }

    // Test or_else() method
    #[test]
    fn test_or_else() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut with_else = conditional.or_else(move |x: &i32, y: &i32| {
            l2.borrow_mut().push(*x * *y);
        });
        with_else.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        with_else.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8, -15]);
    }

    // Test Debug trait implementation
    #[test]
    fn test_debug() {
        let consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let debug_str = format!("{:?}", conditional);
        assert!(debug_str.contains("RcConditionalStatefulBiConsumer"));
        assert!(debug_str.contains("consumer"));
        assert!(debug_str.contains("predicate"));
    }

    // Test Display trait implementation
    #[test]
    fn test_display() {
        let consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert!(display_str.contains("RcConditionalStatefulBiConsumer"));
    }
}

// ============================================================================
// Concrete wrapper composition tests
// ============================================================================

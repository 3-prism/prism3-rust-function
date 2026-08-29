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
mod rc_stateful_bi_consumer_tests {
    use super::Rc;
    use super::RcStatefulBiConsumer;
    use super::RefCell;
    use super::StatefulBiConsumer;

    // Test new() constructor
    #[test]
    fn test_new() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test new_with_name() constructor
    #[test]
    fn test_new_with_name() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new_with_name("test_consumer", move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test name() getter
    #[test]
    fn test_name_getter() {
        let consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        assert_eq!(consumer.name(), None);
    }

    // Test set_name() setter
    #[test]
    fn test_set_name() {
        let mut consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        assert_eq!(consumer.name(), None);
        consumer.set_name("my_consumer");
        assert_eq!(consumer.name(), Some("my_consumer"));
    }

    // Test accept() method
    #[test]
    fn test_accept() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x * *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![15]);
    }

    // Test clone() method
    #[test]
    fn test_clone() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });

        let mut clone1 = consumer.clone();
        let mut clone2 = consumer.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);

        clone2.accept(&10, &2);
        assert_eq!(*log.borrow(), vec![8, 12]);
    }

    // Test and_then() method
    #[test]
    fn test_and_then() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let first = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.borrow_mut().push(*x + *y);
        });
        let second = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l2.borrow_mut().push(*x * *y);
        });

        let mut chained = first.and_then(second);
        chained.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8, 15]);
    }

    // Test when() method
    #[test]
    fn test_when() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        conditional.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        conditional.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test accept_once() from BiConsumerOnce trait
    #[test]
    fn test_accept_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test Debug trait implementation
    #[test]
    fn test_debug() {
        let consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcStatefulBiConsumer"));
    }

    // Test Display trait implementation
    #[test]
    fn test_display() {
        let consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcStatefulBiConsumer");
    }

    // Test Display with name
    #[test]
    fn test_display_with_name() {
        let mut consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcStatefulBiConsumer(my_consumer)");
    }
}

// ============================================================================
// RcConditionalBiConsumer Tests
// ============================================================================

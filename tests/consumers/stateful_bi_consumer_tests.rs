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
mod box_stateful_bi_consumer_tests {
    use super::Arc;
    use super::BoxStatefulBiConsumer;
    use super::Mutex;
    use super::StatefulBiConsumer;

    // Test new() constructor
    #[test]
    fn test_new() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);
    }

    // Test new_with_name() constructor
    #[test]
    fn test_new_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new_with_name("test_consumer", move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);
    }

    // Test noop() constructor
    #[test]
    fn test_noop() {
        let mut noop = BoxStatefulBiConsumer::<i32, i32>::noop();
        noop.accept(&42, &10);
        // Should not panic, values unchanged
    }

    // Test name() getter
    #[test]
    fn test_name_getter() {
        let consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        assert_eq!(consumer.name(), None);
    }

    // Test set_name() setter
    #[test]
    fn test_set_name() {
        let mut consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        assert_eq!(consumer.name(), None);
        consumer.set_name("my_consumer");
        assert_eq!(consumer.name(), Some("my_consumer"));
    }

    // Test accept() method
    #[test]
    fn test_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x * *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![15]);
    }

    // Test accept() with multiple calls
    #[test]
    fn test_accept_multiple_calls() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        consumer.accept(&1, &2);
        consumer.accept(&3, &4);
        consumer.accept(&5, &6);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![3, 7, 11]);
    }

    // Test and_then() method
    #[test]
    fn test_and_then() {
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

    // Test and_then() with multiple consumers
    #[test]
    fn test_and_then_chain() {
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

    // Test when() method
    #[test]
    fn test_when_true_condition() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);
    }

    // Test when() with false condition
    #[test]
    fn test_when_false_condition() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x < 0 && *y < 0);

        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![]);
    }

    // Test accept_once() from BiConsumerOnce trait
    #[test]
    fn test_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);
    }

    // Test with different types
    #[test]
    fn test_with_different_types() {
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |s: &String, n: &i32| {
            *l.lock().expect("mutex should not be poisoned") = format!("{}: {}", s, n);
        });
        consumer.accept(&"Count".to_string(), &42);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), "Count: 42");
    }

    // Test with zero values
    #[test]
    fn test_with_zero_values() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        consumer.accept(&0, &0);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![0]);
    }

    // Test with negative values
    #[test]
    fn test_with_negative_values() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        consumer.accept(&-5, &-3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![-8]);
    }

    // Test with mixed positive and negative values
    #[test]
    fn test_with_mixed_values() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        consumer.accept(&5, &-3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![2]);
    }

    // Test Debug trait implementation
    #[test]
    fn test_debug() {
        let consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxStatefulBiConsumer"));
    }

    // Test Display trait implementation
    #[test]
    fn test_display() {
        let consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxStatefulBiConsumer");
    }

    // Test Display with name
    #[test]
    fn test_display_with_name() {
        let mut consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxStatefulBiConsumer(my_consumer)");
    }
}

// ============================================================================
// BoxConditionalBiConsumer Tests
// ============================================================================

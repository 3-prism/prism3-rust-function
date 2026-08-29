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
mod edge_cases_tests {
    use super::Arc;
    use super::BoxStatefulBiConsumer;
    use super::Mutex;
    use super::StatefulBiConsumer;

    // Test with empty operations
    #[test]
    fn test_noop_multiple_calls() {
        let mut consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        consumer.accept(&5, &3);
        consumer.accept(&10, &20);
        consumer.accept(&1, &2);
        // Should do nothing
    }

    // Test with large values
    #[test]
    fn test_with_large_values() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i64, y: &i64| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        consumer.accept(&i64::MAX, &0);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![i64::MAX]);
    }

    // Test with minimum values
    #[test]
    fn test_with_min_values() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i64, y: &i64| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });
        consumer.accept(&i64::MIN, &0);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![i64::MIN]);
    }

    // Test with string types
    #[test]
    fn test_with_string_types() {
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |s1: &String, s2: &String| {
            *l.lock().expect("mutex should not be poisoned") = format!("{}{}", s1, s2);
        });
        consumer.accept(&"Hello, ".to_string(), &"World!".to_string());
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), "Hello, World!");
    }

    // Test with empty strings
    #[test]
    fn test_with_empty_strings() {
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |s1: &String, s2: &String| {
            *l.lock().expect("mutex should not be poisoned") = format!("{}{}", s1, s2);
        });
        consumer.accept(&"".to_string(), &"".to_string());
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), "");
    }

    // Test with complex types
    #[test]
    fn test_with_complex_types() {
        #[derive(Debug, Clone, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |p1: &Point, p2: &Point| {
            l.lock().expect("mutex should not be poisoned").push(Point {
                x: p1.x + p2.x,
                y: p1.y + p2.y,
            });
        });
        consumer.accept(&Point { x: 1, y: 2 }, &Point { x: 3, y: 4 });
        assert_eq!(
            *log.lock().expect("mutex should not be poisoned"),
            vec![Point { x: 4, y: 6 }]
        );
    }

    // Test stateful behavior
    #[test]
    fn test_stateful_behavior() {
        let counter = Arc::new(Mutex::new(0));
        let c = counter.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().expect("mutex should not be poisoned") += 1;
            std::hint::black_box(x + y);
        });
        consumer.accept(&1, &2);
        consumer.accept(&3, &4);
        consumer.accept(&5, &6);
        assert_eq!(*counter.lock().expect("mutex should not be poisoned"), 3);
    }

    // Test and_then with noop
    #[test]
    fn test_and_then_with_noop() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        })
        .and_then(BoxStatefulBiConsumer::noop());
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);
    }
}

// ============================================================================
// Custom Struct Tests - StatefulBiConsumer Default Implementation to_xxx()
// ============================================================================
